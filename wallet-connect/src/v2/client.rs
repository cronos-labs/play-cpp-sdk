use async_trait::async_trait;
use ethers::prelude::PendingTransaction;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::BlockId;
use ethers::types::U256;
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
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;

use super::core::Connector;
use super::protocol::{Namespaces, RequiredNamespaces};
use super::session::SessionInfo;
use super::Metadata;
use crate::{hex, ClientError};
use relay_client::Error;

#[derive(Debug)]
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

// implement Default for ClientOptions
impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            relay_server: "wss://relay.walletconnect.com".parse().expect("url"),
            project_id: "".into(),
            required_namespaces: RequiredNamespaces::new(
                vec![
                    "eth_sendTransaction".to_owned(),
                    "eth_signTransaction".to_owned(),
                    "eth_sign".to_owned(),
                    "personal_sign".to_owned(),
                    "eth_signTypedData".to_owned(),
                ],
                vec!["eip155:338".to_owned()],
                vec!["chainChanged".to_owned(), "accountsChanged".to_owned()],
            ),
            client_meta: Metadata {
                description: "Defi WalletConnect v2 example.".into(),
                url: "http://localhost:8080/".parse().expect("url"),
                icons: vec![],
                name: "Defi WalletConnect Web3 Example".into(),
            },
            callback_sender: None,
        }
    }
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
    pub fn with_sender(self, address: impl Into<Address>) -> Self {
        WCMiddleware(self.0.with_sender(address))
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

/// make string even length by padding with 0
/// s : string to pad
fn pad_zero(s: String) -> String {
    if s.len() % 2 != 0 {
        format!("0{s}")
    } else {
        s
    }
}

/// add 0x prefix if not present
/// s: string to append 0x to
fn append_hex(s: String) -> String {
    if s.starts_with("0x") {
        s
    } else {
        format!("0x{s}")
    }
}

// tx_bytes is 32 bytes, for defi-wallet , it's txhash
fn make_defiwallet_signature(tx_bytes: &[u8]) -> Option<Signature> {
    // print hex of tx_bytes
    if tx_bytes.len() == 32 {
        let r = U256::from_big_endian(tx_bytes);
        let s = U256::zero();
        let v = 0;
        // r: 32 bytes, s: 32bytes, v: 1 bytes
        let sig = Signature { r, s, v };
        Some(sig)
    } else {
        None
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
            // need for defi wallet, otherwise user rejection error
            tx_obj.insert("data", "0x".to_string());
        }
        if let Some(gas) = tx.gas() {
            // gas not working for webwallet
            tx_obj.insert("gasLimit", append_hex(pad_zero(format!("{gas:x}"))));
        }

        if let Some(gas_price) = tx.gas_price() {
            tx_obj.insert("gasPrice", append_hex(pad_zero(format!("{gas_price:x}"))));
        }
        if let Some(value) = tx.value() {
            tx_obj.insert("value", append_hex(pad_zero(format!("{value:x}"))));
        }
        if let Some(nonce) = tx.nonce() {
            tx_obj.insert("nonce", append_hex(pad_zero(format!("{nonce:x}"))));
        }
        if let Some(c) = tx.chain_id() {
            tx_obj.insert("chainId", append_hex(pad_zero(format!("{c:x}"))));
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

        if tx_rlp.as_raw().len() == 32 {
            // It's not RLP-encoded
            return make_defiwallet_signature(tx_rlp.as_raw()).ok_or_else(|| {
                WCError::ClientError(ClientError::Eyre(eyre!(
                    "failed to decode defiwallet tx-hash signature"
                )))
            });
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
    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        _block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let tx: TypedTransaction = tx.into();

        let mut tx_obj = HashMap::new();
        if let Some(from) = tx.from() {
            let addr = *from;
            tx_obj.insert("from", format!("{addr:?}"));
        }
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
            // need for defi wallet, otherwise user rejection error
            tx_obj.insert("data", "0x".to_string());
        }
        if let Some(gas) = tx.gas() {
            tx_obj.insert("gasLimit", append_hex(pad_zero(format!("{gas:x}"))));
        }
        if let Some(gas_price) = tx.gas_price() {
            tx_obj.insert("gasPrice", append_hex(pad_zero(format!("{gas_price:x}"))));
        }
        if let Some(value) = tx.value() {
            tx_obj.insert("value", append_hex(pad_zero(format!("{value:x}"))));
        }
        if let Some(nonce) = tx.nonce() {
            tx_obj.insert("nonce", append_hex(pad_zero(format!("{nonce:x}"))));
        }

        let tx_hash = self
            .0
            .request("eth_sendTransaction", vec![tx_obj])
            .await
            .map_err(|e| WCError::ClientError(ClientError::Eyre(eyre!(e))))?;

        Ok(PendingTransaction::new(tx_hash, self.provider()))
    }
}
