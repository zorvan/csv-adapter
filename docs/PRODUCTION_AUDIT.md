# CSV Adapter Production Audit Report

**Date:** May 1, 2026  
**Document:** docs/PRODUCTION_AUDIT.md  
**Purpose:** Validate codebase against docs/PRODUCTION_GUARANTEE_PLAN.md requirements  
**Status:** ❌ **NOT PRODUCTION READY**

---

## Executive Summary

This audit validates the CSV Adapter codebase against the Production Guarantee Plan. The project is **architecturally moving toward production readiness, but not yet guaranteeable**.

**Overall Compliance:** Phase 1-2 partially complete, Phases 3-9 require significant work.

---

## Phase 1: Production Surface Audit

### Status: ❌ FAILING

**Audit Script:** `scripts/audit-production-surface.sh`

**Findings:** 74 files contain forbidden markers in production code.

#### Critical Violations by Category

| Pattern | File Count | Severity |
|---------|------------|----------|
| `mock` | 40+ | HIGH |
| `placeholder` | 15+ | HIGH |
| `TODO/FIXME` | 12+ | MEDIUM |
| `simulation/simulate` | 8+ | MEDIUM |
| `stub` | 5+ | MEDIUM |

#### Top Violation Files

| File | Violations | Primary Issues |
|------|------------|----------------|
| `csv-adapter-aptos/src/rpc.rs` | 26 | Mock RPC implementation in production path |
| `csv-adapter-ethereum/src/rpc.rs` | 19 | Mock RPC implementation in production path |
| `csv-adapter-bitcoin/src/rpc.rs` | 18 | Mock RPC implementation in production path |
| `csv-adapter-sui/src/rpc.rs` | 17 | Mock RPC implementation in production path |
| `csv-adapter-aptos/src/adapter.rs` | 14 | Demo/simulation paths |
| `csv-adapter/src/proofs.rs` | 11 | Simulation functions, unimplemented! |
| `csv-explorer/indexer/src/indexer_plugin.rs` | 10 | Demo markers |

#### Critical Issues Found

1. **Mock RPCs in Production Path** (`csv-adapter-{chain}/src/rpc.rs`)
   - Mock implementations return fake transaction hashes (`[0xAB; 32]`, `[0xCD; 32]`)
   - Test code (`#[cfg(test)]`) is mixed with production code
   - The `submit_transaction` methods return deterministic fake hashes in tests
   - **Risk:** Production code could call mock implementations

2. **Placeholder Implementations**
   - `csv-adapter-core/src/schema.rs:439` - placeholder scripts
   - `csv-adapter-ethereum/src/chain_operations.rs:346` - placeholder signing
   - `csv-adapter-bitcoin/src/adapter.rs:285` - placeholder RPC backend
   - `csv-adapter-bitcoin/src/deploy.rs:98` - placeholder txid generation

3. **Simulation/Demo Paths**
   - `csv-adapter/src/proofs.rs:22` - `SimulationResult` struct in production
   - `csv-adapter/src/proofs.rs:52` - `simulate()` method exposed in API
   - `csv-cli/src/commands/cross_chain/transfer.rs` - demo transfer with placeholder proofs
   - `csv-wallet/src/pages/nft_page.rs` - demo NFT data

4. **Unimplemented! Macros**
   - Found in core proof generation paths
   - Should return `CapabilityUnavailable` errors instead

#### Script Bug Identified

**Issue:** The audit script exits with code 0 despite finding violations. The violation counting logic has a bug where it doesn't properly set the exit code.

**Location:** `scripts/audit-production-surface.sh:199`

---

## Phase 2: Single Chain Operation API

### Status: ✅ MOSTLY COMPLETE

**Core Traits Defined:** `csv-adapter-core/src/chain_operations.rs`

| Trait | Status | Notes |
|-------|--------|-------|
| `ChainQuery` | ✅ | Complete with all required methods |
| `ChainSigner` | ✅ | Complete with all required methods |
| `ChainBroadcaster` | ✅ | Complete with all required methods |
| `ChainDeployer` | ✅ | Complete with all required methods |
| `ChainProofProvider` | ✅ | Complete with all required methods |
| `ChainRightOps` | ✅ | Complete with all required methods |
| `FullChainAdapter` | ✅ | Blanket implementation provided |

**Implementation Status by Chain:**

| Chain | Implements Core Traits | Production Ready |
|-------|------------------------|------------------|
| Bitcoin | ✅ | ❌ (mock paths remain) |
| Ethereum | ✅ | ❌ (mock paths remain) |
| Sui | ✅ | ❌ (mock paths remain) |
| Aptos | ✅ | ❌ (mock paths remain) |
| Solana | ✅ | ❌ (mock paths remain) |

**Duplicate Implementation Check:**

```bash
# Per Guarantee Plan Phase 2 exit gate:
rg "csv_adapter_(bitcoin|ethereum|sui|aptos|solana)" csv-cli csv-wallet csv-explorer --glob '*.rs'
```

**Findings:**
- `csv-cli/src/commands/cross_chain/` contains direct chain-specific logic
- `csv-wallet/src/services/blockchain/` contains duplicate chain handling
- These should call `csv-adapter` facade exclusively

---

## Phase 3: Native SDK Enforcement

### Status: ⚠️ PARTIAL

#### Bitcoin Adapter (`csv-adapter-bitcoin/Cargo.toml`)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| `bitcoin` crate | ✅ | v0.32 with serde, rand, consensus |
| `bitcoincore-rpc` | ✅ | v0.19 (optional, production feature) |
| `bitcoin_hashes` | ✅ | v0.14 |
| `secp256k1` | ✅ | v0.29 |
| BIP-32/39/86 | ✅ | `bip32` v0.5 with mnemonic support |

**Compliance:** ✅ Meets requirements

#### Ethereum Adapter (`csv-adapter-ethereum/Cargo.toml`)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Alloy stack | ⚠️ PARTIAL | `alloy` v1.8 (optional, rpc feature) |
| ABI support | ⚠️ | `alloy-sol-types` v1.5 (optional) |
| Contract deploy | ⚠️ | `alloy-contract` v1.0 (optional) |
| EIP-1559 | ❓ | Not verified in dependencies |

**Compliance:** ⚠️ Alloy stack present but feature-gated; needs verification of full EIP-1559 support

#### Sui Adapter (`csv-adapter-sui/Cargo.toml`)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Sui SDK | ❌ | Commented out due to core2 dependency issue |
| JSON-RPC | ⚠️ | `reqwest` for direct HTTP (raw HTTP, not native SDK) |
| BCS | ✅ | `bcs` v0.1 (optional) |
| Ed25519 | ✅ | `ed25519-dalek` v2.0 |

**Compliance:** ❌ **FAILING** - Native Sui SDK not integrated (dependency issues). Uses raw HTTP instead.

#### Aptos Adapter (`csv-adapter-aptos/Cargo.toml`)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Aptos SDK | ⚠️ | `aptos-sdk` v0.4 (optional) |
| REST types | ❓ | Not verified |
| BCS | ✅ | `bcs` v0.1 (optional) |
| Ed25519 | ✅ | `ed25519-dalek` v2.0 |

**Compliance:** ⚠️ Partial - SDK optional, not default

#### Solana Adapter (`csv-adapter-solana/Cargo.toml`)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Solana SDK | ✅ | `solana-sdk` v3.0 |
| Solana Program | ✅ | `solana-program` v3.0 |
| RPC Client | ✅ | `solana-rpc-client` v3.1 |
| Anchor | ❌ | No Anchor SDK found |
| Loader interfaces | ✅ | `solana-loader-v3-interface` v3.0 |

**Compliance:** ⚠️ Good Solana native support, but **no Anchor SDK** for program bindings (requirement not met)

### Raw HTTP Usage Assessment

| Chain | Raw HTTP | Justification |
|-------|----------|---------------|
| Bitcoin | ❌ No | Uses `bitcoincore-rpc` |
| Ethereum | ⚠️ Indirect | Uses `alloy` when available, reqwest fallback |
| Sui | ✅ Yes | Native SDK commented out - **violates plan** |
| Aptos | ⚠️ Partial | SDK optional, reqwest available |
| Solana | ❌ No | Uses official `solana-rpc-client` |

**Recommendation:** Sui adapter needs native SDK integration or documented justification for raw HTTP.

---

## Phase 4: Remove All Production Stubs and Simulations

### Status: ❌ FAILING

#### Fake/Deterministic Outputs Found

| Location | Issue | Risk |
|----------|-------|------|
| `csv-adapter-aptos/src/rpc.rs:310` | `submit_transaction` returns `[0xAB; 32]` | Fake tx hash in test code |
| `csv-adapter-aptos/src/rpc.rs:359` | Hash prefixed with `b"mock"` | Fake deterministic hash |
| `csv-adapter-sui/src/chain_operations.rs:901` | Zero hash placeholder | `0x0000...0000` |
| `csv-cli/src/commands/cross_chain/transfer.rs` | `build_demo_merkle_proof()` | Demo proof, not real |
| `csv-cli/src/commands/cross_chain/utils.rs` | Placeholder tx hashes | Multiple locations |

#### Stub Implementations

| Location | Issue |
|----------|-------|
| `csv-adapter-core/src/vm.rs:192` | `PassthroughVM` labeled as "stub implementation" |
| `csv-adapter-bitcoin/src/rpc.rs:111-133` | Test functions with "stub" in name in production file |
| `csv-adapter-ethereum/src/chain_operations.rs:346` | Placeholder signing implementation |

#### Simulation in Production

| Location | Issue |
|----------|-------|
| `csv-adapter/src/proofs.rs` | `SimulationResult` and `simulate()` exposed in public API |
| `csv-adapter-sui/src/adapter.rs:557` | "Return simulated anchor" comment |
| `csv-adapter-sui/src/error.rs:40` | "transaction simulation error" (may be valid if chain supports) |

#### Required Actions

1. Remove all `#[cfg(test)]` implementations from production-visible modules
2. Replace `SimulationResult` with real proof generation or remove from API
3. Replace placeholder transaction hashes with real RPC results or proper errors
4. Remove or rename functions containing "stub", "mock", "demo", "placeholder" in production paths

---

## Phase 5: Wallet and CLI Convergence

### Status: ❌ FAILING

#### CLI Analysis (`csv-cli/src/`)

| Criterion | Status | Evidence |
|-----------|--------|----------|
| No direct chain imports | ❌ | `csv-cli/src/commands/cross_chain/` imports chain-specific modules |
| No duplicate signing | ❌ | `sign_ethereum_transaction()` in `cross_chain/ethereum.rs` |
| No duplicate broadcast | ❌ | `send_raw_ethereum_transaction()` in cross-chain code |
| Keystore-only keys | ⚠️ | `export_private_key()` exists, gets keys from config/state |
| No plaintext mnemonics | ⚠️ | `export_mnemonic()` with dev mode fallback to env var |

**Critical Issues:**

1. **Direct Chain Logic in CLI** (`csv-cli/src/commands/cross_chain/`)
   - `bitcoin.rs` - Direct Bitcoin RPC handling
   - `ethereum.rs` - Direct Ethereum transaction building and signing
   - `aptos.rs` - Direct Aptos module interaction
   - These should use `csv-adapter` facade only

2. **Private Key Handling** (`csv-cli/src/commands/wallet/import_export.rs`)
   ```rust
   // Lines 220-235: Dev mode allows env var mnemonic
   let dev_mode = std::env::var("CSV_DEV_MODE").map(|v| v == "1").unwrap_or(false);
   if dev_mode {
       if let Ok(mnemonic) = std::env::var("CSV_WALLET_MNEMONIC") {
           return Ok(mnemonic);
       }
   }
   ```

3. **Placeholder XPub Generation** (`csv-cli/src/commands/wallet/import_export.rs:203-209`)
   ```rust
   let placeholder_xpub = format!("xpub{}_{}", chain.to_string(), &address[..8]);
   ```

#### Wallet Analysis (`csv-wallet/src/`)

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Uses facade only | ⚠️ | `blockchain/service.rs` uses `CsvClient`, but also has duplicate logic |
| No duplicate chain APIs | ⚠️ | Duplicate signer/submitter modules exist |
| Encrypted keystore | ✅ | References `csv-adapter-keystore` |
| No mock data in UI | ❌ | `pages/nft_page.rs` has demo data |

**Findings:**
- `csv-wallet/src/services/blockchain/service.rs:30` - Uses `CsvClient` from `csv-adapter`
- But also has `signer.rs`, `submitter.rs` that may duplicate chain logic
- `csv-wallet/src/pages/nft_page.rs` contains "demo" and "simulation" markers

---

## Phase 6: Explorer Plugin Scalability

### Status: ✅ MOSTLY COMPLETE

#### Shared Schema Assessment

| Requirement | Status | Evidence |
|-------------|--------|----------|
| `RightCreated` event | ✅ | `csv-explorer/shared/src/events.rs` |
| `RightConsumed` event | ✅ | Present |
| `CrossChainLock` event | ✅ | Present |
| `CrossChainMint` event | ✅ | Present |
| `CrossChainRefund` event | ✅ | Present |
| `RightTransferred` event | ✅ | Present |
| `NullifierRegistered` event | ✅ | Present |
| `RightMetadataRecorded` event | ✅ | Present |

#### Standard Metadata Fields

| Field | Status | Location |
|-------|--------|----------|
| `right_id` | ✅ | `RightEvent` struct |
| `commitment` | ✅ | `RightEvent` |
| `owner` | ✅ | `RightEvent` |
| `chain_id` | ✅ | `RightEvent` |
| `asset_class` | ✅ | `RightEvent` |
| `asset_id` | ✅ | `RightEvent` |
| `metadata_hash` | ✅ | `RightEvent` |
| `proof_system` | ✅ | `ProofEvent` |
| `proof_root` | ✅ | `ProofEvent` |
| `source_chain` | ✅ | `CrossChainEvent` |
| `destination_chain` | ✅ | `CrossChainEvent` |
| `tx_hash` | ✅ | `RightEvent` |
| `block_height` | ✅ | `RightEvent` |
| `finality_status` | ✅ | `RightEvent` |

#### Plugin Architecture

| Requirement | Status | Evidence |
|-------------|--------|----------|
| `ChainIndexerPlugin` trait | ✅ | `csv-explorer/indexer/src/indexer_plugin.rs` |
| Plugin registry | ✅ | `IndexerPluginRegistry` with factory pattern |
| Per-chain indexers | ✅ | `bitcoin.rs`, `ethereum.rs`, `sui.rs`, `aptos.rs`, `solana.rs` |
| No chain logic in UI | ❓ | Needs verification |

**Exit Gate:** Explorer can index all 5 chains using plugin registration - **PASS**

---

## Phase 7: Cryptography and Key Security Hardening

### Status: ✅ MOSTLY COMPLETE

#### Cryptographic Libraries

| Requirement | Status | Evidence |
|-------------|--------|----------|
| BIP-39 | ✅ | `bip39` v2.0 in workspace deps |
| BIP-32/44/86 | ✅ | `bip32` v0.5 with mnemonic features |
| secp256k1 | ✅ | `secp256k1` v0.28-0.29 |
| Ed25519 | ✅ | `ed25519-dalek` v2.0 |
| Domain-separated hashes | ❓ | Needs code review |
| Canonical serialization | ❓ | BCS present, needs verification |
| AES-GCM | ✅ | `aes-gcm` v0.10 in workspace |
| Memory zeroization | ✅ | `zeroize` v1.7 with derive |

#### Key Storage

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Encrypted keystore | ✅ | `csv-adapter-keystore` crate exists |
| No plaintext persistence | ⚠️ | CLI has dev mode env var fallback |
| User authorization | ❓ | Needs review |
| Validation before broadcast | ❓ | Needs verification |

---

## Phase 8: Contract and Program Production Readiness

### Status: ❓ NEEDS VERIFICATION

#### Contract Vocabulary

| Event | Required | Status |
|-------|----------|--------|
| `RightCreated` | ✅ | Needs verification in contract source |
| `RightConsumed` | ✅ | Needs verification |
| `CrossChainLock` | ✅ | Needs verification |
| `CrossChainMint` | ✅ | Needs verification |
| `CrossChainRefund` | ✅ | Needs verification |
| `RightTransferred` | ✅ | Needs verification |
| `NullifierRegistered` | ✅ | Needs verification |
| `RightMetadataRecorded` | ✅ | Needs verification |

#### Build Status

| Chain | Build Command | Status |
|-------|---------------|--------|
| Ethereum | `forge build` | ❓ Not tested |
| Sui | `sui move build` | ❓ Not tested |
| Aptos | `aptos move compile` | ❓ Not tested |
| Solana | `NO_DNA=1 anchor build` | ❓ Not tested |

**Note:** Contract source files need to be located and built to verify this phase.

---

## Phase 9: CI Guarantee Gates

### Status: ❓ PARTIAL

#### Available CI Checks

| Check | Status | Evidence |
|-------|--------|----------|
| `cargo fmt --check` | ❓ | Not verified |
| `cargo check --all-features` | ❓ | Not verified |
| `cargo test --all-features` | ❓ | Not verified |
| WASM build check | ❓ | Not verified |
| Contract builds | ❓ | Not verified |
| Production surface audit | ❌ | Script exists but has bug (returns 0 on violations) |
| Dependency audit | ❓ | Not verified |

#### GitHub Actions Review

Need to check `.github/workflows/` for configured CI jobs.

---

## Summary by Requirement

| Guarantee Requirement | Status | Blockers |
|-----------------------|--------|----------|
| New chain scalability | ⚠️ Partial | Phase 1, 3, 5 issues |
| Single implementation | ❌ No | CLI/wallet duplicate chain logic |
| Native SDK usage | ⚠️ Partial | Sui SDK missing, Solana no Anchor |
| No stubs/placeholders | ❌ No | 74 files with violations |
| Security first | ⚠️ Partial | Dev mode env var mnemonic |

---

## Recommendations by Priority

### Critical (Block Production)

1. **Fix Audit Script Bug** - Ensure exit code 1 on violations
2. **Remove Mock RPC from Production Path** - Move to test-only modules
3. **Eliminate Placeholder Txid Generation** - Return errors or real RPC results
4. **Fix CLI Direct Chain Imports** - Route through `csv-adapter` facade
5. **Remove Dev Mode Mnemonic Fallback** - Enforce encrypted keystore only

### High Priority

6. **Integrate Sui SDK** - Resolve core2 dependency issue
7. **Add Anchor SDK for Solana** - Required by guarantee plan
8. **Remove SimulationResult from Public API** - Test-only or implement real proofs
9. **Verify Contract Builds** - Run all Phase 8 build commands
10. **Configure Complete CI Gates** - Implement all Phase 9 checks

### Medium Priority

11. **Document Raw HTTP Justifications** - For any chain using non-native SDK
12. **Verify Domain Separated Hashing** - Security audit
13. **Complete Explorer Plugin Testing** - Verify all 5 chains index correctly

---

## Definition of Done Checklist

| Criterion | Status |
|-----------|--------|
| All phases complete | ❌ |
| All exit gates pass | ❌ |
| PRODUCTION_AUDIT.md has zero unresolved findings | ❌ (this document) |
| CLI, wallet, explorer use unified facade | ❌ |
| Every operation uses real chain or typed error | ❌ |
| Mocks only in tests | ❌ |

---

**Status:** Architecturally moving toward production readiness, but **not yet guaranteeable**.

**Next Review Date:** After Phase 4 completion (stub removal)

**Audited By:** Cascade AI Assistant  
**Audit Date:** May 1, 2026
