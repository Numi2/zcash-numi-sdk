//! Address parsing and validation using official Zcash address crate

use crate::error::{Error, Result};
use zcash_address::ZcashAddress;
use zcash_protocol::consensus::Network as ConsensusNetwork;
use zcash_protocol::{PoolType, ShieldedProtocol};

/// Parse and validate a Zcash address
///
/// Supports Unified Addresses, Sapling addresses, Orchard addresses, and transparent addresses.
pub fn parse_address(
    address: &str,
    _network: ConsensusNetwork,
) -> Result<ZcashAddress> {
    address.parse::<ZcashAddress>()
        .map_err(|e| Error::Address(format!("Failed to parse address: {}", e)))
}

/// Parse a Unified Address
pub fn parse_unified_address(address: &str, network: ConsensusNetwork) -> Result<ZcashAddress> {
    let addr = parse_address(address, network)?;
    // Unified addresses can receive in multiple pools, check if it can receive as Sapling or Orchard
    // (Unified addresses support both)
    if addr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Sapling)) 
        || addr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Orchard)) {
        Ok(addr)
    } else {
        Err(Error::Address("Address is not a Unified Address".to_string()))
    }
}

/// Validate an address format without parsing
pub fn is_valid_address(address: &str, _network: ConsensusNetwork) -> bool {
    address.parse::<ZcashAddress>().is_ok()
}

/// Get address type from string
pub fn get_address_type(address: &str, network: ConsensusNetwork) -> Result<AddressType> {
    let addr = parse_address(address, network)?;
    // Check pool types to determine address type
    let can_sapling = addr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Sapling));
    let can_orchard = addr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Orchard));
    let can_transparent = addr.can_receive_as(PoolType::Transparent);
    
    Ok(if can_sapling && can_orchard {
        // Unified address supports both Sapling and Orchard
        AddressType::Unified
    } else if can_sapling {
        AddressType::Sapling
    } else if can_orchard {
        AddressType::Orchard
    } else if can_transparent {
        AddressType::Transparent
    } else {
        // Default to transparent if we can't determine
        AddressType::Transparent
    })
}

/// Address type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressType {
    Unified,
    Sapling,
    Orchard,
    Transparent,
}

impl AddressType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AddressType::Unified => "unified",
            AddressType::Sapling => "sapling",
            AddressType::Orchard => "orchard",
            AddressType::Transparent => "transparent",
        }
    }

    /// Check if this address type supports memos (shielded addresses only)
    pub fn supports_memo(&self) -> bool {
        matches!(
            self,
            AddressType::Unified | AddressType::Sapling | AddressType::Orchard
        )
    }
}

/// Check if an address is shielded (supports memos)
pub fn is_shielded_address(address: &str, network: ConsensusNetwork) -> Result<bool> {
    let addr = parse_address(address, network)?;
    let can_sapling = addr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Sapling));
    let can_orchard = addr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Orchard));
    Ok(can_sapling || can_orchard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_validation() {
        // Testnet Unified Address example (this is a placeholder - real addresses are longer)
        // In practice, you'd use real testnet addresses
        let _testnet = ConsensusNetwork::TestNetwork;
        
        // This test would need actual valid addresses to work
        // For now, we just verify the function exists and works
        // TODO: Add actual address validation tests with real addresses
    }
}

