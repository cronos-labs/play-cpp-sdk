//! Copyright (c) 2020 Nicholas Rodrigues Lordello (licensed under the Apache License, Version 2.0)
//! Modifications Copyright (c) 2022, Cronos Labs (licensed under the Apache License, Version 2.0)
use crate::client::{ClientChannelMessage, ClientChannelMessageType};
use crate::crypto::Key;
use crate::protocol::{
    Metadata, PeerMetadata, SessionParams, SessionRequest, SessionUpdate, Topic,
};
use crate::uri::Uri;
use ethers::prelude::Address;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use url::form_urlencoded::Serializer;
use url::Url;
/// The WalletConnect 1.0 session information
/// based on the initial request-response: https://docs.walletconnect.com/tech-spec#session-request

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// if the wallet approved the connection
    pub connected: bool,
    /// the accounts returned by the wallet
    pub accounts: Vec<Address>,
    /// the chain id returned by the wallet
    pub chain_id: Option<u64>,
    /// the bridge server URL
    pub bridge: Url,
    /// the secret key used in encrypting wallet requests
    /// and decrypting wallet responses as per WalletConnect 1.0
    pub key: Key,
    /// this is the client's randomly generated ID
    pub client_id: Topic,
    /// the client metadata (that will be presented to the wallet in the initial request)
    pub client_meta: Metadata,
    /// the wallet's ID
    pub peer_id: Option<Topic>,
    /// the wallet's metadata
    pub peer_meta: Option<PeerMetadata>,
    /// the one-time request ID
    pub handshake_topic: Topic,
}

impl SessionInfo {
    pub fn uri(&self) -> Uri {
        Uri::parse(&format!(
            "wc:{}@1?{}",
            self.handshake_topic,
            Serializer::new(String::new())
                .append_pair("bridge", self.bridge.as_str())
                .append_pair("key", self.key.display().expose_secret())
                .finish()
        ))
        .expect("WalletConnect URIs from sessions are always valid")
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub info: SessionInfo,

    /// when memory is enough, and receive channel is valid
    /// send will succeed
    /// ref: https://docs.rs/futures/0.1.31/futures/sync/mpsc/fn.unbounded.html
    pub callback_channel: Option<UnboundedSender<ClientChannelMessage>>,
}

impl Session {
    pub fn set_callback(&mut self, myfunc: UnboundedSender<ClientChannelMessage>) {
        self.callback_channel = Some(myfunc);
    }

    /// generate the session URI: https://docs.walletconnect.com/tech-spec#requesting-connection
    /// https://eips.ethereum.org/EIPS/eip-1328
    pub fn uri(&self) -> Uri {
        self.info.uri()
    }

    /// generates a session request from the session: https://docs.walletconnect.com/tech-spec#session-request
    pub fn request(&self) -> SessionRequest {
        SessionRequest {
            peer_id: self.info.client_id.clone(),
            peer_meta: self.info.client_meta.clone(),
            chain_id: self.info.chain_id,
        }
    }

    /// updates the session details from the response
    pub fn apply(&mut self, params: SessionParams) {
        self.info.connected = params.approved;
        self.info.accounts = params.accounts;
        self.info.chain_id = Some(params.chain_id);
        self.info.peer_id = Some(params.peer_id);
        self.info.peer_meta = Some(params.peer_meta);

        if let Some(ref mut callback) = self.callback_channel {
            let msg = ClientChannelMessage {
                state: ClientChannelMessageType::Connected,
                session: Some(self.info.clone()),
            };
            callback
                .send(msg)
                .expect("callback channel should be valid");
        }
    }
    /// when start connecting
    pub fn event_connecting(&self) {
        if let Some(ref callback) = self.callback_channel {
            let msg = ClientChannelMessage {
                state: ClientChannelMessageType::Connecting,
                session: Some(self.info.clone()),
            };
            callback
                .send(msg)
                .expect("callback channel should be valid");
        }
    }

    /// when updated
    pub fn event_updated(&self) {
        if let Some(ref callback) = self.callback_channel {
            let msg = ClientChannelMessage {
                state: ClientChannelMessageType::Updated,
                session: Some(self.info.clone()),
            };
            callback
                .send(msg)
                .expect("callback channel should be valid");
        }
    }

    /// when session is disconnected
    pub fn event_disconnect(&self) {
        if let Some(ref callback) = self.callback_channel {
            let msg = ClientChannelMessage {
                state: ClientChannelMessageType::Disconnected,
                session: Some(self.info.clone()),
            };
            callback
                .send(msg)
                .expect("callback channel should be valid");
        }
    }

    /// updates the session details from the session update: https://docs.walletconnect.com/tech-spec#session-update
    pub fn update(&mut self, update: SessionUpdate) {
        self.info.connected = update.approved;

        if let Some(accounts) = update.accounts {
            self.info.accounts = accounts;
        } else {
            self.info.accounts = vec![];
        }
        self.info.chain_id = update.chain_id;

        if self.info.connected {
            // notify updated information
            self.event_updated();
        } else {
            // by update, session is desroyed
            self.event_disconnect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_topic_is_random() {
        assert_ne!(Topic::new(), Topic::new());
    }

    #[test]
    fn zero_topic() {
        assert_eq!(
            json!(Topic::zero()),
            json!("00000000-0000-0000-0000-000000000000")
        );
    }

    #[test]
    fn topic_serialization() {
        let topic = Topic::new();
        let serialized = serde_json::to_string(&topic).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(topic, deserialized);
    }
}
