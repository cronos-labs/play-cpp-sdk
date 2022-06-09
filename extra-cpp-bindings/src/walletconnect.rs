use crate::ffi::WalletConnectCallback;
use anyhow::{anyhow, Result};
use defi_wallet_connect::session::SessionInfo;
use defi_wallet_connect::ClientChannelMessageType;
use defi_wallet_connect::{Client, Metadata, WCMiddleware};
use ethers_core::types::transaction::eip2718::TypedTransaction;
use url::Url;

use crate::ffi::WalletConnectSessionInfo;
use cxx::UniquePtr;
use ethers::prelude::{Address, NameOrAddress, TransactionRequest, U256};
use ethers::prelude::{Middleware, Signature};
use std::str::FromStr;

pub struct WalletconnectClient {
    pub client: Option<defi_wallet_connect::Client>,
    pub rt: tokio::runtime::Runtime, // need to use the same runtime, otherwise c++ side crash
}

async fn restore_client(contents: String) -> Result<Client> {
    if contents.is_empty() {
        anyhow::bail!("session info is empty");
    }

    let session: SessionInfo = serde_json::from_str(&contents)?;
    let client = Client::restore(session).await?;
    println!("restored client");
    Ok(client)
}

async fn save_client(client: &Client) -> Result<String> {
    let session = client.get_session_info().await?;
    let session_info = serde_json::to_string(&session)?;
    Ok(session_info)
}

// description: "Defi WalletConnect example."
// url: "http://localhost:8080/"
// name: "Defi WalletConnect Web3 Example"
async fn new_client(
    description: String,
    url: String,
    icon_urls: &[String],
    name: String,
) -> Result<Client> {
    // convert string array to url array
    let mut icons: Vec<Url> = Vec::new();
    for icon in icon_urls {
        icons.push(icon.parse().expect("url"));
    }
    let client = Client::new(Metadata {
        description,
        url: url.parse().expect("url"),
        icons,
        name,
    })
    .await?;
    Ok(client)
}

pub fn walletconnect_restore_client(
    rt: &mut tokio::runtime::Runtime,
    session_info: String,
) -> Result<Client> {
    let res = rt.block_on(restore_client(session_info))?;
    Ok(res)
}

pub fn walletconnect_save_client(
    rt: &mut tokio::runtime::Runtime,
    client: &Client,
) -> Result<String> {
    let res = rt.block_on(save_client(client))?;
    Ok(res)
}

// description: "Defi WalletConnect example."
// url: "http://localhost:8080/".parse().expect("url")
// icons: vec![]
// name: "Defi WalletConnect Web3 Example",
pub fn walletconnect_new_client(
    rt: &mut tokio::runtime::Runtime,
    description: String,
    url: String,
    icon_urls: &[String],
    name: String,
) -> Result<Client> {
    let res = rt.block_on(new_client(description, url, icon_urls, name))?;
    Ok(res)
}

fn convert_session_info(sessioninfo: &SessionInfo) -> Result<UniquePtr<WalletConnectSessionInfo>> {
    let mut cppsessioninfo = crate::ffi::new_walletconnect_sessioninfo();
    cppsessioninfo
        .pin_mut()
        .set_connected(sessioninfo.connected);

    let chain_id = match sessioninfo.chain_id {
        Some(id) => id.to_string(),
        None => "".to_string(),
    };
    cppsessioninfo.pin_mut().set_chainid(chain_id);

    let accountstrings = sessioninfo
        .accounts
        .iter()
        .map(|account| format!("{account:#x}"))
        .collect();
    cppsessioninfo.pin_mut().set_accounts(accountstrings);

    cppsessioninfo
        .pin_mut()
        .set_bridge(sessioninfo.bridge.to_string());

    cppsessioninfo
        .pin_mut()
        .set_key(format!("0x{}", hex::encode(sessioninfo.key.as_ref())));

    cppsessioninfo
        .pin_mut()
        .set_clientid(sessioninfo.client_id.to_string());
    cppsessioninfo
        .pin_mut()
        .set_clientmeta(serde_json::to_string(&sessioninfo.client_meta)?);

    cppsessioninfo
        .pin_mut()
        .set_peerid(match sessioninfo.peer_id.as_ref() {
            Some(id) => id.to_string(),
            None => "".to_string(),
        });

    cppsessioninfo
        .pin_mut()
        .set_peermeta(match sessioninfo.peer_meta.as_ref() {
            Some(meta) => serde_json::to_string(&meta)?,
            None => "".to_string(),
        });

    cppsessioninfo
        .pin_mut()
        .set_handshaketopic(sessioninfo.handshake_topic.to_string());

    Ok(cppsessioninfo)
}

async fn do_walletconnect_set_callback(
    client: &mut Client,
    cppcallback: cxx::UniquePtr<WalletConnectCallback>,
) -> Result<()> {
    client.run_callback(Box::new(move |message| {
        match message.state {
            ClientChannelMessageType::Connected => {
                println!("Connected");
                if let Some(info) = message.session {
                    println!("session info: {:?}", info);

                    if let Ok(sessioninfo) = convert_session_info(&info) {
                        cppcallback.onConnected(sessioninfo);
                    } else {
                        println!("invalid session info");
                    }
                }
            }
            ClientChannelMessageType::Disconnected => {
                println!("Disconnected");
                if let Some(info) = message.session {
                    println!("session info: {:?}", info);
                    if let Ok(sessioninfo) = convert_session_info(&info) {
                        cppcallback.onDisconnected(sessioninfo);
                    } else {
                        println!("invalid session info");
                    }
                }
            }
            ClientChannelMessageType::Connecting => {
                println!("Connecting");
                if let Some(info) = &message.session {
                    println!("session info: {:?}", info);
                    if let Ok(sessioninfo) = convert_session_info(info) {
                        cppcallback.onConnecting(sessioninfo);
                    } else {
                        println!("invalid session info");
                    }
                }
            }
            ClientChannelMessageType::Updated => {
                println!("Updated");
                if let Some(info) = &message.session {
                    println!("session info: {:?}", info);
                    if let Ok(sessioninfo) = convert_session_info(info) {
                        cppcallback.onUpdated(sessioninfo);
                    } else {
                        println!("invalid session info");
                    }
                }
            }
        } // end of match
    }));

    Ok(())
}

async fn sign_typed_tx(
    client: Client,
    tx: &TypedTransaction,
    address: Address,
) -> Result<Signature> {
    let middleware = WCMiddleware::new(client);
    let signature = middleware.sign_transaction(tx, address).await?;
    Ok(signature)
}

impl WalletconnectClient {
    /// sign a message
    pub fn sign_personal_blocking(
        &mut self,
        message: String,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if let Some(client) = self.client.as_mut() {
            let signeraddress = Address::from_slice(&address);

            let result = self
                .rt
                .block_on(client.personal_sign(&message, &signeraddress))
                .map_err(|e| anyhow!("sign_personal error {}", e.to_string()))?;

            Ok(result.to_vec())
        } else {
            anyhow::bail!("no client");
        }
    }

    /// sign cronos(eth) legacy transaction
    pub fn sign_legacy_transaction_blocking(
        &mut self,
        userinfo: &crate::ffi::WalletConnectTxLegacy,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if self.client.is_none() {
            anyhow::bail!("no client");
        }

        let client = self.client.take().expect("get client");
        let signeraddress = Address::from_slice(&address);

        let tx = TransactionRequest::new()
            .to(NameOrAddress::Address(Address::from_str(&userinfo.to)?))
            .data(userinfo.data.as_slice().to_vec())
            .gas(U256::from_dec_str(&userinfo.gas)?)
            .gas_price(U256::from_dec_str(&userinfo.gas_price)?)
            .nonce(U256::from_dec_str(&userinfo.nonce)?)
            .value(U256::from_dec_str(&userinfo.value)?);
        let sessioninfo = walletconnect_save_client(&mut self.rt, &client)?;
        let newclient = walletconnect_restore_client(&mut self.rt, sessioninfo)?;
        let typedtx = TypedTransaction::Legacy(tx);

        let result = self
            .rt
            .block_on(sign_typed_tx(newclient, &typedtx, signeraddress))
            .map_err(|e| anyhow!("sign_typed_transaction error {}", e.to_string()))?;

        Ok(result.to_vec())
    }

    pub fn setup_callback(&mut self, usercallback: UniquePtr<WalletConnectCallback>) -> Result<()> {
        let cppcallback = usercallback;
        if let Some(client) = self.client.as_mut() {
            let result = self
                .rt
                .block_on(do_walletconnect_set_callback(client, cppcallback))?;

            Ok(result)
        } else {
            anyhow::bail!("no client");
        }
    }

    /// ensure session, if session does not exist, create a new session
    pub fn ensure_session_blocking(
        self: &mut WalletconnectClient,
    ) -> Result<crate::ffi::WalletConnectEnsureSessionResult> {
        let mut ret = crate::ffi::WalletConnectEnsureSessionResult {
            addresses: Vec::new(),
            chain_id: 0,
        };
        if let Some(client) = self.client.as_mut() {
            let result: (Vec<Address>, u64) = self
                .rt
                .block_on(client.ensure_session())
                .map_err(|e| anyhow!("ensure_session error {}", e.to_string()))?;

            ret.addresses = result
                .0
                .iter()
                .map(|x| crate::ffi::WalletConnectAddress { address: x.0 })
                .collect();
            ret.chain_id = result.1;

            Ok(ret)
        } else {
            anyhow::bail!("no client");
        }
    }

    /// get connection string for qrcode display
    pub fn get_connection_string(&mut self) -> Result<String> {
        if let Some(client) = self.client.as_mut() {
            let result = self
                .rt
                .block_on(client.get_connection_string())
                .map_err(|e| anyhow!("get_connection_string error {}", e.to_string()))?;

            Ok(result)
        } else {
            anyhow::bail!("no client");
        }
    }

    /// save session to string which can be written to file
    pub fn save_client(&mut self) -> Result<String> {
        if let Some(client) = self.client.as_ref() {
            let result = walletconnect_save_client(&mut self.rt, client)?;
            Ok(result)
        } else {
            anyhow::bail!("no client");
        }
    }

    /// print uri(qrcode) for debugging
    pub fn print_uri(&mut self) -> Result<String> {
        if let Some(client) = self.client.as_ref() {
            let result = self
                .rt
                .block_on(client.get_session_info())
                .map_err(|e| anyhow!("get_sesion_info error {}", e.to_string()))?;
            result.uri().print_qr_uri();
            Ok(result.uri().as_url().as_str().into())
        } else {
            anyhow::bail!("no client");
        }
    }
}
