//! Example: Using the Light Client to sync with lightwalletd
//!
//! This example demonstrates how to:
//! 1. Create a wallet
//! 2. Connect to a lightwalletd server via gRPC
//! 3. Sync with the blockchain
//! 4. Check balance
//!
//! # Prerequisites
//! - A running lightwalletd server (or use a public one)
//! - For testnet: Use "https://testnet.lightwalletd.com:9067"
//! - For mainnet: Use "https://mainnet.lightwalletd.com:9067"

use zcash_numi_sdk::light_client::{default_endpoints, LightClient};
use zcash_numi_sdk::wallet::Wallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Creating wallet...");
    let wallet = Wallet::new()?;
    
    // Get a receiving address
    let address = wallet.get_unified_address()?;
    println!("Your address: {}", address);

    // Get default endpoints for testnet
    let endpoints = default_endpoints(wallet.network());
    let endpoint = endpoints.first().ok_or("No endpoints available")?;
    
    println!("Connecting to lightwalletd at {}...", endpoint);
    
    // Connect to lightwalletd
    let mut light_client = match LightClient::connect(endpoint.clone(), wallet).await {
        Ok(client) => {
            println!("✓ Connected to lightwalletd");
            client
        }
        Err(e) => {
            eprintln!("Failed to connect to lightwalletd: {}", e);
            eprintln!("\nNote: Make sure lightwalletd is running or use a public endpoint.");
            eprintln!("For testnet, you can use: https://testnet.lightwalletd.com:9067");
            return Err(e.into());
        }
    };

    // Get latest block height
    println!("\nQuerying latest block height...");
    match light_client.get_latest_block_height().await {
        Ok(height) => {
            println!("✓ Latest block height: {}", height);
        }
        Err(e) => {
            eprintln!("Note: get_latest_block_height is not yet fully implemented.");
            eprintln!("Error: {}", e);
        }
    }

    // Get tip information
    println!("\nGetting tip information...");
    match light_client.get_tip().await {
        Ok((height, hash)) => {
            println!("✓ Tip height: {}", height);
            println!("  Tip hash: {}", hex::encode(&hash));
        }
        Err(e) => {
            eprintln!("Note: get_tip is not yet fully implemented.");
            eprintln!("Error: {}", e);
        }
    }

    // Sync with blockchain (this is a placeholder - full implementation pending)
    println!("\nSyncing with blockchain...");
    println!("Note: Full sync implementation is pending. See zcash_client_backend docs for details.");
    
    // Check balance
    println!("\nChecking balance...");
    // Note: Balance checking requires syncing first
    // For now, we'll just show the address
    println!("Address: {}", address);
    println!("Balance will be available after full sync is implemented.");

    println!("\n✓ Light client example completed!");
    println!("\nNext steps:");
    println!("1. Implement full sync using zcash_client_backend::scanning APIs");
    println!("2. Use scan_cached_blocks to process compact blocks");
    println!("3. Query wallet database for balance and transaction history");

    Ok(())
}

