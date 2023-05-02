use relay_rpc::domain::Topic;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use url::Url;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use crate::{crypto::Key, hex};

use super::{
    crypto::derive_symkey_topic,
    protocol::{
        Namespaces, OptionalNamespaces, Peer, Relay, RequiredNamespaces, WcSessionPropose,
        WcSessionProposeResponse, WcSessionSettle, WcSessionUpdate,
    },
    Metadata,
};

/// The WalletConnect 2.0 session information
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// if the wallet approved the connection
    pub connected: bool,
    /// namespaces required by the client
    pub required_namespaces: RequiredNamespaces,
    /// the accounts, methods, and events returned by the wallet
    pub namespaces: Option<Namespaces>,
    /// the relay server URL
    pub relay_server: Url,
    /// the project id for the walletconnect v2
    pub project_id: String,
    /// the secret key used in encrypting the session proposal request
    pub session_proposal_symkey: Key,
    /// this is the client's secret key for a pairing topic derivation
    pub client_secret_key: Key,
    /// the client metadata (that will be presented to the wallet in the initial request)
    pub client_meta: Peer,
    /// the topic derived from the wallet's public key
    /// and the secret key used in encrypting the paired session requests
    pub pairing_topic_symkey: Option<(Topic, Key)>,
    /// the wallet's metadata
    pub pairing_peer_meta: Option<Peer>,
    /// the one-time request ID
    pub session_proposal_topic: Topic,
}

impl SessionInfo {
    /// Create a new session info.
    /// it will generate a new secret key for the session proposal,
    /// a new secret key for the pairing topic,
    /// a new topic for the session proposal
    /// and prepare the client/peer metadata from it
    /// and provided arguments.
    pub fn new(
        relay_server: Url,
        project_id: String,
        required_namespaces: RequiredNamespaces,
        metadata: Metadata,
    ) -> Self {
        let mut client_secret = StaticSecret::new(relay_rpc::auth::rand::thread_rng());
        let client_public = PublicKey::from(&client_secret);
        let client_secret_key = Key::from_raw(client_secret.to_bytes());
        client_secret.zeroize();
        let session_proposal_symkey = Key::random();

        let session_proposal_topic = Topic::generate();
        let client_meta = Peer {
            public_key: hex::encode(client_public.as_bytes()),
            metadata,
        };
        Self {
            connected: false,
            required_namespaces,
            namespaces: None,
            relay_server,
            project_id,
            session_proposal_symkey,
            client_secret_key,
            client_meta,
            pairing_topic_symkey: None,
            pairing_peer_meta: None,
            session_proposal_topic,
        }
    }

    /// Return the URI for the initial session proposal request
    /// (usually displayed in a QR code or used via a deep link).
    /// ref: https://docs.walletconnect.com/2.0/specs/clients/core/pairing/pairing-uri
    ///
    /// FIXME: currently, it doesn't include the "required methods"
    /// such as &methods=[wc_sessionPropose],[wc_authRequest,wc_authBatchRequest]
    /// but it seems the test react wallet works without it
    pub fn uri(&self) -> String {
        // FIXME: methods
        format!(
            "wc:{}@2?symKey={}&relay-protocol=irn",
            self.session_proposal_topic,
            self.session_proposal_symkey.display().expose_secret()
        )
    }

    /// Return the session proposal request payload
    pub fn session_proposal(&self) -> WcSessionPropose {
        WcSessionPropose {
            required_namespaces: self.required_namespaces.clone(),
            optional_namespaces: OptionalNamespaces {},
            relays: vec![Relay {
                protocol: "irn".to_string(),
            }],
            proposer: self.client_meta.clone(),
        }
    }

    /// Update the session based on the session proposal response
    /// and return the topic for the pairing topic
    /// if the response is valid
    pub fn session_proposal_response(
        &mut self,
        propose_response: &WcSessionProposeResponse,
    ) -> Option<Topic> {
        self.pairing_topic_symkey = derive_symkey_topic(
            &propose_response.responder_public_key,
            &self.client_secret_key,
        );
        self.pairing_topic_symkey.as_ref().map(|(x, _)| x.clone())
    }

    /// Update the session based on the session settle response
    pub fn session_settle(&mut self, settle: WcSessionSettle) {
        self.pairing_peer_meta = Some(settle.controller);
        self.namespaces = Some(settle.namespaces);
    }

    pub fn session_update(&mut self, info: WcSessionUpdate) {
        self.namespaces = Some(info.namespaces);
    }

    // FIXME: 7 days
    pub fn session_extend(&mut self) {}

    pub fn session_delete(&mut self) {
        self.connected = false;
        self.pairing_topic_symkey = None;
        self.pairing_peer_meta = None;
        self.namespaces = None;
    }
}
