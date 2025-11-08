//! Compliance and reporting helpers
//!
//! This module provides:
//! - Exportable viewing keys for compliance reviews
//! - Redaction utilities for safe logging/sharing
//! - CSV export for audit/reporting workflows
//
use crate::error::{Error, Result};
use crate::types::Transaction;
use crate::wallet::Wallet;
use zcash_keys::encoding::AddressCodec;
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus::{MainNetwork, TestNetwork};
use zcash_transparent::keys::IncomingViewingKey;
use zip32::DiversifierIndex;
//
/// Export unified full viewing key (UFVK) and component viewing keys, if available.
pub struct ExportedViewingKeys {
	/// Unified Full Viewing Key (encoded)
	pub ufvk: String,
	/// Sapling Diversified Full Viewing Key (encoded), if present
	pub sapling_fvk: Option<String>,
	/// Transparent external Incoming Viewing Key (encoded), if present
	pub transparent_ivk: Option<String>,
}
//
/// Export viewing keys from the provided wallet for the currently set network.
pub fn export_viewing_keys(wallet: &Wallet) -> Result<ExportedViewingKeys> {
	let ufvk: UnifiedFullViewingKey = wallet
		.unified_full_viewing_key()
		.map_err(|e| Error::KeyDerivation(format!("Failed to get UFVK: {}", e)))?;
	//
	let ufvk_str = match wallet.network() {
		crate::types::Network::Mainnet => ufvk.encode(&MainNetwork),
		crate::types::Network::Testnet | crate::types::Network::Regtest => ufvk.encode(&TestNetwork),
	};
	//
	// Sapling DFVK (encode representative address for attestation)
	let sapling_fvk = ufvk.sapling().map(|dfvk| {
		// Export an address derived from the DFVK for verification (not secret)
		dfvk
			.address(DiversifierIndex::new())
			.and_then(|addr| {
				Some(match wallet.network() {
					crate::types::Network::Mainnet => addr.encode(&MainNetwork),
					crate::types::Network::Testnet | crate::types::Network::Regtest => addr.encode(&TestNetwork),
				})
			})
	}).flatten();
	//
	// Transparent IVK (encode default external address for attestation)
	let transparent_ivk = ufvk.transparent().and_then(|dfvk| {
		let external_ivk = dfvk.derive_external_ivk().ok()?;
		let (addr, _) = external_ivk.default_address();
		Some(match wallet.network() {
			crate::types::Network::Mainnet => addr.encode(&MainNetwork),
			crate::types::Network::Testnet | crate::types::Network::Regtest => addr.encode(&TestNetwork),
		})
	});
	//
	Ok(ExportedViewingKeys {
		ufvk: ufvk_str,
		sapling_fvk,
		transparent_ivk,
	})
}
//
/// Redact a Zcash address or key for safe display/logging.
///
/// Keeps the first N and last M visible characters, replaces the middle with '…'.
pub fn redact_middle(input: &str, keep_start: usize, keep_end: usize) -> String {
	if input.len() <= keep_start + keep_end + 1 {
		return input.to_string();
	}
	let start = &input[..keep_start];
	let end = &input[input.len() - keep_end..];
	format!("{start}…{end}")
}
//
/// Export transactions to a simple CSV for audits.
///
/// Columns: txid, status, height, amount_zec, fee_zec, memo
pub fn export_transactions_csv(transactions: &[Transaction]) -> String {
	let mut out = String::from("txid,status,height,amount_zec,fee_zec,memo\n");
	for tx in transactions {
		let (status, height) = match &tx.status {
			crate::types::TransactionStatus::Pending => ("pending".to_string(), "".to_string()),
			crate::types::TransactionStatus::Confirmed { height } => ("confirmed".to_string(), height.to_string()),
			crate::types::TransactionStatus::Rejected => ("rejected".to_string(), "".to_string()),
		};
		let amount_zec = (tx.amount as f64) / 100_000_000.0;
		let fee_zec = (tx.fee as f64) / 100_000_000.0;
		let memo = tx.memo.clone().unwrap_or_default().replace(',', ";");
		out.push_str(&format!("{},{},{},{:.8},{:.8},{}\n", tx.txid, status, height, amount_zec, fee_zec, memo));
	}
	out
}
//
#[cfg(test)]
mod tests {
	use super::*;
	//
	#[test]
	fn test_redact_middle() {
		let s = "zs1abcdefghijklmnopqrstuvwx1234567890";
		let r = redact_middle(s, 6, 6);
		assert!(r.starts_with("zs1abc"));
		assert!(r.ends_with("67890"));
		assert!(r.contains('…'));
	}
}


