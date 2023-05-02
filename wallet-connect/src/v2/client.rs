use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::{
    prelude::{
        Address, Bytes, JsonRpcClient, Middleware, MiddlewareError, NameOrAddress, Provider,
        Signature, TransactionRequest,
    },
    utils::rlp,
};
use eyre::eyre;
use eyre::Context;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;

use crate::{hex, ClientError};

use super::core::Connector;
use super::protocol::{Namespaces, RequiredNamespaces};
use super::session::SessionInfo;
use super::Metadata;
use relay_client::Error;

/// The WalletConnect 2.0 basic client options
pub struct ClientOptions {
    /// The relay server url
    /// (the default is wss://relay.walletconnect.org)
    /// Note that `Url` will append `/` and
    /// the current relay_client implementation
    /// would return 404 in that case, so make sure
    /// to pop off the trailing `/`
    pub relay_server: Url,
    /// The project id (obtained from the walletconnect.org registration)
    pub project_id: String,
    /// The required namespaces
    /// (methods, chains, events) by the dApp
    /// -- the wallet may not support them though
    pub required_namespaces: RequiredNamespaces,
    /// The client / dApp metadata
    pub client_meta: Metadata,
    /// callback
    pub callback_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
}

/// The WalletConnect 2.0 client
/// (holds the middleware trait implementations for ethers)
/// this is persistent and can be recovered from session-info-string.
/// session-info-string can be saved to the disk
#[derive(Debug, Clone)]
pub struct Client {
    connection: Arc<RwLock<Connector>>, // Client is cloneable
}

impl Client {
    /// Creates a new client from the provided metadata
    pub async fn new(opts: ClientOptions) -> Result<Self, Error> {
        let session = SessionInfo::new(
            opts.relay_server,
            opts.project_id,
            opts.required_namespaces,
            opts.client_meta,
        );

        let connector = Connector::new_client(session, opts.callback_sender).await?;
        Ok(Client {
            connection: Arc::new(RwLock::new(connector)),
        })
    }

    /// Restore a new client from the provided options
    pub async fn restore(
        session_info: SessionInfo,
        callback_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) -> eyre::Result<Self> {
        Ok(Client {
            connection: Arc::new(RwLock::new(
                Connector::new_client(session_info, callback_sender).await?,
            )),
        })
    }

    /// get current session info, can be saved , and later, restored
    pub async fn get_session_info(&self) -> SessionInfo {
        let connection = self.connection.read().await;
        connection.get_session_info().await
    }

    /// create qrcode from this string
    pub async fn get_connection_string(&self) -> String {
        let connection = self.connection.read().await;
        connection.get_uri().await
    }

    /// This will return an existing session or create a new session.
    /// If successful, the returned value is the wallet's addresses and the chain ID.
    /// TODO: more specific error types than eyre
    pub async fn ensure_session(&mut self) -> Result<Namespaces, eyre::Error> {
        let mut connection = self.connection.write().await;

        connection.ensure_session().await?;
        connection
            .get_session_info()
            .await
            .namespaces
            .ok_or_else(|| eyre!("No namespaces in session info"))
    }

    pub async fn send_ping(&mut self) -> Result<String, eyre::Error> {
        let mut connection = self.connection.write().await;
        connection.send_ping().await
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
                    format!("{address:?}"),
                    // "".to_string(), // TODO is password needed?
                ],
            )
            .await?;

        Signature::from_str(&sig_str)
            .context("failed to parse signature")
            .map_err(ClientError::Eyre)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for Client {
    type Error = ClientError;

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync + std::fmt::Debug, R: DeserializeOwned + Send>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        let connection = self.connection.read().await;
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

impl<M: Middleware> MiddlewareError for WCError<M> {
    type Inner = M::Error;
    fn from_err(src: M::Error) -> Self {
        WCError::MiddlewareError(src)
    }
    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            WCError::MiddlewareError(e) => Some(e),
            _ => None,
        }
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
        tx_obj.insert("from", format!("{from:?}"));
        if let Some(to) = tx.to() {
            let addr = match to {
                NameOrAddress::Address(addr) => *addr,
                NameOrAddress::Name(n) => self.resolve_name(n).await?,
            };
            tx_obj.insert("to", format!("{addr:?}"));
        }
        if let Some(data) = tx.data() {
            tx_obj.insert("data", format!("0x{}", hex::encode(data)));
        } else {
            tx_obj.insert("data", "".to_string());
        }
        if let Some(gas) = tx.gas() {
            tx_obj.insert("gas", format!("0x{gas:x}"));
        }
        if let Some(gas_price) = tx.gas_price() {
            tx_obj.insert("gasPrice", format!("0x{gas_price:x}"));
        }
        if let Some(value) = tx.value() {
            tx_obj.insert("value", format!("0x{value:x}"));
        }
        if let Some(nonce) = tx.nonce() {
            tx_obj.insert("nonce", format!("0x{nonce:x}"));
        }
        if let Some(c) = tx.chain_id() {
            tx_obj.insert("chainId", format!("0x{c:x}"));
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
