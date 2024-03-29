use super::pay::CryptoPayErrorObject;

#[derive(Debug, thiserror::Error)]
pub(crate) enum GameSdkError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Crypto Pay Error: {0:?}")]
    CryptoPayError(CryptoPayErrorObject),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid wallet id")]
    InvalidWalletId,
}
