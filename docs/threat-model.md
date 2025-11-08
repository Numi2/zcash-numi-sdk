# Threat Model for Light Clients

This document describes the threat model for Zcash light clients, derived from the lightwalletd documentation and Zcash protocol specifications.

## Overview

Light clients connect to lightwalletd servers via gRPC to sync blockchain data without downloading the full blockchain. This architecture introduces trust assumptions that must be understood and managed.

## Trust Assumptions

### What Light Clients Trust

1. **Lightwalletd Server Integrity**
   - The server provides correct compact block data
   - The server correctly filters and compresses blockchain data
   - The server responds honestly to gRPC queries

2. **Network Connectivity**
   - gRPC connections are not intercepted or modified
   - TLS/HTTPS provides transport security
   - Certificate validation prevents MITM attacks

3. **Full Node Backend**
   - The lightwalletd server connects to a valid zcashd full node
   - The full node enforces consensus rules correctly

### What Light Clients Do NOT Trust

1. **Server Privacy**
   - The server can observe which blocks a client requests
   - The server can correlate requests to identify users
   - The server can track client scanning patterns

2. **Server Availability**
   - The server may be unavailable or rate-limited
   - The server may provide stale data
   - The server may censor certain transactions or blocks

3. **Server Correctness (Partial)**
   - While the server must provide valid blocks, it may:
     - Omit certain blocks (censorship)
     - Provide blocks out of order
     - Delay block delivery

## Threat Categories

### 1. Privacy Threats

**Threat: Transaction Graph Analysis**
- **Description**: A malicious lightwalletd server could analyze which blocks a client requests to infer transaction patterns
- **Mitigation**: 
  - Use multiple lightwalletd servers when possible
  - Request blocks in batches to reduce granularity
  - Consider using Tor or VPN for additional privacy
  - Use shielded addresses (Sapling/Orchard) to reduce linkability

**Threat: Address Correlation**
- **Description**: Server could correlate address queries with block requests
- **Mitigation**:
  - Use Unified Addresses to reduce address reuse
  - Implement address rotation strategies
  - Minimize address queries to the server

**Threat: Timing Analysis**
- **Description**: Server could analyze request timing to infer user behavior
- **Mitigation**:
  - Add random delays to requests
  - Batch requests to reduce timing signals
  - Use background syncing to decouple from user actions

### 2. Availability Threats

**Threat: Server Downtime**
- **Description**: Single lightwalletd server may become unavailable
- **Mitigation**:
  - Support multiple server endpoints
  - Implement failover logic
  - Cache scanned blocks locally
  - Provide fallback to full node RPC when available

**Threat: Rate Limiting**
- **Description**: Server may rate-limit or throttle requests
- **Mitigation**:
  - Implement exponential backoff
  - Respect server rate limits
  - Batch requests efficiently
  - Use incremental syncing to reduce load

**Threat: Censorship**
- **Description**: Server may omit certain blocks or transactions
- **Mitigation**:
  - Verify block hashes against known checkpoints
  - Cross-reference with multiple servers
  - Use full node RPC for critical operations
  - Implement block hash verification

### 3. Correctness Threats

**Threat: Invalid Block Data**
- **Description**: Server provides malformed or invalid compact blocks
- **Mitigation**:
  - Validate block structure and hashes
  - Verify note commitments
  - Check nullifier validity
  - Reject blocks that fail validation

**Threat: Stale Data**
- **Description**: Server provides outdated blockchain state
- **Mitigation**:
  - Track latest block height from multiple sources
  - Verify block heights are reasonable
  - Implement time-based staleness checks
  - Query latest block height regularly

**Threat: Replay Attacks**
- **Description**: Server replays old blocks to confuse client
- **Mitigation**:
  - Track scanned block heights
  - Reject blocks below current scanned height
  - Verify block hashes match expected chain
  - Use checkpoints for chain validation

### 4. Security Threats

**Threat: MITM Attacks**
- **Description**: Attacker intercepts gRPC connection
- **Mitigation**:
  - Use TLS/HTTPS for all connections
  - Validate server certificates
  - Pin certificates for known servers
  - Use secure DNS (DNSSEC, DoH)

**Threat: Malicious Server Code**
- **Description**: Server runs malicious code that compromises client
- **Mitigation**:
  - Validate all received data
  - Sanitize inputs before processing
  - Use type-safe parsing
  - Implement bounds checking
  - Limit resource consumption

**Threat: DoS Attacks**
- **Description**: Server or attacker overwhelms client with data
- **Mitigation**:
  - Implement request timeouts
  - Limit response sizes
  - Rate limit outgoing requests
  - Use connection pooling
  - Implement circuit breakers

## Best Practices

### For Light Client Implementations

1. **Multiple Servers**: Support multiple lightwalletd endpoints and rotate between them
2. **Local Caching**: Cache scanned blocks locally to reduce server dependency
3. **Incremental Sync**: Use incremental syncing to minimize server load
4. **Validation**: Validate all received data before processing
5. **Error Handling**: Implement robust error handling and retry logic
6. **Privacy**: Minimize information leakage through request patterns
7. **Security**: Use TLS, validate certificates, and sanitize inputs

### For Users

1. **Server Selection**: Choose reputable lightwalletd servers
2. **Network Security**: Use secure networks (avoid public WiFi)
3. **Privacy**: Consider using Tor or VPN for additional privacy
4. **Backup**: Keep backups of wallet data and seeds
5. **Verification**: Verify transactions on block explorers when possible

## Comparison with Full Nodes

### Full Node Trust Model
- **Trusts**: Network consensus rules, own validation logic
- **Does NOT Trust**: Other nodes (validates everything)
- **Privacy**: High (no server sees your queries)
- **Resource Requirements**: High (storage, bandwidth, CPU)

### Light Client Trust Model
- **Trusts**: Lightwalletd server, full node backend
- **Does NOT Trust**: Server privacy, server availability
- **Privacy**: Lower (server sees your queries)
- **Resource Requirements**: Low (minimal storage, bandwidth)

## Recommendations

1. **For Maximum Privacy**: Use a full node or run your own lightwalletd server
2. **For Mobile/Web Apps**: Use light clients but implement privacy mitigations
3. **For Critical Operations**: Use full node RPC for transaction submission
4. **For Development**: Use testnet lightwalletd servers for testing
5. **For Production**: Use multiple lightwalletd servers and implement failover

## References

- [Light Client Development Guide](https://zcash.readthedocs.io/en/latest/rtd_pages/lightwalletd.html)
- [Zcash Protocol Specification](https://zips.z.cash/protocol/)
- [lightwalletd Documentation](https://github.com/zcash/lightwalletd)
- [ZIP-316: Unified Addresses](https://zips.z.cash/zip-0316)
- [ZIP-317: Proportional Transfer Fee Mechanism](https://zips.z.cash/zip-0317)

