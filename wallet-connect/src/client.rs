/// The main client state definitions
mod core;
/// The external options to create a client
mod options;
/// The wallet-connect session management
pub mod session;
/// The websocket connection management
mod socket;

use std::collections::HashMap;
use std::str::FromStr;

use self::{
    core::{Connector, ConnectorError},
    options::Options,
    session::SessionInfo,
};
use crate::{hex, protocol::Metadata};
use async_trait::async_trait;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::{
    prelude::{
        Address, Bytes, FromErr, JsonRpcClient, Middleware, NameOrAddress, Provider, ProviderError,
        Signature, TransactionRequest,
    },
    utils::rlp,
};
use eyre::eyre;
use eyre::Context;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
#[derive(Debug, Clone)]
pub enum ClientChannelMessageType {
    Connecting,
    Connected,
    Updated,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct ClientChannelMessage {
    pub state: ClientChannelMessageType,
    pub session: Option<SessionInfo>,
}

impl Default for ClientChannelMessage {
    fn default() -> Self {
        Self {
            state: ClientChannelMessageType::Disconnected,
            session: None,
        }
    }
}

/// The WalletConnect 1.0 client
/// (holds the middleware trait implementations for ethers)
/// this is persistent and can be recovered from session-info-string.
/// session-info-string can be saved to the disk
///
/// for callback channel, if Client is cloned, it can recieve multiple events
/// this channel is mpsc pattern
/// so make sure that only one thread calls ensure_session
#[derive(Debug, Clone)]
pub struct Client {
    connection: Arc<Mutex<Connector>>, // Client is cloneable
    // to make receive channel valid, sender channel should be open
    // if cloned, multiple events can be received by mpsc pattern
    callback_channel: Option<UnboundedSender<ClientChannelMessage>>,
}

impl Client {
    /// Creates a new client from the provided metadata and chain id
    /// (and will connect to the bridge server according to the URI in metadata)
    pub async fn new(
        meta: impl Into<Metadata>,
        chain_id: Option<u64>,
    ) -> Result<Self, ConnectorError> {
        Client::with_options(Options::new(meta.into(), chain_id)).await
    }

    /// Restore a new client from the provided options
    pub async fn restore(session_info: SessionInfo) -> Result<Self, ConnectorError> {
        Ok(Client {
            callback_channel: None,
            connection: Arc::new(Mutex::new(Connector::restore(session_info).await?)),
        })
    }

    /// get current session info, can be saved , and later, restored
    pub async fn get_session_info(&self) -> Result<SessionInfo, ConnectorError> {
        let connection = self.connection.lock().await;
        connection.get_session_info().await
    }

    /// create qrcode from this string
    pub async fn get_connection_string(&self) -> Result<String, ConnectorError> {
        let connection = self.connection.lock().await;
        Ok(connection.get_uri().await?.as_url().as_str().to_string())
    }

    /// manual polling for session
    /// receive client state messages directly though channel
    /// refer to run_callback to create channel
    pub fn set_callback(&mut self, callback_channel: UnboundedSender<ClientChannelMessage>) {
        self.callback_channel = Some(callback_channel);
    }

    /// automatic polling for session
    ///  receive client state messages through callback
    pub async fn run_callback(
        &mut self,
        mycallback: Box<dyn Fn(ClientChannelMessage) -> eyre::Result<()> + Send + Sync>,
    ) -> eyre::Result<tokio::task::JoinHandle<eyre::Result<()>>> {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<ClientChannelMessage>();

        self.set_callback(sender);

        let join_handle = tokio::spawn(async move {
            loop {
                let message = receiver.recv().await;

                if let Some(message) = message {
                    mycallback(message)?;
                }
            }
        });

        Ok(join_handle)
    }
    /// Creates a new client from the provided options
    /// (and will connect to the bridge server according to the URI in metadata)
    pub async fn with_options(options: Options) -> Result<Self, ConnectorError> {
        Ok(Client {
            callback_channel: options.callback_channel.clone(),
            connection: Arc::new(Mutex::new(Connector::new(options).await?)),
        })
    }

    /// This will return an existing session or create a new session.
    /// If successful, the returned value is the wallet's addresses and the chain ID.
    /// TODO: more specific error types than eyre
    pub async fn ensure_session(&mut self) -> Result<(Vec<Address>, u64), eyre::Error> {
        let mut connection = self.connection.lock().await;
        if let Some(v) = &self.callback_channel {
            connection.set_callback(v.clone()).await;
        }

        connection.ensure_session().await
    }

    /// Send a request to sign a message as per https://eips.ethereum.org/EIPS/eip-1271
    pub async fn personal_sign(
        &mut self,
        message: &str,
        address: &Address,
    ) -> Result<Signature, ClientError> {
        let sig_str: String = self
            .request(
                "personal_sign",
                vec![
                    format!("0x{}", hex::encode(message)),
                    format!("{:?}", address),
                    // "".to_string(), // TODO is password needed?
                ],
            )
            .await?;

        Signature::from_str(&sig_str)
            .context("failed to parse signature")
            .map_err(ClientError::Eyre)
    }
}

/// Error thrown when sending an HTTP request
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("{0}")]
    Eyre(#[from] eyre::Report),
    #[error("Deserialization Error: {err}. Response: {text}")]
    /// Serde JSON Error
    SerdeJson {
        err: serde_json::Error,
        text: String,
    },
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for Client {
    type Error = ClientError;

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync + std::fmt::Debug, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        let connection = self.connection.lock().await;
        connection.request(method, params).await
    }
}

/// The wrapper struct for `ethers` middleware
/// TODO: override transaction-related middleware methods,
/// so that the client broadcasts the transaction (instead of the wallet)?
#[derive(Debug)]
pub struct WCMiddleware<M>(M);

impl WCMiddleware<Provider<Client>> {
    /// Creates a new wrapper for `ethers` middleware
    pub fn new(client: Client) -> Self {
        WCMiddleware(Provider::new(client))
    }
}

/// The wrapper error type for `ethers` middleware-related issues
#[derive(Error, Debug)]
pub enum WCError<M: Middleware> {
    #[error("{0}")]
    MiddlewareError(M::Error),
    #[error("client error: {0}")]
    ClientError(ClientError),
}

impl<M: Middleware> FromErr<M::Error> for WCError<M> {
    fn from(src: M::Error) -> WCError<M> {
        WCError::MiddlewareError(src)
    }
}

#[async_trait]
impl Middleware for WCMiddleware<Provider<Client>> {
    type Error = WCError<Provider<Client>>;
    type Provider = Client;
    type Inner = Provider<Client>;

    fn inner(&self) -> &Provider<Client> {
        &self.0
    }

    async fn sign_transaction(
        &self,
        tx: &TypedTransaction,
        from: Address,
    ) -> Result<Signature, Self::Error> {
        let mut tx_obj = HashMap::new();
        tx_obj.insert("from", format!("{:?}", from));
        if let Some(to) = tx.to() {
            let addr = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(n) => self.resolve_name(n).await?,
            };
            tx_obj.insert("to", format!("{:?}", addr));
        }
        if let Some(data) = tx.data() {
            tx_obj.insert("data", format!("0x{}", hex::encode(data)));
        } else {
            tx_obj.insert("data", "".to_string());
        }
        if let Some(gas) = tx.gas() {
            tx_obj.insert("gas", format!("0x{:x}", gas));
        }
        if let Some(gas_price) = tx.gas_price() {
            tx_obj.insert("gasPrice", format!("0x{:x}", gas_price));
        }
        if let Some(value) = tx.value() {
            tx_obj.insert("value", format!("0x{:x}", value));
        }
        if let Some(nonce) = tx.nonce() {
            tx_obj.insert("nonce", format!("0x{:x}", nonce));
        }
        if let Some(c) = tx.chain_id() {
            tx_obj.insert("chainId", format!("0x{:x}", c));
        }
        // TODO: put those error cases to WCError instead of wrapping in eyre
        let tx_bytes: Bytes = self
            .0
            .request("eth_signTransaction", vec![tx_obj])
            .await
            .map_err(|e| WCError::ClientError(ClientError::Eyre(eyre!(e))))?;
        let tx_rlp = rlp::Rlp::new(tx_bytes.as_ref());
        if tx_rlp.as_raw().is_empty() {
            return Err(WCError::ClientError(ClientError::Eyre(eyre!(
                "failed to decode transaction , empty rlp"
            ))));
        }

        let first_byte = tx_rlp.as_raw()[0];
        // TODO: check that the decoded request matches the typed transaction content here? or a new `sign_transaction` function that returns both request+signature?
        if first_byte <= 0x7f {
            let decoded_request = TypedTransaction::decode_signed(&tx_rlp);
            decoded_request
                .map(|x| x.1)
                .map_err(|e| WCError::ClientError(ClientError::Eyre(eyre!(e))))
        } else if (0xc0..=0xfe).contains(&first_byte) {
            let decoded_request = TransactionRequest::decode_signed_rlp(&tx_rlp);
            decoded_request
                .map(|x| x.1)
                .map_err(|e| WCError::ClientError(ClientError::Eyre(eyre!(e))))
        } else {
            Err(WCError::ClientError(ClientError::Eyre(eyre!(
                "failed to decode transaction"
            ))))
        }
    }
}
