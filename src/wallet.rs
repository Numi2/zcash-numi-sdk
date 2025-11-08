//! Wallet management functionality

use crate::error::{Error, Result};
use crate::types::{Balance, Network};
use dirs;
use getrandom::getrandom;
use rand::thread_rng;
use secrecy::SecretVec;
use std::path::PathBuf;
use zcash_client_backend::data_api::{wallet::ConfirmationsPolicy, WalletRead};
use zcash_client_sqlite::{util::SystemClock, wallet::init::init_wallet_db, WalletDb};
use zcash_keys::encoding::AddressCodec;
use zcash_keys::keys::{
	ReceiverRequirement,
	ReceiverRequirements,
	UnifiedAddressRequest,
	UnifiedFullViewingKey,
	UnifiedSpendingKey,
};
use zcash_protocol::consensus::{MainNetwork, Network as ConsensusNetwork, TestNetwork};
use zip32::{AccountId, DiversifierIndex};

/// Wallet structure for managing Zcash addresses and keys
pub struct Wallet {
    db_path: PathBuf,
    network: Network,
    seed: Vec<u8>,
    account_id: AccountId,
}

impl Wallet {
    /// Create a new wallet with a random seed
    pub fn new() -> Result<Self> {
        Self::with_path(
            dirs::data_dir()
                .ok_or_else(|| {
                    Error::InvalidParameter("Cannot determine data directory".to_string())
                })?
                .join("zcash-numi-sdk")
                .join("wallet.db"),
        )
    }

    /// Create a new wallet with a custom database path
    pub fn with_path(db_path: PathBuf) -> Result<Self> {
        Self::with_path_and_seed(db_path, None)
    }

    /// Create a new wallet with a custom database path and seed
    pub fn with_path_and_seed(db_path: PathBuf, seed: Option<Vec<u8>>) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let seed_bytes = match seed {
            Some(bytes) => bytes,
            None => {
                let mut generated = vec![0u8; 32];
                getrandom(&mut generated).map_err(|e| {
                    Error::KeyDerivation(format!("Failed to generate wallet seed: {}", e))
                })?;
                generated
            }
        };

        let wallet = Wallet {
            db_path,
            network: Network::default(),
            seed: seed_bytes,
            account_id: AccountId::ZERO,
        };

        wallet.initialize_database()?;

        Ok(wallet)
    }

    /// Create a wallet from an existing seed
    pub fn from_seed(seed: Vec<u8>) -> Result<Self> {
        let db_path = dirs::data_dir()
            .ok_or_else(|| Error::InvalidParameter("Cannot determine data directory".to_string()))?
            .join("zcash-numi-sdk")
            .join("wallet.db");

        Self::with_path_and_seed(db_path, Some(seed))
    }

    pub(crate) fn consensus_network(&self) -> ConsensusNetwork {
        match self.network {
            Network::Mainnet => ConsensusNetwork::MainNetwork,
            Network::Testnet | Network::Regtest => ConsensusNetwork::TestNetwork,
        }
    }

    fn open_initialized_wallet_db(
        &self,
    ) -> Result<WalletDb<rusqlite::Connection, ConsensusNetwork, SystemClock, rand::rngs::ThreadRng>>
    {
        let mut wallet_db = WalletDb::for_path(
            &self.db_path,
            self.consensus_network(),
            SystemClock,
            thread_rng(),
        )
        .map_err(|e| Error::Database(format!("Failed to open wallet database: {}", e)))?;

        init_wallet_db(&mut wallet_db, Some(SecretVec::new(self.seed.clone())))
            .map_err(|e| Error::Database(format!("Failed to initialize wallet database: {}", e)))?;

        Ok(wallet_db)
    }

    fn initialize_database(&self) -> Result<()> {
        self.open_initialized_wallet_db().map(|_| ())
    }

    /// Set the network for this wallet
    pub fn set_network(&mut self, network: Network) {
        self.network = network;
    }

    /// Get the current network
    pub fn network(&self) -> Network {
        self.network
    }

    /// Get the unified spending key for this wallet
    fn get_unified_spending_key(&self) -> Result<UnifiedSpendingKey> {
        match self.network {
            Network::Mainnet => {
                UnifiedSpendingKey::from_seed(&MainNetwork, &self.seed, self.account_id)
            }
            Network::Testnet => {
                UnifiedSpendingKey::from_seed(&TestNetwork, &self.seed, self.account_id)
            }
            Network::Regtest => {
                UnifiedSpendingKey::from_seed(&TestNetwork, &self.seed, self.account_id)
            }
        }
        .map_err(|e| Error::KeyDerivation(format!("Failed to derive unified spending key: {}", e)))
    }

    /// Get the unified full viewing key for this wallet
    fn get_unified_full_viewing_key(&self) -> Result<UnifiedFullViewingKey> {
        let usk = self.get_unified_spending_key()?;
        Ok(usk.to_unified_full_viewing_key())
    }

    /// Get the unified full viewing key (public method for light client use)
    pub fn unified_full_viewing_key(&self) -> Result<UnifiedFullViewingKey> {
        self.get_unified_full_viewing_key()
    }

    /// Generate a new unified address
    pub fn get_unified_address(&self) -> Result<String> {
        let ufvk = self.get_unified_full_viewing_key()?;
        let (ua, _) = ufvk
            .default_address(UnifiedAddressRequest::ALLOW_ALL)
            .map_err(|e| Error::Address(format!("Failed to generate unified address: {}", e)))?;

        match self.network {
            Network::Mainnet => Ok(ua.encode(&MainNetwork)),
            Network::Testnet => Ok(ua.encode(&TestNetwork)),
            Network::Regtest => Ok(ua.encode(&TestNetwork)),
        }
    }
}

/// ZIP-316 policy for Unified Address receiver selection
///
/// Policies align with priority rules:
/// - Prefer Orchard, otherwise Sapling; optionally include/exclude transparent
/// - Shielded-only variants never include transparent receivers
pub enum Zip316ReceiverPolicy {
	/// Require Orchard if available; allow Sapling; omit transparent
	OrchardPreferred,
	/// Require Sapling; omit Orchard and transparent
	SaplingOnly,
	/// Allow any shielded (Orchard or Sapling); omit transparent
	ShieldedOnly,
	/// Prefer shielded; allow transparent if present
	AllowTransparent,
}

impl Wallet {
	/// Generate a unified address using a ZIP-316 receiver policy
	pub fn get_unified_address_with_policy(&self, policy: Zip316ReceiverPolicy) -> Result<String> {
		let ufvk = self.get_unified_full_viewing_key()?;

		// Map policy to receiver requirements
		let reqs = match policy {
			Zip316ReceiverPolicy::OrchardPreferred => {
				// Require Orchard, allow Sapling, omit transparent
				ReceiverRequirements::new(
					ReceiverRequirement::Require,
					ReceiverRequirement::Allow,
					ReceiverRequirement::Omit,
				)
				.map_err(|_| Error::Address("Invalid receiver requirement combination".to_string()))?
			}
			Zip316ReceiverPolicy::SaplingOnly => {
				ReceiverRequirements::new(
					ReceiverRequirement::Omit,
					ReceiverRequirement::Require,
					ReceiverRequirement::Omit,
				)
				.map_err(|_| Error::Address("Invalid receiver requirement combination".to_string()))?
			}
			Zip316ReceiverPolicy::ShieldedOnly => {
				// Any shielded allowed, no transparent
				ReceiverRequirements::SHIELDED
			}
			Zip316ReceiverPolicy::AllowTransparent => {
				// Prefer shielded but allow transparent if present
				ReceiverRequirements::ALLOW_ALL
			}
		};

		let (ua, _) = ufvk
			.default_address(UnifiedAddressRequest::Custom(reqs))
			.map_err(|e| Error::Address(format!("Failed to generate unified address: {}", e)))?;

		match self.network {
			Network::Mainnet => Ok(ua.encode(&MainNetwork)),
			Network::Testnet => Ok(ua.encode(&TestNetwork)),
			Network::Regtest => Ok(ua.encode(&TestNetwork)),
		}
	}

    /// Get a Sapling address
    pub fn get_sapling_address(&self) -> Result<String> {
        let ufvk = self.get_unified_full_viewing_key()?;
        let sapling_dfvk = ufvk
            .sapling()
            .ok_or_else(|| Error::Address("No Sapling component in unified key".to_string()))?;

        let sapling_address = sapling_dfvk
            .address(DiversifierIndex::new())
            .ok_or_else(|| Error::Address("Failed to generate Sapling address".to_string()))?;

        match self.network {
            Network::Mainnet => Ok(sapling_address.encode(&MainNetwork)),
            Network::Testnet => Ok(sapling_address.encode(&TestNetwork)),
            Network::Regtest => Ok(sapling_address.encode(&TestNetwork)),
        }
    }

    /// Get an Orchard address
    pub fn get_orchard_address(&self) -> Result<String> {
        // Orchard addresses are best accessed through unified addresses
        // Return the unified address which includes Orchard support
        self.get_unified_address()
    }

    /// Get a transparent address
    pub fn get_transparent_address(&self) -> Result<String> {
        let ufvk = self.get_unified_full_viewing_key()?;
        let transparent_dfvk = ufvk
            .transparent()
            .ok_or_else(|| Error::Address("No transparent component in unified key".to_string()))?;

        let external_ivk = transparent_dfvk
            .derive_external_ivk()
            .map_err(|e| Error::Address(format!("Failed to derive external IVK: {}", e)))?;

        use zcash_transparent::keys::IncomingViewingKey;
        let (transparent_address, _) = external_ivk.default_address();

        match self.network {
            Network::Mainnet => Ok(transparent_address.encode(&MainNetwork)),
            Network::Testnet => Ok(transparent_address.encode(&TestNetwork)),
            Network::Regtest => Ok(transparent_address.encode(&TestNetwork)),
        }
    }

    /// Get the current balance
    pub fn get_balance(&self) -> Result<Balance> {
        let wallet_db = self.open_initialized_wallet_db()?;

        let summary = wallet_db
            .get_wallet_summary(ConfirmationsPolicy::default())
            .map_err(|e| Error::Database(format!("Failed to read wallet summary: {}", e)))?;

        if let Some(summary) = summary {
            let mut transparent_total = 0u64;
            let mut sapling_total = 0u64;
            let mut orchard_total = 0u64;

            for account_balance in summary.account_balances().values() {
                transparent_total = transparent_total
                    .checked_add(u64::from(account_balance.unshielded_balance().total()))
                    .ok_or_else(|| {
                        Error::Wallet("Transparent balance exceeds u64 range".to_string())
                    })?;

                sapling_total = sapling_total
                    .checked_add(u64::from(account_balance.sapling_balance().total()))
                    .ok_or_else(|| {
                        Error::Wallet("Sapling balance exceeds u64 range".to_string())
                    })?;

                orchard_total = orchard_total
                    .checked_add(u64::from(account_balance.orchard_balance().total()))
                    .ok_or_else(|| {
                        Error::Wallet("Orchard balance exceeds u64 range".to_string())
                    })?;
            }

            let total = transparent_total
                .checked_add(sapling_total)
                .and_then(|value| value.checked_add(orchard_total))
                .ok_or_else(|| Error::Wallet("Total balance exceeds u64 range".to_string()))?;

            Ok(Balance {
                transparent: transparent_total,
                sapling: sapling_total,
                orchard: orchard_total,
                total,
            })
        } else {
            Ok(Balance::default())
        }
    }

    /// Get transaction history
    ///
    /// Retrieves transaction history from the wallet database using zcash_client_backend APIs.
    /// 
    /// Note: Full transaction history retrieval requires scanning the blockchain and
    /// maintaining transaction metadata. For production use, consider using zcashd RPC
    /// methods like `z_listreceivedbyaddress` or `z_viewtransaction` for transaction details.
    pub fn get_transactions(
        &self,
        _limit: Option<usize>,
    ) -> Result<Vec<crate::types::Transaction>> {
        // TODO: Implement full transaction history using zcash_client_backend APIs
        // This requires:
        // 1. Scanning blocks with viewing keys
        // 2. Maintaining transaction metadata in the wallet database
        // 3. Querying received/sent notes per transaction
        // 
        // For now, return empty vector. Use RPC client methods for transaction queries.
        Ok(Vec::new())
    }

    /// Get the wallet database handle for advanced operations
    ///
    /// This provides direct access to the underlying WalletDb for use with
    /// zcash_client_backend APIs that require WalletRead/WalletWrite traits.
    pub fn wallet_db(
        &self,
    ) -> Result<WalletDb<rusqlite::Connection, ConsensusNetwork, SystemClock, rand::rngs::ThreadRng>>
    {
        self.open_initialized_wallet_db()
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new().expect("Failed to create default wallet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_wallet.db");
        let wallet = Wallet::with_path(db_path.clone()).unwrap();
        assert_eq!(wallet.network(), Network::Mainnet);
    }
}
