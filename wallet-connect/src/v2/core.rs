use std::{sync::Arc, time::Duration};

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

use crate::{ClientError, Request, Response};

use super::{
    crypto::{decode_decrypt, encrypt_and_encode},
    protocol::{
        WcSessionProposeResponse, WcSessionRequest, WcSessionSettle,
        WC_SESSION_PROPOSE_REQUEST_METHOD, WC_SESSION_PROPOSE_REQUEST_TAG,
        WC_SESSION_REQUEST_METHOD, WC_SESSION_REQUEST_TAG, WC_SESSION_SETTLE_RESPONSE_TAG,
    },
    session::SessionInfo,
};

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
    sender: mpsc::Sender<ConnectorMessage>,
}

impl MessageHandler {
    fn new(context: SharedContext, sender: mpsc::Sender<ConnectorMessage>) -> Self {
        Self {
            context,
            connected: false,
            last_connection_error: None,
            sender,
        }
    }
}

#[async_trait]
impl ConnectionHandler for MessageHandler {
    async fn connected(&mut self) {
        self.connected = true;
    }

    async fn disconnected(&mut self, _frame: Option<CloseFrame<'static>>) {
        self.connected = false;
    }

    async fn message_received(&mut self, message: PublishedMessage) {
        let mut session = self.context.session.lock().await;
        match (&message.topic, &session.pairing_topic_symkey) {
            // this case is for the session proposal
            // so expecting the session proposal response there
            (t, _) if t == &session.session_proposal_topic => {
                if let Ok(plain) =
                    decode_decrypt(&session.session_proposal_symkey, &message.message)
                {
                    if let Ok(response) =
                        serde_json::from_slice::<Response<WcSessionProposeResponse>>(&plain)
                    {
                        if let Ok(r) = response.data.into_result() {
                            // derive the new topic and symkey
                            if let Some(t) = session.session_proposal_response(&r) {
                                // subscribe to the new topic
                                let _ = self
                                    .sender
                                    .send(ConnectorMessage::Subscribe(t.clone()))
                                    .await;
                            }
                        }
                        if let Some((_, sender)) =
                            self.context.pending_requests.remove(&response.id)
                        {
                            // notify the client app that the session is being established
                            let _ = sender.send(serde_json::Value::Null);
                        }
                    }
                }
            }
            // this case is for the session settlement and normal requests
            // (and events? TODO: check if session updates are sent here)
            (t1, Some((t2, key))) if t1 == t2 => {
                if let Ok(plain) = decode_decrypt(&key, &message.message) {
                    // the response to normal RPC requests (e.g. eth_sendTransaction)
                    if let Ok(response) =
                        serde_json::from_slice::<Response<serde_json::Value>>(&plain)
                    {
                        if let Some((_, sender)) =
                            self.context.pending_requests.remove(&response.id)
                        {
                            if let Ok(value) = response.data.into_value() {
                                // notify the client dApp that the response is received
                                let _ = sender.send(value);
                            }
                        }
                    } else {
                        // the request for session settlement
                        if let Ok(request) =
                            serde_json::from_slice::<Request<WcSessionSettle>>(&plain)
                        {
                            let response = Response::new(request.id, true);
                            let response_str =
                                serde_json::to_string(&response).expect("serialize response");
                            let message = encrypt_and_encode(&key, response_str.as_bytes());
                            // need to send a reply to the wallet
                            let _ = self
                                .sender
                                .send(ConnectorMessage::Publish(
                                    t1.clone(),
                                    message,
                                    WC_SESSION_SETTLE_RESPONSE_TAG,
                                ))
                                .await;
                            session.session_settle(request.params);
                            session.connected = true;
                            // notify the client dApp that the session is settled
                            self.context.session_pending_notify.notify_waiters();
                        }
                    }
                }
            }
            _ => {
                // unknown topic
                // TODO: send back error?
            }
        }
    }

    async fn inbound_error(&mut self, error: Error) {
        self.last_connection_error = Some(error);
    }

    async fn outbound_error(&mut self, error: Error) {
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

    /// establishes the session
    pub async fn ensure_session(&mut self) -> eyre::Result<()> {
        // the session proposal topic
        let topic = self
            .context
            .session
            .lock()
            .await
            .session_proposal_topic
            .clone();
        use eyre::Context;
        // subscribe to that topic
        self.sender
            .send(ConnectorMessage::Subscribe(topic.clone()))
            .await
            .wrap_err("subscribe")?;
        // create a session proposal request
        let session_request_id = get_safe_random();
        let session = self.context.session.lock().await;
        let session_request = Request::new(
            session_request_id,
            WC_SESSION_PROPOSE_REQUEST_METHOD,
            session.session_proposal(),
        );
        let session_request_str =
            serde_json::to_string(&session_request).expect("serialize session request");
        let message = encrypt_and_encode(
            &session.session_proposal_symkey,
            session_request_str.as_bytes(),
        );
        // release the lock
        drop(session);
        let (proposal_sender, proposal_receiver) = oneshot::channel();
        self.context
            .pending_requests
            .insert(session_request_id, proposal_sender);
        // request for publishing the session proposal request
        self.sender
            .send(ConnectorMessage::Publish(
                topic,
                message,
                WC_SESSION_PROPOSE_REQUEST_TAG,
            ))
            .await
            .wrap_err("publish")?;
        // wait for the session proposal response
        let _ = proposal_receiver.await;
        // wait for the session settle request
        self.context.session_pending_notify.notified().await;
        Ok(())
    }

    /// creates a new connector
    pub async fn new_client(session: SessionInfo) -> Result<Self, Error> {
        let mut relay_address = session.relay_server.clone().to_string();
        // remove "/"
        relay_address.pop();
        let project_id = session.project_id.clone();
        let context = Arc::new(Context::new(session));
        let (sender, mut receiver) = mpsc::channel(10);
        let handler = MessageHandler::new(context.clone(), sender.clone());
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
