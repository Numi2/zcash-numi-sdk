//! # Zcash Numi SDK
//!
//! A comprehensive Rust SDK for building Zcash decentralized applications.
//!
//! This SDK provides high-level abstractions for:
//! - Wallet management (addresses, keys, balances)
//! - Transaction creation and signing
//! - RPC client integration
//! - Shielded and transparent operations
//!
//! ## Features
//!
//! - **Wallet Management**: Create and manage Zcash wallets with support for
//!   Unified Addresses, Sapling, Orchard, and transparent addresses
//! - **RPC Client**: Interact with zcashd nodes via RPC for full node operations
//! - **Transaction Building**: Create and sign shielded and transparent transactions
//! - **Address Parsing**: Parse and validate Zcash addresses (UA, Sapling, Orchard, transparent)
//!
//! ## Example
//!
//! ```no_run
//! use zcash_numi_sdk::wallet::Wallet;
//! use zcash_numi_sdk::client::RpcClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new wallet
//! let wallet = Wallet::new()?;
//!
//! // Get a receiving address
//! let address = wallet.get_unified_address()?;
//!
//! // Connect to a zcashd node via RPC
//! let client = RpcClient::new("http://localhost:8232");
//!
//! // Get blockchain info
//! let info = client.get_blockchain_info().await?;
//!
//! // Get balance
//! let balance = wallet.get_balance()?;
//! # Ok(())
//! # }
//! ```

pub mod address;
pub mod client;
pub mod error;
pub mod fees;
pub mod compliance;
pub mod light_client;
pub mod rpc;
pub mod transaction;
pub mod types;
pub mod wallet;

pub use error::{Error, Result};

/// Re-export commonly used types
pub use types::*;

/// Re-export utility functions
pub use types::utils;

/// Re-export fee calculation functions
pub use fees::{calculate_zip317_fee, calculate_fee_from_payments, fee_zatoshis_to_zec, fee_zec_to_zatoshis};

/// Re-export compliance helpers
pub use compliance::*;
