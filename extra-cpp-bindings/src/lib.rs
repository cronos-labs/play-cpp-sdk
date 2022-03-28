use anyhow::Result;
use ethers_core::types::{BlockNumber, Chain};
use ethers_etherscan::{
    account::{
        ERC20TokenTransferEvent, ERC721TokenTransferEvent, NormalTransaction, TokenQueryOption,
    },
    Client,
};
use serde::{Deserialize, Serialize};

#[cxx::bridge(namespace = "com::crypto::game_sdk")]
mod ffi {
    /// Raw transaction details (extracted from Cronoscan/Etherscan API)
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
        /// the human-readable name of the token
        pub name: String,
        /// the ticker for the token
        pub symbol: String,
        /// the token type (ERC-20, ERC-721, ERC-1155)
        #[serde(rename = "type")]
        pub token_type: String,
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
        pub fn get_tokens_blocking(
            blockscout_base_url: String,
            account_address: String,
        ) -> Result<Vec<RawTokenResult>>;
    }
}

use ffi::{QueryOption, RawTokenResult, RawTxDetail};

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

/// given the BlockScout REST API base url and the account address (hexadecimal),
/// it will return the list of all owned tokens
/// (ref: https://cronos.org/explorer/testnet3/api-docs)
pub fn get_tokens_blocking(
    blockscout_base_url: String,
    account_address: String,
) -> Result<Vec<RawTokenResult>> {
    let blockscout_url =
        format!("{blockscout_base_url}?module=account&action=tokenlist&address={account_address}");
    let resp = reqwest::blocking::get(&blockscout_url)?.json::<RawTokenResponse>()?;
    Ok(resp.result)
}

#[derive(Serialize, Deserialize)]
struct RawTokenResponse {
    message: String,
    result: Vec<RawTokenResult>,
    status: String,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_get_tokens() {
        let expected: Vec<RawTokenResult> = serde_json::from_str(r#"[{"balance":"1","contractAddress":"0x1e0edbea442cfeff05ed1b01f0c38ecb768de0e0","decimals":"","name":"NFT Gold","symbol":"Gold","type":"ERC-1155"},{"balance":"36616853110389525899548","contractAddress":"0x27b9c2bd4baea18abdf49169054c1c1c12af9862","decimals":"18","name":"SNAFU","symbol":"SNAFU","type":"ERC-20"},{"balance":"1","contractAddress":"0x57aaaf5a61b6a370f981b7826843694cfa4774e1","decimals":"","name":"Protector","symbol":"サイタマ","type":"ERC-1155"},{"balance":"73000000000000000000","contractAddress":"0x586f8a53c24d8d35a9f49e94d09058560791803e","decimals":"18","name":"NFTOPIUM","symbol":"NTP","type":"ERC-20"},{"balance":"763467280363239051","contractAddress":"0x6a023ccd1ff6f2045c3309768ead9e68f978f6e1","decimals":"18","name":"Wrapped Ether on xDai","symbol":"WETH","type":"ERC-20"},{"balance":"1","contractAddress":"0x90fda259cfbdb74f1804e921f523e660bfbe698d","decimals":"","name":"Unique Pixie","symbol":"UPIXIE","type":"ERC-721"},{"balance":"4","contractAddress":"0x93d0c9a35c43f6bc999416a06aadf21e68b29eba","decimals":"","name":"Unique One","symbol":"UNE","type":"ERC-1155"},{"balance":"3000000000000000000","contractAddress":"0x9c58bacc331c9aa871afd802db6379a98e80cedb","decimals":"18","name":"Gnosis Token on xDai","symbol":"GNO","type":"ERC-20"},{"balance":"1","contractAddress":"0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a","decimals":"","name":"SNAFU","symbol":"SNAFU","type":"ERC-1155"}]"#).expect("parse");
        let actual = get_tokens_blocking(
            "https://blockscout.com/xdai/mainnet/api".into(),
            "0x652d53227d7013f3FbBeA542443Dc2eeF05719De".into(),
        )
        .expect("api");
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected.iter()) {
            // assert_eq!(a.balance, b.balance);
            assert_eq!(a.contract_address, b.contract_address);
            assert_eq!(a.decimals, b.decimals);
            assert_eq!(a.name, b.name);
            assert_eq!(a.symbol, b.symbol);
            assert_eq!(a.token_type, b.token_type);
        }
    }
}
