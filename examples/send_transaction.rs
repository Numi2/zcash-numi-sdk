//! Transaction sending example demonstrating how to build and send Zcash transactions

use zcash_numi_sdk::transaction::TransactionBuilder;
use zcash_numi_sdk::wallet::Wallet;
use zcash_numi_sdk::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Zcash Transaction Example");
    println!("==========================");

    // Create a wallet
    let wallet = Wallet::new()?;
    println!("✓ Wallet loaded");

    // Get balance
    let balance = match wallet.get_balance() {
        Ok(balance) => {
            println!(
                "Current balance: {} ZEC",
                balance.total as f64 / 100_000_000.0
            );
            balance
        }
        Err(e) => {
            println!("⚠ Could not get balance: {}", e);
            return Ok(());
        }
    };

    // Check if we have sufficient balance
    if balance.total == 0 {
        println!("⚠ Wallet has no balance. Please fund the wallet first.");
        println!("Receiving address: {}", wallet.get_unified_address()?);
        return Ok(());
    }

    // Create transaction builder
    // Note: To actually send transactions, you need to connect to a zcashd node
    // via RPC. For this example, we'll demonstrate the API without an RPC client.
    let _tx_builder = TransactionBuilder::new(wallet);

    println!("\n📝 Transaction Builder Ready");
    println!("The TransactionBuilder uses the official Zcash Payment API (z_sendmany)");
    println!("which handles note selection, fee calculation, proof generation,");
    println!("and transaction signing automatically via zcashd.");
    println!("\nTo send a transaction:");
    println!("  1. Connect to a zcashd node: RpcClient::with_auth(...)");
    println!("  2. Create builder with RPC: TransactionBuilder::with_rpc_client(wallet, rpc_client)");
    println!("  3. Call send_to_address() or send_many()");
    println!("\nExample (requires zcashd RPC connection):");
    println!("  let rpc_client = RpcClient::with_auth(\"http://localhost:8232\", \"user\", \"pass\");");
    println!("  let tx_builder = TransactionBuilder::with_rpc_client(wallet, rpc_client);");
    println!("  let op_id = tx_builder.send_to_address(");
    println!("      from_address,");
    println!("      recipient_address,");
    println!("      0.001,");
    println!("      Some(\"Memo text\".to_string()),");
    println!("      None,");
    println!("      None,");
    println!("  ).await?;");
    println!("  let txid = tx_builder.wait_for_operation(&op_id, None).await?;");

    Ok(())
}
