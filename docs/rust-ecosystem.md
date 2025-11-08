# Zcash Rust Ecosystem

This SDK is built on top of the official Zcash Rust crates, providing battle-tested implementations of Zcash's protocol. The Rust ecosystem is primarily maintained by Electric Coin Co. (ECC) and the Zcash Foundation.

## ECC's librustzcash Repository

The Electric Coin Co. hosts [librustzcash](https://github.com/zcash/librustzcash), a repository of Rust crates for Zcash. This is essentially the Rust counterpart to Zcashd's functionality, covering everything from low-level primitives to wallet logic.

### Core Crates

#### `zcash_primitives`
**Purpose:** Core cryptographic routines and structures
**Key Components:**
- Note structures and serialization
- Transaction building and validation
- zk-SNARK circuit definitions for Sapling
- Cryptographic constants and parameters

**Usage in SDK:**
```rust
use zcash_primitives::transaction::TransactionData;
use zcash_primitives::note::Note;
```

#### `zcash_proofs`
**Purpose:** SNARK proving/verifying logic
**Features:**
- Sapling proof generation and verification
- Orchard proof generation and verification
- Circuit parameter loading
- Proof serialization/deserialization

**Usage:**
```rust
use zcash_proofs::prover::LocalProver;
use zcash_proofs::sapling::SaplingProvingContext;
```

#### `zcash_note_encryption` & `zcash_encoding`
**Purpose:** Utilities for shielded transaction encryption and data encoding
**Features:**
- Note encryption/decryption for shielded transactions
- Memo field encryption
- Address encoding/decoding
- Base58check and Bech32 encoding

**Usage:**
```rust
use zcash_note_encryption::NoteEncryption;
use zcash_encoding::AddressCodec;
```

#### `zcash_keys` & `zcash_address`
**Purpose:** Key derivation and address formatting
**Features:**
- Spending key and viewing key derivation
- Unified Address support (Sapling, Orchard, transparent)
- Key serialization and deserialization
- Address validation and parsing

**Usage:**
```rust
use zcash_keys::keys::{SpendingKey, ViewingKey};
use zcash_address::unified::{UnifiedAddress, Receiver};
```

#### `zip32` & `zip321`
**Purpose:** Hierarchical deterministic wallets and payment URIs
**Features:**
- ZIP 32 HD wallet implementation
- Key derivation paths
- ZIP 321 payment URI parsing and generation
- Payment request encoding

**Usage:**
```rust
use zip32::{AccountId, DiversifierIndex};
use zip321::{Payment, TransactionRequest};
```

### Wallet-Level Crates

#### `zcash_client_backend`
**Purpose:** High-level wallet logic for light clients
**Key Features:**
- Block scanning with viewing keys
- Transaction construction and signing
- Note management and selection
- Wallet state management
- Shielded transaction building

**Core Traits:**
- `WalletRead`: Reading wallet data
- `WalletWrite`: Writing wallet data
- `WalletCommitmentTrees`: Managing commitment trees

**Usage:**
```rust
use zcash_client_backend::wallet::Wallet;
use zcash_client_backend::data_api::{WalletRead, WalletWrite};
```

#### `zcash_client_sqlite`
**Purpose:** SQLite-backed wallet datastore
**Features:**
- Persistent wallet storage
- Transaction history tracking
- Address management
- Balance calculation
- Migration support

**Usage:**
```rust
use zcash_client_sqlite::{WalletDb, SqliteClientCache};
```

## Shielded Protocol Crates

The Sapling and Orchard protocols have their own dedicated Rust implementations.

### Sapling Implementation
- **Legacy sapling-crypto**: zk-SNARK circuits using Bellman/Groth16
- **Note encryption**: Sapling-specific note encryption
- **Key components**: SpendAuthSig, BindingSig verification

### Orchard Implementation
- **orchard crate**: Halo 2 proving system for Orchard
- **Features**: Action descriptions, bundle construction
- **Integration**: Enable with "orchard" feature flag

**Enabling Orchard Support:**
```toml
zcash_client_backend = { version = "0.21", features = ["orchard"] }
```

## Zebra (Zcash Foundation)

The Zcash Foundation maintains [Zebra](https://github.com/ZcashFoundation/zebra), an independent full-node implementation in Rust.

### Zebra Components
- **zebrad**: Full node daemon
- **zebra-chain**: Consensus-critical code
- **zebra-network**: Network protocol implementation
- **zebra-state**: State management and validation

### Wallet SDK (Under Development)
- **zebra-client**: Wallet functionality
- Note/key management
- Blockchain scanning with viewing keys
- Transaction creation
- RPC API for wallet operations

**Current Status:**
- Focused on transparent and Sapling support
- Orchard support forthcoming
- Active development with RFCs available

## Supporting Cryptographic Libraries

Zcash's Rust codebase relies on several important cryptographic libraries:

### Zero-Knowledge Proof Systems
- **Bellman**: Groth16 proving for Sapling (ECC)
- **Halo 2**: Zcash Foundation's proving system for Orchard
- **arithmetic circuits**: Custom circuit implementations

### Elliptic Curve Cryptography
- **jubjub**: Twisted Edwards curve for Zcash shielded keys
- **bls12_381**: Pairing-friendly curve for Sapling proofs
- **secp256k1**: For transparent Bitcoin-compatible operations

### Additional Dependencies
- **ff**: Finite field arithmetic
- **group**: Elliptic curve group operations
- **subtle**: Constant-time operations for security

## Version Compatibility

The Zcash Rust crates follow semantic versioning with coordinated releases:

```toml
# Recommended versions (as of 2025)
zcash_primitives = "0.26"
zcash_client_backend = "0.21"
zcash_client_sqlite = "0.19"
zcash_keys = "0.12"
zcash_address = "0.10"
zip32 = "0.2"
zip321 = "0.6"
```

## Feature Flags

Common feature flags across Zcash crates:

- `"orchard"`: Enable Orchard protocol support
- `"transparent-inputs"`: Enable transparent address operations
- `"test-dependencies"`: Additional testing utilities
- `"multicore"`: Parallel proof generation

## Integration Patterns

### Basic Wallet Setup
```rust
use zcash_client_backend::wallet::Wallet;
use zcash_client_sqlite::WalletDb;
use zcash_keys::keys::UnifiedSpendingKey;

// Initialize wallet database
let wallet_db = WalletDb::for_path("/path/to/wallet.db", network)?;

// Create or load spending key
let usk = UnifiedSpendingKey::from_seed(&seed, 0, network)?;

// Initialize wallet
let wallet = Wallet::new(usk, wallet_db);
```

### Transaction Construction
```rust
use zcash_client_backend::data_api::WalletRead;
use zcash_client_backend::wallet::TransparentAddress;

// Read wallet state
let account = wallet.get_account(0)?;
let balance = wallet.get_balance(account, &TransparentAddress::PublicKeyHash)?;

// Build transaction request
let request = TransactionRequest::new(vec![
    Payment::new(
        Address::Unified(ua),
        Amount::from_u64(1000).unwrap(),
        Some(Memo::from_str("Payment memo").unwrap())
    )
]);

// Create and send transaction
let txid = wallet.send_to_address(request).await?;
```

### Light Client Synchronization
```rust
use zcash_client_backend::sync::sync;

// Connect to lightwalletd
let client = CompactTxStreamerClient::connect("https://lightwalletd.example.com").await?;

// Sync wallet
sync(
    &client,
    &wallet,
    last_sync_height,
    target_height
).await?;
```

## Best Practices

### Error Handling
```rust
use zcash_client_backend::data_api::Error;

match wallet_operation() {
    Ok(result) => println!("Success: {:?}", result),
    Err(Error::InsufficientFunds { available, required }) => {
        println!("Insufficient funds: {} available, {} required",
                available, required);
    }
    Err(e) => println!("Error: {}", e),
}
```

### Memory Management
- Use streaming for large block ranges
- Implement proper cleanup for wallet databases
- Consider memory-mapped databases for performance

### Testing
```rust
#[cfg(test)]
mod tests {
    use zcash_client_backend::data_api::testing::{MockWalletDb, MockBlockSource};

    #[test]
    fn test_wallet_operations() {
        let wallet_db = MockWalletDb::new();
        let block_source = MockBlockSource::new();
        // Test wallet operations with mocks
    }
}
```

## Contributing to the Ecosystem

### Reporting Issues
- ECC crates: [GitHub Issues](https://github.com/zcash/librustzcash/issues)
- Zebra: [GitHub Issues](https://github.com/ZcashFoundation/zebra/issues)

### Development Workflow
1. Fork the repository
2. Create a feature branch
3. Implement changes with comprehensive tests
4. Submit a pull request
5. Participate in code review

### Staying Updated
- Follow ECC's [blog](https://electriccoin.co/blog/) for announcements
- Monitor [Zcash Community Forum](https://forum.zcashcommunity.com/) for discussions
- Watch release notes for breaking changes
