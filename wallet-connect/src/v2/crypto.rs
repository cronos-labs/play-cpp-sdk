use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use relay_rpc::domain::Topic;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use crate::{crypto::Key, hex};

/// After the session proposal response, we obtain the wallet's public key
/// and derive a new topic and symmetric key for the pairing topic
pub fn derive_symkey_topic(responder_public_key: &str, secret: &Key) -> Option<(Topic, Key)> {
    let mut secret_buf = [0u8; 32];
    secret_buf.copy_from_slice(&secret.as_ref());
    let mut client_secret = StaticSecret::from(secret_buf);
    match hex::decode(responder_public_key) {
        Ok(pk) if pk.len() == 32 => {
            let mut pk_b = [0u8; 32];
            pk_b.copy_from_slice(&pk);
            let responder_public = PublicKey::from(pk_b);
            let shared_secret = client_secret.diffie_hellman(&responder_public);
            let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
            let mut sym_key = [0u8; 32];

            hkdf.expand(&[], &mut sym_key).expect("expand sym key");

            let hashed = Sha256::digest(&sym_key[..]);
            let new_topic = Topic::from(hex::encode(hashed));
            secret_buf.zeroize();
            client_secret.zeroize();
            Some((new_topic, Key::from_raw(sym_key)))
        }
        _ => {
            secret_buf.zeroize();
            client_secret.zeroize();
            None
        }
    }
}

/// Encrypt using ChaCha20Poly1305 and encode using base64
/// The first byte is a version byte, the next 12 bytes are the nonce
/// (see https://docs.walletconnect.com/2.0/specs/clients/core/crypto/crypto-envelopes#type-0-envelope)
pub fn encrypt_and_encode(key: &Key, data: &[u8]) -> String {
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_ref()).expect("correct key");
    let nonce = ChaCha20Poly1305::generate_nonce(OsRng {});
    let ciphertext = cipher.encrypt(&nonce, data).expect("encryption");
    let mut buf = vec![0];
    buf.extend_from_slice(&nonce);
    buf.extend_from_slice(&ciphertext);
    general_purpose::STANDARD.encode(buf)
}

/// Decode using base64 and decrypt using ChaCha20Poly1305
/// The first byte is a version byte, the next 12 bytes are the nonce
/// (see https://docs.walletconnect.com/2.0/specs/clients/core/crypto/crypto-envelopes#type-0-envelope)
pub fn decode_decrypt(key: &Key, data: &str) -> Result<Vec<u8>, ()> {
    let decoded = general_purpose::STANDARD.decode(data).map_err(|_| ())?;
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_ref()).expect("correct key");
    let nonce = Nonce::clone_from_slice(&decoded[1..13]);
    cipher.decrypt(&nonce, &decoded[13..]).map_err(|_| ())
}
