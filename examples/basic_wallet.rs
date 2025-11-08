//! Basic wallet example demonstrating wallet creation and address generation

use zcash_numi_sdk::wallet::Wallet;
use zcash_numi_sdk::Result;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Creating a new Zcash wallet...");

    // Create a new wallet
    let wallet = Wallet::new()?;
    println!("✓ Wallet created successfully");

    // Get a unified address (recommended for receiving payments)
    match wallet.get_unified_address() {
        Ok(address) => println!("✓ Unified Address: {}", address),
        Err(e) => println!("⚠ Unified Address not available: {}", e),
    }

    // Get a Sapling address
    match wallet.get_sapling_address() {
        Ok(address) => println!("✓ Sapling Address: {}", address),
        Err(e) => println!("⚠ Sapling Address not available: {}", e),
    }

    // Get balance
    match wallet.get_balance() {
        Ok(balance) => {
            println!("✓ Balance:");
            println!(
                "  Transparent: {} ZEC",
                balance.transparent as f64 / 100_000_000.0
            );
            println!("  Sapling: {} ZEC", balance.sapling as f64 / 100_000_000.0);
            println!("  Orchard: {} ZEC", balance.orchard as f64 / 100_000_000.0);
            println!("  Total: {} ZEC", balance.total as f64 / 100_000_000.0);
        }
        Err(e) => println!("⚠ Balance not available: {}", e),
    }

    Ok(())
}
