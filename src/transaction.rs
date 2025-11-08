//! Transaction building and sending functionality
//!
//! This module provides transaction building capabilities for Zcash using the
//! official Zcash Payment API (z_sendmany) via RPC, which is the recommended
//! approach for new integrations according to the Zcash Integration Guide.

use crate::address::{is_shielded_address, parse_address};
use crate::client::RpcClient;
use crate::error::{Error, Result};
use crate::fees::{calculate_fee_from_payments, fee_zatoshis_to_zec};
use crate::rpc::Payment;
use crate::wallet::Wallet;

/// Maximum memo size in bytes (Zcash protocol limit)
const MAX_MEMO_SIZE: usize = 512;

/// Maximum ZEC amount (sanity check - 21 million ZEC total supply)
const MAX_ZEC_AMOUNT: f64 = 21_000_000.0;

/// Transaction builder for creating and sending Zcash transactions
///
/// This builder uses the official Zcash Payment API (z_sendmany) which handles
/// all the complexity of note selection, fee calculation, proof generation,
/// and transaction signing automatically via zcashd.
pub struct TransactionBuilder {
    wallet: Wallet,
    rpc_client: Option<RpcClient>,
}

impl TransactionBuilder {
    /// Create a new transaction builder for a wallet
    ///
    /// The builder can work with or without an RPC client. If no RPC client
    /// is provided, transactions can be built but not sent until one is set.
    pub fn new(wallet: Wallet) -> Self {
        TransactionBuilder {
            wallet,
            rpc_client: None,
        }
    }

    /// Create a new transaction builder with an RPC client
    ///
    /// This allows immediate sending of transactions via zcashd RPC.
    pub fn with_rpc_client(wallet: Wallet, rpc_client: RpcClient) -> Self {
        TransactionBuilder {
            wallet,
            rpc_client: Some(rpc_client),
        }
    }

    /// Set the RPC client for sending transactions
    pub fn set_rpc_client(&mut self, rpc_client: RpcClient) {
        self.rpc_client = Some(rpc_client);
    }

    /// Estimate ZIP-317 fee for a transaction based on payments
    ///
    /// This estimates the fee using ZIP-317 fee calculation:
    /// fee = 5000 zatoshis × max(2, logical_actions)
    ///
    /// Logical actions include note spends, note outputs, and transparent inputs/outputs.
    ///
    /// # Arguments
    /// * `payments` - Vector of payments to be included in the transaction
    /// * `from_address` - Source address (to determine if it's shielded)
    ///
    /// # Returns
    /// Estimated fee in ZEC
    ///
    /// # Note
    /// This is an estimation. The actual fee will be calculated by zcashd based on
    /// the final transaction structure. This method is useful for pre-validation
    /// and user feedback before sending the transaction.
    ///
    /// See [ZIP-317](https://zips.z.cash/zip-0317) for detailed fee parameters.
    pub fn estimate_fee(&self, payments: &[Payment], from_address: &str) -> Result<f64> {
        let network = self.wallet.consensus_network();
        let has_shielded_input = is_shielded_address(from_address, network)?;
        
        let fee_zatoshis = calculate_fee_from_payments(payments, has_shielded_input);
        Ok(fee_zatoshis_to_zec(fee_zatoshis))
    }

    /// Build and send a transaction to one or more recipients using z_sendmany
    ///
    /// This uses the official Zcash Payment API which is the recommended approach
    /// for sending shielded transactions according to the Zcash Integration Guide.
    ///
    /// # Arguments
    /// * `from_address` - Source address (must be in the wallet managed by zcashd)
    /// * `payments` - Vector of payments to send
    /// * `minconf` - Minimum confirmations for source funds (default: 1)
    /// * `fee` - Optional transaction fee in ZEC
    ///
    /// # Returns
    /// Operation ID (string) that can be used to check transaction status
    ///
    /// # Note
    /// This method requires a zcashd node with the source address in its wallet.
    /// For light client transactions, use the lightwalletd integration instead.
    pub async fn send_many(
        &self,
        from_address: &str,
        payments: Vec<Payment>,
        minconf: Option<u32>,
        fee: Option<f64>,
    ) -> Result<String> {
        let rpc_client = self
            .rpc_client
            .as_ref()
            .ok_or_else(|| Error::Transaction("RPC client not configured".to_string()))?;

        // Validate the from address format
        let network = self.wallet.consensus_network();
        parse_address(from_address, network)?;

        // Validate all payment addresses and payments
        for (idx, payment) in payments.iter().enumerate() {
            // Validate address format
            parse_address(&payment.address, network)?;

            // Validate amount
            if payment.amount <= 0.0 {
                return Err(Error::Transaction(format!(
                    "Payment {} has invalid amount: {} ZEC (must be positive)",
                    idx, payment.amount
                )));
            }
            if payment.amount > MAX_ZEC_AMOUNT {
                return Err(Error::Transaction(format!(
                    "Payment {} has excessive amount: {} ZEC (max: {} ZEC)",
                    idx, payment.amount, MAX_ZEC_AMOUNT
                )));
            }

            // Validate memo
            if let Some(ref memo) = payment.memo {
                // Check memo size (512 bytes max)
                let memo_bytes = memo.as_bytes();
                if memo_bytes.len() > MAX_MEMO_SIZE {
                    return Err(Error::Transaction(format!(
                        "Payment {} has memo exceeding {} bytes: {} bytes",
                        idx, MAX_MEMO_SIZE, memo_bytes.len()
                    )));
                }

                // Check if address supports memos (shielded addresses only)
                let is_shielded = is_shielded_address(&payment.address, network)?;
                if !is_shielded {
                    return Err(Error::Transaction(format!(
                        "Payment {} includes memo but recipient address is transparent (memos only supported for shielded addresses)",
                        idx
                    )));
                }
            }
        }

        rpc_client
            .z_sendmany(from_address, payments, minconf, fee)
            .await
    }

    /// Send a simple payment to a single address
    ///
    /// This is a convenience wrapper around `send_many` for single payments.
    ///
    /// # Arguments
    /// * `from_address` - Source address (must be in the wallet managed by zcashd)
    /// * `to_address` - Recipient address (Unified, Sapling, Orchard, or Transparent)
    /// * `amount_zec` - Amount to send in ZEC
    /// * `memo` - Optional memo (for shielded addresses only)
    /// * `minconf` - Minimum confirmations for source funds (default: 1)
    /// * `fee` - Optional transaction fee in ZEC
    ///
    /// # Returns
    /// Operation ID (string) that can be used to check transaction status
    pub async fn send_to_address(
        &self,
        from_address: &str,
        to_address: &str,
        amount_zec: f64,
        memo: Option<String>,
        minconf: Option<u32>,
        fee: Option<f64>,
    ) -> Result<String> {
        // Validate amount before creating payment
        if amount_zec <= 0.0 {
            return Err(Error::Transaction(format!(
                "Invalid amount: {} ZEC (must be positive)",
                amount_zec
            )));
        }
        if amount_zec > MAX_ZEC_AMOUNT {
            return Err(Error::Transaction(format!(
                "Excessive amount: {} ZEC (max: {} ZEC)",
                amount_zec, MAX_ZEC_AMOUNT
            )));
        }

        // Validate memo if provided
        if let Some(ref memo) = memo {
            let memo_bytes = memo.as_bytes();
            if memo_bytes.len() > MAX_MEMO_SIZE {
                return Err(Error::Transaction(format!(
                    "Memo exceeds {} bytes: {} bytes",
                    MAX_MEMO_SIZE, memo_bytes.len()
                )));
            }

            // Check if address supports memos
            let network = self.wallet.consensus_network();
            let is_shielded = is_shielded_address(to_address, network)?;
            if !is_shielded {
                return Err(Error::Transaction(
                    "Memo provided but recipient address is transparent (memos only supported for shielded addresses)".to_string()
                ));
            }
        }

        let payments = vec![Payment {
            address: to_address.to_string(),
            amount: amount_zec,
            memo,
        }];

        self.send_many(from_address, payments, minconf, fee).await
    }

    /// Build and send a transaction using ZIP-321 payment requests
    ///
    /// Converts ZIP-321 Payment objects to the format required by z_sendmany.
    ///
    /// # Arguments
    /// * `from_address` - Source address (must be in the wallet managed by zcashd)
    /// * `payments` - Vector of ZIP-321 payments
    /// * `minconf` - Minimum confirmations for source funds (default: 1)
    /// * `fee` - Optional transaction fee in ZEC
    ///
    /// # Returns
    /// Operation ID (string) that can be used to check transaction status
    pub async fn send_zip321(
        &self,
        from_address: &str,
        payments: Vec<zip321::Payment>,
        minconf: Option<u32>,
        fee: Option<f64>,
    ) -> Result<String> {
        let network = self.wallet.consensus_network();
        
        // Convert ZIP-321 payments to RPC Payment format
        let rpc_payments: Result<Vec<Payment>> = payments
            .into_iter()
            .enumerate()
            .map(|(idx, p)| {
                // Extract address string from ZIP-321 payment
                // zip321::Payment uses ZcashAddress which can be encoded directly
                let address_str = p.recipient_address().encode();

                // Validate address format matches network
                parse_address(&address_str, network)
                    .map_err(|e| Error::Transaction(format!(
                        "ZIP-321 payment {} has invalid address: {}",
                        idx, e
                    )))?;

                // Extract memo if present
                let memo = p.memo().and_then(|m| {
                    // Convert memo bytes to string if possible
                    // ZIP-321 memos are UTF-8 encoded
                    String::from_utf8(m.as_array().to_vec()).ok()
                });

                // Validate memo size if present
                if let Some(ref memo_str) = memo {
                    let memo_bytes = memo_str.as_bytes();
                    if memo_bytes.len() > MAX_MEMO_SIZE {
                        return Err(Error::Transaction(format!(
                            "ZIP-321 payment {} has memo exceeding {} bytes: {} bytes",
                            idx, MAX_MEMO_SIZE, memo_bytes.len()
                        )));
                    }
                }

                // Convert amount from zatoshis to ZEC
                // Zatoshis implements From<Zatoshis> for u64
                let zatoshis: u64 = p.amount().into();
                let amount_zec = zatoshis as f64 / 100_000_000.0;

                // Validate amount
                if amount_zec <= 0.0 {
                    return Err(Error::Transaction(format!(
                        "ZIP-321 payment {} has invalid amount: {} ZEC (must be positive)",
                        idx, amount_zec
                    )));
                }
                if amount_zec > MAX_ZEC_AMOUNT {
                    return Err(Error::Transaction(format!(
                        "ZIP-321 payment {} has excessive amount: {} ZEC (max: {} ZEC)",
                        idx, amount_zec, MAX_ZEC_AMOUNT
                    )));
                }

                Ok(Payment {
                    address: address_str,
                    amount: amount_zec,
                    memo,
                })
            })
            .collect();

        self.send_many(from_address, rpc_payments?, minconf, fee).await
    }

    /// Check the status of a transaction operation
    ///
    /// # Arguments
    /// * `operation_id` - The operation ID returned by send methods
    ///
    /// # Returns
    /// Vector of operation status objects
    pub async fn get_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let rpc_client = self
            .rpc_client
            .as_ref()
            .ok_or_else(|| Error::Transaction("RPC client not configured".to_string()))?;

        rpc_client.z_getoperationstatus(operation_id).await
    }

    /// Get the result of a transaction operation
    ///
    /// # Arguments
    /// * `operation_id` - The operation ID returned by send methods
    ///
    /// # Returns
    /// Vector of operation result objects (includes transaction ID when complete)
    pub async fn get_operation_result(
        &self,
        operation_id: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let rpc_client = self
            .rpc_client
            .as_ref()
            .ok_or_else(|| Error::Transaction("RPC client not configured".to_string()))?;

        rpc_client.z_getoperationresult(operation_id).await
    }

    /// Wait for a transaction operation to complete
    ///
    /// Polls the operation status until it completes or fails.
    ///
    /// # Arguments
    /// * `operation_id` - The operation ID returned by send methods
    /// * `max_wait_seconds` - Maximum time to wait in seconds (default: 300)
    ///
    /// # Returns
    /// Transaction ID when the operation completes successfully
    pub async fn wait_for_operation(
        &self,
        operation_id: &str,
        max_wait_seconds: Option<u64>,
    ) -> Result<String> {
        use std::time::Duration;
        use tokio::time::sleep;

        let max_wait = max_wait_seconds.unwrap_or(300);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed().as_secs() > max_wait {
                return Err(Error::Transaction(format!(
                    "Operation {} timed out after {} seconds",
                    operation_id, max_wait
                )));
            }

            let results = self.get_operation_result(operation_id).await?;

            for result in results {
                if let Some(status) = result.get("status") {
                    if status == "success" {
                        if let Some(txid) = result.get("txid").and_then(|t| t.as_str()) {
                            return Ok(txid.to_string());
                        }
                    } else if status == "failed" {
                        let error = result
                            .get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("Unknown error");
                        return Err(Error::Transaction(format!(
                            "Operation {} failed: {}",
                            operation_id, error
                        )));
                    }
                }
            }

            // Wait before polling again
            sleep(Duration::from_secs(2)).await;
        }
    }
}
