mod error;
/// Crypto.com Pay basic support
mod pay;
/// Wallect Connect registry of wallets/apps support
mod wallectconnectregistry;
mod walletconnect;
mod walletconnect2;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use ethers::core::types::{BlockNumber, Chain};
use ethers::etherscan::{
    account::{
        ERC20TokenTransferEvent, ERC721TokenTransferEvent, NormalTransaction, TokenQueryOption,
    },
    Client,
};
use ffi::{
    CryptoComPaymentResponse, ImageUrl, Platform, QueryOption, RawTokenResult, RawTxDetail,
    TokenHolderDetail, WalletEntry,
};
use qrcodegen::QrCode;
use qrcodegen::QrCodeEcc;
use serde::{Deserialize, Serialize};
use walletconnect::WalletconnectClient;
use walletconnect2::Walletconnect2Client;

#[cxx::bridge(namespace = "com::crypto::game_sdk")]
mod ffi {
    #[derive(Debug, Default)]
    pub struct WalletConnectTransactionReceiptRaw {
        pub transaction_hash: Vec<u8>,
        pub transaction_index: String,
        pub block_hash: Vec<u8>,
        pub block_number: String,
        pub cumulative_gas_used: String,
        pub gas_used: String,
        pub contract_address: String,
        pub logs: Vec<String>,
        /// Status: either 1 (success) or 0 (failure)
        pub status: String,
        pub root: Vec<u8>,
        pub logs_bloom: Vec<u8>,
        pub transaction_type: String,
        pub effective_gas_price: String,
    }

    unsafe extern "C++" {
        include!("extra-cpp-bindings/include/walletconnectcallback.h");

        type WalletConnectCallback;

        fn onConnected(&self, sessioninfo: &WalletConnectSessionInfo);
        fn onDisconnected(&self, sessioninfo: &WalletConnectSessionInfo);
        fn onConnecting(&self, sessioninfo: &WalletConnectSessionInfo);
        fn onUpdated(&self, sessioninfo: &WalletConnectSessionInfo);
    }

    unsafe extern "C++" {
        include!("extra-cpp-bindings/include/walletconnectcallback.h");

        type WalletConnectSessionInfo;

        fn new_walletconnect_sessioninfo() -> UniquePtr<WalletConnectSessionInfo>;
        fn set_chainid(self: Pin<&mut WalletConnectSessionInfo>, chainid: String);
        fn set_connected(self: Pin<&mut WalletConnectSessionInfo>, connected: bool);
        fn set_accounts(self: Pin<&mut WalletConnectSessionInfo>, accounts: Vec<String>);
        fn set_bridge(self: Pin<&mut WalletConnectSessionInfo>, bridge: String);
        fn set_key(self: Pin<&mut WalletConnectSessionInfo>, key: String);
        fn set_clientid(self: Pin<&mut WalletConnectSessionInfo>, clientid: String);
        fn set_clientmeta(self: Pin<&mut WalletConnectSessionInfo>, clientmeta: String);
        fn set_peerid(self: Pin<&mut WalletConnectSessionInfo>, peerid: String);
        fn set_peermeta(self: Pin<&mut WalletConnectSessionInfo>, peermeta: String);
        fn set_handshaketopic(self: Pin<&mut WalletConnectSessionInfo>, handshaketopic: String);
    }

    /// The wallet registry entry
    pub struct WalletEntry {
        /// wallet id
        pub id: String,
        /// its name
        pub name: String,
        /// icon URLs
        pub image_url: ImageUrl,
        /// mobile native link, empty if none
        pub mobile_native_link: String,
        /// mobile universal link, empty if none
        pub mobile_universal_link: String,
        /// desktop native link, empty if none
        pub desktop_native_link: String,
        /// desktop universal link, empty if none
        pub desktop_universal_link: String,
    }

    /// The target platform
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub enum Platform {
        Mobile,
        Desktop,
    }

    /// The icon URLs
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ImageUrl {
        /// small
        pub sm: String,
        /// medium
        pub md: String,
        /// large
        pub lg: String,
    }

    #[derive(Debug, Default)]
    pub struct WalletQrcode {
        pub qrcode: String,
        pub image: Vec<u8>, /* size* size*/
        pub size: u32,
    }
    #[derive(Debug, Default)]
    pub struct WalletConnectTxCommon {
        pub gas_limit: String,   // decimal string, "1"
        pub gas_price: String,   // decimal string
        pub nonce: String,       // decimal string
        pub chainid: u64,        // integer u64
        pub web3api_url: String, // string
    }

    /// wallet connect cronos(eth) eip155-tx signing info
    #[derive(Debug, Default)]
    pub struct WalletConnectTxEip155 {
        pub to: String,    // hexstring, "0x..."
        pub value: String, // decimal string, in wei units
        pub data: Vec<u8>, // data, as bytes

        pub common: WalletConnectTxCommon,
    }

    /// cronos address info
    #[derive(Debug, Default)]
    pub struct WalletConnectAddress {
        pub address: [u8; 20], // address, as bytes, 20 bytes
    }

    /// walletconnect ensure-session result
    #[derive(Debug, Default)]
    pub struct WalletConnectEnsureSessionResult {
        pub addresses: Vec<WalletConnectAddress>,
        pub chain_id: u64,
    }

    #[derive(Debug, Default)]
    pub struct WalletConnect2Eip155Accounts {
        pub address: WalletConnectAddress,
        pub chain_id: u64,
    }
    #[derive(Debug, Default)]
    pub struct WalletConnect2Eip155 {
        pub(crate) accounts: Vec<WalletConnect2Eip155Accounts>,
        pub(crate) methods: Vec<String>,
        pub(crate) events: Vec<String>,
    }

    #[derive(Debug, Default)]
    pub struct WalletConnect2EnsureSessionResult {
        pub eip155: WalletConnect2Eip155,
    }

    /// the subset of payment object from https://pay-docs.crypto.com
    #[derive(Debug)]
    pub struct CryptoComPaymentResponse {
        /// uuid of the payment object
        pub id: String,
        /// the base64 payload to be displayed as QR code that
        /// can be scanned by the main app
        pub main_app_qr_code: String,
        /// if the on-chain payment is desired, this will
        /// have the cryptocurrency address that can be displayed
        /// as a QR code or put in a tx to be signed via WalletConnect
        pub onchain_deposit_address: String,
        /// the amount in base denomination
        /// e.g. for USD, it's cents (1 USD == 100 cents)
        pub base_amount: String,
        /// the 3-letter currency code
        pub currency: String,
        /// expiration time in unix timestamp (10 minutes)
        pub expiration: u64,
        /// the status of the payment
        pub status: String,
    }

    /// Raw transaction details (extracted from Cronoscan/Etherscan or BlockScout API)
    #[derive(Debug, PartialEq, Eq)]
    pub struct RawTxDetail {
        /// Transaction hash
        pub hash: String,
        /// the hexadecimal address of the receiver
        pub to_address: String,
        /// the hexadecimal address of the sender
        pub from_address: String,
        /// the value sent in decimal (in base tokens)
        pub value: String,
        /// block number when it happened
        pub block_no: u64,
        /// the time it happened
        pub timestamp: String,
        /// the address of the contract (if no contract, it's an empty string)
        pub contract_address: String,
    }

    /// Token ownership result detail from BlockScout API
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct RawTokenResult {
        /// how many tokens are owned by the address
        pub balance: String,
        /// the deployed contract address
        #[serde(rename = "contractAddress")]
        pub contract_address: String,
        /// the number of decimal places
        pub decimals: String,
        /// the token id
        #[serde(default)]
        pub id: String,
        /// the human-readable name of the token
        pub name: String,
        /// the ticker for the token
        pub symbol: String,
        /// the token type (ERC-20, ERC-721, ERC-1155)
        #[serde(rename = "type")]
        pub token_type: String,
    }

    /// Token holder detail from BlockScout API
    ///
    /// tokenid is not supported yet.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct TokenHolderDetail {
        /// the holder address
        pub address: String,
        /// balance of the target token
        pub value: String,
    }

    pub enum QueryOption {
        ByContract,
        ByAddressAndContract,
        ByAddress,
    }

    extern "Rust" {
        /// filter wallets by platform
        /// (`registry_local_path` can be empty string if it is not needed to store the `cached` registry result)
        pub fn filter_wallets(
            cached: bool,
            registry_local_path: String,
            platform: Platform,
        ) -> Result<Vec<WalletEntry>>;
        /// get all possible wallets
        /// (`registry_local_path` can be empty string if it is not needed to store the `cached` registry result)
        pub fn get_all_wallets(
            cached: bool,
            registry_local_path: String,
        ) -> Result<Vec<WalletEntry>>;
        /// check wallet by `id` for supported `platform` listing or not
        /// Check wallet id at https://explorer.walletconnect.com/
        /// (`registry_local_path` can be empty string if it is not needed to store the `cached` registry result)
        pub fn check_wallet(
            cached: bool,
            registry_local_path: String,
            id: String,
            platform: Platform,
        ) -> Result<bool>;
        /// get a wallet by `id`
        /// Check wallet id at https://explorer.walletconnect.com/
        /// (`registry_local_path` can be empty string if it is not needed to store the `cached` registry result)
        pub fn get_wallet(
            cached: bool,
            registry_local_path: String,
            id: String,
        ) -> Result<WalletEntry>;
        pub fn generate_qrcode(qrcodestring: String) -> Result<WalletQrcode>;
        /// WallnetConnect API
        type WalletconnectClient;
        type Walletconnect2Client;
        /// restore walletconnect-session from string
        pub fn walletconnect_restore_client(
            session_info: String,
        ) -> Result<Box<WalletconnectClient>>;
        pub fn walletconnect2_restore_client(
            session_info: String,
        ) -> Result<Box<Walletconnect2Client>>;
        /// create walletconnect-session
        /// the chain id (if 0, retrived and decided by wallet, if > 0, decided by the client)
        pub fn walletconnect_new_client(
            description: String,
            url: String,
            icon_urls: Vec<String>,
            name: String,
            chain_id: u64,
        ) -> Result<Box<WalletconnectClient>>;
        pub fn walletconnect2_client_new(
            relayserver: String,
            project_id: String,
            required_namespaces: String,
            client_meta: String,
        ) -> Result<Box<Walletconnect2Client>>;

        /// setup callback
        pub fn setup_callback_blocking(
            self: &mut WalletconnectClient,
            usercallback: UniquePtr<WalletConnectCallback>,
        ) -> Result<()>;
        /// create or restore a session
        /// once session is created, it will be reused
        pub fn ensure_session_blocking(
            self: &mut WalletconnectClient,
        ) -> Result<WalletConnectEnsureSessionResult>;

        pub fn ensure_session_blocking(
            self: &mut Walletconnect2Client,
            waitmillis: u64,
        ) -> Result<WalletConnect2EnsureSessionResult>;

        pub fn poll_events_blocking(
            self: &mut Walletconnect2Client,
            waitmillis: u64,
        ) -> Result<String>;

        /// get connection string for qrcode
        pub fn get_connection_string(self: &mut WalletconnectClient) -> Result<String>;
        pub fn get_connection_string(self: &mut Walletconnect2Client) -> Result<String>;
        /// write session-info to string, which can be written to file
        pub fn save_client(self: &mut WalletconnectClient) -> Result<String>;
        pub fn save_client(self: &mut Walletconnect2Client) -> Result<String>;
        /// print qrcode in termal, for debugging
        pub fn print_uri(self: &mut WalletconnectClient) -> Result<String>;
        pub fn print_uri(self: &mut Walletconnect2Client) -> Result<String>;
        /// sign message
        pub fn sign_personal_blocking(
            self: &mut WalletconnectClient,
            message: String,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn sign_personal_blocking(
            self: &mut Walletconnect2Client,
            message: String,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn ping_blocking(self: &mut Walletconnect2Client, waitmillis: u64) -> Result<String>;

        /// build cronos(eth) eip155 transaction
        /// Supported Wallets: Trust Wallet, Crypto.com Desktop Defi Wallet
        pub fn sign_eip155_transaction_blocking(
            self: &mut WalletconnectClient,
            info: &WalletConnectTxEip155,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn sign_eip155_transaction_blocking(
            self: &mut Walletconnect2Client,
            info: &WalletConnectTxEip155,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;

        /// send cronos(eth) eip155 transaction
        /// Supported Wallets: Trust Wallet, MetaMask and Crypto.com Mobile Defi Wallet
        pub fn send_eip155_transaction_blocking(
            self: &mut WalletconnectClient,
            info: &WalletConnectTxEip155,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn send_eip155_transaction_blocking(
            self: &mut Walletconnect2Client,
            info: &WalletConnectTxEip155,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;

        /// eip1559_transaction_request: json string of Eip1559TransactionRequest
        /// return signed transaction bytes
        pub fn sign_transaction(
            self: &mut WalletconnectClient,
            eip1559_transaction_request: String,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn sign_transaction(
            self: &mut Walletconnect2Client,
            eip1559_transaction_request: String,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;

        /// eip1559_transaction_request: json string of Eip1559TransactionRequest
        /// return transaction hash bytes
        pub fn send_transaction(
            self: &mut WalletconnectClient,
            eip1559_transaction_request: String,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn send_transaction(
            self: &mut Walletconnect2Client,
            eip1559_transaction_request: String,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;

        /// sign a contract transaction
        /// contract_action is a json string of `ContractAction` type, for example:
        /// for example, transfer Erc20 token
        /// {
        ///     "ContractTransfer": {
        ///         "Erc20Transfer": {
        ///             "contract_address": "0xxxxx",
        ///             "to_address": "0xxxxx",
        ///             "amount": "1000000000000000000"
        ///         }
        ///     }
        /// }
        /// return signed transaction bytes
        pub fn sign_contract_transaction(
            self: &mut WalletconnectClient,
            contract_action: String,
            common: &WalletConnectTxCommon,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn sign_contract_transaction(
            self: &mut Walletconnect2Client,
            contract_action: String,
            common: &WalletConnectTxCommon,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;

        // send a contract transaction
        /// contract_action is a json string of `ContractAction` type
        /// for example, transfer Erc20 token
        /// {
        ///     "ContractTransfer": {
        ///         "Erc20Transfer": {
        ///             "contract_address": "0xxxxx",
        ///             "to_address": "0xxxxx",
        ///             "amount": "1000000000000000000"
        ///         }
        ///     }
        /// }
        // return transaction hash bytes
        pub fn send_contract_transaction(
            self: &mut WalletconnectClient,
            contract_action: String,
            common: &WalletConnectTxCommon,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;
        pub fn send_contract_transaction(
            self: &mut Walletconnect2Client,
            contract_action: String,
            common: &WalletConnectTxCommon,
            address: [u8; 20],
        ) -> Result<Vec<u8>>;

        /// returns the transactions of a given address.
        /// The API key can be obtained from https://cronoscan.com
        pub fn get_transaction_history_blocking(
            address: String,
            api_key: String,
        ) -> Result<Vec<RawTxDetail>>;
        /// returns the ERC20 transfers of a given address of a given contract.
        /// (address can be empty if option is ByContract)
        /// default option is by address
        /// The API key can be obtained from https://cronoscan.com
        pub fn get_erc20_transfer_history_blocking(
            address: String,
            contract_address: String,
            option: QueryOption,
            api_key: String,
        ) -> Result<Vec<RawTxDetail>>;
        /// returns the ERC721 transfers of a given address of a given contract.
        /// (address can be empty if option is ByContract)
        /// default option is by address
        /// The API key can be obtained from https://cronoscan.com
        pub fn get_erc721_transfer_history_blocking(
            address: String,
            contract_address: String,
            option: QueryOption,
            api_key: String,
        ) -> Result<Vec<RawTxDetail>>;
        /// given the BlockScout REST API base url and the account address (hexadecimal),
        /// it will return the list of all owned tokens
        /// (ref: https://cronos.org/explorer/testnet3/api-docs)
        pub fn get_tokens_blocking(
            blockscout_base_url: String,
            account_address: String,
        ) -> Result<Vec<RawTokenResult>>;
        /// given the BlockScout REST API base url and the account address (hexadecimal; required)
        /// and optional contract address (hexadecimal; optional -- it can be empty if the option is ByAddress),
        /// it will return all the token transfers (ERC20, ERC721... in the newer BlockScout
        /// releases, also ERC1155)
        /// (ref: https://cronos.org/explorer/testnet3/api-docs)
        /// NOTE: QueryOption::ByContract is not supported by BlockScout
        pub fn get_token_transfers_blocking(
            blockscout_base_url: String,
            address: String,
            contract_address: String,
            option: QueryOption,
        ) -> Result<Vec<RawTxDetail>>;
        /// given the BlockScout REST API base url and the contract address (hexadecimal),
        ///
        /// page: A nonnegative integer that represents the page number to be used for
        /// pagination. 'offset' must be provided in conjunction.
        ///
        /// offset: A nonnegative integer that represents the maximum number of records to
        /// return when paginating. 'page' must be provided in conjunction.
        ///
        /// it will return the list of owners and balances (sorting from largest to smallest),
        /// but no token ids.
        ///
        /// (ref: https://cronos.org/explorer/api-docs#token)
        ///
        /// ::TIPS:: Use another functions to get more token/owner details, e.g.
        /// `get_tokens_blocking` to get owned tokens by account_address
        pub fn get_token_holders(
            blockscout_base_url: String,
            contract_address: String,
            page: u64,
            offset: u64,
        ) -> Result<Vec<TokenHolderDetail>>;
        /// it creates the payment object
        /// https://pay-docs.crypto.com/#api-reference-resources-payments-create-a-payment
        /// This API can be called using either your Secret Key or Publishable Key.
        /// The amount should be given in base units (e.g. for USD, the base unit is cents 1 USD == 100 cents).
        pub fn create_payment(
            secret_or_publishable_api_key: String,
            base_unit_amount: String,
            currency: String,
            optional_args: &OptionalArguments,
        ) -> Result<CryptoComPaymentResponse>;
        /// it returns the payment object by id
        /// https://pay-docs.crypto.com/#api-reference-resources-payments-get-payment-by-id
        /// This API can be called using either your Secret Key or Publishable Key.
        pub fn get_payment(
            secret_or_publishable_api_key: String,
            payment_id: String,
        ) -> Result<CryptoComPaymentResponse>;
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("extra-cpp-bindings/include/pay.h");

        type OptionalArguments;
        fn get_description(&self) -> &str;
        fn get_metadata(&self) -> &str;
        fn get_order_id(&self) -> &str;
        fn get_return_url(&self) -> &str;
        fn get_cancel_url(&self) -> &str;
        fn get_sub_merchant_id(&self) -> &str;
        fn get_onchain_allowed(&self) -> bool;
        fn get_expired_at(&self) -> u64;
    }
}

/// returns the transactions of a given address.
/// The API key can be obtained from https://cronoscan.com
pub fn get_transaction_history_blocking(
    address: String,
    api_key: String,
) -> Result<Vec<RawTxDetail>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move { get_transaction_history(&address, api_key).await })
}

/// returns the ERC20 transfers of a given address of a given contract.
/// (address can be empty if option is ByContract)
/// default option is by address
/// The API key can be obtained from https://cronoscan.com
pub fn get_erc20_transfer_history_blocking(
    address: String,
    contract_address: String,
    option: QueryOption,
    api_key: String,
) -> Result<Vec<RawTxDetail>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        get_erc20_transfer_history(&address, &contract_address, option, api_key).await
    })
}

/// returns the ERC721 transfers of a given address of a given contract.
/// (address can be empty if option is ByContract)
/// default option is by address
/// The API key can be obtained from https://cronoscan.com
pub fn get_erc721_transfer_history_blocking(
    address: String,
    contract_address: String,
    option: QueryOption,
    api_key: String,
) -> Result<Vec<RawTxDetail>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        get_erc721_transfer_history(&address, &contract_address, option, api_key).await
    })
}

/// given the BlockScout REST API base url and the account address (hexadecimal),
/// it will return the list of all owned tokens
/// (ref: https://cronos.org/explorer/testnet3/api-docs)
pub fn get_tokens_blocking(
    blockscout_base_url: String,
    account_address: String,
) -> Result<Vec<RawTokenResult>> {
    let blockscout_url =
        format!("{blockscout_base_url}?module=account&action=tokenlist&address={account_address}");
    let resp = reqwest::blocking::get(blockscout_url)?.json::<RawResponse<RawTokenResult>>()?;
    Ok(resp.result)
}

/// given the BlockScout REST API base url and the account address (hexadecimal; required)
/// and optional contract address (hexadecimal; optional -- it can be empty if the option is ByAddress),
/// it will return all the token transfers (ERC20, ERC721... in the newer BlockScout
/// releases, also ERC1155)
/// (ref: https://cronos.org/explorer/testnet3/api-docs)
/// NOTE: QueryOption::ByContract is not supported by BlockScout
pub fn get_token_transfers_blocking(
    blockscout_base_url: String,
    address: String,
    contract_address: String,
    option: QueryOption,
) -> Result<Vec<RawTxDetail>> {
    let blockscout_url = match option {
        QueryOption::ByAddress => {
            format!("{blockscout_base_url}?module=account&action=tokentx&address={address}")
        }
        QueryOption::ByAddressAndContract => {
            format!(
                "{blockscout_base_url}?module=account&action=tokentx&address={address}&contractaddress={contract_address}"
            )
        }
        _ => {
            anyhow::bail!("unsupported option")
        }
    };
    let resp =
        reqwest::blocking::get(blockscout_url)?.json::<RawResponse<RawBlockScoutTransfer>>()?;

    Ok(resp.result.iter().flat_map(TryInto::try_into).collect())
}

/// given the BlockScout REST API base url and the contract address (hexadecimal),
///
/// page: A nonnegative integer that represents the page number to be used for
/// pagination. 'offset' must be provided in conjunction.
///
/// offset: A nonnegative integer that represents the maximum number of records to
/// return when paginating. 'page' must be provided in conjunction.
///
/// it will return the list of owners and balances (sorting from largest to smallest), but no
/// token ids.
///
/// (ref: https://cronos.org/explorer/api-docs#token)
///
/// ::TIPS:: Use another functions to get more token/owner details, e.g.
/// `get_tokens_blocking` to get owned tokens by account_address
pub fn get_token_holders<S: AsRef<str> + std::fmt::Display>(
    blockscout_base_url: S,
    contract_address: S,
    page: u64,
    offset: u64,
) -> Result<Vec<TokenHolderDetail>> {
    let blockscout_url =
        format!("{blockscout_base_url}?module=token&action=getTokenHolders&contractaddress={contract_address}&page={page}&offset={offset}");
    let resp = reqwest::blocking::get(blockscout_url)?.json::<RawResponse<TokenHolderDetail>>()?;
    Ok(resp.result)
}

/// it creates the payment object
/// https://pay-docs.crypto.com/#api-reference-resources-payments-create-a-payment
/// This API can be called using either your Secret Key or Publishable Key.
/// The amount should be given in base units (e.g. for USD, the base unit is cents 1 USD == 100 cents).
pub fn create_payment(
    secret_or_publishable_api_key: String,
    base_unit_amount: String,
    currency: String,
    optional_args: &ffi::OptionalArguments,
) -> Result<CryptoComPaymentResponse> {
    Ok(pay::create_payment(
        &secret_or_publishable_api_key,
        &base_unit_amount,
        &currency,
        optional_args,
    )?
    .into())
}

/// it returns the payment object by id
/// https://pay-docs.crypto.com/#api-reference-resources-payments-get-payment-by-id
/// This API can be called using either your Secret Key or Publishable Key.
pub fn get_payment(
    secret_or_publishable_api_key: String,
    payment_id: String,
) -> Result<CryptoComPaymentResponse> {
    Ok(pay::get_payment(&secret_or_publishable_api_key, &payment_id)?.into())
}

impl From<pay::CryptoPayObject> for CryptoComPaymentResponse {
    fn from(obj: pay::CryptoPayObject) -> Self {
        Self {
            id: obj.id,
            main_app_qr_code: obj.qr_code.unwrap_or_default(),
            onchain_deposit_address: obj.deposit_address.unwrap_or_default(),
            base_amount: serde_json::to_string(&obj.amount).unwrap_or_default(),
            currency: obj.currency,
            expiration: obj.expired_at.unwrap_or_default(),
            status: obj.status,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct RawResponse<R> {
    message: String,
    result: Vec<R>,
    status: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawBlockScoutTransfer {
    block_hash: String,
    block_number: String,
    confirmations: String,
    contract_address: String,
    cumulative_gas_used: String,
    from: String,
    gas: String,
    gas_price: String,
    gas_used: String,
    hash: String,
    input: String,
    log_index: String,
    nonce: String,
    time_stamp: String,
    to: String,
    token_decimal: String,
    token_name: String,
    token_symbol: String,
    transaction_index: String,
    value: String,
}

impl TryFrom<&RawBlockScoutTransfer> for RawTxDetail {
    type Error = anyhow::Error;

    fn try_from(tx: &RawBlockScoutTransfer) -> Result<Self, Self::Error> {
        let block_no = tx.block_number.parse::<u64>()?;
        Ok(Self {
            hash: tx.hash.clone(),
            to_address: tx.to.clone(),
            from_address: tx.from.clone(),
            value: tx.value.clone(),
            block_no,
            timestamp: tx.time_stamp.clone(),
            contract_address: tx.contract_address.clone(),
        })
    }
}

impl From<&NormalTransaction> for RawTxDetail {
    fn from(tx: &NormalTransaction) -> Self {
        let block_no: u64 = match tx.block_number {
            BlockNumber::Number(block_no) => block_no.0[0],
            _ => 0,
        };
        RawTxDetail {
            hash: tx
                .hash
                .value()
                .map(|x| format!("{x:?}"))
                .unwrap_or_default(),
            to_address: tx.to.map(|x| format!("{x:?}")).unwrap_or_default(),
            from_address: tx
                .from
                .value()
                .map(|x| format!("{x:?}"))
                .unwrap_or_default(),
            value: tx.value.to_string(),
            block_no,
            timestamp: tx.time_stamp.clone(),
            contract_address: format!("{:?}", tx.contract_address.unwrap_or_default()),
        }
    }
}

impl From<&ERC20TokenTransferEvent> for RawTxDetail {
    fn from(tx: &ERC20TokenTransferEvent) -> Self {
        let block_no: u64 = match tx.block_number {
            BlockNumber::Number(block_no) => block_no.0[0],
            _ => 0,
        };
        RawTxDetail {
            hash: format!("{:?}", tx.hash),
            to_address: tx.to.map(|x| format!("{x:?}")).unwrap_or_default(),
            from_address: format!("{:?}", tx.from),
            value: tx.value.to_string(),
            block_no,
            timestamp: tx.time_stamp.clone(),
            contract_address: format!("{:?}", tx.contract_address),
        }
    }
}

impl From<&ERC721TokenTransferEvent> for RawTxDetail {
    fn from(tx: &ERC721TokenTransferEvent) -> Self {
        let block_no: u64 = match tx.block_number {
            BlockNumber::Number(block_no) => block_no.0[0],
            _ => 0,
        };
        RawTxDetail {
            hash: format!("{:?}", tx.hash),
            to_address: tx.to.map(|x| format!("{x:?}")).unwrap_or_default(),
            from_address: format!("{:?}", tx.from),
            value: tx.token_id.to_string(),
            block_no,
            timestamp: tx.time_stamp.clone(),
            contract_address: format!("{:?}", tx.contract_address),
        }
    }
}

async fn get_transaction_history(address: &str, api_key: String) -> Result<Vec<RawTxDetail>> {
    let client = Client::new(Chain::Cronos, api_key)?;
    let transactions = client.get_transactions(&address.parse()?, None).await?;
    Ok(transactions.iter().map(|tx| tx.into()).collect())
}

async fn get_erc20_transfer_history(
    address: &str,
    contract_address: &str,
    option: QueryOption,
    api_key: String,
) -> Result<Vec<RawTxDetail>> {
    let client = Client::new(Chain::Cronos, api_key)?;
    let token_query = match option {
        QueryOption::ByContract => TokenQueryOption::ByContract(contract_address.parse()?),
        QueryOption::ByAddressAndContract => {
            TokenQueryOption::ByAddressAndContract(address.parse()?, contract_address.parse()?)
        }
        _ => TokenQueryOption::ByAddress(address.parse()?),
    };
    let transactions = client
        .get_erc20_token_transfer_events(token_query, None)
        .await?;
    Ok(transactions.iter().map(|tx| tx.into()).collect())
}

async fn get_erc721_transfer_history(
    address: &str,
    contract_address: &str,
    option: QueryOption,
    api_key: String,
) -> Result<Vec<RawTxDetail>> {
    let client = Client::new(Chain::Cronos, api_key)?;
    let token_query = match option {
        QueryOption::ByContract => TokenQueryOption::ByContract(contract_address.parse()?),
        QueryOption::ByAddressAndContract => {
            TokenQueryOption::ByAddressAndContract(address.parse()?, contract_address.parse()?)
        }
        _ => TokenQueryOption::ByAddress(address.parse()?),
    };
    let transactions = client
        .get_erc721_token_transfer_events(token_query, None)
        .await?;
    Ok(transactions.iter().map(|tx| tx.into()).collect())
}

fn walletconnect_restore_client(session_info: String) -> Result<Box<WalletconnectClient>> {
    let mut rt = tokio::runtime::Runtime::new()?;
    let client = walletconnect::walletconnect_restore_client(&mut rt, session_info)?;

    Ok(Box::new(WalletconnectClient {
        client: Some(client),
        rt,
    }))
}

fn walletconnect2_restore_client(session_info: String) -> Result<Box<Walletconnect2Client>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rt = tokio::runtime::Runtime::new()?;
    let client = rt.block_on(walletconnect2::restore_client(
        session_info,
        Some(tx.clone()),
    ))?;
    let client = Walletconnect2Client {
        client: Some(client),
        rt,
        tx,
        rx,
    };
    Ok(Box::new(client))
}

fn walletconnect_new_client(
    description: String,
    url: String,
    icon_urls: Vec<String>,
    name: String,
    chain_id: u64,
) -> Result<Box<WalletconnectClient>> {
    let mut rt = tokio::runtime::Runtime::new()?;
    let client = walletconnect::walletconnect_new_client(
        &mut rt,
        description,
        url,
        &icon_urls,
        name,
        chain_id,
    )?;

    Ok(Box::new(WalletconnectClient {
        client: Some(client),
        rt,
    }))
}
unsafe impl Send for ffi::WalletConnectCallback {}
unsafe impl Sync for ffi::WalletConnectCallback {}

fn check_wallet(
    cached: bool,
    registry_local_path: String,
    id: String,
    platform: crate::ffi::Platform,
) -> Result<bool> {
    let path = if registry_local_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(registry_local_path))
    };
    let reg = if cached {
        wallectconnectregistry::Registry::load_cached(path)?
    } else {
        wallectconnectregistry::Registry::fetch_new(path)?
    };

    Ok(reg.check_wallet(id, platform)?)
}

fn get_wallet(
    cached: bool,
    registry_local_path: String,
    id: String,
) -> Result<crate::ffi::WalletEntry> {
    let path = if registry_local_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(registry_local_path))
    };
    let reg = if cached {
        wallectconnectregistry::Registry::load_cached(path)?
    } else {
        wallectconnectregistry::Registry::fetch_new(path)?
    };

    Ok(reg.get_wallet(id)?)
}

fn get_all_wallets(
    cached: bool,
    registry_local_path: String,
) -> Result<Vec<crate::ffi::WalletEntry>> {
    let path = if registry_local_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(registry_local_path))
    };
    let reg = if cached {
        wallectconnectregistry::Registry::load_cached(path)?
    } else {
        wallectconnectregistry::Registry::fetch_new(path)?
    };

    Ok(reg.filter_wallets(None))
}

fn filter_wallets(
    cached: bool,
    registry_local_path: String,
    platform: crate::ffi::Platform,
) -> Result<Vec<crate::ffi::WalletEntry>> {
    let path = if registry_local_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(registry_local_path))
    };
    let reg = if cached {
        wallectconnectregistry::Registry::load_cached(path)?
    } else {
        wallectconnectregistry::Registry::fetch_new(path)?
    };

    Ok(reg.filter_wallets(Some(platform)))
}

fn generate_qrcode(qrcodestring: String) -> Result<crate::ffi::WalletQrcode> {
    let qr: QrCode = QrCode::encode_text(&qrcodestring, QrCodeEcc::Medium)?;
    let border: i32 = 2;
    let size = (qr.size() + border * 2) as u32;
    let mut image: Vec<u8> = Vec::with_capacity((size * size) as usize);
    for y in -border..qr.size() + border {
        for x in -border..qr.size() + border {
            image.push(u8::from(!qr.get_module(x, y)));
        }
    }
    assert!(image.len() as u32 == size * size);

    let qrcode = crate::ffi::WalletQrcode {
        qrcode: qrcodestring,
        image,
        size,
    };
    Ok(qrcode)
}

use defi_wallet_core_common::TransactionReceipt;
use ffi::WalletConnectTransactionReceiptRaw;
impl From<TransactionReceipt> for WalletConnectTransactionReceiptRaw {
    fn from(src: TransactionReceipt) -> Self {
        ffi::WalletConnectTransactionReceiptRaw {
            transaction_hash: src.transaction_hash,
            transaction_index: src.transaction_index,
            block_hash: src.block_hash,
            block_number: src.block_number,
            cumulative_gas_used: src.cumulative_gas_used,
            gas_used: src.gas_used,
            contract_address: src.contract_address,
            status: src.status,
            root: src.root,
            logs_bloom: src.logs_bloom,
            transaction_type: src.transaction_type,
            effective_gas_price: src.effective_gas_price,
            logs: src.logs,
        }
    }
}

// relay_server_string: "wss://relay.walletconnect.com"
// project_id: hex string without 0x prefix
//required_namespaces_json: {"eip155":{"methods":["eth_sendTransaction","eth_signTransaction","eth_sign","personal_sign","eth_signTypedData"],"chains":["eip155:5"],"events":["chainChanged","accountsChanged"]}}
//client_meta_json: {"description":"Defi WalletConnect v2 example.","url":"http://localhost:8080/","icons":[],"name":"Defi WalletConnect Web3 Example"}
pub fn walletconnect2_client_new(
    relay_server_string: String,
    project_id: String,
    required_namespaces_json: String,
    client_meta_json: String,
) -> Result<Box<Walletconnect2Client>> {
    // project_id is "", return error with anyhow
    if project_id.is_empty() {
        return Err(anyhow!("project_id is empty"));
    }
    // print all arguments
    println!("relay_server_string: {relay_server_string:?}");
    println!("project_id: {project_id:?}");
    println!("required_namespaces_json: {required_namespaces_json:?}");
    println!("client_meta_json: {client_meta_json:?}");

    let mut opts = defi_wallet_connect::v2::ClientOptions::default();

    if !relay_server_string.is_empty() {
        let relay_server = url::Url::parse(&relay_server_string)?;
        opts.relay_server = relay_server;
    }

    if !project_id.is_empty() {
        opts.project_id = project_id;
    }

    if !required_namespaces_json.is_empty() {
        let required_namespaces: defi_wallet_connect::v2::RequiredNamespaces =
            serde_json::from_str(&required_namespaces_json)?;
        opts.required_namespaces = required_namespaces;
    }

    if !client_meta_json.is_empty() {
        let client_meta: defi_wallet_connect::v2::Metadata =
            serde_json::from_str(&client_meta_json)?;
        opts.client_meta = client_meta;
    }

    println!("opts: {opts:?}");
    let required_namespaces = serde_json::to_string(&opts.required_namespaces)?;
    println!("required_namespaces_json: {required_namespaces}",);
    let client_meta = serde_json::to_string(&opts.client_meta)?;
    println!("client_meta_json: {client_meta}");

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    opts.callback_sender = Some(tx.clone());

    /* {
        relay_server,
        project_id,
        required_namespaces,
        client_meta,
        callback_sender: Some(tx.clone()),
    };*/
    let rt = tokio::runtime::Runtime::new()?;
    let client = rt.block_on(walletconnect2::new_client(opts))?;
    let client = Walletconnect2Client {
        client: Some(client),
        rt,
        tx,
        rx,
    };
    Ok(Box::new(client))
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;
    use sha2::Digest;

    // run this test manually and adjust the expect balance, as this changes on the testnet
    // TODO: deploy a contract and test a more stable balance
    #[test]
    #[ignore]
    pub fn test_get_tokens() {
        let expected: Vec<RawTokenResult> = serde_json::from_str(r#"[{"balance":"36330128084034373866325","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2883410031878","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"161","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0x1e0edbea442cfeff05ed1b01f0c38ecb768de0e0","decimals":"","name":"NFT Gold","symbol":"Gold","type":"ERC-1155"},{"balance":"1","contractAddress":"0x93d0c9a35c43f6bc999416a06aadf21e68b29eba","decimals":"","name":"Unique One","symbol":"UNE","type":"ERC-1155"},{"balance":"1","contractAddress":"0x93d0c9a35c43f6bc999416a06aadf21e68b29eba","decimals":"","name":"Unique One","symbol":"UNE","type":"ERC-1155"},{"balance":"1","contractAddress":"0x57aaaf5a61b6a370f981b7826843694cfa4774e1","decimals":"","name":"Protector","symbol":"サイタマ","type":"ERC-1155"},{"balance":"1","contractAddress":"0x57aaaf5a61b6a370f981b7826843694cfa4774e1","decimals":"","name":"Protector","symbol":"サイタマ","type":"ERC-1155"},{"balance":"1","contractAddress":"0x57aaaf5a61b6a370f981b7826843694cfa4774e1","decimals":"","name":"Protector","symbol":"サイタマ","type":"ERC-1155"},{"balance":"4","contractAddress":"0x93d0c9a35c43f6bc999416a06aadf21e68b29eba","decimals":"","name":"Unique One","symbol":"UNE","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"2","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"5","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"4","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"9","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"4","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"9","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"9","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"},{"balance":"36616853110389525899548","contractAddress":"0x27b9c2bd4baea18abdf49169054c1c1c12af9862","decimals":"18","name":"SNAFU","symbol":"SNAFU","type":"ERC-20"},{"balance":"73000000000000000000","contractAddress":"0x586f8a53c24d8d35a9f49e94d09058560791803e","decimals":"18","name":"NFTOPIUM","symbol":"NTP","type":"ERC-20"},{"balance":"763467280363239051","contractAddress":"0x6a023ccd1ff6f2045c3309768ead9e68f978f6e1","decimals":"18","name":"Wrapped Ether on xDai","symbol":"WETH","type":"ERC-20"},{"balance":"1","contractAddress":"0x90fda259cfbdb74f1804e921f523e660bfbe698d","decimals":"","name":"Unique Pixie","symbol":"UPIXIE","type":"ERC-721"},{"balance":"3000000000000000000","contractAddress":"0x9c58bacc331c9aa871afd802db6379a98e80cedb","decimals":"18","name":"Gnosis Token on xDai","symbol":"GNO","type":"ERC-20"}]"#).expect("parse");

        // blacksout somestimes works, sometimes not
        let max_count = 10;
        for _ in 0..max_count {
            let actual_result = get_tokens_blocking(
                "https://blockscout.com/xdai/mainnet/api".into(),
                "0x652d53227d7013f3FbBeA542443Dc2eeF05719De".into(),
            );
            match actual_result {
                Ok(actual) => {
                    assert_eq!(actual.len(), expected.len());
                    for (a, b) in actual.iter().zip(expected.iter()) {
                        // assert_eq!(a.balance, b.balance);
                        assert_eq!(a.contract_address, b.contract_address);
                        assert_eq!(a.decimals, b.decimals);
                        assert_eq!(a.name, b.name);
                        assert_eq!(a.symbol, b.symbol);
                        assert_eq!(a.token_type, b.token_type);
                    }
                    return;
                }
                Err(_e) => {
                    // try more
                    // wait for 1 second
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
        } // end for for loop
        panic!("test_get_tokens failed");
    }

    #[test]
    pub fn test_get_token_transactions() {
        let expected: Vec<RawBlockScoutTransfer> = serde_json::from_str(
            r#"[
            {
              "value": "200000000000000000000",
              "blockHash": "0x1456b7934898b7c735967e849effc4dc45e84ff32c6c2e130d572cb9589ca652",
              "blockNumber": "2088372",
              "confirmations": "537920",
              "contractAddress": "0x715b4d660148c477e03358f8b0315ed4088fe89a",
              "cumulativeGasUsed": "31760308",
              "from": "0x9ad08de843158b0a4f8efdae6ea49caf77bbf13f",
              "gas": "145930",
              "gasPrice": "2021527882398",
              "gasUsed": "97287",
              "hash": "0x0890c4dce61da8713db5844fa0ae0aa73b74ea6ccfa91c90065ae80471cd908c",
              "input": "0x3363315c000000000000000000000000841a15d12aec9c6039fd132c2fbff112ed355700",
              "logIndex": "104",
              "nonce": "13",
              "timeStamp": "1646318156",
              "to": "0x841a15d12aec9c6039fd132c2fbff112ed355700",
              "tokenDecimal": "18",
              "tokenName": "DAI",
              "tokenSymbol": "DAI",
              "transactionIndex": "37"
            },
            {
              "value": "200000000",
              "blockHash": "0x307edc85e8d5bde617857831427cb90e0c9b2b6c455d6f3a56918c3ee4aa009d",
              "blockNumber": "2039569",
              "confirmations": "586723",
              "contractAddress": "0xf0307093f23311fe6776a7742db619eb3df62969",
              "cumulativeGasUsed": "97352",
              "from": "0x9ad08de843158b0a4f8efdae6ea49caf77bbf13f",
              "gas": "146028",
              "gasPrice": "5017552095418",
              "gasUsed": "97352",
              "hash": "0x7939078155138ed15a10343627a4e6b8d623d63bdd69479bf62d7872dee587d6",
              "input": "0xd8215830000000000000000000000000841a15d12aec9c6039fd132c2fbff112ed355700",
              "logIndex": "0",
              "nonce": "18",
              "timeStamp": "1646036708",
              "to": "0x841a15d12aec9c6039fd132c2fbff112ed355700",
              "tokenDecimal": "6",
              "tokenName": "USDC",
              "tokenSymbol": "USDC",
              "transactionIndex": "0"
            }
          ]"#,
        )
        .expect("parse");
        let expected: Vec<RawTxDetail> = expected.iter().flat_map(TryInto::try_into).collect();
        let actual = get_token_transfers_blocking(
            "https://cronos.org/explorer/testnet3/api".to_string(),
            "0x841a15D12aEc9c6039FD132c2FbFF112eD355700".to_string(),
            "".to_string(),
            QueryOption::ByAddress,
        )
        .expect("blockscout query");
        assert_eq!(actual, expected);
    }

    #[test]
    pub fn test_generate_qrcode() {
        let qrcode = generate_qrcode("play-cpp-sdk".to_string()).expect("get qrcode");
        assert!("play-cpp-sdk" == qrcode.qrcode);
        assert!(qrcode.image.len() as u32 == qrcode.size * qrcode.size);
        let mut hasher = sha2::Sha256::new();
        hasher.update(&qrcode.image);
        let result = hasher.finalize();
        assert!(
            result[..]
                == hex!("8C64C3C66FD5C11DDD7926664D311056F6C6F08CE7371AADE166D5E5B0A6754C")[..]
        );
    }
    #[test]
    #[ignore]
    pub fn test_get_token_holders() {
        let expected: Vec<TokenHolderDetail> = serde_json::from_str(
            r#"[
                {
                    "address": "0x8bd0e10424255e71ab18d192503f751ef62167b0",
                    "value": "67738518631326470638584975590"
                },
                {
                    "address": "0x652d53227d7013f3fbbea542443dc2eef05719de",
                    "value": "36330128084034373866325"
                },
                {
                    "address": "0x8bd0e10424255e71ab18d192503f751ef62167b0",
                    "value": "14678105397136827551403"
                },
                {
                    "address": "0xaf3a0f20580dcb7d251126fc6a45897ac760c550",
                    "value": "2331440245657701757142"
                },
                {
                    "address": "0x1253594843798ff0fcd7fa221b820c2d3ca58fd5",
                    "value": "1457125011469162132201"
                },
                {
                    "address": "0x8bd0e10424255e71ab18d192503f751ef62167b0",
                    "value": "906981481481475281365"
                },
                {
                    "address": "0x097f16e2931a86107dd0f900c0b5f060889f65a2",
                    "value": "3158512111945682"
                },
                {
                    "address": "0x1253594843798ff0fcd7fa221b820c2d3ca58fd5",
                    "value": "1755139738543195"
                },
                {
                    "address": "0x1253594843798ff0fcd7fa221b820c2d3ca58fd5",
                    "value": "426803243704652"
                },
                {
                    "address": "0x1253594843798ff0fcd7fa221b820c2d3ca58fd5",
                    "value": "426803243704652"
                }
          ]"#,
        )
        .expect("parse");
        let actual = get_token_holders(
            "https://blockscout.com/xdai/mainnet/api",
            "0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a",
            1,
            10,
        )
        .expect("blockscout query");
        assert_eq!(actual, expected);
    }
}
