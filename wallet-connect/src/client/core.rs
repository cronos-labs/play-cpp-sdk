//! Copyright (c) 2020 Nicholas Rodrigues Lordello (licensed under the Apache License, Version 2.0)
//! Modifications Copyright (c) 2022, Cronos Labs (licensed under the Apache License, Version 2.0)
use super::{
    options::{Connection, Options},
    session::Session,
    session::SessionInfo,
    socket::{MessageHandler, Socket},
};
use crate::client::ClientChannelMessage;
use crate::protocol::Topic;
use crate::uri::Uri;
use crate::ClientError;
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::prelude::{Address, JsonRpcClient};
use rand::Rng;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{atomic::AtomicBool, Arc};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{oneshot, Mutex};

/// This `Context` holds the wallet-connect client state
#[derive(Debug)]
pub struct Context {
    /// the current session information
    /// it's under mutex, as it's accessed by multiple threads
    /// and may be updated from the connected wallet
    /// (e.g. when a new address is added)
    pub session: Mutex<Session>,
    /// indicates whether the session is being established
    pub session_pending: AtomicBool,
    /// the map of the requests that were sent to the wallet
    /// and the client app is awaiting a response.
    /// When the response is received, the request is removed
    /// and the response is sent to the receiver via the one-shot channel.
    /// TODO: limit size or record the time of the request and have a regular cleanup?
    pub pending_requests: DashMap<u64, oneshot::Sender<serde_json::Value>>,
}

/// `SharedContext` holds the thread-safe reference to the wallet-connect client state
#[derive(Clone, Debug)]
pub struct SharedContext(pub Arc<Context>);

impl SharedContext {
    /// Creates a new client state context from the provided session
    /// (empty pending requests)
    pub fn new(session: Session) -> Self {
        Self(Arc::new(Context {
            session: Mutex::new(session),
            session_pending: AtomicBool::new(false),
            pending_requests: DashMap::new(),
        }))
    }
}

/// It holds the wallet-connect connection state
#[derive(Debug)]
pub struct Connector {
    /// the websocket connection
    socket: Socket,
    /// the client state
    context: SharedContext,
}

/// maximum is 9007199254740991 , 2^53 -1
/// cannot be zero
fn get_safe_random() -> u64 {
    let random_request_id: u64 = rand::thread_rng().gen();
    random_request_id % 9007199254740990 + 1
}
impl Connector {
    ///  create qrcode with this uri
    pub async fn get_uri(&self) -> Result<Uri, ConnectorError> {
        let session = self.context.0.session.lock().await;
        Ok(session.uri())
    }

    /// get session info, can be saved for restoration
    pub async fn get_session_info(&self) -> Result<SessionInfo, ConnectorError> {
        let session = self.context.0.session.lock().await;
        Ok(session.info.clone())
    }

    pub async fn set_callback(&mut self, myfunc: UnboundedSender<ClientChannelMessage>) {
        self.context.0.session.lock().await.set_callback(myfunc);
    }

    /// This will return an existing session or create a new session.
    /// If successful, the returned value is the wallet's addresses and the chain ID.
    /// TODO: more specific error types than eyre
    pub async fn ensure_session(&mut self) -> Result<(Vec<Address>, u64), eyre::Error> {
        let session = self.context.0.session.lock().await;
        if session.info.connected {
            Ok((
                session.info.accounts.clone(),
                session.info.chain_id.unwrap_or_default(),
            ))
        } else {
            session.event_connecting();
            // no need to hold the session lock, hence this explicit drop
            drop(session);
            self.socket
                .create_session(get_safe_random(), &mut self.context)
                .await
        }
    }

    pub async fn new_client(
        handshake_topic: Option<Topic>,
        session: Session,
    ) -> Result<Self, ConnectorError> {
        let client_id = session.info.client_id.clone();
        // NOTE: WalletConnect bridge URLs are expected to be automatically
        // converted from a `http(s)` to `ws(s)` protocol for the WebSocket
        // connection.
        let mut url = session.info.bridge.clone();
        match url.scheme() {
            "http" => url.set_scheme("ws").unwrap(),
            "https" => url.set_scheme("wss").unwrap(),
            "ws" | "wss" => {}
            scheme => return Err(ConnectorError::BadScheme(scheme.into())),
        }
        let key = session.info.key.clone();
        let context = SharedContext::new(session);
        let handler = MessageHandler {
            context: context.clone(),
        };
        let mut socket = Socket::connect(url, key, handler).await?;
        socket.subscribe(client_id.clone()).await?;
        if let Some(topic) = handshake_topic {
            socket.subscribe(topic).await?;
        }
        Ok(Self { socket, context })
    }

    pub async fn restore(session_info: SessionInfo) -> Result<Self, ConnectorError> {
        // invalidate old topic, because  it's pending
        let handshake_topic = Some(session_info.handshake_topic.clone());

        let session = Session {
            info: session_info,
            callback_channel: None,
        };
        Connector::new_client(handshake_topic, session).await
    }

    /// Given the options (that contain the connection string),
    /// this will create a new connector (i.e. try to connect to the bridge server
    /// via websockets and wait for responses).
    pub async fn new(options: Options) -> Result<Self, ConnectorError> {
        let handshake_topic = match &options.connection {
            Connection::Uri(uri) => Some(uri.handshake_topic().clone()),
            _ => None,
        };
        let session = options.create_session();
        Connector::new_client(handshake_topic, session).await
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
        self.socket
            .json_rpc_request::<T, R>(get_safe_random(), method, params, &self.context)
            .await
            .map_err(ClientError::Eyre)
    }
}

/// The errors when creating a connector
/// TODO: more specific error types than eyre
#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("invalid URL scheme '{0}', must be 'http(s)' or 'ws(s)'")]
    BadScheme(String),
    #[error("socket error: {0}")]
    SocketError(#[from] eyre::Report),
}

#[cfg(test)]
mod test {
    use crate::{uri::Uri, SocketMessage};

    #[test]
    pub fn test_payloads() {
        let u = "wc:c0254d9e-b523-4b7e-845a-e457abe05df4@1?bridge=https%3A%2F%2Fl.bridge.walletconnect.org&key=f674df12094c46f96f41fd6a6ec7702eadb41f706480369d5be9729716147807";
        let uri = Uri::parse(u).unwrap();
        let msg: SocketMessage = serde_json::from_str(r#"
        {"topic":"c0254d9e-b523-4b7e-845a-e457abe05df4","type":"pub","payload":"{\"data\":\"8e3e03af98d72bb0ef41ba129004c3bd6600bfa1783d2da5b00283f092d5ae6f6a6867f36734dc98f48d167c504ef7e8e2d3a65d8e33c48357150b5eec3c7f7fc56e96b08267b270e25818d4b25d0242a21b44dd886496e6ea14a8889138353d266eccd32e79d98444419a9960e342e19c8d04f0baacd5e936ac88e77c35f68f8e3145dda4f9d116b91c9f610c936c4a7ce74d616d8eebecca8f419f93d613bf77a1a4acce2d6baa1aab29518b87d753722501caffa7ad8f6d5e48845eb3ac0c334993d19df4998cf8762892f3d3b787e7072153435c7396e94604c74a709aaf703fae9bf22124e35757af345e1aef500f2dfd0a8b3ef47148f5cb210ea255676a069998276ed508ee139e0ceefb19573aef6554eda744535df51fcf21b99a4e42804e698fcdb8520f7d1d59471c7d76bcbf19b3d212ef5eff68aea280beeca4\",\"hmac\":\"28aa4d7db35149c431885e8d17d4a0cef434d8933688bd15dd4da809a51e25a2\",\"iv\":\"4ab2c77513e0857b08f59fba56c92634\"}","silent":true}
        "#).unwrap();
        dbg!(String::from_utf8(uri.key().open(&msg.payload.unwrap()).unwrap()).unwrap());
    }
}
