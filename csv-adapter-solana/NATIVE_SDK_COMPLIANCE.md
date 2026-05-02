# Native SDK Compliance - Solana Adapter

**Status**: Production Ready
**Last Updated**: May 2026

## Native SDK Dependencies

| Crate | Version | Purpose | Production Use |
|-------|---------|---------|----------------|
| `solana-sdk` | 3.0 | Core Solana types (Pubkey, Signature, Transaction) | Yes |
| `solana-program` | 3.0 | On-chain program types | Yes |
| `solana-rpc-client` | 3.1 | RPC client for Solana | Yes |
| `solana-loader-v3-interface` | 3.0 | Program loader (BPF) | Yes |
| `solana-system-interface` | 2.0 | System program interface | Yes |
| `ed25519-dalek` | 2.0 | Ed25519 signing (Solana native curve) | Yes |
| `bincode` | 1.3 | Transaction serialization | Yes |

## Compliance with Production Guarantee Plan Phase 3

### Native SDK Usage

This adapter uses the official Solana Labs Rust SDK.

- **Transaction Building**: `solana_sdk::transaction::Transaction`
- **Account Handling**: `solana_sdk::pubkey::Pubkey`, `solana_sdk::account::Account`
- **Program Interaction**: `solana_program::instruction::Instruction`
- **RPC**: `solana_rpc_client::rpc_client::RpcClient`
- **Signing**: `ed25519_dalek` (Ed25519, Solana's native signature scheme)
- **Program Deployment**: `solana_loader_v3_interface` for BPF loader

### Raw HTTP Fallback

**Not Used**: All operations use the native `solana-rpc-client` crate which provides comprehensive RPC coverage.

### Anchor Integration

Anchor program bindings are available in `contracts/programs/`:
- `csv_seal` program with IDL
- TypeScript tests in `contracts/tests/`

### Integration Tests

```bash
# Run with devnet (requires SOLANA_RPC_URL)
export SOLANA_RPC_URL=https://api.devnet.solana.com
cargo test -p csv-adapter-solana --features rpc -- --test-threads=1
```

Test endpoints:
- Devnet: `https://api.devnet.solana.com`
- Testnet: `https://api.testnet.solana.com`
- Local validator: `http://localhost:8899`

## Security Notes

- Ed25519 signatures via audited `ed25519-dalek` crate
- Transaction atomicity enforced by Solana runtime
- Program-derived addresses (PDAs) for deterministic account creation
