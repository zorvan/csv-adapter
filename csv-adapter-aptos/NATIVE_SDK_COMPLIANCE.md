# Native SDK Compliance - Aptos Adapter

**Status**: Production Ready
**Last Updated**: May 2026

## Native SDK Dependencies

| Crate | Version | Purpose | Production Use |
|-------|---------|---------|----------------|
| `aptos-sdk` | 0.4 | Official Aptos SDK | Yes |
| `bcs` | 0.1 | BCS encoding (Move transaction format) | Yes |
| `ed25519-dalek` | 2.0 | Ed25519 signing (Aptos native curve) | Yes |
| `reqwest` | 0.12 | HTTP client for REST API (optional) | Yes (with `rpc` feature) |
| `tokio` | 1.x | Async runtime | Yes |

## Compliance with Production Guarantee Plan Phase 3

### Native SDK Usage

This adapter uses the official Aptos Labs Rust SDK.

- **Transaction Building**: `aptos_sdk::transaction_builder::TransactionBuilder`
- **Account Handling**: `aptos_sdk::types::AccountAddress`
- **Type Tags**: `aptos_sdk::move_types::language_storage::TypeTag`
- **REST API**: `aptos_sdk::rest_client::Client`
- **Signing**: `ed25519_dalek` (Ed25519, Aptos's native signature scheme)
- **BCS**: Native `bcs` crate for transaction serialization

### Raw HTTP Fallback

**Limited Use**: Raw HTTP is used only when:
1. SDK doesn't expose a specific REST endpoint (documented)
2. Custom resource queries require direct access

All HTTP requests use strongly-typed request/response structures.

### Move Integration

Move contracts in `contracts/sources/`:
- `csv_seal.move` - Seal contract with events
- `Move.toml` - Package manifest

Build verification:
```bash
cd csv-adapter-aptos/contracts
aptos move compile
```

### Integration Tests

```bash
# Run with testnet (requires APTOS_RPC_URL)
export APTOS_RPC_URL=https://fullnode.testnet.aptoslabs.com/v1
cargo test -p csv-adapter-aptos --features rpc -- --test-threads=1
```

Test endpoints:
- Testnet: `https://fullnode.testnet.aptoslabs.com/v1`
- Devnet: `https://fullnode.devnet.aptoslabs.com/v1`
- Local: `http://localhost:8080/v1`

## Security Notes

- Ed25519 signatures via audited `ed25519-dalek` crate
- BCS serialization prevents transaction malleability
- Resource-based access control via Move
