# Native SDK Compliance - Bitcoin Adapter

**Status**: Production Ready
**Last Updated**: May 2026

## Native SDK Dependencies

| Crate | Version | Purpose | Production Use |
|-------|---------|---------|----------------|
| `bitcoin` | 0.32 | Core Bitcoin library (transactions, scripts, addresses) | Yes |
| `bitcoincore-rpc` | 0.19 | RPC client for Bitcoin Core/bitcoind | Yes (with `rpc` feature) |
| `secp256k1` | 0.29 | Elliptic curve cryptography (signing/verification) | Yes |
| `bip32` | 0.5 | HD wallet derivation (BIP-32/39/44) | Yes |
| `bitcoin_hashes` | 0.14 | Bitcoin hash functions | Yes |

## Compliance with Production Guarantee Plan Phase 3

### Native SDK Usage

This adapter uses the official `bitcoin` crate from rust-bitcoin, the community-standard Rust implementation for Bitcoin.

- **Transaction Building**: `bitcoin::Transaction`, `bitcoin::TxIn`, `bitcoin::TxOut`
- **Address Handling**: `bitcoin::Address`, BIP-86 Taproot derivation via `bip32`
- **Script Operations**: `bitcoin::ScriptBuf`, `bitcoin::opcodes`
- **Cryptography**: `secp256k1` for signing (Schnorr for Taproot)
- **RPC**: `bitcoincore-rpc` for bitcoind integration

### Raw HTTP Fallback

**Not Used**: All Bitcoin operations use the native `bitcoin` crate types. No raw HTTP fallback is required as the `bitcoin` crate provides comprehensive transaction construction and script handling.

### Integration Tests

Integration tests require a running Bitcoin node:

```bash
# Run with regtest node
cargo test -p csv-adapter-bitcoin --features rpc -- --test-threads=1
```

Test endpoints:
- Regtest: `http://localhost:18443`
- Signet REST API: `https://mempool.space/signet/api` (for `signet-rest` feature)

## Security Notes

- All cryptographic operations use the audited `secp256k1` library
- BIP-32 derivation follows standard hardened paths
- Taproot (BIP-86) is the default address type for new seals
