//! Client implementations for connecting to Zcash infrastructure
use crate::error::{Error, Result};
use crate::rpc::{
    AddressInfo, BlockchainInfo, Payment, RpcRequest, RpcResponse, TransactionDetails,
};
use rand::random;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// RPC client for connecting to `zcashd` nodes.
///
/// This client implements the official Zcash Payment API, which extends
/// Bitcoin-compatible RPC calls with Zcash-specific methods for shielded operations.
pub struct RpcClient {
    endpoint: String,
    http: reqwest::Client,
    auth: Option<String>,
}

impl RpcClient {
    /// Create a new RPC client without authentication.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            http: reqwest::Client::new(),
            auth: None,
        }
    }

    /// Create a new RPC client with HTTP basic authentication.
    ///
    /// This is the standard authentication method for zcashd RPC endpoints.
    pub fn with_auth(endpoint: impl Into<String>, username: String, password: String) -> Self {
        use base64::Engine;
        let mut client = Self::new(endpoint);
        let credentials = format!("{}:{}", username, password);
        client.auth = Some(base64::engine::general_purpose::STANDARD.encode(credentials));
        client
    }

    /// Call a JSON-RPC method and deserialize the result into the requested type.
    ///
    /// This is the low-level method for making RPC calls. Prefer using the
    /// typed convenience methods when available.
    pub async fn call<T, P>(&self, method: &str, params: P) -> Result<T>
    where
        T: DeserializeOwned,
        P: Serialize,
    {
        let params = serde_json::to_value(params)?;
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: random::<u64>(),
            method: method.to_string(),
            params,
        };

        let mut req = self
            .http
            .post(&self.endpoint)
            .json(&request)
            .header("Content-Type", "application/json");

        if let Some(ref auth) = self.auth {
            req = req.header("Authorization", format!("Basic {}", auth));
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            return Err(Error::Rpc(format!(
                "RPC request failed with status: {}",
                response.status()
            )));
        }

        let rpc_response: RpcResponse<T> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(Error::Rpc(format!(
                "RPC error {}: {}",
                error.code, error.message
            )));
        }

        rpc_response
            .result
            .ok_or_else(|| Error::Rpc("RPC response missing result".to_string()))
    }

    // ============================================================================
    // Bitcoin-Compatible RPC Methods
    // ============================================================================

    /// Get blockchain information (Bitcoin-compatible).
    ///
    /// Returns information about the blockchain state including chain name,
    /// block height, difficulty, and sync status.
    pub async fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        self.call("getblockchaininfo", serde_json::json!([])).await
    }

    /// Get the raw blockchain info as JSON value.
    pub async fn get_blockchain_info_raw(&self) -> Result<serde_json::Value> {
        self.call("getblockchaininfo", serde_json::json!([])).await
    }

    /// Get block hash for a given block height.
    pub async fn get_block_hash(&self, height: u64) -> Result<String> {
        self.call("getblockhash", serde_json::json!([height])).await
    }

    /// Get block information by hash.
    pub async fn get_block(&self, hash: &str) -> Result<serde_json::Value> {
        self.call("getblock", serde_json::json!([hash])).await
    }

    /// Get the current block count.
    pub async fn get_block_count(&self) -> Result<u64> {
        self.call("getblockcount", serde_json::json!([])).await
    }

    /// Get network information.
    pub async fn get_network_info(&self) -> Result<serde_json::Value> {
        self.call("getnetworkinfo", serde_json::json!([])).await
    }

    // ============================================================================
    // Zcash-Specific Shielded RPC Methods (Zcash Payment API)
    // ============================================================================

    /// Get a new shielded address (Unified Address).
    ///
    /// Generates a new Unified Address that supports Sapling, Orchard, and
    /// transparent receivers.
    ///
    /// # Arguments
    /// * `account` - Optional account name (defaults to empty string)
    /// * `address_type` - Optional address type: "unified", "sapling", "orchard", or "transparent"
    pub async fn z_getnewaddress(
        &self,
        account: Option<&str>,
        address_type: Option<&str>,
    ) -> Result<String> {
        let mut params = vec![];
        if let Some(acc) = account {
            params.push(serde_json::json!(acc));
            if let Some(addr_type) = address_type {
                params.push(serde_json::json!(addr_type));
            }
        } else if let Some(addr_type) = address_type {
            params.push(serde_json::json!(""));
            params.push(serde_json::json!(addr_type));
        }
        self.call("z_getnewaddress", params).await
    }

    /// Get the balance for a shielded address.
    ///
    /// Returns the balance for a given shielded address (Unified, Sapling, or Orchard).
    /// For Unified Addresses, returns the total balance across all pools.
    ///
    /// # Arguments
    /// * `address` - The shielded address to query
    /// * `minconf` - Minimum confirmations (default: 1)
    pub async fn z_getbalance(&self, address: &str, minconf: Option<u32>) -> Result<f64> {
        let params = if let Some(conf) = minconf {
            serde_json::json!([address, conf])
        } else {
            serde_json::json!([address])
        };
        self.call("z_getbalance", params).await
    }

    /// Get the total balance for all addresses in the wallet.
    ///
    /// Returns the total balance across all shielded addresses in the wallet.
    ///
    /// # Arguments
    /// * `minconf` - Minimum confirmations (default: 1)
    /// * `include_watchonly` - Include watch-only addresses (default: false)
    pub async fn z_gettotalbalance(
        &self,
        minconf: Option<u32>,
        include_watchonly: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut params = vec![];
        if let Some(conf) = minconf {
            params.push(serde_json::json!(conf));
            if let Some(watchonly) = include_watchonly {
                params.push(serde_json::json!(watchonly));
            }
        }
        self.call("z_gettotalbalance", params).await
    }

    /// List all addresses in the wallet.
    ///
    /// Returns a list of all addresses (shielded and transparent) in the wallet
    /// with their associated information.
    pub async fn z_listaddresses(&self) -> Result<Vec<AddressInfo>> {
        self.call("z_listaddresses", serde_json::json!([])).await
    }

    /// View transaction details.
    ///
    /// Returns detailed information about a transaction, including shielded
    /// transaction details if applicable.
    ///
    /// # Arguments
    /// * `txid` - Transaction ID to view
    pub async fn z_viewtransaction(&self, txid: &str) -> Result<TransactionDetails> {
        self.call("z_viewtransaction", serde_json::json!([txid]))
            .await
    }

    /// Send funds to multiple recipients (Zcash Payment API).
    ///
    /// This is the primary method for sending shielded transactions. It supports
    /// sending to Unified Addresses, Sapling addresses, Orchard addresses, and
    /// transparent addresses.
    ///
    /// # Arguments
    /// * `from_address` - Source address (must be in wallet)
    /// * `payments` - Vector of payments to send
    /// * `minconf` - Minimum confirmations for source funds (default: 1)
    /// * `fee` - Optional transaction fee in ZEC
    ///
    /// # Returns
    /// Operation ID (string) that can be used to check transaction status
    pub async fn z_sendmany(
        &self,
        from_address: &str,
        payments: Vec<Payment>,
        minconf: Option<u32>,
        fee: Option<f64>,
    ) -> Result<String> {
        let mut params = vec![serde_json::json!(from_address)];
        
        let payment_json: Vec<serde_json::Value> = payments
            .into_iter()
            .map(|p| {
                let mut payment_obj = serde_json::json!({
                    "address": p.address,
                    "amount": p.amount
                });
                if let Some(memo) = p.memo {
                    payment_obj["memo"] = serde_json::json!(memo);
                }
                payment_obj
            })
            .collect();
        params.push(serde_json::json!(payment_json));

        if let Some(conf) = minconf {
            params.push(serde_json::json!(conf));
            if let Some(fee_amount) = fee {
                params.push(serde_json::json!(fee_amount));
            }
        } else if let Some(fee_amount) = fee {
            params.push(serde_json::json!(1)); // default minconf
            params.push(serde_json::json!(fee_amount));
        }

        self.call("z_sendmany", params).await
    }

    /// Get the status of a z_sendmany operation.
    ///
    /// # Arguments
    /// * `operation_id` - The operation ID returned by z_sendmany
    pub async fn z_getoperationstatus(
        &self,
        operation_id: &str,
    ) -> Result<Vec<serde_json::Value>> {
        self.call("z_getoperationstatus", serde_json::json!([[operation_id]]))
            .await
    }

    /// Get the result of a z_sendmany operation.
    ///
    /// # Arguments
    /// * `operation_id` - The operation ID returned by z_sendmany
    pub async fn z_getoperationresult(
        &self,
        operation_id: &str,
    ) -> Result<Vec<serde_json::Value>> {
        self.call("z_getoperationresult", serde_json::json!([[operation_id]]))
            .await
    }

    /// List all pending z_sendmany operations.
    pub async fn z_listoperationids(&self) -> Result<Vec<String>> {
        self.call("z_listoperationids", serde_json::json!([])).await
    }

    /// Get notes for a shielded address.
    ///
    /// Returns all notes (UTXOs) for a given shielded address.
    ///
    /// # Arguments
    /// * `address` - The shielded address
    /// * `minconf` - Minimum confirmations (default: 1)
    pub async fn z_listnotes(
        &self,
        address: &str,
        minconf: Option<u32>,
    ) -> Result<Vec<serde_json::Value>> {
        let params = if let Some(conf) = minconf {
            serde_json::json!([address, conf])
        } else {
            serde_json::json!([address])
        };
        self.call("z_listnotes", params).await
    }

    /// Get received notes for a shielded address.
    ///
    /// Returns all received notes for a given shielded address.
    ///
    /// # Arguments
    /// * `address` - The shielded address
    /// * `minconf` - Minimum confirmations (default: 1)
    pub async fn z_listreceivedbyaddress(
        &self,
        address: &str,
        minconf: Option<u32>,
    ) -> Result<Vec<serde_json::Value>> {
        let params = if let Some(conf) = minconf {
            serde_json::json!([address, conf])
        } else {
            serde_json::json!([address])
        };
        self.call("z_listreceivedbyaddress", params).await
    }

    // ============================================================================
    // Convenience Methods (Backward Compatibility)
    // ============================================================================

    /// Retrieve the shielded balance for an address using `z_getbalance`.
    ///
    /// This is a convenience wrapper that uses default minconf=1.
    /// For more control, use `z_getbalance` directly.
    #[deprecated(note = "Use z_getbalance instead for more control")]
    pub async fn get_balance(&self, address: &str) -> Result<f64> {
        self.z_getbalance(address, None).await
    }
}
