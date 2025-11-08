//! ZIP-317 fee calculation for Zcash transactions
//!
//! This module implements the ZIP-317 Proportional Transfer Fee Mechanism,
//! which obsoletes ZIP-313. All transactions should use ZIP-317 fee calculation.
//!
//! Fee formula: 5000 zatoshis × max(2, logical_actions)
//!
//! Logical actions include:
//! - Note spends (Sapling or Orchard)
//! - Note outputs (Sapling or Orchard)
//! - Transparent inputs
//! - Transparent outputs
//!
//! See [ZIP-317](https://zips.z.cash/zip-0317) for detailed fee parameters and action accounting rules.

use crate::error::{Error, Result};
use crate::rpc::Payment;

/// ZIP-317 fee parameters
const FEE_BASE: u64 = 5000; // zatoshis per logical action
const MIN_LOGICAL_ACTIONS: u64 = 2; // minimum logical actions for fee calculation

/// Calculate ZIP-317 conventional fee for a transaction
///
/// This calculates the fee based on logical actions in the transaction.
/// The fee formula is: 5000 zatoshis × max(2, logical_actions)
///
/// # Arguments
/// * `logical_actions` - Number of logical actions in the transaction
///
/// # Returns
/// Fee in zatoshis
///
/// # Example
/// ```
/// use zcash_numi_sdk::fees::calculate_zip317_fee;
///
/// // Transaction with 3 logical actions
/// let fee = calculate_zip317_fee(3);
/// assert_eq!(fee, 15000); // 5000 * 3
///
/// // Transaction with 1 logical action (minimum is 2)
/// let fee = calculate_zip317_fee(1);
/// assert_eq!(fee, 10000); // 5000 * 2
/// ```
pub fn calculate_zip317_fee(logical_actions: u64) -> u64 {
    FEE_BASE * logical_actions.max(MIN_LOGICAL_ACTIONS)
}

/// Estimate logical actions for a transaction based on payments
///
/// This is a simplified estimation that counts:
/// - Each payment as a potential note output (shielded) or transparent output
/// - Assumes at least one input (spend or transparent input)
///
/// # Arguments
/// * `payments` - Vector of payments to be included in the transaction
/// * `has_shielded_input` - Whether the transaction will have shielded inputs
///
/// # Returns
/// Estimated number of logical actions
///
/// # Note
/// This is a simplified estimation. For accurate fee calculation, you need
/// to know the exact transaction structure including:
/// - Number of note spends
/// - Number of note outputs
/// - Number of transparent inputs
/// - Number of transparent outputs
///
/// The actual transaction builder (zcashd or light client) will calculate
/// the exact fee based on the final transaction structure.
pub fn estimate_logical_actions(payments: &[Payment], has_shielded_input: bool) -> u64 {
    // We need at least one input (spend or transparent input)
    let mut actions = 1u64;
    
    // Count outputs based on payment addresses
    // Note: This is an estimation - actual transaction may have change outputs
    for payment in payments {
        // Try to determine if address is shielded (best effort)
        // Check common shielded address prefixes
        let is_shielded = payment.address.starts_with("zs") 
            || payment.address.starts_with("u")
            || payment.address.starts_with("ur")
            || payment.address.starts_with("ztestsapling")
            || payment.address.starts_with("test");
        
        if is_shielded {
            // Shielded output (note output)
            actions += 1;
        } else {
            // Transparent output
            actions += 1;
        }
    }
    
    // If we have shielded inputs, we need at least one note spend
    if has_shielded_input {
        actions += 1; // Note spend
    }
    
    actions
}

/// Calculate ZIP-317 fee for a transaction based on payments
///
/// This is a convenience function that estimates logical actions from payments
/// and calculates the fee.
///
/// # Arguments
/// * `payments` - Vector of payments to be included in the transaction
/// * `has_shielded_input` - Whether the transaction will have shielded inputs
///
/// # Returns
/// Fee in zatoshis
pub fn calculate_fee_from_payments(payments: &[Payment], has_shielded_input: bool) -> u64 {
    let logical_actions = estimate_logical_actions(payments, has_shielded_input);
    calculate_zip317_fee(logical_actions)
}

/// Convert fee from zatoshis to ZEC
pub fn fee_zatoshis_to_zec(fee_zatoshis: u64) -> f64 {
    fee_zatoshis as f64 / 100_000_000.0
}

/// Convert fee from ZEC to zatoshis
pub fn fee_zec_to_zatoshis(fee_zec: f64) -> Result<u64> {
    if fee_zec < 0.0 {
        return Err(Error::Transaction("Fee cannot be negative".to_string()));
    }
    Ok((fee_zec * 100_000_000.0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_zip317_fee_minimum() {
        // Minimum fee is 5000 * 2 = 10000 zatoshis
        assert_eq!(calculate_zip317_fee(0), 10000);
        assert_eq!(calculate_zip317_fee(1), 10000);
        assert_eq!(calculate_zip317_fee(2), 10000);
    }

    #[test]
    fn test_calculate_zip317_fee_above_minimum() {
        // Fee scales with logical actions
        assert_eq!(calculate_zip317_fee(3), 15000); // 5000 * 3
        assert_eq!(calculate_zip317_fee(4), 20000); // 5000 * 4
        assert_eq!(calculate_zip317_fee(10), 50000); // 5000 * 10
    }

    #[test]
    fn test_fee_conversion() {
        assert_eq!(fee_zatoshis_to_zec(10000), 0.0001);
        assert_eq!(fee_zatoshis_to_zec(5000), 0.00005);
        
        assert_eq!(fee_zec_to_zatoshis(0.0001).unwrap(), 10000);
        assert_eq!(fee_zec_to_zatoshis(0.00005).unwrap(), 5000);
    }

    #[test]
    fn test_fee_conversion_negative() {
        assert!(fee_zec_to_zatoshis(-0.0001).is_err());
    }

    #[test]
    fn test_estimate_logical_actions_shielded() {
        let payments = vec![
            Payment {
                address: "zs1test".to_string(),
                amount: 1.0,
                memo: None,
            },
        ];
        
        let actions = estimate_logical_actions(&payments, true);
        // At least 1 input + 1 note spend + 1 note output = 3
        assert!(actions >= 3);
    }

    #[test]
    fn test_estimate_logical_actions_transparent() {
        let payments = vec![
            Payment {
                address: "t1test".to_string(),
                amount: 1.0,
                memo: None,
            },
        ];
        
        let actions = estimate_logical_actions(&payments, false);
        // At least 1 transparent input + 1 transparent output = 2
        assert!(actions >= 2);
    }
}

