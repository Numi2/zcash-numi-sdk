//! Common types and data structures for the Zcash Numi SDK

use serde::{Deserialize, Serialize};

/// Network type (Mainnet, Testnet, or Regtest)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Network {
    #[default]
    Mainnet,
    Testnet,
    Regtest,
}

/// Address type supported by Zcash
/// Addresses are stored as strings for serialization compatibility
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    Unified(String),
    Sapling(String),
    Orchard(String),
    Transparent(String),
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed { height: u64 },
    Rejected,
}

/// Balance information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Balance {
    pub transparent: u64,
    pub sapling: u64,
    pub orchard: u64,
    pub total: u64,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub status: TransactionStatus,
    pub amount: i64, // Negative for sent, positive for received
    pub fee: u64,
    pub memo: Option<String>,
    pub timestamp: Option<u64>,
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub time: u64,
    pub size: u64,
}

/// Utility functions for Zcash amounts
pub mod utils {
    /// Convert zatoshis (smallest unit) to ZEC
    ///
    /// # Arguments
    /// * `zatoshis` - Amount in zatoshis (1 ZEC = 100,000,000 zatoshis)
    ///
    /// # Returns
    /// Amount in ZEC as f64
    pub fn zatoshis_to_zec(zatoshis: u64) -> f64 {
        zatoshis as f64 / 100_000_000.0
    }

    /// Convert ZEC to zatoshis (smallest unit)
    ///
    /// # Arguments
    /// * `zec` - Amount in ZEC
    ///
    /// # Returns
    /// Amount in zatoshis as u64
    ///
    /// # Panics
    /// Panics if the amount exceeds u64::MAX when converted to zatoshis
    pub fn zec_to_zatoshis(zec: f64) -> u64 {
        (zec * 100_000_000.0) as u64
    }

    /// Format ZEC amount as a string with proper decimal places
    ///
    /// # Arguments
    /// * `zec` - Amount in ZEC
    ///
    /// # Returns
    /// Formatted string (e.g., "1.23456789 ZEC")
    pub fn format_zec(zec: f64) -> String {
        format!("{:.8} ZEC", zec)
    }

    /// Format zatoshis amount as a string
    ///
    /// # Arguments
    /// * `zatoshis` - Amount in zatoshis
    ///
    /// # Returns
    /// Formatted string (e.g., "123456789 zatoshis")
    pub fn format_zatoshis(zatoshis: u64) -> String {
        format!("{} zatoshis", zatoshis)
    }
}
