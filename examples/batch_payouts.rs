//! Batch payouts example using z_sendmany with memos
use zcash_numi_sdk::rpc::Payment;
use zcash_numi_sdk::transaction::TransactionBuilder;
use zcash_numi_sdk::wallet::Wallet;
use zcash_numi_sdk::client::RpcClient;
use zcash_numi_sdk::Result;
//
#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();
	//
	let wallet = Wallet::new()?;
	let rpc = RpcClient::with_auth(
		"http://localhost:8232",
		"rpcuser".to_string(),
		"rpcpassword".to_string(),
	);
	//
	let builder = TransactionBuilder::with_rpc_client(wallet, rpc);
	//
	let payouts = vec![
		Payment { address: "u1…replace…".to_string(), amount: 0.1234, memo: Some("Payroll batch A".into()) },
		Payment { address: "zs1…replace…".to_string(), amount: 0.0500, memo: Some("Reimbursement #42".into()) },
	];
	//
	// Optional: estimate ZIP-317 fee (zcashd will compute final fee)
	let fee_estimate = builder.estimate_fee(&payouts, "u1…from…")?;
	println!("Estimated ZIP-317 fee: {:.8} ZEC", fee_estimate);
	//
	let opid = builder
		.send_many("u1…from…", payouts, Some(1), None)
		.await?;
	println!("Operation ID: {}", opid);
	//
	Ok(())
}


