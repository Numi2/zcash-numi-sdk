//! Shielding flow example (move funds to a Unified Address)
use zcash_numi_sdk::transaction::TransactionBuilder;
use zcash_numi_sdk::wallet::Wallet;
use zcash_numi_sdk::client::RpcClient;
use zcash_numi_sdk::rpc::Payment;
use zcash_numi_sdk::Result;
//
#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();
	//
	let mut wallet = Wallet::new()?;
	// For test usage:
	// wallet.set_network(zcash_numi_sdk::types::Network::Testnet);
	//
	let ua = wallet.get_unified_address()?;
	println!("Destination UA: {}", ua);
	//
	let rpc = RpcClient::with_auth(
		"http://localhost:8232",
		"rpcuser".to_string(),
		"rpcpassword".to_string(),
	);
	let builder = TransactionBuilder::with_rpc_client(wallet, rpc);
	//
	let payment = Payment {
		address: ua,
		amount: 0.0100,
		memo: Some("Shielding".into()),
	};
	//
	let opid = builder
		.send_many("t1…replace…", vec![payment], Some(1), None)
		.await?;
	println!("Operation ID: {}", opid);
	Ok(())
}


