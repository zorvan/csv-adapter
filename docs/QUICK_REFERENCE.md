# CSV Adapter - Quick Reference Card

**Last Updated:** April 10, 2026  
**Status:** ✅ Builds successfully, 427 tests passing

---

## Current State

```
Build: ✅ SUCCESS
Tests: ✅ 427 PASSING (all crates)
  - csv-adapter-core: 221 tests
  - csv-adapter-bitcoin: 75 tests
  - csv-adapter-ethereum: 60 tests
  - csv-adapter-sui: 48 tests
  - csv-adapter-aptos: 10 tests
  - csv-adapter-store: 3 tests
  - Integration tests: 10 tests
```

---

## Network Support

### Bitcoin
| Network | Status | Default RPC | Finality |
|---------|--------|-------------|----------|
| Mainnet | ✅ Ready | `127.0.0.1:8332` | 6 blocks |
| Testnet3 | ✅ Ready | `127.0.0.1:18332` | 6 blocks |
| **Signet** | ✅ **Default** | `127.0.0.1:38332` | 6 blocks |
| Regtest | ✅ Ready | `127.0.0.1:18443` | 1 block |

### Ethereum
| Network | Status | Default RPC | Finality |
|---------|--------|-------------|----------|
| Mainnet | ✅ Ready | `127.0.0.1:8545` | Checkpoint |
| **Sepolia** | ✅ **Default** | `127.0.0.1:8545` | 15 blocks |
| Holesky | ⚠️ Planned | - | 15 blocks |
| Dev | ✅ Ready | `127.0.0.1:8545` | 1 block |

### Sui
| Network | Status | Default RPC | Finality |
|---------|--------|-------------|----------|
| Mainnet | ✅ Ready | `fullnode.mainnet.sui.io:443` | Certified |
| **Testnet** | ✅ **Default** | `fullnode.testnet.sui.io:443` | Certified |
| Devnet | ✅ Ready | `fullnode.devnet.sui.io:443` | Certified |
| Local | ✅ Ready | `127.0.0.1:9000` | 1 checkpoint |

### Aptos
| Network | Status | Default RPC | Finality |
|---------|--------|-------------|----------|
| Mainnet | ✅ Ready | `fullnode.mainnet.aptoslabs.com/v1` | HotStuff 2f+1 |
| **Testnet** | ✅ **Default** | `fullnode.testnet.aptoslabs.com/v1` | HotStuff 2f+1 |
| Devnet | ✅ Ready | `fullnode.devnet.aptoslabs.com/v1` | HotStuff 2f+1 |

---

## Critical Gaps (Priority Order)

### 🔴 Critical - Must Fix Before Production

1. **Signature Scheme Mismatches** (30 minutes)
   - Files: `csv-adapter-sui/src/adapter.rs:~420`, `csv-adapter-aptos/src/adapter.rs:~420`
   - Fix: Change `Secp256k1` to `Ed25519`
   - Impact: BREAKING - proof verification fails without this

2. **Bitcoin Proof Extraction** (4-6 hours)
   - File: `csv-adapter-bitcoin/src/proofs.rs:146-158`
   - Fix: Extract real Merkle branches from blocks
   - Impact: Cannot produce verifiable proofs

3. **Real RPC Wiring** (8-12 hours)
   - Sui/Aptos: Create `real_rpc.rs` modules
   - Bitcoin: Wire existing `real_rpc.rs` to adapter
   - Impact: Cannot publish real transactions

### 🟡 High - Should Fix Soon

4. **Rollback Implementation** (2 hours)
   - All adapters: Actually unmark seals on reorg
   - Impact: Reorgs leave seals permanently marked

5. **Integration Tests** (2-3 days)
   - End-to-end tests on testnets
   - Impact: No confidence in real-world behavior

### 🟠 RGB Compatibility - For RGB Interop

6. **RGB Tapret Verification** (1-2 days)
   - Verify Bitcoin Tapret matches RGB specification
   - Impact: Cannot interoperate with RGB tools

7. **Consignment Format** (1-2 days)
   - Wire-compatible with RGB consignment format
   - Impact: Cannot exchange state with RGB peers

---

## Documentation

| Document | Purpose | Path |
|----------|---------|------|
| **README** | Project overview | `/README.md` |
| **Production Readiness** | Complete roadmap | `/docs/PRODUCTION_READINESS_RGB.md` |
| **Bitcoin RGB Guide** | RGB compatibility details | `/docs/BITCOIN_RGB_COMPATIBILITY.md` |
| **Implementation Analysis** | Detailed code analysis | `/docs/IMPLEMENTATION_ANALYSIS.md` |
| **Rewrite Status** | Current implementation state | `/REWRITE_STATUS.md` |

---

## Quick Commands

```bash
# Build entire workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Run tests for specific adapter
cargo test -p csv-adapter-bitcoin
cargo test -p csv-adapter-ethereum
cargo test -p csv-adapter-sui
cargo test -p csv-adapter-aptos

# Build with RPC support
cargo build -p csv-adapter-bitcoin --features rpc
cargo build -p csv-adapter-ethereum --features rpc

# Check for compilation errors
cargo check --workspace

# Run linter
cargo clippy --workspace
```

---

## Key Libraries

| Blockchain | Library | Version | Purpose |
|------------|---------|---------|---------|
| Bitcoin | `bitcoin` | 0.30 | Block/tx parsing, Merkle trees |
| Bitcoin | `bitcoin_hashes` | 0.12 | Hash types |
| Bitcoin | `bitcoincore-rpc` | 0.17 | Node RPC (optional) |
| Ethereum | `alloy` | 0.9 | Transaction building, signing |
| Ethereum | `alloy-sol-types` | 0.8 | ABI encoding |
| Sui/Aptos | `ed25519-dalek` | 2.0 | Signature verification |
| Sui | `sui-sdk` | 0.0.0 ⚠️ | Placeholder - needs update |
| Aptos | `aptos-sdk` | 0.4 | Optional - not wired |
| All | `rusqlite` | 0.30 | Persistence |

---

## Architecture

```
┌──────────────────────────────────────────┐
│          csv-adapter-core                 │
│  AnchorLayer trait + shared types        │
└──────────────────────────────────────────┘
         │         │         │         │
    ┌────┴───┐ ┌───┴────┐ ┌─┴────┐ ┌┴─────┐
    │Bitcoin │ │Ethereum│ │ Sui  │ │Aptos │
    │(rust-  │ │(Alloy) │ │(sdk) │ │(sdk) │
    │bitcoin)│ │        │ │      │ │      │
    └────────┘ └────────┘ └──────┘ └──────┘
```

Each adapter implements:
- `publish()` - Anchor commitment to blockchain
- `verify_inclusion()` - Extract inclusion proof
- `verify_finality()` - Verify finality per chain rules
- `enforce_seal()` - Prevent seal replay
- `create_seal()` - Create new authorization token
- `hash_commitment()` - Compute commitment hash
- `build_proof_bundle()` - Build verifiable proof
- `rollback()` - Handle chain reorgs

---

## Next Steps

1. **Today**: Fix signature schemes (30 min)
2. **This week**: Implement proof extraction (2-3 days)
3. **Next week**: Wire real RPC clients (3-4 days)
4. **Following**: Testing, hardening, RGB verification

**Estimated to Production:** 6 weeks (60-80 hours total)

---

## Contact & Resources

- [RGB Protocol](https://rgb.tech/)
- [LNP/BP Standards](https://github.com/LNP-BP)
- [rust-bitcoin Docs](https://docs.rs/bitcoin)
- [Alloy Documentation](https://alloy.rs/)
- [Sui SDK Docs](https://docs.sui.io/sdk)
- [Aptos SDK Docs](https://aptos.dev/sdks)
