use std::{sync::Arc, time::Duration};

use super::{
    crypto::{decode_decrypt, encrypt_and_encode},
    protocol::{
        WcSessionDelete, WcSessionExtend, WcSessionPing, WcSessionProposeResponse,
        WcSessionRequest, WcSessionSettle, WcSessionUpdate, WC_SESSION_DELETE_RESPONSE_TAG,
        WC_SESSION_EVENT_RESPONSE_TAG, WC_SESSION_EXTEND_RESPONSE_TAG,
        WC_SESSION_PING_REQUEST_METHOD, WC_SESSION_PING_REQUEST_TAG, WC_SESSION_PING_RESPONSE_TAG,
        WC_SESSION_PROPOSE_REQUEST_METHOD, WC_SESSION_PROPOSE_REQUEST_TAG,
        WC_SESSION_REQUEST_METHOD, WC_SESSION_REQUEST_TAG, WC_SESSION_SETTLE_RESPONSE_TAG,
        WC_SESSION_UPDATE_RESPONSE_TAG,
    },
    session::SessionInfo,
};
use crate::crypto::Key;
use crate::v2::WcSessionPropose;
use crate::{v2::WcSessionEvent, ClientError, Request, Response};
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::providers::JsonRpcClient;
use relay_client::{
    Client, CloseFrame, ConnectionHandler, ConnectionOptions, Error, PublishedMessage,
};
use relay_rpc::{
    auth::{ed25519_dalek::Keypair, rand, rand::Rng, AuthToken},
    domain::{AuthSubject, SubscriptionId, Topic},
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{mpsc, oneshot, Mutex, Notify};

/// This `Context` holds the wallet-connect client state
#[derive(Debug)]
pub struct Context {
    /// the current session information
    /// it's under mutex, as it's accessed by multiple threads
    /// and may be updated from the connected wallet
    /// (e.g. when a new address is added)
    pub session: Mutex<SessionInfo>,
    /// will notify when the session is established
    /// (after receiving the `wc_sessionSettle` request)
    pub session_pending_notify: Notify,
    /// record the time of the request and have a regular cleanup
    pub pending_requests_timeout: Duration,
    /// limit pending requests size
    pub pending_requests_limit: usize,
    /// the map of the requests that were sent to the wallet
    /// and the client app is awaiting a response.
    /// When the response is received, the request is removed
    /// and the response is sent to the receiver via the one-shot channel.
    pub pending_requests: DashMap<u64, oneshot::Sender<serde_json::Value>>,
    /// the map of existing subscriptions
    /// (currently unused; but may be used for deleting subscriptions etc.)
    pub subscriptions: DashMap<Topic, SubscriptionId>,
}

/// `SharedContext` holds the thread-safe reference to the wallet-connect client state
pub type SharedContext = Arc<Context>;

impl Context {
    /// Creates a new client state context from the provided session
    /// (empty pending requests)
    pub fn new(session: SessionInfo) -> Self {
        Self {
            session: Mutex::new(session),
            session_pending_notify: Notify::new(),
            pending_requests_timeout: Duration::from_millis(60000),
            pending_requests_limit: 2,
            pending_requests: DashMap::new(),
            subscriptions: DashMap::new(),
        }
    }

    async fn send_response<T: Serialize>(
        &self,
        argresponse: Response<T>,
        sender: &mpsc::Sender<ConnectorMessage>,
        tag: u32,
    ) -> eyre::Result<()> {
        let response_str = serde_json::to_string(&argresponse)?;
        let session = self.session.lock().await;
        if let Some((t, key)) = &session.pairing_topic_symkey {
            let message = encrypt_and_encode(key, response_str.as_bytes());
            let _ = sender
                .send(ConnectorMessage::Publish(t.clone(), message, tag))
                .await;
        }
        Ok(())
    }
    async fn send_callback<T: Serialize>(
        &self,
        message: T,
        callback_sender: Option<mpsc::UnboundedSender<String>>,
    ) -> eyre::Result<()> {
        let request_str = serde_json::to_string(&message)?;
        if let Some(sender) = callback_sender {
            sender.send(request_str).unwrap();
        }
        Ok(())
    }

    async fn handle_session_proposal_response(
        &self,
        plain: &[u8],
        sender: &mpsc::Sender<ConnectorMessage>,
    ) -> eyre::Result<()> {
        let response = serde_json::from_slice::<Response<WcSessionProposeResponse>>(plain)?;
        {
            let response_json = serde_json::to_value(&response)?;
            if let Ok(r) = response.data.into_result() {
                let mut session = self.session.lock().await;
                if let Some(t) = session.session_proposal_response(&r) {
                    let _ = sender.send(ConnectorMessage::Subscribe(t.clone())).await;
                }
            }
            if let Some((_, sender)) = self.pending_requests.remove(&response.id) {
                let _ = sender.send(response_json);
            }
        }
        Ok(())
    }

    async fn handle_session_settle_request(
        &self,
        plain: &[u8],
        sender: &mpsc::Sender<ConnectorMessage>,
    ) -> eyre::Result<()> {
        let request = serde_json::from_slice::<Request<WcSessionSettle>>(plain)?;
        {
            let response = Response::new(request.id, true);

            self.send_response(response, sender, WC_SESSION_SETTLE_RESPONSE_TAG)
                .await?;
            let mut session = self.session.lock().await;
            session.session_settle(request.params);
            session.connected = true;
            self.session_pending_notify.notify_waiters();
        }
        Ok(())
    }

    async fn handle_session_event_request(
        &self,
        plain: &[u8],
        sender: &mpsc::Sender<ConnectorMessage>,
        callback_sender: Option<mpsc::UnboundedSender<String>>,
    ) -> eyre::Result<()> {
        let request = serde_json::from_slice::<Request<WcSessionEvent>>(plain)?;
        let response = Response::new(request.id, true);
        self.send_response(response, sender, WC_SESSION_EVENT_RESPONSE_TAG)
            .await?;
        self.send_callback(request, callback_sender).await?;
        Ok(())
    }

    async fn handle_session_delete_request(
        &self,
        plain: &[u8],
        sender: &mpsc::Sender<ConnectorMessage>,
        callback_sender: Option<mpsc::UnboundedSender<String>>,
    ) -> eyre::Result<()> {
        let request = serde_json::from_slice::<Request<WcSessionDelete>>(plain)?;
        {
            let mut session = self.session.lock().await;
            session.session_delete();
        }
        let response = Response::new(request.id, true);
        self.send_response(response, sender, WC_SESSION_DELETE_RESPONSE_TAG)
            .await?;

        self.send_callback(request, callback_sender).await?;
        Ok(())
    }

    async fn handle_session_update_request(
        &self,
        plain: &[u8],
        sender: &mpsc::Sender<ConnectorMessage>,
        callback_sender: Option<mpsc::UnboundedSender<String>>,
    ) -> eyre::Result<()> {
        let request = serde_json::from_slice::<Request<WcSessionUpdate>>(plain)?;

        {
            let mut session = self.session.lock().await;
            session.session_update(request.params.clone());
        }

        let response = Response::new(request.id, true);
        self.send_response(response, sender, WC_SESSION_UPDATE_RESPONSE_TAG)
            .await?;
        self.send_callback(request, callback_sender).await?;
        Ok(())
    }

    async fn handle_session_extend_request(
        &self,
        plain: &[u8],
        sender: &mpsc::Sender<ConnectorMessage>,
        callback_sender: Option<mpsc::UnboundedSender<String>>,
    ) -> eyre::Result<()> {
        let request = serde_json::from_slice::<Request<WcSessionExtend>>(plain)?;

        {
            let mut session = self.session.lock().await;
            session.session_extend();
        }

        let response = Response::new(request.id, true);
        self.send_response(response, sender, WC_SESSION_EXTEND_RESPONSE_TAG)
            .await?;
        self.send_callback(request, callback_sender).await?;
        Ok(())
    }

    async fn handle_session_ping_request(
        &self,
        plain: &[u8],
        sender: &mpsc::Sender<ConnectorMessage>,
        callback_sender: Option<mpsc::UnboundedSender<String>>,
    ) -> eyre::Result<()> {
        let request = serde_json::from_slice::<Request<WcSessionPing>>(plain)?;
        let response = Response::new(request.id, true);
        self.send_response(response, sender, WC_SESSION_PING_RESPONSE_TAG)
            .await?;
        self.send_callback(request, callback_sender).await?;
        Ok(())
    }

    async fn handle_normal_rpc_response(&self, plain: &[u8]) -> eyre::Result<()> {
        let response = serde_json::from_slice::<Response<serde_json::Value>>(plain)
            .map_err(eyre::Report::from)?;

        let (_, sender) = self
            .pending_requests
            .remove(&response.id)
            .ok_or_else(|| eyre::eyre!("Request not found"))?;
        let value = response.data.into_value().map_err(eyre::Report::from)?;
        let _ = sender.send(value);
        Ok(())
    }
}

/// The handler of WC 2.0 messages
struct MessageHandler {
    /// the shared context of the client
    context: SharedContext,
    /// if websocket is connected
    /// (currently not used; for debugging purposes)
    connected: bool,
    /// the last error if any
    /// (currently not used; for debugging purposes)
    last_connection_error: Option<Error>,
    sender: mpsc::Sender<ConnectorMessage>, // send queue

    callback_sender: Option<mpsc::UnboundedSender<String>>, // callback
}

impl MessageHandler {
    fn new(
        context: SharedContext,
        sender: mpsc::Sender<ConnectorMessage>,

        callback_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> Self {
        Self {
            context,
            connected: false,
            last_connection_error: None,
            sender,
            callback_sender,
        }
    }
}

impl ConnectionHandler for MessageHandler {
    fn connected(&mut self) {
        self.connected = true;
    }

    fn disconnected(&mut self, _frame: Option<CloseFrame<'static>>) {
        self.connected = false;
    }

    // TODO: collect the JoinHandle and await them in a separate loop/task?
    // or rewrite this whole thing, such that here it'll just push `message`
    // onto a channel and the processing will be done in a separate task spawned elsewhere?
    //
    // send event back to a channel (whole json)
    // in c++ bindings, also whole json can be sent
    fn message_received(&mut self, message: PublishedMessage) {
        let context = self.context.clone();
        let sender = self.sender.clone();
        let callback_sender = self.callback_sender.clone();

        tokio::spawn(async move {
            let session = context.session.lock().await;
            match (&message.topic, &session.pairing_topic_symkey) {
                // this case is for the session proposal
                // so expecting the session proposal response there
                (t, _) if t == &session.session_proposal_topic => {
                    if let Ok(plain) =
                        decode_decrypt(&session.session_proposal_symkey, &message.message)
                    {
                        drop(session);
                        let _ = context
                            .handle_session_proposal_response(&plain, &sender)
                            .await;
                    }
                }
                // this case is for the session settlement and normal requests
                // (and events? TODO: check if session updates are sent here)
                (t1, Some((t2, key))) if t1 == t2 => {
                    if let Ok(plain) = decode_decrypt(key, &message.message) {
                        drop(session);
                        let plain = plain.as_slice();
                        let plainjson = serde_json::from_slice::<serde_json::Value>(plain).unwrap();
                        // request json
                        // jsonrpc, id, method, params
                        if let Some(method_value) = plainjson.get("method") {
                            if let Some(method) = method_value.as_str() {
                                match method {
                                    "wc_sessionSettle" => {
                                        let _ = context
                                            .handle_session_settle_request(plain, &sender)
                                            .await;
                                    }

                                    "wc_sessionUpdate" => {
                                        let _ = context
                                            .handle_session_update_request(
                                                plain,
                                                &sender,
                                                callback_sender,
                                            )
                                            .await;
                                    }

                                    "wc_sessionExtend" => {
                                        let _ = context
                                            .handle_session_extend_request(
                                                plain,
                                                &sender,
                                                callback_sender,
                                            )
                                            .await;
                                    }

                                    "wc_sessionPing" => {
                                        let _ = context
                                            .handle_session_ping_request(
                                                plain,
                                                &sender,
                                                callback_sender,
                                            )
                                            .await;
                                    }

                                    "wc_sessionDelete" => {
                                        let _ = context
                                            .handle_session_delete_request(
                                                plain,
                                                &sender,
                                                callback_sender,
                                            )
                                            .await;
                                    }

                                    "wc_sessionEvent" => {
                                        let _ = context
                                            .handle_session_event_request(
                                                plain,
                                                &sender,
                                                callback_sender,
                                            )
                                            .await;
                                    }

                                    _ => (),
                                }
                            }
                        } else {
                            // response json
                            // jsonrpc, id, result
                            let _ = context.handle_normal_rpc_response(plain).await;
                        }
                    }
                }
                _ => {
                    // unknown topic
                    // TODO: send back error?
                }
            }
        });
    }

    fn inbound_error(&mut self, error: Error) {
        self.last_connection_error = Some(error);
    }

    fn outbound_error(&mut self, error: Error) {
        self.last_connection_error = Some(error);
    }
}

/// maximum is 9007199254740991 , 2^53 -1
/// cannot be zero
fn get_safe_random() -> u64 {
    let random_request_id: u64 = rand::thread_rng().gen();
    random_request_id % 9007199254740990 + 1
}

/// Handles publishing messages and subscribing to topics
#[derive(Debug)]
pub struct Connector {
    context: SharedContext,
    _task_handler: tokio::task::JoinHandle<()>,
    sender: mpsc::Sender<ConnectorMessage>,
}

/// messages processed in the task loop
#[derive(Debug)]
enum ConnectorMessage {
    Publish(Topic, String, u32),
    Subscribe(Topic),
}

impl Connector {
    ///  create qrcode with this uri
    pub async fn get_uri(&self) -> String {
        let session = self.context.session.lock().await;
        session.uri()
    }

    /// get session info, can be saved for restoration
    pub async fn get_session_info(&self) -> SessionInfo {
        let session = self.context.session.lock().await;
        session.clone()
    }

    pub async fn do_request<T: Serialize>(
        &self,
        topic: Topic,
        key: &Key,
        method: &str,
        params: T,
        tag: u32,
    ) -> eyre::Result<serde_json::Value> {
        let request_id = get_safe_random();
        let req = Request::new(request_id, method, params);
        use eyre::Context;
        let request_str = serde_json::to_string(&req).wrap_err("serialize request")?;
        let message = encrypt_and_encode(key, request_str.as_bytes());

        let (ping_sender, ping_receiver) = oneshot::channel();
        self.context
            .pending_requests
            .insert(request_id, ping_sender);

        self.sender
            .send(ConnectorMessage::Publish(
                topic.clone(),
                message.clone(),
                tag,
            ))
            .await
            .map_err(|e| ClientError::Eyre(eyre::eyre!(e)))?;
        let receivedpacket = ping_receiver.await?;
        Ok(receivedpacket)
    }

    pub async fn send_ping(&mut self) -> eyre::Result<String> {
        let params = serde_json::json!({});

        let session = self.context.session.lock().await;
        let topickey = if let Some((topic, key)) = session.pairing_topic_symkey.as_ref() {
            Some((topic.clone(), key.clone()))
        } else {
            None
        };
        drop(session);
        // if pairing was established, we should have a topic + symmetric key
        if let Some((topic, key)) = topickey {
            let receivedpacket = self
                .do_request(
                    topic,
                    &key,
                    WC_SESSION_PING_REQUEST_METHOD,
                    params,
                    WC_SESSION_PING_REQUEST_TAG,
                )
                .await?;
            let receivedpacket_str = serde_json::to_string(&receivedpacket)?;

            Ok(receivedpacket_str)
        } else {
            Err(eyre::eyre!("no pairing established"))
        }
    }

    /// establishes the session
    pub async fn ensure_session(&mut self) -> eyre::Result<()> {
        let session = self.context.session.lock().await;
        // the session proposal topic
        let topic = session.session_proposal_topic.clone();
        use eyre::Context;
        // subscribe to that topic
        self.sender
            .send(ConnectorMessage::Subscribe(topic.clone()))
            .await
            .wrap_err("subscribe")?;

        let proposal: WcSessionPropose = session.session_proposal();
        let key: Key = session.session_proposal_symkey.clone();
        drop(session);

        let response = self
            .do_request(
                topic,
                &key,
                WC_SESSION_PROPOSE_REQUEST_METHOD,
                proposal,
                WC_SESSION_PROPOSE_REQUEST_TAG,
            )
            .await?;

        if let Some(error) = response.get("error") {
            return Err(eyre::eyre!(
                "EnsureSessionFail {}",
                serde_json::to_string(&error)?
            ));
        }

        // wait for the session settle request
        self.context.session_pending_notify.notified().await;
        Ok(())
    }

    /// creates a new connector
    pub async fn new_client(
        session: SessionInfo,
        callback_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> Result<Self, Error> {
        let mut relay_address = session.relay_server.clone().to_string();
        // remove "/"
        relay_address.pop();
        let project_id = session.project_id.clone();
        let context = Arc::new(Context::new(session));
        let (sender, mut receiver) = mpsc::channel(10);
        let handler = MessageHandler::new(context.clone(), sender.clone(), callback_sender);
        let client = Client::new(handler);
        let key = Keypair::generate(&mut rand::thread_rng());
        let auth = AuthToken::new(AuthSubject::generate())
            .aud(relay_address.clone())
            .ttl(Duration::from_secs(60 * 60))
            .as_jwt(&key)
            .expect("jwt token");
        let opts = ConnectionOptions::new(project_id, auth).with_address(relay_address);
        client.connect(opts).await?;

        let task_context = context.clone();
        // a task loop to handle messages
        // that we need to send to the walletconnect relay server
        let _task_handler = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Some(ConnectorMessage::Publish(topic, message, tag)) => {
                        let _ = client
                            .publish(topic, message, tag, task_context.pending_requests_timeout)
                            .await;
                    }
                    Some(ConnectorMessage::Subscribe(topic)) => {
                        let sid = client.subscribe(topic.clone()).await;
                        if let Ok(id) = sid {
                            task_context.subscriptions.insert(topic, id);
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        });
        Ok(Self {
            context,
            _task_handler,
            sender,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for Connector {
    type Error = ClientError;

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        let session = self.context.session.lock().await;
        let topickey = if let Some((topic, key)) = session.pairing_topic_symkey.as_ref() {
            Some((topic.clone(), key.clone()))
        } else {
            None
        };
        // get chain id or default (cronos mainnet)
        let chain_id = session
            .required_namespaces
            .eip155
            .chains
            .get(0)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "eip155:25".to_owned());
        // release the lock
        drop(session);
        // if pairing was established, we should have a topic + symmetric key
        if let Some((topic, key)) = topickey {
            let request_id = get_safe_random();
            let params = WcSessionRequest::new(method.to_string(), params, chain_id);
            let req = Request::new(request_id, WC_SESSION_REQUEST_METHOD, params);
            use eyre::Context;
            let request_str = serde_json::to_string(&req).wrap_err("serialize request")?;
            let message = encrypt_and_encode(&key, request_str.as_bytes());
            let (sender, receiver) = oneshot::channel();
            self.context.pending_requests.insert(request_id, sender);
            self.sender
                .send(ConnectorMessage::Publish(
                    topic.clone(),
                    message,
                    WC_SESSION_REQUEST_TAG,
                ))
                .await
                .map_err(|e| ClientError::Eyre(eyre::eyre!(e)))?;
            let response = receiver
                .await
                .map_err(|e| ClientError::Eyre(eyre::eyre!(e)))?;
            let resp: R = serde_json::from_value(response).wrap_err("failed to parse response")?;
            Ok(resp)
        } else {
            Err(ClientError::Eyre(eyre::eyre!(
                "no pairing topic and symkey"
            )))
        }
    }
}
