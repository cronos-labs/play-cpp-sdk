use std::{fmt::Display, str::FromStr};

use ethers::types::Address;
// https://docs.walletconnect.com/2.0/specs/clients/sign/rpc-methods
// FIXME: wc_sessionUpdate
// FIXME: wc_sessionExtend
// FIXME: wc_sessionEvent
// FIXME: wc_sessionDelete
// FIXME: wc_sessionPing OK
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
/// https://docs.walletconnect.com/2.0/specs/clients/sign/rpc-methods#wc_sessionpropose
pub const WC_SESSION_PROPOSE_REQUEST_METHOD: &str = "wc_sessionPropose";
pub const WC_SESSION_PING_REQUEST_METHOD: &str = "wc_sessionPing";
/// https://docs.walletconnect.com/2.0/specs/clients/sign/rpc-methods#wc_sessionpropose
pub const WC_SESSION_PROPOSE_REQUEST_TAG: u32 = 1100;
pub const WC_SESSION_PING_REQUEST_TAG: u32 = 1114;

/// Method: wc_sessionPropose
#[derive(Serialize, Deserialize)]
pub struct WcSessionPropose {
    #[serde(rename = "requiredNamespaces")]
    pub(crate) required_namespaces: RequiredNamespaces,
    #[serde(rename = "optionalNamespaces")]
    pub(crate) optional_namespaces: OptionalNamespaces,
    pub(crate) relays: Vec<Relay>,
    pub(crate) proposer: Peer,
}

#[derive(Serialize, Deserialize)]
pub struct OptionalNamespaces {
    // FIXME: eip155 etc.
}

/// FIXME: is it duplicate with WalletConnect 1.0?
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Metadata {
    pub description: String,
    pub url: String,
    pub icons: Vec<String>,
    pub name: String,
}

/// the relay protocol info
#[derive(Serialize, Deserialize)]
pub struct Relay {
    pub(crate) protocol: String,
}

/// the namespaces required by the dApp / client
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RequiredNamespaces {
    pub(crate) eip155: Eip155,
    // FIXME: Cosmos, Solana, Stellar...
}

impl RequiredNamespaces {
    /// Create a new required namespaces.
    pub fn new(methods: Vec<String>, chains: Vec<String>, events: Vec<String>) -> Self {
        Self {
            eip155: Eip155 {
                methods,
                chains,
                events,
            },
        }
    }
}

/// the required EIP155 namespace
/// chains are the chain IDs prefixed with "eip155:"
/// TODO: just store the chain IDs and have custom deserialize/serialize?
/// TODO: methods/events -- use Enum of known Ethereum methods/events?
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Eip155 {
    methods: Vec<String>,
    pub(crate) chains: Vec<String>,
    events: Vec<String>,
}

/// The response to the session proposal request.
#[derive(Serialize, Deserialize)]
pub struct WcSessionProposeResponse {
    relay: Relay,
    #[serde(rename = "responderPublicKey")]
    pub(crate) responder_public_key: String,
}

/// Method: wc_sessionSettle
/// https://docs.walletconnect.com/2.0/specs/clients/sign/rpc-methods#wc_sessionsettle
#[derive(Serialize, Deserialize)]
pub struct WcSessionSettle {
    relay: Relay,
    pub namespaces: Namespaces,
    #[serde(rename = "requiredNamespaces")]
    required_namespaces: RequiredNamespaces,
    pub controller: Peer,
    expiry: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WcSessionUpdate {
    pub namespaces: Namespaces,
}

#[derive(Serialize, Deserialize)]
pub struct WcSessionEventEvent {
    pub name: String,
    pub data: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct WcSessionEvent {
    pub event: WcSessionEventEvent,
    pub chain_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct WcSessionPing {}

#[derive(Serialize, Deserialize)]
pub struct WcSessionExtend {}

#[derive(Serialize, Deserialize)]
pub struct WcSessionDelete {
    pub code: i64,
    pub message: String,
}

/// https://docs.walletconnect.com/2.0/specs/clients/sign/rpc-methods#wc_sessionsettle
pub const WC_SESSION_SETTLE_RESPONSE_TAG: u32 = 1103;
pub const WC_SESSION_UPDATE_RESPONSE_TAG: u32 = 1105;
pub const WC_SESSION_PING_RESPONSE_TAG: u32 = 1115;
pub const WC_SESSION_DELETE_RESPONSE_TAG: u32 = 1113;
pub const WC_SESSION_EVENT_RESPONSE_TAG: u32 = 1111;
pub const WC_SESSION_EXTEND_RESPONSE_TAG: u32 = 1107;

/// The peer metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Peer {
    /// public key encoded in hexadecimal
    #[serde(rename = "publicKey")]
    pub(crate) public_key: String,
    pub(crate) metadata: Metadata,
}

/// The namespaces returned by the wallet
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Namespaces {
    pub eip155: NamespacesEip155,
}

/// The address with EIP155 chain ID
/// e.g. eip155:1:0x1234567890123456789012345678901234567890
#[derive(SerializeDisplay, DeserializeFromStr, Debug, Clone)]
pub struct Eip155AddressWithChainId {
    pub address: Address,
    pub chain_id: u64,
}

impl FromStr for Eip155AddressWithChainId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let prefix = parts.next().ok_or(anyhow::anyhow!("invalid prefix"))?;
        if prefix != "eip155" {
            return Err(anyhow::anyhow!("invalid prefix"));
        }
        let chain_id = parts.next().ok_or(anyhow::anyhow!("invalid chain id"))?;
        let chain_id = chain_id.parse::<u64>()?;
        let address = parts.next().ok_or(anyhow::anyhow!("invalid address"))?;
        let address = Address::from_str(address)?;
        Ok(Self { address, chain_id })
    }
}

impl Display for Eip155AddressWithChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = self.address;
        let full_address = format!("{address:?}");
        assert!(
            full_address.starts_with("0x"),
            "Address must start with '0x'"
        );
        write!(f, "eip155:{}:{}", self.chain_id, full_address)
    }
}

impl Namespaces {
    pub fn get_ethereum_addresses(&self) -> Vec<Eip155AddressWithChainId> {
        self.eip155.accounts.clone()
    }
}

/// The EIP155 namespace
/// FIXME: parse events and methods to the known Ethereum events and methods?
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NamespacesEip155 {
    pub accounts: Vec<Eip155AddressWithChainId>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

/// ref: https://docs.walletconnect.com/2.0/specs/clients/sign/rpc-methods#wc_sessionrequest
pub const WC_SESSION_REQUEST_METHOD: &str = "wc_sessionRequest";
/// ref: https://docs.walletconnect.com/2.0/specs/clients/sign/rpc-methods#wc_sessionrequest
pub const WC_SESSION_REQUEST_TAG: u32 = 1108;

/// Method: wc_sessionRequest
#[derive(Serialize, Deserialize)]
pub struct WcSessionRequest<T> {
    request: WcSessionRequestData<T>,
    #[serde(rename = "chainId")]
    chain_id: String,
}

impl<T> WcSessionRequest<T> {
    pub fn new(method: String, params: T, chain_id: String) -> Self {
        Self {
            request: WcSessionRequestData { method, params },
            chain_id,
        }
    }
}

/// this wraps the RPC requests,
/// such as https://docs.walletconnect.com/2.0/advanced/rpc-reference/ethereum-rpc
#[derive(Serialize, Deserialize)]
pub struct WcSessionRequestData<T> {
    method: String,
    params: T,
    // expiry: Option<u64>,
}

#[cfg(test)]
mod test {
    use crate::Request;

    use super::WcSessionSettle;

    #[test]
    pub fn test_deserialize_wc_settle() {
        let request = "{\"id\":1678415342621744,\"jsonrpc\":\"2.0\",\"method\":\"wc_sessionSettle\",\"params\":{\"relay\":{\"protocol\":\"irn\"},\"namespaces\":{\"eip155\":{\"accounts\":[\"eip155:5:0xcE915a3b937261853EE2C60B8010c22c295200B0\"],\"methods\":[\"eth_sendTransaction\",\"eth_signTransaction\",\"eth_sign\",\"personal_sign\",\"eth_signTypedData\"],\"events\":[\"chainChanged\",\"accountsChanged\"]}},\"requiredNamespaces\":{\"eip155\":{\"methods\":[\"eth_sendTransaction\",\"eth_signTransaction\",\"eth_sign\",\"personal_sign\",\"eth_signTypedData\"],\"chains\":[\"eip155:5\"],\"events\":[\"chainChanged\",\"accountsChanged\"]}},\"optionalNamespaces\":{},\"controller\":{\"publicKey\":\"94f705551213e83822c9a0c29063bb79223eec36433ad411f2de7bbaa4ae496f\",\"metadata\":{\"name\":\"React Wallet\",\"description\":\"React Wallet for WalletConnect\",\"url\":\"https://walletconnect.com/\",\"icons\":[\"https://avatars.githubusercontent.com/u/37784886\"]}},\"expiry\":1679020142}}";
        let req: Request<WcSessionSettle> = serde_json::from_str(request).unwrap();
        let data = req.params;
        assert_eq!(data.namespaces.eip155.accounts.len(), 1);
        assert_eq!(data.namespaces.eip155.methods.len(), 5);
        assert_eq!(data.namespaces.eip155.events.len(), 2);
        assert_eq!(data.required_namespaces.eip155.methods.len(), 5);
        assert_eq!(data.required_namespaces.eip155.chains.len(), 1);
        assert_eq!(data.required_namespaces.eip155.events.len(), 2);
        assert_eq!(
            data.namespaces.eip155.accounts[0].address,
            "0xcE915a3b937261853EE2C60B8010c22c295200B0"
                .parse()
                .unwrap()
        );
    }
}
