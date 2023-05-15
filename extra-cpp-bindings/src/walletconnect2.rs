use crate::ffi::WalletConnect2Eip155Accounts;
use crate::ffi::WalletConnect2EnsureSessionResult;
use crate::ffi::WalletConnectAddress;
use anyhow::{anyhow, Result};
use defi_wallet_connect::v2::Namespaces;
use defi_wallet_connect::v2::{Client, ClientOptions, SessionInfo};
pub struct Walletconnect2Client {
    pub client: Option<defi_wallet_connect::v2::Client>,
    pub rt: tokio::runtime::Runtime, // need to use the same runtime, otherwise c++ side crash
    pub tx: tokio::sync::mpsc::UnboundedSender<String>, // sender
    pub rx: tokio::sync::mpsc::UnboundedReceiver<String>, // receiver
}

pub async fn restore_client(
    contents: String,
    callback_sender: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<Client> {
    if contents.is_empty() {
        anyhow::bail!("session info is empty");
    }

    let session_info: SessionInfo = serde_json::from_str(&contents)?;
    let client = Client::restore(session_info, callback_sender)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(client)
}

pub async fn save_client(client: &Client) -> Result<String> {
    let session = client.get_session_info().await;
    let session_info = serde_json::to_string(&session)?;
    Ok(session_info)
}

pub fn walletconnect_save_client(
    rt: &mut tokio::runtime::Runtime,
    client: &Client,
) -> Result<String> {
    let res = rt.block_on(save_client(client))?;
    Ok(res)
}

pub async fn new_client(opts: ClientOptions) -> Result<Client> {
    let client = Client::new(opts).await?;
    Ok(client)
}

impl Walletconnect2Client {
    /// save session to string which can be written to file
    pub fn save_client(&mut self) -> Result<String> {
        if let Some(client) = self.client.as_ref() {
            let result = walletconnect_save_client(&mut self.rt, client)?;
            Ok(result)
        } else {
            anyhow::bail!("no client");
        }
    }

    pub fn get_connection_string(self: &mut Walletconnect2Client) -> Result<String> {
        self.client.as_mut().map_or_else(
            || Err(anyhow!("no client")),
            |client| {
                let result = self.rt.block_on(client.get_connection_string());
                Ok(result)
            },
        )
    }
    pub fn sign_personal_blocking(
        &mut self,
        message: String,
        useraddress: [u8; 20],
    ) -> Result<Vec<u8>> {
        let address = ethers::types::Address::from_slice(&useraddress);
        self.client.as_mut().map_or_else(
            || Err(anyhow!("no client")),
            |client| {
                let result = self
                    .rt
                    .block_on(client.personal_sign(&message, &address))
                    .map_err(|e| anyhow!("ensure_session error {}", e.to_string()))?;
                Ok(result.to_vec())
            },
        )
    }

    pub fn ping_blocking(&mut self, waitmillis: u64) -> Result<String> {
        if let Some(client) = self.client.as_mut() {
            self.rt.block_on(async {
                tokio::time::timeout(
                    std::time::Duration::from_millis(waitmillis),
                    client.send_ping(),
                )
                .await
                .map_err(|_| anyhow!("send_ping timed out"))?
                .map_err(|e| anyhow!("ensure_session error {}", e.to_string()))
                .and_then(|result| serde_json::to_string(&result).map_err(|e| e.into()))
            })
        } else {
            Err(anyhow!("no client"))
        }
    }

    pub fn poll_events_blocking(&mut self, waitmillis: u64) -> Result<String> {
        let rt = &self.rt;

        let res = rt.block_on(async {
            tokio::time::timeout(std::time::Duration::from_millis(waitmillis), self.rx.recv())
                .await
                .map_err(anyhow::Error::new)
                .and_then(|res| res.map_or(Ok("".to_string()), Ok))
        })?;

        Ok(res)
    }

    pub fn ensure_session_blocking(
        &mut self,
        waitmillis: u64,
    ) -> Result<crate::ffi::WalletConnect2EnsureSessionResult> {
        let mut ret: WalletConnect2EnsureSessionResult =
            crate::ffi::WalletConnect2EnsureSessionResult::default();

        self.client.as_mut().map_or_else(
            || Err(anyhow!("no client")),
            |client| {
                let result: Namespaces = self.rt.block_on(async {
                    tokio::time::timeout(
                        std::time::Duration::from_millis(waitmillis),
                        client.ensure_session(),
                    )
                    .await
                    .map_err(|_| anyhow!("ensure_session timed out"))?
                    .map_err(|e| anyhow!("ensure_session error {}", e.to_string()))
                })?;

                // convert Names into ret
                let src = result.eip155;
                ret.eip155.accounts = src
                    .accounts
                    .iter()
                    .map(|account| WalletConnect2Eip155Accounts {
                        address: WalletConnectAddress {
                            address: account.address.into(),
                        },
                        chain_id: account.chain_id,
                    })
                    .collect();
                ret.eip155.methods = src.methods;
                ret.eip155.events = src.events;

                Ok(ret)
            },
        )
    }
}
