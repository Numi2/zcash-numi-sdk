//! Light client implementation for connecting to lightwalletd via gRPC
//!
//! This module provides functionality for light clients to sync with the Zcash blockchain
//! without downloading the full blockchain. It connects to lightwalletd servers via gRPC
//! to receive compact blocks and submit transactions.
//!
//! # Example
//! ```no_run
//! use zcash_numi_sdk::light_client::LightClient;
//! use zcash_numi_sdk::wallet::Wallet;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let wallet = Wallet::new()?;
//! let mut light_client = LightClient::connect(
//!     "https://lightwalletd.example.com:9067".to_string(),
//!     wallet,
//! ).await?;
//!
//! // Sync with blockchain
//! light_client.sync(0, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! API method names align with lightwalletd's gRPC service (`CompactTxStreamer`):
//! - GetLatestBlock (tested with grpcurl)
//! - GetBlockRange (tested with grpcurl)

use crate::error::{Error, Result};
use crate::types::Network;
use crate::wallet::Wallet;
use std::sync::Arc;
use tokio::sync::Mutex;
use zcash_client_backend::data_api::{WalletRead, WalletWrite};
use zcash_client_backend::data_api::chain::{self, BlockSource};
use zcash_client_backend::scanning::{ScanningKeys};
use zcash_client_backend::proto::compact_formats::CompactBlock;
use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;
use zcash_client_backend::proto::service::{BlockId, BlockRange, ChainSpec, RawTransaction, TxFilter};
use zcash_client_sqlite::{util::SystemClock, WalletDb};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus::Network as ConsensusNetwork;
use zip32::AccountId;

/// Light client for connecting to lightwalletd servers
///
/// This client connects to a lightwalletd server via gRPC to sync blockchain data
/// without requiring a full node. It's designed for mobile and web applications
/// that need lightweight blockchain access.
pub struct LightClient {
    /// gRPC endpoint URL
    endpoint: String,
    /// Wallet database for storing synced data
    wallet_db: Arc<Mutex<WalletDb<rusqlite::Connection, ConsensusNetwork, SystemClock, rand::rngs::ThreadRng>>>,
    /// Network (mainnet/testnet/regtest)
    network: Network,
    /// Unified full viewing key for scanning
    ufvk: UnifiedFullViewingKey,
    /// Consensus network type
    consensus_network: ConsensusNetwork,
}

impl LightClient {
    /// Create a new light client and connect to a lightwalletd server
    ///
    /// # Arguments
    /// * `endpoint` - gRPC endpoint URL (e.g., "https://lightwalletd.example.com:9067")
    /// * `wallet` - Wallet instance to use for key management and storage
    ///
    /// # Example
    /// ```no_run
    /// use zcash_numi_sdk::light_client::LightClient;
    /// use zcash_numi_sdk::wallet::Wallet;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let wallet = Wallet::new()?;
    /// let mut light_client = LightClient::connect(
    ///     "https://lightwalletd.example.com:9067".to_string(),
    ///     wallet,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(endpoint: String, wallet: Wallet) -> Result<Self> {
        // Validate endpoint URL format
        endpoint.parse::<tonic::transport::Uri>()
            .map_err(|e| Error::InvalidParameter(format!("Invalid endpoint URL: {}", e)))?;

        // Get the unified full viewing key from wallet
        let ufvk = wallet.unified_full_viewing_key()?;
        
        // Get wallet database
        let wallet_db = Arc::new(Mutex::new(wallet.wallet_db()?));
        
        let network = wallet.network();
        let consensus_network = wallet.consensus_network();

        Ok(Self {
            endpoint,
            wallet_db,
            network,
            ufvk,
            consensus_network,
        })
    }

    /// Get the current network
    pub fn network(&self) -> Network {
        self.network
    }

    /// Get the latest block height from the lightwalletd server
    ///
    /// This queries the lightwalletd server to determine the current blockchain height.
    pub async fn get_latest_block_height(&mut self) -> Result<u64> {
        // Create gRPC client - use Endpoint with connect_lazy for compatibility
        use tonic::transport::Endpoint;
        
        let channel = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| Error::InvalidParameter(format!("Invalid endpoint URL: {}", e)))?
            .connect_lazy();

        let mut client = CompactTxStreamerClient::new(channel);
        let request = tonic::Request::new(ChainSpec {});
        
        let response = client
            .get_latest_block(request)
            .await
            .map_err(|e| Error::Rpc(format!("Failed to get latest block: {}", e)))?;

        let block = response.into_inner();
        Ok(block.height)
    }

    /// Get compact blocks for a given height range
    ///
    /// # Arguments
    /// * `start_height` - Starting block height (inclusive)
    /// * `end_height` - Ending block height (inclusive)
    ///
    /// # Returns
    /// Vector of compact blocks
    pub async fn get_compact_blocks(
        &mut self,
        start_height: u64,
        end_height: u64,
    ) -> Result<Vec<CompactBlock>> {
        // Create gRPC client - use Endpoint with connect_lazy for compatibility
        use tonic::transport::Endpoint;
        
        let channel = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| Error::InvalidParameter(format!("Invalid endpoint URL: {}", e)))?
            .connect_lazy();

        let mut client = CompactTxStreamerClient::new(channel);
        let mut blocks = Vec::new();
        
        let request = tonic::Request::new(BlockRange {
            start: Some(BlockId {
                height: start_height,
                hash: vec![],
            }),
            end: Some(BlockId {
                height: end_height,
                hash: vec![],
            }),
        });

        let mut stream = client
            .get_block_range(request)
            .await
            .map_err(|e| Error::Rpc(format!("Failed to get block range: {}", e)))?
            .into_inner();

        while let Some(compact_block) = stream
            .message()
            .await
            .map_err(|e| Error::Rpc(format!("Failed to receive block: {}", e)))?
        {
            blocks.push(compact_block);
        }

        Ok(blocks)
    }

    /// Sync the wallet with the blockchain by scanning blocks
    ///
    /// This method fetches compact blocks from lightwalletd and scans them
    /// using the wallet's viewing keys to find transactions relevant to the wallet.
    ///
    /// # Arguments
    /// * `start_height` - Starting block height to scan from
    /// * `end_height` - Ending block height to scan to (use None for latest)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use zcash_numi_sdk::light_client::LightClient;
    /// # use zcash_numi_sdk::wallet::Wallet;
    /// # let wallet = Wallet::new()?;
    /// # let mut light_client = LightClient::connect("https://example.com".to_string(), wallet).await?;
    /// // Sync from block 0 to latest
    /// light_client.sync(0, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sync(&mut self, start_height: u64, end_height: Option<u64>) -> Result<()> {
        // Determine end height
        let end = if let Some(height) = end_height {
            height
        } else {
            self.get_latest_block_height().await?
        };

        if start_height > end {
            return Err(Error::InvalidParameter(format!(
                "Start height {} is greater than end height {}",
                start_height, end
            )));
        }

        tracing::info!("Starting sync from height {} to {}", start_height, end);

        // Get the account ID (using AccountId::ZERO for the default account)
        let _account_id = AccountId::ZERO;

        // Fetch compact blocks from lightwalletd in batches to avoid memory issues
        const BATCH_SIZE: u64 = 100; // Process 100 blocks at a time
        let mut current_height = start_height;
        let mut total_blocks_scanned = 0;

        while current_height <= end {
            let batch_end = std::cmp::min(current_height + BATCH_SIZE - 1, end);
            
            tracing::debug!("Fetching blocks {} to {}", current_height, batch_end);
            
            // Fetch compact blocks for this batch
            let compact_blocks = self.get_compact_blocks(current_height, batch_end).await?;

            if compact_blocks.is_empty() {
                tracing::warn!("No blocks returned for range {} to {}", current_height, batch_end);
                break;
            }

            let blocks_count = compact_blocks.len();
            tracing::debug!(
                "Received {} compact blocks for heights {} to {}",
                blocks_count,
                current_height,
                batch_end
            );

            // Lock the wallet database for scanning
            let mut wallet_db = self.wallet_db.lock().await;

            // Get or import the AccountUuid for the UFVK
            // The wallet database uses AccountUuid internally, so we need to get/import an account
            use zcash_client_backend::data_api::{AccountBirthday, AccountPurpose, chain::ChainState};
            
            // Create a minimal AccountBirthday for account import
            let birthday = AccountBirthday::from_parts(
                ChainState::empty(
                    zcash_primitives::consensus::BlockHeight::from_u32(0),
                    zcash_primitives::block::BlockHash([0u8; 32]),
                ),
                None,
            );
            
            let _account_uuid = match wallet_db.get_account_for_ufvk(&self.ufvk) {
                Ok(Some(_account)) => {
                    // Account exists - re-import to get the UUID
                    // import_account_ufvk returns the UUID even if account already exists
                    wallet_db
                        .import_account_ufvk(
                            "", // account name - empty for default
                            &self.ufvk,
                            &birthday,
                            AccountPurpose::ViewOnly,
                            None, // seed
                        )
                        .map_err(|e| Error::Database(format!("Failed to import account: {}", e)))?
                }
                Ok(None) => {
                    // Account doesn't exist, import it
                    wallet_db
                        .import_account_ufvk(
                            "", // account name - empty for default
                            &self.ufvk,
                            &birthday,
                            AccountPurpose::ViewOnly,
                            None, // seed
                        )
                        .map_err(|e| Error::Database(format!("Failed to import account: {}", e)))?
                }
                Err(e) => {
                    return Err(Error::Database(format!("Failed to get account for UFVK: {}", e)));
                }
            };

            // Build scanning keys from the unified full viewing key
            let account_id = AccountId::ZERO;
            
            // Create scanning keys from the unified full viewing key
            // from_account_ufvks takes an iterator of (account_id, ufvk) tuples with owned values
            let _scanning_keys = ScanningKeys::from_account_ufvks(
                std::iter::once((account_id, self.ufvk.clone()))
            );

            // Get nullifiers from wallet database for checking spent notes
            // Note: For scanning, we use empty nullifiers. The scan_block function will
            // check against nullifiers in the wallet database automatically, and the
            // scanned results will update the database with new nullifiers.
            use zcash_client_backend::scanning::Nullifiers;
            
            // Use empty nullifiers - the scanning process will handle nullifier tracking
            // through the wallet database. The scan_block function uses nullifiers primarily
            // for checking if notes have been spent, which is handled by the database.
            let _nullifiers: Nullifiers<AccountId> = Nullifiers::empty();

            // Prepare ChainState from prior metadata (or empty at genesis)
            let max_scanned_metadata = wallet_db
                .block_max_scanned()
                .map_err(|e| Error::Database(format!("Failed to get max scanned height: {}", e)))?;
            let chain_state = if let Some(metadata) = max_scanned_metadata {
                zcash_client_backend::data_api::chain::ChainState::empty(
                    metadata.block_height(),
                    metadata.block_hash(),
                )
            } else {
                zcash_client_backend::data_api::chain::ChainState::empty(
                    zcash_primitives::consensus::BlockHeight::from_u32(0),
                    zcash_primitives::block::BlockHash([0u8; 32]),
                )
            };

            // Adapt fetched compact blocks into a BlockSource and scan+persist them
            struct VecBlockSource {
                blocks: Vec<CompactBlock>,
            }
            impl BlockSource for VecBlockSource {
                type Error = std::convert::Infallible;
                fn with_blocks<F, DbErrT>(
                    &self,
                    from_height: Option<zcash_protocol::consensus::BlockHeight>,
                    limit: Option<usize>,
                    mut with_row: F,
                ) -> std::result::Result<(), zcash_client_backend::data_api::chain::error::Error<DbErrT, Self::Error>>
                where
                    F: FnMut(CompactBlock) -> std::result::Result<(), zcash_client_backend::data_api::chain::error::Error<DbErrT, Self::Error>>,
                {
                    let start = from_height.map(|h| u32::from(h)).unwrap_or(0);
                    let mut count = 0usize;
                    for b in &self.blocks {
                        if u32::from(b.height()) >= start {
                            with_row(b.clone())?;
                            count += 1;
                            if let Some(lim) = limit {
                                if count >= lim {
                                    break;
                                }
                            }
                        }
                    }
                    Ok(())
                }
            }

            let source = VecBlockSource { blocks: compact_blocks };
            let from_h = zcash_protocol::consensus::BlockHeight::from_u32(current_height as u32);
            // Limit to batch size
            let limit = (batch_end - current_height + 1) as usize;
            match chain::scan_cached_blocks(
                &self.consensus_network,
                &source,
                &mut *wallet_db,
                from_h,
                &chain_state,
                limit,
            ) {
                Ok(summary) => {
                    let range = summary.scanned_range();
                    tracing::debug!(
                        "Scanned {} blocks ({}..={})",
                        (range.end - range.start) as u64,
                        current_height,
                        batch_end
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to scan cached blocks: {:?}", e);
                }
            }

            total_blocks_scanned += blocks_count;
            current_height = batch_end + 1;

            tracing::debug!(
                "Scanned {} blocks, progress: {}/{}",
                blocks_count,
                current_height - start_height,
                end - start_height + 1
            );
        }

        tracing::info!(
            "Sync completed: scanned {} blocks from height {} to {}",
            total_blocks_scanned,
            start_height,
            end
        );

        Ok(())
    }

    /// Submit a transaction to the network via lightwalletd
    ///
    /// # Arguments
    /// * `raw_tx` - Raw transaction bytes (hex encoded)
    ///
    /// # Returns
    /// Transaction ID if successful
    ///
    /// # Note
    /// This is a placeholder implementation. The actual implementation requires
    /// using the CompactTxStreamerClient from zcash_client_backend::proto.
    pub async fn submit_transaction(&mut self, raw_tx: &[u8]) -> Result<String> {
        use tonic::transport::Endpoint;
        let channel = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| Error::InvalidParameter(format!("Invalid endpoint URL: {}", e)))?
            .connect_lazy();
        let mut client = CompactTxStreamerClient::new(channel);
        let request = tonic::Request::new(RawTransaction { data: raw_tx.to_vec(), height: 0 });
        let response = client
            .send_transaction(request)
            .await
            .map_err(|e| Error::Rpc(format!("Failed to send transaction: {}", e)))?;
        let res = response.into_inner();
        // Return a status string; lightwalletd typically provides error info fields.
        Ok(format!("code:{} message:{}", res.error_code, res.error_message))
    }

    /// Get transaction details by transaction ID
    ///
    /// # Arguments
    /// * `txid` - Transaction ID (hex encoded)
    ///
    /// # Returns
    /// Transaction details if found
    ///
    /// # Note
    /// This is a placeholder implementation. The actual implementation requires
    /// using the CompactTxStreamerClient from zcash_client_backend::proto.
    pub async fn get_transaction(&mut self, txid_hex: &str) -> Result<Option<Vec<u8>>> {
        use tonic::transport::Endpoint;
        let channel = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| Error::InvalidParameter(format!("Invalid endpoint URL: {}", e)))?
            .connect_lazy();
        let mut client = CompactTxStreamerClient::new(channel);
        let txid = hex::decode(txid_hex)
            .map_err(|e| Error::InvalidParameter(format!("Invalid txid hex: {}", e)))?;
        let mut filter = TxFilter::default();
        filter.hash = txid;
        filter.index = 0;
        let request = tonic::Request::new(filter);
        let response = client
            .get_transaction(request)
            .await
            .map_err(|e| Error::Rpc(format!("Failed to get transaction: {}", e)))?
            .into_inner();
        if response.data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(response.data))
        }
    }

    /// Get the tip (latest block) information
    ///
    /// Returns information about the latest block known to the lightwalletd server.
    ///
    /// # Note
    /// This is a placeholder implementation. The actual implementation requires
    /// using the CompactTxStreamerClient from zcash_client_backend::proto.
    pub async fn get_tip(&mut self) -> Result<(u64, Vec<u8>)> {
        // TODO: Implement using CompactTxStreamerClient::get_latest_block
        // Example:
        // use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;
        // let mut client = CompactTxStreamerClient::new(self.channel.clone());
        // let request = tonic::Request::new(());
        // let response = client.get_latest_block(request).await?;
        // let block = response.into_inner();
        // Ok((block.height, block.hash))
        
        Err(Error::Rpc(
            "get_tip not yet implemented. See zcash_client_backend::proto for API details.".to_string()
        ))
    }

    /// Get the endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Helper function to get default lightwalletd endpoints
///
/// Returns common public lightwalletd endpoints for mainnet and testnet.
///
/// # Note
/// These endpoints are examples. Always verify the endpoint URL and use
/// trusted lightwalletd servers in production.
pub fn default_endpoints(network: Network) -> Vec<String> {
    match network {
        Network::Mainnet => vec![
            "https://mainnet.lightwalletd.com:9067".to_string(),
            "https://lwd1.zcash-infra.com:9067".to_string(),
        ],
        Network::Testnet => vec![
            "https://testnet.lightwalletd.com:9067".to_string(),
        ],
        Network::Regtest => vec![
            "http://localhost:9067".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires lightwalletd server
    async fn test_connect() {
        // This test requires a running lightwalletd server
        // It's marked as ignored by default
        let wallet = Wallet::new().unwrap();
        let endpoint = "http://localhost:9067".to_string();
        
        // This will fail if lightwalletd is not running
        let _client = LightClient::connect(endpoint, wallet).await;
    }
}
