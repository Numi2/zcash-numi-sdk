# Development Tools & Infrastructure

## Full Node Infrastructure

### zcashd (Official Full Node)

**Overview:**
zcashd is the official Zcash full node, maintained by Electric Coin Co. as a fork of Bitcoin Core with Zcash enhancements.

**Key Features:**
- Complete blockchain validation and consensus enforcement
- Built-in wallet with private key management
- RPC interface for programmatic access
- Support for shielded and transparent operations

**Installation:**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install zcash

# macOS with Homebrew
brew install zcash

# From source
git clone https://github.com/zcash/zcash.git
cd zcash
./zcutil/build.sh -j$(nproc)
```

**Configuration:**
```bash
# Create zcash.conf
mkdir ~/.zcash
cat > ~/.zcash/zcash.conf << EOF
rpcuser=username
rpcpassword=password
rpcport=8232
port=8233
testnet=0  # Set to 1 for testnet
EOF
```

**Running zcashd:**
```bash
# Mainnet
zcashd

# Testnet
zcashd -testnet

# Regtest (local testing)
zcashd -regtest
```

**RPC Usage:**
```bash
# Using zcash-cli
zcash-cli getblockchaininfo
zcash-cli z_getnewaddress
zcash-cli z_getbalance

# Using curl
curl --user username:password \
     --data-binary '{"jsonrpc": "1.0", "id":"curltest", "method": "getblockchaininfo", "params": [] }' \
     -H 'content-type: text/plain;' \
     http://127.0.0.1:8232/
```

### Zebra (Alternative Full Node)

**Overview:**
Zebra is an independent full-node implementation written in Rust by the Zcash Foundation.

**Advantages:**
- Independent validation of consensus rules
- Improved security through diversity
- Rust-based implementation
- Active development and research

**Running Zebra:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/ZcashFoundation/zebra.git
cd zebra
cargo build --release

# Run zebrad
./target/release/zebrad
```

## Light Client Infrastructure

### Lightwalletd

**Overview:**
Lightwalletd is a stateless proxy server that connects to full nodes and serves compressed blockchain data to light clients.

**Architecture:**
- Connects to zcashd with `lightwalletd=1` and `txindex=1` enabled
- Stores compressed summaries (compact blocks)
- Serves data via gRPC to light clients
- Reduces bandwidth requirements by ~1000x

**Setup Requirements:**
```bash
# zcashd configuration for lightwalletd
lightwalletd=1
txindex=1
rpcuser=user
rpcpassword=pass
```

**Running Lightwalletd:**
```bash
# Download binary or build from source
git clone https://github.com/zcash/lightwalletd.git
cd lightwalletd
go build .

# Run with zcashd connection
./lightwalletd --zcash-conf-path ~/.zcash/zcash.conf --grpc-bind-addr 0.0.0.0:9067
```

**Public Lightwalletd Servers:**
- ECC provides public lightwalletd instances for development
- Community-operated servers available
- Can run your own for production applications

### Light Client SDK Integration

**Connection Setup:**
```rust
use tonic::transport::Channel;
use zcash_client_backend::proto::compact_tx_streamer_client::CompactTxStreamerClient;

// Connect to lightwalletd
let client = CompactTxStreamerClient::connect("https://lightwalletd.example.com").await?;
```

## Testing Infrastructure

### Testnet

**Overview:**
The Zcash testnet provides a public network that mirrors mainnet functionality but uses valueless TAZ coins.

**Key Features:**
- Identical protocol to mainnet
- Coordinated with network upgrades
- Public faucets for test coins
- Multiple block explorers available

**Getting Test Coins:**
1. Join [Zcash Community Discord](https://discord.gg/zcash)
2. Request TAZ from faucet channels
3. Use public faucets like [Zecpages](https://zecpages.com/faucet)

**Testnet Endpoints:**
- RPC: `testnet.z.cash` (port 18232)
- Lightwalletd: Public instances available
- Explorers: Multiple testnet explorers

### Regtest Mode

**Overview:**
Regtest mode allows for local, private testing networks with complete control.

**Setup:**
```bash
# Run zcashd in regtest mode
zcashd -regtest -daemon

# Generate blocks for testing
zcash-cli -regtest generate 101

# Get test coins
zcash-cli -regtest getnewaddress
zcash-cli -regtest sendtoaddress <address> 10.0
zcash-cli -regtest generate 1
```

**Benefits:**
- Instant block generation
- No network dependencies
- Deterministic testing
- Multiple nodes for complex scenarios

### Testnet-in-a-Box

**Overview:**
Kubernetes-based setup for spinning up private mini-networks.

**Use Cases:**
- Advanced testing scenarios
- Continuous integration
- Multi-node network testing
- Protocol development

## Block Explorers & Indexers

### Zcash Block Explorer (Nighthawk)
- **URL:** [zcashblockexplorer.com](https://zcashblockexplorer.com)
- **Technology:** Elixir/Phoenix backend
- **Features:** Transaction lookup, address search, block details
- **Open Source:** Available for self-hosting

### Other Explorers
- **Blockchair:** General-purpose explorer with Zcash support
- **Zchain:** Zcash-specific explorer
- **Local Indexers:** Run your own for development

### Zaino (Zcash Indexer)
**Overview:**
Rust-based indexer for the Zcash blockchain by ECC.

**Features:**
- Optimized database queries
- Address-based transaction lookup
- Advanced filtering capabilities
- REST API for data access

**Current Status:**
- Under active development
- Not yet widely deployed
- Future: Enhanced query capabilities

## Development Tooling

### Network Analysis Tools

**Ziggurat:**
Framework for network-level testing of Zcash nodes.
- Protocol conformance testing
- Performance stress testing
- Network robustness validation

### Monitoring & Debugging

**Logging:**
```rust
use tracing_subscriber;
use zcash_client_backend::logging::init_logging;

// Initialize logging
init_logging();
```

**RPC Debugging:**
```bash
# Enable debug logging in zcashd
debug=rpc

# Monitor lightwalletd logs
./lightwalletd --log-level debug
```

## CI/CD Infrastructure

### GitHub Actions Setup

**Basic CI Configuration:**
```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cargo test --verbose
```

### Docker Development Environment

**Dockerfile for Zcash Development:**
```dockerfile
FROM rust:1.70

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Zcash dependencies
RUN cargo install zcashd
RUN cargo install lightwalletd

WORKDIR /app
COPY . .

RUN cargo build --release
```

## Performance Optimization Tools

### Profiling

**Cargo Flamegraph:**
```bash
cargo install flamegraph
cargo flamegraph --bin my_binary
```

**Heap Profiling:**
```bash
cargo install heaptrack
heaptrack cargo run
```

### Benchmarking

**Criterion.rs:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_transaction_creation(c: &mut Criterion) {
    c.bench_function("create_transaction", |b| {
        b.iter(|| {
            // Benchmark transaction creation
            black_box(create_transaction());
        });
    });
}

criterion_group!(benches, bench_transaction_creation);
criterion_main!(benches);
```

## Security Testing

### Fuzzing

**Cargo Fuzz:**
```bash
cargo install cargo-fuzz
cargo fuzz init
cargo fuzz run fuzz_target
```

### Static Analysis

**Clippy:**
```bash
cargo clippy -- -D warnings
```

**Rust Audit:**
```bash
cargo install cargo-audit
cargo audit
```

## Deployment Considerations

### Production Deployment

**Full Node Deployment:**
- Dedicated server with sufficient resources
- SSD storage for blockchain data
- Backup strategies for wallet data
- Monitoring and alerting setup

**Lightwalletd Deployment:**
- Container orchestration (Kubernetes/Docker)
- Load balancing for multiple instances
- Connection pooling for zcashd
- Rate limiting for client requests

### Cloud Deployment

**AWS/GCP/Azure:**
- Use spot instances for cost optimization
- Implement auto-scaling for lightwalletd
- Use managed databases for indexing
- CDN for static assets

## Community Tools

### Zcash Community Grants (ZCG)
- Funding for open-source Zcash projects
- Includes infrastructure and tooling projects
- Application process via forum proposals

### Third-Party Tools
- **ZecHub Wiki:** Community knowledge base
- **Zcash Community Discord:** Real-time support
- **Zcash Community Forum:** Technical discussions

## Getting Help

### Documentation Resources
- [Zcash Developer Hub](https://zcash.readthedocs.io/)
- [Zcash RPC Reference](https://zcash.github.io/rpc/)
- [Zcash Community Forum](https://forum.zcashcommunity.com/)

### Community Support
- Discord: `#engineering` channel
- Forum: Development category
- GitHub Issues: For specific crate issues

### Professional Services
- ECC provides integration support: ecosystem@z.cash
- Consulting firms available for enterprise integration
- ZCG-funded projects may offer support
