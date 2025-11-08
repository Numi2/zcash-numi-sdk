//! RPC client implementation for zcashd

use serde::{Deserialize, Serialize};

/// RPC request structure
#[derive(Debug, Serialize)]
pub(crate) struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC response structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

/// RPC error structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Payment structure for z_sendmany
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Recipient address
    pub address: String,
    /// Amount in ZEC
    pub amount: f64,
    /// Optional memo (for shielded addresses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// Blockchain info response
#[derive(Debug, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub verificationprogress: f64,
    pub chainwork: String,
    pub pruned: bool,
    pub commitments: u64,
}

/// Transaction details from z_viewtransaction
#[derive(Debug, Deserialize)]
pub struct TransactionDetails {
    pub txid: String,
    pub hex: Option<String>,
    pub fee: Option<f64>,
    pub time: Option<u64>,
    pub blocktime: Option<u64>,
    pub blockheight: Option<u64>,
    pub confirmations: Option<u64>,
    pub details: Vec<TransactionDetail>,
}

/// Transaction detail entry
#[derive(Debug, Deserialize)]
pub struct TransactionDetail {
    pub address: Option<String>,
    pub category: String,
    pub amount: f64,
    pub vout: Option<u64>,
    pub fee: Option<f64>,
    pub memo: Option<String>,
}

/// Address info from z_listaddresses
#[derive(Debug, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub account: Option<String>,
    pub label: Option<String>,
    pub balance: Option<f64>,
    pub receivedby: Option<f64>,
}
