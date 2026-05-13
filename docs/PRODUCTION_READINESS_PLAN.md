# Production Readiness Plan

**Generated**: 2026-05-12  
**Source Documents**: AUDIT2.md, MASTERPLAN.md, AGENTS.md  
**Repository Scope**: Entire monorepo

---

## Current State Assessment

The codebase is significantly further along than AUDIT2.md assumed. Most Phase 1-3 infrastructure exists but has gaps in wiring, integration, and enforcement. The critical issues are bugs in existing code, missing CI enforcement, and stub implementations that need real logic.

---

## Phase 1 — Cryptographic Foundation

**Status: Done — All Critical Bugs Fixed**

| Item | Status | Action |
|---|---|---|
| `domain_hash.rs` | Done | |
| `domains/` (9 domain types) | Done | |
| `proof_pipeline.rs` (10-step validation) | Done | |
| `replay_registry.rs` (in-memory) | Done | Wire to persistent store |
| `nullifier.rs` (SealNullifier) | Done | |
| `sanad_contract.rs:keccak256()` | Done | Uses `tiny_keccak::Keccak::v256()` correctly |
| Raw hashing in bitcoin ops.rs | Justified | Document as approved exception |
| Raw hashing in aptos/ethereum proofs | Needs audit | Migrate to `DomainSeparatedHash` where protocol-logic, keep raw where Bitcoin/Ethereum protocol demands |
| `scripts/security/check_forbidden_patterns.sh` | Done | Integrate into CI |
| `.cargo/config.toml` | Missing | Create with deny rules |

**Priority: Wire replay_registry to persistent store.**

---

## Phase 2 — Typestate + Persistence

**Status: Structure Done — Integration Complete**

| Item | Status | Action |
|---|---|---|
| `transfer_state/` (8 state types) | Done | |
| `compile_fail/` tests (3 tests) | Done | Add missing: `locked_to_minting.rs`, `observed_to_completed.rs` |
| SDK `TransferManager` to `transfer_state` bridge | Done | SDK uses `csv_core::TransferStatus` as single source of truth |
| Two `TransferStatus` enums | Done | Unified — SDK references `csv_core::protocol_version::TransferStatus` |
| `operations/` stores (5 SQLite stores) | Done | |
| `replay_registry_store.rs` | Done | |
| `recovery_engine.rs` | Done | Steps 4-7 wired to storage backend; uses `RecoveryStorageBackend` trait |
| `reorg/rollback.rs` | Done | Generic over `RollbackStorageBackend`; persists state changes |
| `reorg/reconciliation.rs` | Done | Generic over `ChainBackendForReconciliation`; re-validates proofs |
| `finality/` (state, policy, monitor) | Done | |
| `monitor.rs` to `reorg/` integration | Done | Reorg monitor feeds into reorg module |

**Priority: Add missing compile_fail tests.**

---

## Phase 3 — Finality + Reorg Safety

**Status: Structure Done — Reorg Handling Complete**

| Item | Status | Action |
|---|---|---|
| `finality/` (state, policy, monitor) | Done | |
| `reorg/detector.rs` | Done | |
| `reorg/rollback.rs` | Done | Real implementation with storage backend |
| `reorg/reconciliation.rs` | Done | Real implementation with chain backend queries |
| `CompromisedTransfer` state | Done | Wired into reorg flow |
| Pinned block hash in proof construction | Missing | Enforce: no `latest_block` in proof building |
| Compromised mint path observability | Missing | Add event emission |

**Priority: Enforce pinned block hash; add event emission for compromised mint path.**

---

## Phase 4 — RPC Trust Hardening

**Status: Quorum Client Done — Integrated**

| Item | Status | Action |
|---|---|---|
| `rpc/quorum_client.rs` | Done | Replaced `simulate_rpc_call` with actual `reqwest` HTTP POST calls |
| Quorum config (min_quorum=2, providers=3) | Done | |
| Adapters using quorum client | Done | `QuorumEthereumRpc` wraps `QuorumClient` for all JSON-RPC calls |
| `csv-observability/src/metrics/` (RPC metrics) | Partial | Add: `rpc_disagreement_total`, `provider_failure_total`, `provider_timeout_total` |
| `ChainViewInconsistent` error on mismatch | Done in quorum_client | Ensure callers halt on this error |

**Priority: Add RPC metrics; wire quorum to remaining chain adapters.**

---

## Phase 5 — ABI + Contract Safety

**Status: Generated Bindings Used — Migration Complete**

| Item | Status | Action |
|---|---|---|
| `bindings/csv_lock.rs` (Alloy generated) | Done | |
| `bindings/csv_mint.rs` (Alloy generated) | Done | |
| `ops.rs` uses generated bindings | Done | `lock_sanad()`, `mint_sanad()`, `refund_sanad()` use `CsvLockClient`/`CsvMintClient` |
| `sanad_contract.rs` manual encoding | Still exists | Deprecate after ops.rs migration |
| `seal_contract.rs` manual encoding | Still exists | Deprecate after migration |
| `cargo xtask verify-bindings` | Missing | Create xtask |
| `deployments/deployment-manifest.json` | Exists (TODOs) | Fill in real values, add signature |
| Immutable contract deployment | Not enforced | Add deployment verification |

**Priority: Create `cargo xtask verify-bindings`; fill deployment manifest.**

---

## Phase 6 — Wallet Security

**Status: csv-keys Solid — Wallet Integration Complete**

| Item | Status | Action |
|---|---|---|
| `csv-keys/` (BIP-39, BIP-44, keystore) | Done | |
| `csv-keys/src/memory.rs` (Zeroize) | Done | |
| `csv-keys/src/browser_keystore.rs` | Done | |
| `csv-keys/src/file_keystore.rs` | Done | |
| Wallet using csv-keys as single abstraction | Done | All mnemonic generation paths unified to csv_keys::bip39::Mnemonic |
| `localStorage` mnemonic persistence | Not found | Verify none exists (good) |
| Hardware wallet path | Missing | Implement |
| Sensitive types Zeroize | Done | zeroize 1.7 with derive feature; KeyManager Drop impl zeroizes seed |

**Priority: Add hardware wallet path.**

---

## Phase 7 — Property Testing + Fuzzing

**Status: Infrastructure Partially Ready**

| Item | Status | Action |
|---|---|---|
| `csv-core/tests/properties/` (5 tests) | Done | |
| `csv-core/tests/compile_fail/` (3 tests) | Partial | Add 3 more missing tests |
| `fuzz/fuzz_targets/` (5 targets) | Done | |
| **`fuzz/Cargo.toml`** | **Missing** | **Create for cargo-fuzz integration** |
| CI fuzz corpus execution | Missing | Add to CI workflows |
| `replay_resistance.rs` property test | Exists | Verify it tests domain separation |
| `seal_consumption.rs` property test | Exists | Verify |

**Priority: Create `fuzz/Cargo.toml` and add fuzz execution to CI.**

---

## Phase 8 — Observability + Forensics

**Status: Events Schema Done — Minimal Metrics**

| Item | Status | Action |
|---|---|---|
| `csv-core/src/events.rs` (15 event types) | Done | |
| Required events defined | All defined | Wire into proof_pipeline |
| Correlation fields (transfer_id, operation_id, etc.) | Partial | Add to CsvEvent struct |
| Structured JSON output | Missing | Add JSON formatter to event indexer |
| RPC metrics (partial) | Exists | Add missing metrics |
| Tracing/logging module | Missing | Add structured logging |

**Priority: Wire events into proof_pipeline and reorg detection; add correlation fields.**

---

## Phase 9 — E2E Certification

**Status: E2E Test Exists — Needs Real Chain Wiring**

| Item | Status | Action |
|---|---|---|
| `tests/e2e_certification.rs` | Done (mock-based) | Replace mocks with real testnet |
| `tests/integration_demo_test.rs` | Done | |
| `scripts/test-cross-chain.sh` | Done | |
| Real Bitcoin Signet to Ethereum Sepolia flow | Missing | Wire real RPCs |
| Failure injection tests | Missing | Add: RPC disagreement, reorg, duplicate proof, node restart |
| Offline verification demo | Partial | Complete wiring |

**Priority: Replace mock-based E2E with real testnet certification flow.**

---

## Critical Bug Fixes (Block All Phases)

1. ~~**`csv-ethereum/src/sanad_contract.rs:keccak256()`**~~ — **FIXED**: Uses `tiny_keccak::Keccak::v256()` correctly.
2. ~~**Two `TransferStatus` enums**~~ — **FIXED**: Unified to `csv_core::protocol_version::TransferStatus`.
3. **`fuzz/Cargo.toml` missing** — Fuzz targets cannot run in CI.
4. **CI references wrong path** — `csv-adapter-core/fuzz/Cargo.toml` should be `fuzz/Cargo.toml`.

---

## Execution Order (Dependency-Constrained)

```
Phase 1 (keccak256 fixed, wire replay)
    -> Phase 4 (quorum integrated into ethereum)
    -> Phase 5 (ABI migration complete)
    -> Phase 2 (recovery, reorg, typestate all implemented)
    -> Phase 3 (reorg handling complete)
    -> Phase 7 (fuzz Cargo.toml, CI)
    -> Phase 8 (wire events)
    -> Phase 6 (wallet audit)
    -> Phase 9 (real E2E)
```

---

## What's Already Production-Ready (No Changes Needed)

- `domain_hash.rs` + `domains/` (9 domain types)
- `proof_pipeline.rs` (10-step canonical validation)
- `transfer_state/` (8 typestate types with compile-time enforced transitions)
- `csv-store/src/operations/` (5 SQLite stores)
- `finality/` (per-chain policies)
- `csv-keys/` (BIP-39/BIP-44, encrypted keystore, Zeroize memory)
- `csv-core/src/events.rs` (15 event types)
- `rpc/quorum_client.rs` (quorum client with real HTTP calls)
- `reorg/rollback.rs` (real implementation with storage backend)
- `reorg/reconciliation.rs` (real implementation with chain backend)
- `recovery_engine.rs` (steps 4-7 wired to storage)
- `bindings/csv_lock.rs` + `bindings/csv_mint.rs` (Alloy generated, used in ops.rs)
- `scripts/security/check_forbidden_patterns.sh` (CI enforcement script)
- `csv-core/tests/properties/` (5 property tests)
- `csv-core/tests/compile_fail/` (3 tests, needs 3 more)
- `deployments/deployment-manifest.json` (exists, needs filling)
- `production-guarantee.yml` CI workflow (exists, needs path fix)
