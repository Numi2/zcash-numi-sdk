//! Zcash CLI - Command-line interface for the Zcash Numi SDK
//!
//! This CLI provides a comprehensive interface for managing Zcash wallets,
//! generating addresses, checking balances, sending transactions, and syncing
//! with the blockchain.

use clap::{Parser, Subcommand};
use zcash_numi_sdk::client::RpcClient;
use zcash_numi_sdk::light_client::{default_endpoints, LightClient};
use zcash_numi_sdk::transaction::TransactionBuilder;
use zcash_numi_sdk::types::{Network, utils};
use zcash_numi_sdk::wallet::Wallet;
use zcash_numi_sdk::Result;

#[derive(Parser)]
#[command(name = "zcash-cli")]
#[command(about = "Zcash Numi SDK Command Line Interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Network to use (mainnet, testnet, regtest)
    #[arg(short, long, default_value = "mainnet")]
    network: String,

    /// Wallet database path (optional, defaults to standard location)
    #[arg(short, long)]
    wallet_path: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet management commands
    Wallet {
        #[command(subcommand)]
        action: WalletAction,
    },
    /// Address generation commands
    Address {
        #[command(subcommand)]
        action: AddressAction,
    },
    /// Check wallet balance
    Balance {
        /// Use RPC to check balance instead of local wallet
        #[arg(short, long)]
        rpc: bool,
        /// Address to check balance for (RPC mode only)
        #[arg(short, long)]
        address: Option<String>,
        /// RPC endpoint URL (required if --rpc is used)
        #[arg(long)]
        rpc_url: Option<String>,
        /// RPC username
        #[arg(long)]
        rpc_user: Option<String>,
        /// RPC password
        #[arg(long)]
        rpc_password: Option<String>,
    },
    /// Send Zcash transactions
    Send {
        /// Source address (must be in wallet)
        #[arg(short, long)]
        from: String,
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount in ZEC
        #[arg(short, long)]
        amount: f64,
        /// Optional memo (for shielded addresses)
        #[arg(short, long)]
        memo: Option<String>,
        /// RPC endpoint URL
        #[arg(short, long)]
        rpc_url: String,
        /// RPC username
        #[arg(long)]
        rpc_user: Option<String>,
        /// RPC password
        #[arg(long)]
        rpc_password: Option<String>,
        /// Minimum confirmations
        #[arg(long, default_value = "1")]
        minconf: u32,
        /// Transaction fee in ZEC (optional)
        #[arg(long)]
        fee: Option<f64>,
    },
    /// Sync with blockchain using light client
    Sync {
        /// Lightwalletd endpoint URL
        #[arg(short, long)]
        endpoint: Option<String>,
        /// Start height for sync (default: 0)
        #[arg(long, default_value = "0")]
        start_height: u64,
        /// End height for sync (default: latest)
        #[arg(long)]
        end_height: Option<u64>,
    },
    /// Get blockchain information
    Info {
        /// RPC endpoint URL
        #[arg(short, long)]
        rpc_url: String,
        /// RPC username
        #[arg(long)]
        rpc_user: Option<String>,
        /// RPC password
        #[arg(long)]
        rpc_password: Option<String>,
        /// Show network information
        #[arg(short, long)]
        network: bool,
        /// Show block count only
        #[arg(short, long)]
        count: bool,
    },
}

#[derive(Subcommand)]
enum WalletAction {
    /// Create a new wallet
    Create,
    /// Show wallet information
    Info,
    /// List addresses from RPC node (requires RPC connection)
    List {
        /// RPC endpoint URL
        #[arg(short, long)]
        rpc_url: String,
        /// RPC username
        #[arg(long)]
        rpc_user: Option<String>,
        /// RPC password
        #[arg(long)]
        rpc_password: Option<String>,
    },
}

#[derive(Subcommand)]
enum AddressAction {
    /// Generate a unified address (supports all address types)
    Unified,
    /// Generate a Sapling address
    Sapling,
    /// Generate an Orchard address (via unified address)
    Orchard,
    /// Generate a transparent address
    Transparent,
}

fn parse_network(network_str: &str) -> Network {
    match network_str.to_lowercase().as_str() {
        "testnet" => Network::Testnet,
        "regtest" => Network::Regtest,
        _ => Network::Mainnet,
    }
}

fn load_wallet(cli: &Cli) -> Result<Wallet> {
    let network = parse_network(&cli.network);
    
    let wallet = if let Some(ref path) = cli.wallet_path {
        let db_path = std::path::PathBuf::from(path);
        let mut wallet = Wallet::with_path(db_path)?;
        wallet.set_network(network);
        wallet
    } else {
        let mut wallet = Wallet::new()?;
        wallet.set_network(network);
        wallet
    };
    
    Ok(wallet)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging if verbose
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .init();
    }

    match &cli.command {
        Commands::Wallet { action } => {
            match action {
                WalletAction::Create => {
                    println!("Creating new wallet...");
                    let wallet = load_wallet(&cli)?;
                    let address = wallet.get_unified_address()?;
                    println!("✓ Wallet created successfully!");
                    println!("Network: {:?}", wallet.network());
                    println!("Unified Address: {}", address);
                }
                WalletAction::Info => {
                    let wallet = load_wallet(&cli)?;
                    let address = wallet.get_unified_address()?;
                    println!("Wallet Information");
                    println!("==================");
                    println!("Network: {:?}", wallet.network());
                    println!("Unified Address: {}", address);
                    
                    match wallet.get_sapling_address() {
                        Ok(addr) => println!("Sapling Address: {}", addr),
                        Err(_) => println!("Sapling Address: Not available"),
                    }
                    
                    match wallet.get_transparent_address() {
                        Ok(addr) => println!("Transparent Address: {}", addr),
                        Err(_) => println!("Transparent Address: Not available"),
                    }
                }
                WalletAction::List {
                    rpc_url,
                    rpc_user,
                    rpc_password,
                } => {
                    // Create RPC client
                    let rpc_client = if let (Some(user), Some(pass)) = (rpc_user, rpc_password) {
                        RpcClient::with_auth(rpc_url.clone(), user.clone(), pass.clone())
                    } else {
                        println!("Warning: No RPC credentials provided. Using unauthenticated connection.");
                        RpcClient::new(rpc_url.clone())
                    };

                    println!("Fetching addresses from RPC node...");
                    
                    match rpc_client.z_listaddresses().await {
                        Ok(addresses) => {
                            if addresses.is_empty() {
                                println!("No addresses found in wallet.");
                            } else {
                                println!("Addresses in wallet:");
                                println!("=====================");
                                for (idx, addr_info) in addresses.iter().enumerate() {
                                    println!("\n{}. Address: {}", idx + 1, addr_info.address);
                                    if let Some(ref account) = addr_info.account {
                                        println!("   Account: {}", account);
                                    }
                                    if let Some(ref label) = addr_info.label {
                                        println!("   Label: {}", label);
                                    }
                                    if let Some(balance) = addr_info.balance {
                                        println!("   Balance: {} ZEC", balance);
                                    }
                                    if let Some(received) = addr_info.receivedby {
                                        println!("   Total Received: {} ZEC", received);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error fetching addresses: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Address { action } => {
            let wallet = load_wallet(&cli)?;
            match action {
                AddressAction::Unified => {
                    let address = wallet.get_unified_address()?;
                    println!("{}", address);
                }
                AddressAction::Sapling => {
                    let address = wallet.get_sapling_address()?;
                    println!("{}", address);
                }
                AddressAction::Orchard => {
                    // Orchard addresses are accessed via unified addresses
                    let address = wallet.get_unified_address()?;
                    println!("{}", address);
                    println!("\nNote: Orchard addresses are included in Unified Addresses");
                }
                AddressAction::Transparent => {
                    let address = wallet.get_transparent_address()?;
                    println!("{}", address);
                }
            }
        }
        Commands::Balance {
            rpc,
            address,
            rpc_url,
            rpc_user,
            rpc_password,
        } => {
            if *rpc {
                // RPC-based balance check
                let rpc_url = rpc_url.as_ref().ok_or_else(|| {
                    zcash_numi_sdk::Error::InvalidParameter(
                        "RPC URL required when using --rpc flag".to_string()
                    )
                })?;

                let rpc_client = if let (Some(user), Some(pass)) = (rpc_user, rpc_password) {
                    RpcClient::with_auth(rpc_url.clone(), user.clone(), pass.clone())
                } else {
                    println!("Warning: No RPC credentials provided. Using unauthenticated connection.");
                    RpcClient::new(rpc_url.clone())
                };

                if let Some(ref addr) = address {
                    // Check balance for specific address
                    println!("Checking balance for address: {}", addr);
                    match rpc_client.z_getbalance(addr, None).await {
                        Ok(balance) => {
                            println!("Balance: {} ZEC", balance);
                        }
                        Err(e) => {
                            eprintln!("Error getting balance: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    // Get total balance
                    println!("Fetching total wallet balance...");
                    match rpc_client.z_gettotalbalance(None, None).await {
                        Ok(total_balance) => {
                            println!("Total Wallet Balance");
                            println!("====================");
                            println!("{}", serde_json::to_string_pretty(&total_balance)?);
                        }
                        Err(e) => {
                            eprintln!("Error getting total balance: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                // Local wallet balance
                let wallet = load_wallet(&cli)?;
                match wallet.get_balance() {
                    Ok(balance) => {
                        println!("Wallet Balance");
                        println!("==============");
                        println!("Network: {:?}", wallet.network());
                        println!("Transparent: {}", utils::format_zec(balance.transparent as f64 / 100_000_000.0));
                        println!("Sapling: {}", utils::format_zec(balance.sapling as f64 / 100_000_000.0));
                        println!("Orchard: {}", utils::format_zec(balance.orchard as f64 / 100_000_000.0));
                        println!("Total: {}", utils::format_zec(balance.total as f64 / 100_000_000.0));
                    }
                    Err(e) => {
                        eprintln!("Error getting balance: {}", e);
                        eprintln!("\nNote: Balance requires syncing with the blockchain first.");
                        eprintln!("Use 'zcash-cli sync' to sync with a lightwalletd server.");
                        eprintln!("Or use 'zcash-cli balance --rpc --rpc-url <url>' to check via RPC.");
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Send {
            from,
            to,
            amount,
            memo,
            rpc_url,
            rpc_user,
            rpc_password,
            minconf,
            fee,
        } => {
            let wallet = load_wallet(&cli)?;
            
            // Create RPC client
            let rpc_client = if let (Some(user), Some(pass)) = (rpc_user, rpc_password) {
                RpcClient::with_auth(rpc_url.clone(), user.clone(), pass.clone())
            } else {
                println!("Warning: No RPC credentials provided. Using unauthenticated connection.");
                RpcClient::new(rpc_url.clone())
            };

            println!("Sending transaction...");
            println!("From: {}", from);
            println!("To: {}", to);
            println!("Amount: {} ZEC", amount);
            if let Some(ref m) = memo {
                println!("Memo: {}", m);
            }

            let tx_builder = TransactionBuilder::with_rpc_client(wallet, rpc_client);
            
            match tx_builder
                .send_to_address(from, to, *amount, memo.clone(), Some(*minconf), *fee)
                .await
            {
                Ok(op_id) => {
                    println!("✓ Transaction submitted!");
                    println!("Operation ID: {}", op_id);
                    println!("\nWaiting for transaction to be confirmed...");
                    
                    match tx_builder.wait_for_operation(&op_id, Some(300)).await {
                        Ok(txid) => {
                            println!("✓ Transaction confirmed!");
                            println!("Transaction ID: {}", txid);
                        }
                        Err(e) => {
                            eprintln!("⚠ Transaction submitted but confirmation check failed: {}", e);
                            eprintln!("Operation ID: {}", op_id);
                            eprintln!("You can check the status using zcashd RPC: z_getoperationstatus");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error sending transaction: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Sync {
            endpoint,
            start_height,
            end_height,
        } => {
            let wallet = load_wallet(&cli)?;
            
            let endpoint_url = if let Some(ref ep) = endpoint {
                ep.clone()
            } else {
                // Use default endpoints for the network
                let endpoints = default_endpoints(wallet.network());
                endpoints
                    .first()
                    .ok_or_else(|| zcash_numi_sdk::Error::InvalidParameter(
                        "No default endpoints available for this network".to_string()
                    ))?
                    .clone()
            };

            println!("Connecting to lightwalletd at {}...", endpoint_url);
            
            match LightClient::connect(endpoint_url.clone(), wallet).await {
                Ok(mut light_client) => {
                    println!("✓ Connected to lightwalletd");
                    
                    // Get latest block height
                    let latest_height = match light_client.get_latest_block_height().await {
                        Ok(height) => {
                            println!("✓ Latest block height: {}", height);
                            height
                        }
                        Err(e) => {
                            eprintln!("⚠ Could not get latest block height: {}", e);
                            return Err(e.into());
                        }
                    };
                    
                    // Get tip information
                    match light_client.get_tip().await {
                        Ok((height, hash)) => {
                            println!("✓ Tip height: {}", height);
                            println!("  Tip hash: {}", hex::encode(&hash));
                        }
                        Err(e) => {
                            eprintln!("⚠ Could not get tip: {}", e);
                        }
                    }
                    
                    // Determine sync range
                    let sync_start = *start_height;
                    let sync_end = end_height.unwrap_or(latest_height);
                    
                    if sync_start > sync_end {
                        eprintln!("Error: Start height {} is greater than end height {}", sync_start, sync_end);
                        std::process::exit(1);
                    }
                    
                    if sync_start == sync_end {
                        println!("\nNo blocks to sync (start == end)");
                        return Ok(());
                    }
                    
                    println!("\nStarting blockchain sync...");
                    println!("Sync range: {} to {} ({} blocks)", sync_start, sync_end, sync_end - sync_start + 1);
                    
                    match light_client.sync(sync_start, Some(sync_end)).await {
                        Ok(_) => {
                            println!("✓ Sync completed successfully!");
                            println!("\nYou can now check your balance with: zcash-cli balance");
                        }
                        Err(e) => {
                            eprintln!("⚠ Sync encountered errors: {}", e);
                            eprintln!("Some blocks may have been synced. Check balance to verify.");
                            // Don't exit with error - partial sync may be useful
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error connecting to lightwalletd: {}", e);
                    eprintln!("\nMake sure lightwalletd is running or use a public endpoint.");
                    eprintln!("For testnet: https://testnet.lightwalletd.com:9067");
                    eprintln!("For mainnet: https://mainnet.lightwalletd.com:9067");
                    std::process::exit(1);
                }
            }
        }
        Commands::Info {
            rpc_url,
            rpc_user,
            rpc_password,
            network,
            count,
        } => {
            // Create RPC client
            let rpc_client = if let (Some(user), Some(pass)) = (rpc_user, rpc_password) {
                RpcClient::with_auth(rpc_url.clone(), user.clone(), pass.clone())
            } else {
                println!("Warning: No RPC credentials provided. Using unauthenticated connection.");
                RpcClient::new(rpc_url.clone())
            };

            if *count {
                // Just show block count
                match rpc_client.get_block_count().await {
                    Ok(count) => {
                        println!("{}", count);
                    }
                    Err(e) => {
                        eprintln!("Error fetching block count: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("Fetching blockchain information...");
                
                match rpc_client.get_blockchain_info().await {
                    Ok(info) => {
                        println!("Blockchain Information");
                        println!("=====================");
                        println!("Chain: {}", info.chain);
                        println!("Blocks: {}", info.blocks);
                        println!("Headers: {}", info.headers);
                        println!("Best Block Hash: {}", info.bestblockhash);
                        println!("Difficulty: {}", info.difficulty);
                        println!("Verification Progress: {:.2}%", info.verificationprogress * 100.0);
                        println!("Chainwork: {}", info.chainwork);
                        println!("Pruned: {}", info.pruned);
                        println!("Commitments: {}", info.commitments);
                        
                        if *network {
                            println!("\nFetching network information...");
                            match rpc_client.get_network_info().await {
                                Ok(net_info) => {
                                    println!("\nNetwork Information");
                                    println!("===================");
                                    println!("{}", serde_json::to_string_pretty(&net_info)?);
                                }
                                Err(e) => {
                                    eprintln!("⚠ Could not fetch network info: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching blockchain info: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    Ok(())
}

