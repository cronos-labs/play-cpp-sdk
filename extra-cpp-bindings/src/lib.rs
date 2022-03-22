use anyhow::Result;
use ethers_core::types::{BlockNumber, Chain};
use ethers_etherscan::{
    account::{
        ERC20TokenTransferEvent, ERC721TokenTransferEvent, NormalTransaction, TokenQueryOption,
    },
    Client,
};

#[cxx::bridge(namespace = "com::crypto::game_sdk")]
mod ffi {
    /// Raw transaction details
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

    pub enum QueryOption {
        ByContract,
        ByAddressAndContract,
    }

    extern "Rust" {
        pub fn get_transaction_history_blocking(
            address: String,
            api_key: String,
        ) -> Result<Vec<RawTxDetail>>;
        pub fn get_erc20_transfer_history_blocking(
            address: String,
            contract_address: String,
            option: QueryOption,
            api_key: String,
        ) -> Result<Vec<RawTxDetail>>;
        pub fn get_erc721_transfer_blocking(
            address: String,
            contract_address: String,
            option: QueryOption,
            api_key: String,
        ) -> Result<Vec<RawTxDetail>>;
    }
}

use ffi::{QueryOption, RawTxDetail};

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
pub fn get_erc721_transfer_blocking(
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

impl From<&NormalTransaction> for RawTxDetail {
    fn from(tx: &NormalTransaction) -> Self {
        let block_no: u64 = match tx.block_number {
            BlockNumber::Number(block_no) => block_no.0[0],
            _ => 0,
        };
        RawTxDetail {
            hash: tx.hash.value().map(|x| x.to_string()).unwrap_or_default(),
            to_address: tx.to.map(|x| x.to_string()).unwrap_or_default(),
            from_address: tx.from.value().map(|x| x.to_string()).unwrap_or_default(),
            value: tx.value.to_string(),
            block_no,
            timestamp: tx.time_stamp.clone(),
            contract_address: tx
                .contract_address
                .value()
                .map(|x| x.to_string())
                .unwrap_or_default(),
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
            hash: tx.hash.to_string(),
            to_address: tx.to.map(|x| x.to_string()).unwrap_or_default(),
            from_address: tx.from.to_string(),
            value: tx.value.to_string(),
            block_no,
            timestamp: tx.time_stamp.clone(),
            contract_address: tx.contract_address.to_string(),
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
            hash: tx.hash.to_string(),
            to_address: tx.to.map(|x| x.to_string()).unwrap_or_default(),
            from_address: tx.from.to_string(),
            value: tx.token_id.to_string(),
            block_no,
            timestamp: tx.time_stamp.clone(),
            contract_address: tx.contract_address.to_string(),
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
