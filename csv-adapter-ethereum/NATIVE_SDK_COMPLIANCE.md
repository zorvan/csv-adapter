# Native SDK Compliance - Ethereum Adapter

**Status**: Production Ready
**Last Updated**: May 2026

## Native SDK Dependencies

| Crate | Version | Purpose | Production Use |
|-------|---------|---------|----------------|
| `alloy` | 1.8 | Full Ethereum stack (RPC, contracts, signing) | Yes (with `rpc` feature) |
| `alloy-primitives` | 0.8 | Ethereum primitive types (Address, U256, etc.) | Yes |
| `alloy-rlp` | 0.3 | RLP encoding/decoding | Yes |
| `alloy-trie` | 0.7 | Merkle Patricia Trie proofs | Yes |
| `alloy-sol-types` | 1.5 | Solidity type encoding | Yes (with `rpc` feature) |
| `alloy-contract` | 1.0 | Contract interaction | Yes (with `rpc` feature) |
| `secp256k1` | 0.28 | ECDSA signing (Ethereum addresses) | Yes |
| `sha3` | 0.10 | Keccak-256 hashing | Yes |

## Compliance with Production Guarantee Plan Phase 3

### Native SDK Usage

This adapter uses the Alloy stack from Paradigm, the modern Rust Ethereum SDK.

- **Transaction Building**: `alloy::consensus::TxEip1559` for EIP-1559 transactions
- **Contract Interaction**: `alloy_contract::ContractInstance` for seal contract calls
- **RPC**: `alloy::providers::Provider` for Ethereum RPC
- **Proofs**: `alloy_trie` for storage slot proofs
- **Encoding**: `alloy_rlp` for transaction RLP encoding
- **Signing**: `alloy::signers::local::PrivateKeySigner` for transaction signing

### Raw HTTP Fallback

**Limited Use**: Raw HTTP is used only when:
1. The native SDK doesn't support a specific RPC method (documented per-method)
2. Light client proofs require custom verification

All HTTP requests use strongly-typed request/response structures.

### Integration Tests

```bash
# Run with Sepolia testnet (requires ETH_RPC_URL)
export ETH_RPC_URL=https://rpc.sepolia.org
cargo test -p csv-adapter-ethereum --features rpc -- --test-threads=1
```

Test endpoints:
- Sepolia: `https://rpc.sepolia.org`
- Local Anvil: `http://localhost:8545`

## Security Notes

- Keccak-256 via `sha3` crate for Ethereum-compatible hashing
- secp256k1 ECDSA for transaction signing
- EIP-1559 transaction format supported
- Domain-separated proof verification via `alloy_trie`
