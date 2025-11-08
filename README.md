# Zcash Numi SDK

A comprehensive Rust SDK for building Zcash decentralized applications. This SDK provides high-level abstractions for wallet management, transaction creation, and RPC integration.

## Features

- 🎯 **Wallet Management**: Create and manage Zcash wallets with support for Unified Addresses, Sapling, Orchard, and transparent addresses
- 🔌 **RPC Client**: Interact with zcashd nodes via RPC for full node operations
- 📝 **Transaction Building**: Create and sign shielded and transparent transactions
- 🔍 **Address Parsing**: Parse and validate Zcash addresses (UA, Sapling, Orchard, transparent)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zcash-numi-sdk = "0.1"
```

Note: RPC client functionality is always included, as Zcash operations typically require RPC interaction with zcashd nodes or lightwalletd servers.

## Quick Start

```rust
use zcash_numi_sdk::wallet::Wallet;
use zcash_numi_sdk::client::RpcClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new wallet
    let wallet = Wallet::new()?;

    // Get a receiving address
    let address = wallet.get_unified_address()?;
    println!("Your address: {}", address);

    // Connect to a zcashd node via RPC
    // Note: Most zcashd nodes require authentication. Use RpcClient::with_auth()
    // for production. For testing without auth, use RpcClient::new()
    let client = RpcClient::with_auth(
        "http://localhost:8232",
        "rpcuser".to_string(),
        "rpcpassword".to_string(),
    );

    // Get blockchain info
    let info = client.get_blockchain_info().await?;
    println!("Blockchain info: {:?}", info);

    // Get balance
    let balance = wallet.get_balance()?;
    println!("Balance: {} ZEC", balance.total as f64 / 100_000_000.0);

    Ok(())
}
```

### Sending Transactions

To send transactions, use the `TransactionBuilder`:

```rust
use zcash_numi_sdk::transaction::TransactionBuilder;
use zcash_numi_sdk::wallet::Wallet;
use zcash_numi_sdk::client::RpcClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new()?;
    let rpc_client = RpcClient::with_auth(
        "http://localhost:8232",
        "rpcuser".to_string(),
        "rpcpassword".to_string(),
    );
    
    let tx_builder = TransactionBuilder::with_rpc_client(wallet, rpc_client);
    
    // Send a transaction
    let op_id = tx_builder.send_to_address(
        "your_from_address",
        "recipient_address",
        0.001, // amount in ZEC
        Some("Memo text".to_string()), // optional memo for shielded addresses
        None, // minconf
        None, // fee
    ).await?;
    
    // Wait for transaction to complete
    let txid = tx_builder.wait_for_operation(&op_id, None).await?;
    println!("Transaction sent: {}", txid);
    
    Ok(())
}
```

## Resources for Building on Zcash

### Address Formats (correct)
- Unified Address (UA): Bech32m with HRP `u` (mainnet) or `ur` (Rev-1); testnet HRPs add `test`. Uses F4Jumble. UA strings start with `u…`/`ur…`.
- Sapling: Bech32 with HRP `zs` (mainnet) or `ztestsapling` (testnet), approximately 78 characters.
- Orchard: No standalone string format; Orchard receivers appear only inside a Unified Address.

### Fees (ZIP-317)
- ZIP-317 obsoletes ZIP-313. Conventional fee = 5000 zatoshis × max(2, logical_actions).
- Logical actions include note spends, note outputs, and transparent inputs/outputs.
- See `https://zips.z.cash/zip-0317` for parameters and full accounting rules.

### Official Zcash Protocol Documentation & Specifications

- **[Zcash Protocol Specification](https://zips.z.cash/)**: The formal protocol spec for Zcash provides the complete technical description of the blockchain and its privacy features. The latest spec (updated for network upgrades like Canopy, Orchard, etc.) is available on the Zcash Improvement Proposals site. This is the go-to reference for understanding Zcash's consensus rules, transaction format, and cryptographic protocols.

- **[Zcash Improvement Proposals (ZIPs)](https://zips.z.cash/)**: New features and changes to Zcash are proposed and documented through ZIPs. Anyone can author a ZIP to propose protocol improvements, gather community feedback, and record design decisions.

- **[Zcash Developer Hub](https://zcash.readthedocs.io/)**: Zcash's official documentation site is an entry point for developers and end-users, featuring quick-start guides, tutorials, API references, and example code. Key sections include the Network Upgrade Guide (bi-annual upgrade details), Architecture & Cryptography (Zcash's design and zk-SNARK basics), and Development Best Practices. This hub also links to more specific guides (full node, light client, integration, etc.) described below.

### Developer Integration Guides and APIs

- **[Zcash Integration Guide](https://zcash.readthedocs.io/en/latest/rtd_pages/zcashd.html)**: For developers adding Zcash support to applications (exchanges, services, etc.), the integration guide is essential. Zcashd (the Zcash node) offers two RPC interfaces: a Bitcoin-compatible API and the Zcash Payment API. The Bitcoin-compatible RPC calls mirror Bitcoin Core's interface (useful if you already support Bitcoin), but only handle transparent addresses. The Zcash Payment API extends RPC calls (like z_sendmany) to support shielded addresses and encrypted memos. New integrations are encouraged to use the Zcash API for full shielded functionality, falling back to Bitcoin-like calls only if needed (e.g. for multisig, which is currently transparent-only).

- **[Zcashd Full Node & RPC Reference](https://zcash.github.io/rpc/)**: Running a Zcash full node (zcashd) gives you the complete functionality of the network via RPC and a built-in wallet. The official docs explain how to install and run zcashd and zcash-cli (the command-line RPC client). A fully synced node is needed for most operations, and it can be run on Mainnet or Testnet. Zcashd's RPC interface includes all Bitcoin Core calls plus Zcash-specific calls for shielded operations. The Zcash RPC online documentation is a handy reference listing every RPC method, parameters, and example usage.

- **[Light Client Development Guide](https://zcash.readthedocs.io/en/latest/rtd_pages/lightwalletd.html)**: Zcash supports light clients, which can send/receive transactions without downloading the entire blockchain. The Light Client Development section of the docs covers the ecosystem of tools enabling this. Notably, it introduces Lightwalletd – a lightweight backend service (written in Go) that connects to a full node (zcashd), filters and compresses blockchain data (sending compact blocks), and serves this to wallets over gRPC. Lightwalletd drastically reduces bandwidth requirements for wallets.

### Official Mobile SDKs

Electric Coin Co. (ECC) provides open-source mobile wallet SDKs that wrap the light client logic:

- **[Android SDK](https://github.com/zcash/zcash-android-wallet-sdk)** (in Kotlin) and **[iOS SDK](https://github.com/zcash/ZcashLightClientKit)** (in Swift) include functionality for address management, key storage, sending transactions, viewing balance, and memo support.
- They come with demo wallet apps and API docs.
- Under the hood, these SDKs use Rust libraries (librustzcash) via JNI (Android) or FFI (iOS) for the heavy cryptography and syncing.
- Even if your goal is a new Rust-based SDK, reviewing these can be informative – e.g. how they call into Rust for creating shielded transactions.
- There's also a WASM demo (a minimal web wallet using Rust compiled to WebAssembly) though it's an older prototype and not actively maintained.

### Rust Libraries and Crates

This SDK is built on top of the official Zcash Rust crates maintained largely by ECC and the Zcash Foundation:

#### ECC's librustzcash and Zcash Rust Crates

The Electric Coin Co. hosts [librustzcash](https://github.com/zcash/librustzcash), a repository of Rust crates for Zcash. This is essentially the Rust counterpart to Zcashd's functionality, covering everything from low-level primitives to wallet logic:

- **`zcash_primitives`**: Core cryptographic routines and structures (notes, transactions, zk-SNARK circuits for Sapling, etc.)
- **`zcash_proofs`**: SNARK proving/verifying logic (for Sapling and Orchard proofs)
- **`zcash_note_encryption`** and **`zcash_encoding`**: Utilities for note encryption (used in shielded transactions) and data encoding/decoding
- **`zcash_keys`** and **`zcash_address`**: Key derivation (including viewing keys, spending keys) and address formatting (supports Unified Addresses, Sapling, Orchard, transparent)
- **`zip32`** and **`zip321`**: Implementations of ZIP 32 (hierarchical deterministic wallets for Zcash) and ZIP 321 (payment URIs)
- **`zcash_client_backend`**: High-level wallet logic for light clients. The zcash_client_backend crate provides the traits and data structures to build a shielded light wallet – handling block scanning, trial decryption with viewing keys, transaction construction, etc.
- **`zcash_client_sqlite`**: SQLite-backed wallet datastore (so you don't have to implement data storage from scratch)

#### Shielded Protocol Crates

The Sapling and Orchard protocols have their own Rust implementations. The repository includes the legacy sapling-crypto (for Sapling's zk-SNARK circuits, using Bellman/Groth16) and orchard (for the Orchard protocol using the Halo 2 proving system). These handle the heavy cryptography – you typically wouldn't interact with them directly in an SDK, but they're the engines that, for example, create proofs when sending shielded transactions. Simply enabling the "orchard" feature flag in the client SDK crates will activate Orchard support.

All of these crates are published on [crates.io](https://crates.io/) and documented on [docs.rs](https://docs.rs/). By using these, you get a battle-tested implementation of Zcash's protocol. For example, rather than writing your own transaction builder, you can use the crate APIs to create shielded transactions, sign them, and serialize for broadcast.

#### Zebra (Rust Full Node by Zcash Foundation)

The Zcash Foundation maintains [Zebra](https://github.com/ZcashFoundation/zebra), an independent full-node implementation written in Rust. Zebra fully validates Zcash consensus rules and can interoperate with zcashd on the network. While Zebra's primary purpose is running nodes (to improve decentralization and security), it also provides libraries you might leverage. In particular, Zebra includes a wallet SDK component (zebra-client) under active development: it handles note/key management, blockchain scanning with viewing keys, transaction creation, etc., and exposes a wallet RPC API in Zebra.

#### Supporting Rust/crypto libraries

Zcash's Rust codebase relies on several important cryptographic libraries, which might be relevant if your SDK needs lower-level operations:

- **Bellman** (for Groth16 proving, used in Sapling) and **Halo2** (for Orchard's proof system) – these are the zk-SNARK frameworks developed by Zcash engineers. Halo 2 in particular is a Zcash Foundation project (part of the Orchard implementation).
- **jubjub** (the elliptic curve for Zcash's shielded keys) and **bls12_381** (the pairing-friendly curve for Sapling's original proofs) – Rust crates providing the mathematical primitives.

### Tools for Interacting with the Zcash Network

- **[zcashd](https://github.com/zcash/zcash)** (Full Node) & RPC: As mentioned, running a zcashd full node gives you direct access to the network. zcashd is maintained by ECC (a fork of Bitcoin Core with Zcash enhancements). It enforces all protocol rules and includes a wallet for private keys and addresses. You can interact with it via RPC calls (using zcash-cli or any HTTP client). This is useful for backend services or for testing your SDK's outputs (e.g., broadcasting a transaction or checking an address balance via RPC). The Zcashd RPC interface is documented online and includes Zcash-specific calls for shielded operations.

- **[Zcash Testnet](https://zcash.readthedocs.io/en/latest/rtd_pages/testnet.html)** and Devnets: The Testnet is an official public network that mirrors Zcash mainnet's functionality but uses valueless coins (usually denoted "TAZ"). It's reset or coordinated with major upgrades, allowing developers to trial-run transactions and new features safely. Testnet has its own blockchain and block explorers. There is also a "testnet-in-a-box" Kubernetes setup for spinning up a private mini-network – useful for advanced testing scenarios or continuous integration (this tool deploys multiple zcashd nodes in a containerized cluster). Unless you need a custom network, the public Testnet is usually sufficient for development.

- **[Lightwalletd](https://github.com/zcash/lightwalletd)** (Light Client Server): For wallet or dApp development targeting light clients, lightwalletd is key. It acts as a bridge between full nodes and lite clients. Rather than having each mobile app download gigabytes of blocks, a single lightwalletd instance can serve thousands of clients. It connects to a zcashd (which must run with lightwalletd=1 and txindex=1 enabled in zcash.conf) to fetch raw blockchain data. Lightwalletd then stores a compressed summary (only including compact blocks – essentially shielded note commitment info and other minimal details). Clients connect to lightwalletd via a gRPC API to obtain block range data and to submit transactions. This setup is the standard architecture for Zcash mobile wallets like Nighthawk, YWallet, and Zecwallet Lite.
  - gRPC methods aligned with `CompactTxStreamer`: GetLatestBlock, GetBlockRange (validated with `grpcurl`).

- **Block Explorers and Indexers**: Sometimes you need to query blockchain data (transactions, addresses, etc.) without relying on your own node's RPC. There are several Zcash block explorers that provide web interfaces and APIs:
  - The Zcash Block Explorer by Nighthawk (available at zcashblockexplorer.com) is an open-source explorer (Elixir/Phoenix backend) that you can use to look up transactions, addresses, blocks etc.
  - Third-party explorers like Blockchair and Zchain (if still operational) can provide APIs for querying transaction histories, address balances, etc.
  - For a Rust SDK, you might not need direct explorer integration, but these are useful for debugging and verifying your SDK's behavior (for instance, cross-checking that a transaction your SDK created appears on the blockchain and has the expected outputs).

- **[Zcash Indexer (Zaino)](https://github.com/zcashfoundation/zaino)**: The ECC has a project called Zaino – an indexer for the Zcash blockchain written in Rust. While not widely used yet, an indexer can maintain optimized databases of chain data (for example, to quickly look up transactions by address, or to support features like transparent address tracking without a full wallet). If your dApp needs complex queries (beyond what lightwalletd or RPC provide efficiently), keeping an eye on Zaino or similar projects might be worthwhile.

### Community & Ecosystem Resources

Building on Zcash means you're not alone – there's an active community and support structure for developers:

- **[Developer Community Forum](https://forum.zcashcommunity.com/)**: The Zcash Community Forum is a major hub for discussion. There's a dedicated Development category where you can ask technical questions, find proposal discussions, and read updates from Zcash engineers. Many Zcash Improvement Proposals and grant project updates are discussed here.

- **[Discord & Chat](https://discord.gg/zcash)**: Electric Coin Company and the Zcash community maintain an official Discord server ("Zcash Community Chat") where developers and users hang out. There are channels for different topics (engineering, ecosystem projects, support, etc.). The Discord is a good place for quick questions or real-time chat with Zcash engineers (both ECC and Zcash Foundation devs are often present). There's also a Telegram group for the Zcash community if you prefer Telegram; it's more general, but sometimes development topics pop up there as well.

- **Zcash Foundation & ECC**: The two organizations behind Zcash both have resources:
  - The Electric Coin Co. (ECC) website (z.cash) has an ecosystem section listing wallets, exchanges, and developer tools. It also has a blog where ECC publishes developer updates, release announcements, and technical explainers.
  - The Zcash Foundation site (zfnd.org) and GitHub have information on their projects (like Zebra, wallet SDK efforts, research initiatives). The Foundation occasionally runs community calls or technical AMAs which can be informative.

- **[ZecHub Wiki](https://zechub.wiki/)**: A community-driven knowledge base, ZecHub is an organized repository of Zcash information. It includes technical explainers, developer how-tos, and curated links. For example, it has introductions to Zebra, Lightwalletd, and other concepts. ZecHub can be a convenient starting point to discover resources (it's maintained by the community and even funded via grants to improve Zcash education).

- **[Grants and Funding Programs](https://zcashcommunitygrants.org/)**: If you plan to build a substantial open-source Zcash project (like a Rust SDK or dApp framework!), consider the Zcash Community Grants (ZCG) program. ZCG (formerly called ZOMG) is a grants committee that funds independent teams working to improve Zcash. It is financed through 8% of the block rewards, allocated specifically for major grants. The committee accepts proposals – if your project aligns with ecosystem needs, you could receive funding and support. In fact, on ZCG's wishlist of desired projects, one item is a "Zcash SDK based on Mozilla's UniFFI (a multi-language bindings generator for Rust)", indicating strong interest in developer tools exactly like what you're aiming to build. Many ecosystem projects have been funded through ZCG, including wallets (YWallet), libraries, and educational resources.

### Open Source Projects & Examples

Reviewing existing Zcash projects can provide guidance and even building blocks for your SDK:

- **[Zingo](https://github.com/zingolabs/zingolib)**: An open-source project (by an independent team, with support from ECC) that provides a Rust API and CLI for Zcash light wallets. The zingolib library exposes Zcash light client functionality for apps, and zingo-cli is a command-line wallet that uses it. It interacts with Lightwalletd and demonstrates how to build a client that handles addresses, syncing, and transfers in Rust. Zingo could serve as both inspiration and possibly a codebase to contribute to or reuse, since your goals may overlap.

- **[Zecwallet Lite](https://github.com/adityapk00/zecwallet-light-cli)**: The earlier light wallet (now superseded by Nighthawk and others) whose core was a Rust lightclient library. The project by Aditya (adityapk00 on GitHub) created zecwallet-light-cli, which was both a library and a CLI tool for shielded transactions. This has since been archived in favor of ECC's official SDKs, but the code is still available. It can be instructive to see another implementation of a Rust light client and how it manages scanning, keys, and transaction sending.

- **[Nighthawk](https://github.com/nighthawk-apps/nighthawk-wallet)**, **[YWallet](https://github.com/ywalletapp/ywallet)**, **[Zecwallet](https://github.com/adityapk00/zecwallet-lite)**, **[Zashi](https://github.com/zcash/zashi)**: These are user-facing wallets, but each has components you might learn from. For instance, YWallet (by Hanh) is known for performance and was funded by ZCG; it likely uses Rust libraries and has custom sync code optimized for speed. Zashi is ECC's new reference wallet (currently mobile apps in Swift/Kotlin) meant to showcase best practices and use of the official SDKs. While these apps aren't Rust-based in their UI, they rely on the same Rust backend logic you'll be using, and their design decisions (e.g., how to handle multiple address pools in a unified address, how to sync in chunks, etc.) are valuable.

- **[Zebra Client & Wallet](https://github.com/ZcashFoundation/zebra)**: As mentioned, Zcash Foundation's Zebra is implementing a wallet internally. Their approach (detailed in the Zebra RFCs) emphasizes security, isolation of secret data, and uses Rust's async features (Tokio) for concurrency. Even if you don't use Zebra directly, concepts like scanning with viewing keys in an isolated task or using a sled database for wallet state, as described in the Zebra design, could guide your SDK architecture.

## Project Status

✅ **Core Functionality Ready**: This SDK provides core wallet management, RPC integration, and transaction building capabilities. The essential features for building Zcash dApps are implemented and tested.

### Key Features Implemented

- ✅ **Wallet Management**: Full support for Unified Addresses, Sapling, Orchard, and transparent addresses
- ✅ **RPC Client**: Complete implementation of Zcash Payment API (z_sendmany) and Bitcoin-compatible RPC methods
- ✅ **Transaction Building**: Full transaction creation with validation (amounts, memos, addresses)
- ✅ **Address Parsing**: Comprehensive address validation using official `zcash_address` crate
- ✅ **Memo Support**: Proper memo handling with 512-byte limit validation for shielded addresses
- ✅ **ZIP-321 Support**: Support for sending ZIP-321 payment requests (converts ZIP-321 Payment objects to RPC format)
- ✅ **Error Handling**: Comprehensive error types and clear error messages
- ✅ **Utility Functions**: Helper functions for zatoshis/ZEC conversion and formatting

### Known Limitations

- **Transaction History**: The `get_transactions()` method is not yet fully implemented. For transaction history, use RPC methods `z_listreceivedbyaddress` or `z_viewtransaction` instead
- **ZIP-321**: Currently supports sending ZIP-321 payments but does not include ZIP-321 URI parsing or payment request creation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This SDK is built on the excellent work of:
- Electric Coin Company (ECC) and their Zcash Rust crates
- Zcash Foundation and the Zebra project
- The Zcash community and all contributors to the Zcash ecosystem

## Resources Summary

### Core Documentation
- [Zcash Protocol Specification](https://zips.z.cash/)
- [Zcash Improvement Proposals (ZIPs)](https://zips.z.cash/)
- [Zcash Developer Hub](https://zcash.readthedocs.io/)

### Integration Guides
- [Zcash Integration Guide](https://zcash.readthedocs.io/en/latest/rtd_pages/zcashd.html)
- [Zcashd RPC Reference](https://zcash.github.io/rpc/)
- [Light Client Development Guide](https://zcash.readthedocs.io/en/latest/rtd_pages/lightwalletd.html)

### Rust Crates
- [zcash_primitives](https://crates.io/crates/zcash_primitives)
- [zcash_client_backend](https://crates.io/crates/zcash_client_backend)
- [zcash_client_sqlite](https://crates.io/crates/zcash_client_sqlite)
- [zcash_keys](https://crates.io/crates/zcash_keys)
- [zcash_address](https://crates.io/crates/zcash_address)
- [zip32](https://crates.io/crates/zip32)
- [zip321](https://crates.io/crates/zip321)

### Tools
- [zcashd GitHub](https://github.com/zcash/zcash)
- [Lightwalletd GitHub](https://github.com/zcash/lightwalletd)
- [Zebra GitHub](https://github.com/ZcashFoundation/zebra)

### Community
- [Zcash Community Forum](https://forum.zcashcommunity.com/)
- [Zcash Discord](https://discord.gg/zcash)
- [ZecHub Wiki](https://zechub.wiki/)
- [Zcash Community Grants](https://zcashcommunitygrants.org/)

## Production Readiness Checklist

✅ **Core Functionality**
- Wallet creation and management with seed support
- Address generation (Unified, Sapling, Orchard, Transparent)
- Balance queries using `zcash_client_backend` APIs
- Transaction building and sending via Zcash Payment API
- RPC client with full zcashd integration

✅ **Validation & Safety**
- Address format and network validation
- Amount validation (positive, reasonable limits)
- Memo validation (512-byte limit, shielded addresses only)
- Comprehensive error handling with clear messages

✅ **Standards Compliance**
- ZIP-32 (HD wallets) support
- ZIP-321 (Payment sending support - converts ZIP-321 Payment objects to RPC format)
- ZIP-316 (Unified Addresses) support
- Official Zcash Payment API integration

✅ **Developer Experience**
- Well-documented API with examples
- Utility functions for common operations
- Type-safe RPC client
- Clear error messages

## Next Steps

The SDK provides core functionality for building Zcash dApps. To get started:

1. **Set up a zcashd node** (for RPC operations) or use a lightwalletd server (for light clients)
2. **Create a wallet** using `Wallet::new()` or `Wallet::from_seed()`
3. **Connect to RPC** using `RpcClient::with_auth()` for authenticated connections (most zcashd nodes require authentication)
4. **Build transactions** using `TransactionBuilder` with the RPC client
5. **Send transactions** using `send_to_address()` or `send_many()`

For advanced use cases, you can access the underlying `WalletDb` directly for custom blockchain scanning and transaction history management. Note that full transaction history requires implementing blockchain scanning logic using `zcash_client_backend` APIs.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Additional Resources

- [Zcash Client SQLite Documentation](https://docs.rs/zcash_client_sqlite/) - Complete SQLite-based wallet implementation
- [Zcash Protocol Documentation](https://docs.rs/zcash_protocol/) - Protocol-level types and utilities

**Sources**: Official Zcash documentation and developer guides, Electric Coin Co. and Zcash Foundation resources, and Zcash community references.

