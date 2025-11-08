//! Integration tests for the Zcash Numi SDK

use zcash_numi_sdk::wallet::Wallet;

#[test]
fn test_wallet_creation() {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join("test_wallet_integration.db");

    // Clean up any existing test database
    let _ = std::fs::remove_file(&db_path);

    let wallet = Wallet::with_path(db_path.clone()).unwrap();
    assert_eq!(wallet.network(), zcash_numi_sdk::types::Network::Mainnet);

    // Clean up
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_wallet_network_switching() {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join("test_wallet_network.db");
    let _ = std::fs::remove_file(&db_path);

    let mut wallet = Wallet::with_path(db_path.clone()).unwrap();

    wallet.set_network(zcash_numi_sdk::types::Network::Testnet);
    assert_eq!(wallet.network(), zcash_numi_sdk::types::Network::Testnet);

    wallet.set_network(zcash_numi_sdk::types::Network::Mainnet);
    assert_eq!(wallet.network(), zcash_numi_sdk::types::Network::Mainnet);

    let _ = std::fs::remove_file(&db_path);
}
