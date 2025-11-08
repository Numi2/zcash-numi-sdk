use thiserror::Error;

/// Error types for the Zcash Numi SDK
#[derive(Error, Debug)]
pub enum Error {
    #[error("Zcash protocol error: {0}")]
    Protocol(String),

    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Address parsing error: {0}")]
    Address(String),

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result type alias for SDK operations
pub type Result<T> = std::result::Result<T, Error>;
