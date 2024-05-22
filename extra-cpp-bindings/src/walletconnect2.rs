use crate::ffi::WalletConnect2Eip155Accounts;
use crate::ffi::WalletConnect2EnsureSessionResult;
use crate::ffi::WalletConnectAddress;
use crate::ffi::WalletConnectTxCommon;
use anyhow::{anyhow, Result};
use defi_wallet_connect::v2::Namespaces;
use defi_wallet_connect::v2::{Client, ClientOptions, SessionInfo};
use qrcodegen::{QrCode, QrCodeEcc};

use defi_wallet_connect::v2::WCMiddleware;

use ethers::core::types::transaction::eip2718::TypedTransaction;

use ethers::prelude::{Address, Eip1559TransactionRequest, NameOrAddress, U256};
use ethers::prelude::{Middleware, Signature, TxHash};
use ethers::types::H160;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub struct Walletconnect2Client {
    pub client: Option<defi_wallet_connect::v2::Client>,
    pub rt: tokio::runtime::Runtime, // need to use the same runtime, otherwise c++ side crash
    pub tx: tokio::sync::mpsc::UnboundedSender<String>, // sender
    pub rx: tokio::sync::mpsc::UnboundedReceiver<String>, // receiver
}

#[derive(Serialize, Deserialize)]
enum ContractAction {
    ContractApproval(defi_wallet_core_common::ContractApproval),
    ContractTransfer(defi_wallet_core_common::ContractTransfer),
}

/// sign eip-155 transaction
/// client: client for walletconnect
/// tx: transaction info
/// address: address of the signer
/// returns: Signature
async fn sign_typed_tx(
    client: Client,
    tx: &TypedTransaction,
    address: Address,
) -> Result<Signature> {
    let middleware = WCMiddleware::new(client);
    let signature = middleware.sign_transaction(tx, address).await?;
    Ok(signature)
}

/// send eip-155 transaction
/// client: client for walletconnect
/// tx: transaction info
/// address: address of the signer
/// returns: TxHash
async fn send_typed_tx(client: Client, tx: TypedTransaction, address: Address) -> Result<TxHash> {
    let middleware = WCMiddleware::new(client).with_sender(address);
    let receipt = middleware.send_transaction(tx, None).await?.tx_hash();
    Ok(receipt)
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

    fn print_qr(qr: &QrCode) {
        let border: i32 = 1;
        for y in -border..qr.size() + border {
            for x in -border..qr.size() + border {
                let c = if qr.get_module(x, y) {
                    "\x1b[40m  \x1b[0m"
                } else {
                    "\x1b[47m  \x1b[0m"
                };
                print!("{c}");
            }
            println!();
        }
        println!();
    }

    /// print uri(qrcode) for debugging
    pub fn print_uri(&mut self) -> Result<String> {
        if let Some(client) = self.client.as_ref() {
            let result = self.rt.block_on(client.get_session_info());
            let uristring = result.uri();
            if let Ok(qr) = QrCode::encode_text(&uristring, QrCodeEcc::Medium) {
                Self::print_qr(&qr);
            }
            Ok(uristring)
        } else {
            anyhow::bail!("no client");
        }
    }

    // signature: 65 bytes (r:32, s:32,v:1)
    pub fn verify_personal_blocking(
        &mut self,
        message: String,
        signature_bytes: Vec<u8>,
        user_address: [u8; 20],
    ) -> Result<bool> {
        let address = ethers::types::Address::from_slice(&user_address);
        let signature = Signature::try_from(signature_bytes.as_slice())
            .map_err(|e| anyhow!("Invalid signature: {}", e))?;

        Ok(signature.verify(message, address).is_ok())
    }

    // signature
    // r: 32 bytes
    // s: 32 bytees
    // v: 1 byte
    // total 65 bytes
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

    /// build cronos(eth) eip155 transaction
    pub fn sign_eip155_transaction_blocking(
        &mut self,
        userinfo: &crate::ffi::WalletConnectTxEip155,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if self.client.is_none() {
            anyhow::bail!("no client");
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("get walllet-connect client error"))?;
        let signeraddress = Address::from_slice(&address);

        let mut tx = Eip1559TransactionRequest::new();

        if !userinfo.to.is_empty() {
            tx = tx.to(NameOrAddress::Address(Address::from_str(&userinfo.to)?));
        }
        if !userinfo.data.is_empty() {
            tx = tx.data(userinfo.data.as_slice().to_vec());
        }
        if !userinfo.common.gas_limit.is_empty() {
            tx = tx.gas(U256::from_dec_str(&userinfo.common.gas_limit)?);
        }
        if !userinfo.common.gas_price.is_empty() {
            tx = tx
                .max_priority_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?)
                .max_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?);
        }
        if !userinfo.common.nonce.is_empty() {
            tx = tx.nonce(U256::from_dec_str(&userinfo.common.nonce)?);
        }
        if userinfo.common.chainid != 0 {
            tx = tx.chain_id(userinfo.common.chainid);
        }
        if !userinfo.value.is_empty() {
            tx = tx.value(U256::from_dec_str(&userinfo.value)?);
        }
        let newclient = client.clone();
        let typedtx = TypedTransaction::Eip1559(tx);

        let sig = self
            .rt
            .block_on(sign_typed_tx(newclient, &typedtx, signeraddress))
            .map_err(|e| anyhow!("sign_typed_transaction error {}", e.to_string()))?;

        let signed_tx = &typedtx.rlp_signed(&sig);
        Ok(signed_tx.to_vec())
    }

    /// send cronos(eth) eip155 transaction
    pub fn send_eip155_transaction_blocking(
        &mut self,
        userinfo: &crate::ffi::WalletConnectTxEip155,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if self.client.is_none() {
            anyhow::bail!("no client");
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("get walllet-connect client error"))?;
        let signeraddress = Address::from_slice(&address);

        let mut tx = Eip1559TransactionRequest::new();

        if !userinfo.from.is_empty() {
            // from address is necessary for wc.20 , metamask
            tx = tx.from(Address::from_str(&userinfo.from)?);
        }
        if !userinfo.to.is_empty() {
            tx = tx.to(NameOrAddress::Address(Address::from_str(&userinfo.to)?));
        }
        if !userinfo.data.is_empty() {
            tx = tx.data(userinfo.data.as_slice().to_vec());
        } else {
            // for defiwallet
            tx = tx.data(vec![]);
        }

        if !userinfo.common.gas_limit.is_empty() {
            tx = tx.gas(U256::from_dec_str(&userinfo.common.gas_limit)?);
        }
        if !userinfo.common.gas_price.is_empty() {
            tx = tx
                .max_priority_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?)
                .max_fee_per_gas(U256::from_dec_str(&userinfo.common.gas_price)?);
        }
        if !userinfo.common.nonce.is_empty() {
            tx = tx.nonce(U256::from_dec_str(&userinfo.common.nonce)?);
        }
        if userinfo.common.chainid != 0 {
            tx = tx.chain_id(userinfo.common.chainid);
        }
        if !userinfo.value.is_empty() {
            tx = tx.value(U256::from_dec_str(&userinfo.value)?);
        }

        let newclient = client.clone();
        let typedtx = TypedTransaction::Eip1559(tx);

        let tx_bytes = self
            .rt
            .block_on(send_typed_tx(newclient, typedtx, signeraddress))
            .map_err(|e| anyhow!("send_typed_transaction error {}", e.to_string()))?;

        Ok(tx_bytes.0.to_vec())
    }

    fn get_signed_tx_raw_bytes(
        &self,
        newclient: Client,
        signeraddress: H160,
        typedtx: &mut TypedTransaction,
        common: &WalletConnectTxCommon,
    ) -> Result<Vec<u8>> {
        let mynonce = U256::from_dec_str(&common.nonce)?;
        if !mynonce.is_zero() {
            typedtx.set_nonce(mynonce);
        }
        typedtx.set_from(signeraddress);
        if !common.chainid == 0 {
            typedtx.set_chain_id(common.chainid);
        }
        if !common.gas_limit.is_empty() {
            typedtx.set_gas(U256::from_dec_str(&common.gas_limit)?);
        }
        if !common.gas_price.is_empty() {
            typedtx.set_gas_price(U256::from_dec_str(&common.gas_price)?);
        }

        let sig = self
            .rt
            .block_on(sign_typed_tx(newclient, typedtx, signeraddress))
            .map_err(|e| anyhow!("sign_typed_transaction error {}", e.to_string()))?;

        let signed_tx = &typedtx.rlp_signed(&sig);
        Ok(signed_tx.to_vec())
    }

    fn get_sent_tx_raw_bytes(
        &self,
        newclient: Client,
        signeraddress: H160,
        typedtx: &mut TypedTransaction,
        common: &WalletConnectTxCommon,
    ) -> Result<Vec<u8>> {
        let mynonce = U256::from_dec_str(&common.nonce)?;
        if !mynonce.is_zero() {
            typedtx.set_nonce(mynonce);
        }
        typedtx.set_from(signeraddress);
        if !common.chainid == 0 {
            typedtx.set_chain_id(common.chainid);
        }
        if !common.gas_limit.is_empty() {
            typedtx.set_gas(U256::from_dec_str(&common.gas_limit)?);
        }
        if !common.gas_price.is_empty() {
            typedtx.set_gas_price(U256::from_dec_str(&common.gas_price)?);
        }

        let tx_bytes = self
            .rt
            .block_on(send_typed_tx(newclient, typedtx.clone(), signeraddress))
            .map_err(|e| anyhow!("send_typed_transaction error {}", e.to_string()))?;

        Ok(tx_bytes.0.to_vec())
    }

    pub fn sign_transaction(
        &mut self,
        eip1559_transaction_request: String,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if self.client.is_none() {
            anyhow::bail!("no client");
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("get walllet-connect client error"))?;
        let signeraddress = Address::from_slice(&address);

        // parse json string transaction_info to TransactionRequest
        let tx: Eip1559TransactionRequest = serde_json::from_str(&eip1559_transaction_request)?;
        let typedtx = TypedTransaction::Eip1559(tx);

        let newclient = client.clone();
        let sig = self
            .rt
            .block_on(sign_typed_tx(newclient, &typedtx, signeraddress))
            .map_err(|e| anyhow!("sign_typed_transaction error {}", e.to_string()))?;

        let signed_tx = &typedtx.rlp_signed(&sig);
        Ok(signed_tx.to_vec())
    }

    pub fn send_transaction(
        &mut self,
        eip1559_transaction_request: String,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if self.client.is_none() {
            anyhow::bail!("no client");
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("get walllet-connect client error"))?;
        let signeraddress = Address::from_slice(&address);

        // parse json string transaction_info to TransactionRequest
        let tx: Eip1559TransactionRequest = serde_json::from_str(&eip1559_transaction_request)?;
        let typedtx = TypedTransaction::Eip1559(tx);

        let newclient = client.clone();
        let tx_bytes = self
            .rt
            .block_on(send_typed_tx(newclient, typedtx, signeraddress))
            .map_err(|e| anyhow!("send_typed_transaction error {}", e.to_string()))?;

        Ok(tx_bytes.0.to_vec())
    }

    pub fn sign_contract_transaction(
        &mut self,
        contract_action: String,
        common: &WalletConnectTxCommon,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if self.client.is_none() {
            anyhow::bail!("no client");
        }
        let signeraddress = Address::from_slice(&address);
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("get walllet-connect client error"))?;
        let newclient = client.clone();

        let action: ContractAction = serde_json::from_str(&contract_action)?;
        // parse json string transaction_info to TransactionRequest
        // let tx: ContractTransfer = serde_json::from_str(&contract_transaction_info)?;

        let mut typedtx = match action {
            ContractAction::ContractApproval(approval) => {
                self.rt
                    .block_on(defi_wallet_core_common::construct_contract_approval_tx(
                        approval,
                        defi_wallet_core_common::EthNetwork::Custom {
                            chain_id: common.chainid,
                            legacy: false,
                        },
                        common.web3api_url.as_str(),
                    ))?
            }
            ContractAction::ContractTransfer(transfer) => {
                self.rt
                    .block_on(defi_wallet_core_common::construct_contract_transfer_tx(
                        transfer,
                        defi_wallet_core_common::EthNetwork::Custom {
                            chain_id: common.chainid,
                            legacy: false,
                        },
                        // TODO unnessary for walletconnect
                        common.web3api_url.as_str(),
                    ))?
            }
        };

        let tx = self.get_signed_tx_raw_bytes(newclient, signeraddress, &mut typedtx, common)?;
        Ok(tx.to_vec())
    }

    pub fn send_contract_transaction(
        &mut self,
        contract_action: String,
        common: &WalletConnectTxCommon,
        address: [u8; 20],
    ) -> Result<Vec<u8>> {
        if self.client.is_none() {
            anyhow::bail!("no client");
        }
        let signeraddress = Address::from_slice(&address);
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("get walllet-connect client error"))?;
        let newclient = client.clone();

        let action: ContractAction = serde_json::from_str(&contract_action)?;
        // parse json string transaction_info to TransactionRequest
        // let tx: ContractTransfer = serde_json::from_str(&contract_transaction_info)?;

        let mut typedtx = match action {
            ContractAction::ContractApproval(approval) => {
                self.rt
                    .block_on(defi_wallet_core_common::construct_contract_approval_tx(
                        approval,
                        defi_wallet_core_common::EthNetwork::Custom {
                            chain_id: common.chainid,
                            legacy: false,
                        },
                        common.web3api_url.as_str(),
                    ))?
            }
            ContractAction::ContractTransfer(transfer) => {
                self.rt
                    .block_on(defi_wallet_core_common::construct_contract_transfer_tx(
                        transfer,
                        defi_wallet_core_common::EthNetwork::Custom {
                            chain_id: common.chainid,
                            legacy: false,
                        },
                        // TODO unnessary for walletconnect
                        common.web3api_url.as_str(),
                    ))?
            }
        };

        let tx = self.get_sent_tx_raw_bytes(newclient, signeraddress, &mut typedtx, common)?;
        Ok(tx.to_vec())
    }
}
