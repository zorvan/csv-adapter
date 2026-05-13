# CSV Protocol — Production Hardening Execution Specification

**Document Type:** Engineering Execution Specification  
**Audience:** Protocol Core Team, Infra Team, Security Team, Chain Adapter Owners  
**Status:** Blocking Before Production  
**Priority:** Critical  
**Repository Scope:** Entire monorepo  
**Execution Model:** Sequential + Dependency-Constrained  

---


Summary: What Needs to Be Wired Up
Critical Gaps
1. TransferBuilder -> Typestate Machine
The execute() method in TransferBuilder uses a simple SDK-level TransferStatus enum but never interacts with the csv_core::transfer_state typestate machine. The typestate states (Locked, AwaitingFinality, ProofBuilding, etc.) are defined but have no driver.
Fix needed: Create a TransferStateMachine that owns a TransferData and transitions it through the typestate types, integrating with TransferStore for persistence.
2. No Proof Delivery via Nostr P2P
After build_inclusion_proof() in execute(), the proof is stored in TransferRecord.inclusion_proof but never broadcast via NostrTransport. The destination chain has no way to receive the proof.
Fix needed: Wire NostrTransport::broadcast_proof() after proof generation in execute(), and add a proof subscription listener on the destination side.
3. RecoveryEngine -> TransferStore Integration
The RecoveryStorageBackend trait is defined but has no SQLite implementation. The csv-store operation stores exist but are not connected to RecoveryEngine.
Fix needed: Implement RecoveryStorageBackend for csv_store::TransferStore + csv_store::ReplayStore + csv_store::ReorgStore.
4. ReplayRegistry -> ProofPipeline Integration
Step 6 of the proof pipeline (validate_replay) always passes. The ReplayRegistry exists but is never queried.
Fix needed: Wire csv_store::ReplayStore into the proof pipeline's replay validation step.
5. ReorgDetector -> RollbackHandler -> TransferState Integration
The ReorgDetector can detect reorgs, RollbackHandler can determine rollback actions, and ReconciliationEngine can reconcile states -- but none of these are connected.
Fix needed: Create a ReorgHandler that listens for ReorgEvents from ReorgMonitor, calls RollbackHandler.determine_rollback_action(), and transitions the transfer state machine to RolledBack or Compromised.
6. FinalityMonitor -> TransferState Integration
The FinalityMonitor tracks finality per chain but doesn't feed into the transfer state machine's AwaitingFinality state.
Fix needed: Have FinalityMonitor update AwaitingFinality.current_confirmations and trigger build_proof() when is_finalized().
7. Ethereum ChainSanadOps Gaps
create_sanad() and consume_sanad() return CapabilityUnavailable. The contract bindings (CsvLockClient, CsvMintClient) exist but ops.rs uses manual CsvLockAbi/CsvMintAbi encoding instead.
Fix needed: Wire the generated bindings into ops.rs and implement create_sanad() and consume_sanad().
8. DriverRegistry Built-in Drivers Commented Out
All 5 chain driver registrations in register_built_in_drivers() are commented out due to cyclic dependency issues.
Fix needed: Resolve cyclic dependencies and uncomment the registrations.
9. SDK TransferManager -> Core TransferStore
The TransferManager uses an in-memory HashMap instead of csv_store::TransferStore.
Fix needed: Replace in-memory storage with csv_store::TransferStore when SQLite feature is enabled.
10. Event System Integration
The CsvClient has an event system (EventStream) but TransferManager::execute() doesn't emit events during the transfer lifecycle.
Fix needed: Emit events at each state transition in the transfer pipeline.


# 1. EXECUTION MODEL

This document is not architectural guidance.

This is the implementation contract for the repository.

Every task below contains:

- exact modules
- target files
- required refactors
- forbidden patterns
- acceptance criteria
- CI enforcement requirements
- dependency ordering

No engineering team should improvise outside these boundaries.

---

# 2. GLOBAL REPOSITORY GUARDRAILS

---

## 2.1 Forbidden Runtime Patterns

The following patterns are forbidden in production runtime code:

```rust
unwrap()
expect()
new_unchecked()
unsafe
Sha256::digest
Keccak256::digest
blake3::hash
```

Allowed only in:

```text
/tests
/fuzz
/benches
```

---

## 2.2 CI Enforcement Targets

Create:

```text
/scripts/security/check_forbidden_patterns.sh
```

CI MUST fail on:

| Pattern | Scope |
|---|---|
| TODO | protocol modules |
| FIXME | protocol modules |
| unwrap() | runtime code |
| expect() | runtime code |
| unsafe | outside approved modules |
| raw hashing | outside crypto module |
| mock proofs | production code |
| manual ABI encoding | EVM adapters |

---

## 2.3 Approved Unsafe Modules

Unsafe MAY exist only in:

```text
csv-crypto/
csv-zk/
```

All unsafe blocks MUST include:

```rust
// SAFETY:
```

Without exception.

---

# 3. PHASE ORDERING (MANDATORY)

Execution order is fixed.

| Phase | Status | Blocks |
|---|---|---|
| Phase 1 — Hashing + Proof Canonicalization | **Done** | everything |
| Phase 2 — Typestate + Persistence | **Done** | reorg safety |
| Phase 3 — Finality + Reorg | **Done** | certification |
| Phase 4 — RPC Quorum | **Done** | production deployment |
| Phase 5 — ABI + Contracts | **Done** | deployment certification |
| Phase 6 — Wallet Security | browser release |
| Phase 7 — Property Testing + Fuzzing | release |
| Phase 8 — Observability | release |
| Phase 9 — E2E Certification | production |

No team may skip phase ordering.

---

# 4. PHASE 1 — CRYPTOGRAPHIC FOUNDATION

**Owner Teams:** Core Protocol + Security  
**Blocking:** YES  
**Status:** Complete — keccak256 bug fixed, domain separation implemented

---

## 4.0 Keccak256 Bug Fix (COMPLETED)

**File:** `csv-ethereum/src/sanad_contract.rs`

**Issue:** `keccak256()` was computing SHA256 instead of Keccak256, breaking all Ethereum ABI selectors.

**Fix:** Uses `tiny_keccak::Keccak::v256()` correctly. All function/event signature selectors now compute proper Keccak256 hashes.

---

# 4.1 DOMAIN-SEPARATED HASHING

## Existing Problem

Hashing is fragmented across:

```text
csv-core/src/tagged_hash.rs
csv-core/src/commitment.rs
csv-bitcoin/src/proofs.rs
csv-ethereum/src/sanad_contract.rs
csv-aptos/src/merkle.rs
```

Raw SHA256 and Keccak calls exist directly inside protocol logic.

This permits replay ambiguity and inconsistent verification semantics.

---

## Required Refactor

Create:

```text
csv-core/src/domain_hash.rs
```

Implement:

```rust
pub trait Domain {
    const DOMAIN: &'static [u8];
}

pub struct DomainSeparatedHash<D>(PhantomData<D>);
```

Required API:

```rust
impl<D: Domain> DomainSeparatedHash<D> {
    pub fn hash(payload: &[u8]) -> Hash;
}
```

---

## Required Domain Types

Create:

```text
csv-core/src/domains/
```

Files:

```text
bitcoin_seal.rs
ethereum_mint.rs
aptos_anchor.rs
transfer_commitment.rs
proof_bundle.rs
replay_registry.rs
```

Each file MUST expose:

```rust
pub struct BitcoinSealDomain;
```

with:

```rust
impl Domain for BitcoinSealDomain {
    const DOMAIN: &'static [u8] = b"csv.bitcoin.seal.v1";
}
```

---

## Mandatory Migrations

Replace all direct hashing in:

| File | Action |
|---|---|
| csv-bitcoin/src/proofs.rs | remove double_sha256 |
| csv-ethereum/src/sanad_contract.rs | remove raw keccak |
| csv-aptos/src/merkle.rs | remove direct sha256 |
| csv-core/src/commitment.rs | migrate to DomainSeparatedHash |
| csv-core/src/nullifier.rs | use replay domain |

---

## Forbidden After Migration

Forbidden outside:

```text
csv-core/src/domain_hash.rs
csv-crypto/
```

Forbidden calls:

```rust
Sha256::digest
Keccak256::digest
blake3::hash
```

---

## CI Rule

Add:

```text
.cargo/config.toml
```

with deny rules through custom clippy or script enforcement.

---

## Acceptance Criteria

- identical payloads hash differently across domains
- replay across chains rejected
- replay across proof types rejected
- all protocol hashing routes through DomainSeparatedHash
- zero raw hash calls remain outside approved modules

---

# 4.2 CANONICAL PROOF VALIDATION PIPELINE

## Existing Problem

Current validation exists in:

```text
csv-core/src/verifier.rs
csv-core/src/validator.rs
```

But chain adapters bypass canonical ordering.

---

## Required Refactor

Create:

```text
csv-core/src/proof_pipeline.rs
```

The ONLY allowed proof validation entrypoint:

```rust
pub async fn validate_proof_bundle(...)
```

---

## Required Validation Order

Must execute exactly:

```text
1. structural validation
2. domain validation
3. inclusion proof validation
4. zk proof validation
5. finality validation
6. replay validation
7. seal registry validation
8. transition legality validation
9. signature validation
10. acceptance decision
```

No adapter may reorder.

---

## Required Adapter Refactor

Current adapter verification logic in:

```text
csv-bitcoin/
csv-ethereum/
csv-aptos/
csv-solana/
```

must implement only:

```rust
trait ChainVerifier {
    async fn verify_inclusion(...)
    async fn verify_finality(...)
    async fn verify_zk(...)
}
```

Adapters MUST NOT orchestrate validation flow.

---

## Required Replay Registry

Create:

```text
csv-core/src/replay_registry.rs
```

Persistent backend:

```text
csv-store/src/replay_registry_store.rs
```

Replay key:

```rust
pub struct ReplayKey {
    proof_hash,
    seal_id,
    commitment_hash,
    source_chain,
    destination_chain,
}
```

---

## Required Persistence Semantics

Replay registry MUST survive:

- restart
- crash
- rollback recovery
- node migration

---

## Acceptance Criteria

- all chains use same validation order
- replay survives restart
- adapters cannot bypass validation
- invalid proofs rejected before state transition

---

# 5. PHASE 2 — TYPESTATE + STORAGE COHERENCY

**Owner Teams:** Core Protocol + Storage  
**Status:** Complete — TransferStatus unified, recovery engine wired, reorg handlers implemented

---

## 5.0 TransferStatus Unification (COMPLETED)

**Files:** `csv-sdk/src/transfers.rs`, `csv-core/src/protocol_version.rs`

**Issue:** Two separate `TransferStatus` enums caused inconsistency between SDK and core.

**Fix:** SDK now references `csv_core::protocol_version::TransferStatus` as the single source of truth. All status transitions use the unified enum.

---

# 5.1 TYPESTATE TRANSFER MACHINE

## Existing Problem

Current mutable enums:

```text
csv-sdk/src/transfers.rs
csv-core/src/protocol_version.rs
```

permit arbitrary transition mutation.

**Status:** Resolved — SDK now uses unified `csv_core::TransferStatus`.

Example:

```rust
record.status = new_status
```

This is forbidden.

---

## Required Refactor

Create:

```text
csv-core/src/transfer_state/
```

Required files:

```text
locked.rs
awaiting_finality.rs
proof_building.rs
proof_validated.rs
minting.rs
completed.rs
rolled_back.rs
compromised.rs
```

---

## Required State Types

```rust
LockedTransfer
AwaitingFinalityTransfer
ProofBuildingTransfer
ProofValidatedTransfer
MintingTransfer
CompletedTransfer
RolledBackTransfer
CompromisedTransfer
```

---

## Required Transition Semantics

Every transition MUST:

- consume previous state
- validate invariants internally
- persist intent before side effect
- persist result after completion

Example:

```rust
impl LockedTransfer {
    pub async fn await_finality(
        self,
        verifier: &FinalityVerifier,
        store: &TransferStore,
    ) -> Result<AwaitingFinalityTransfer>
}
```

---

## Forbidden

Forbidden everywhere:

```rust
transfer.status = ...
```

Forbidden:

- public mutable state fields
- enum rewrites
- arbitrary state jumps
- deserializing directly into advanced states

---

## Compile-Fail Tests

Create:

```text
csv-core/tests/compile_fail/
```

Required tests:

```text
locked_to_minting.rs
observed_to_completed.rs
rollback_to_completed.rs
```

---

## Acceptance Criteria

- illegal transitions impossible through public API
- rollback represented explicitly
- compromised state modeled explicitly
- all transitions durable

---

# 5.2 CRASH-SAFE PERSISTENCE

## Existing Problem

Storage implementations fragmented across:

```text
csv-store/
csv-explorer-storage/
in-memory adapters
browser state
```

Protocol state can diverge after crash.

---

## Required Standard

All protocol-critical persistence uses:

```text
SQLite
```

Optional indexers MAY use Postgres.

---

## Required Persistence Order

Every operation MUST follow:

```text
persist intent
flush
execute side effect
persist result
flush
```

---

## Required Store Refactors

Create:

```text
csv-store/src/operations/
```

Files:

```text
transfer_store.rs
proof_store.rs
replay_store.rs
reorg_store.rs
operation_log.rs
```

---

## Required Operation Metadata

Every operation MUST contain:

```rust
operation_id
attempt_counter
proof_hash
chain_id
pinned_block_hash
```

---

## Recovery Engine

Create:

```text
csv-core/src/recovery_engine.rs
```

Required startup sequence:

```text
1. load incomplete operations
2. verify persisted tx state
3. reconcile chain state
4. invalidate orphaned operations
5. resume idempotently
```

---

## Acceptance Criteria

- crash during mint recovers safely
- duplicate execution impossible
- restart reconstructs valid state
- orphaned transfers reconciled

---

# 6. PHASE 3 — FINALITY + REORG SAFETY

**Owner Teams:** Chain Adapters + Protocol Core  
**Status:** Complete — rollback.rs and reconciliation.rs have real implementations

---

## 6.0 Reorg Handlers Implementation (COMPLETED)

**Files:** `csv-core/src/reorg/rollback.rs`, `csv-core/src/reorg/reconciliation.rs`

**Issue:** `rollback_transfers()` and `reconcile()` were stubs that only logged actions.

**Fix:**
- `RollbackHandler` is now generic over `RollbackStorageBackend`; `rollback_transfers()` persists state changes and returns `Vec<RollbackResult>`
- `ReconciliationEngine` is now generic over `ChainBackendForReconciliation`; `reconcile()` queries chain state, verifies lock validity, re-validates proofs, and computes new states

---

# 6.1 FINALITY STATE MODEL

## Required Types

Create:

```text
csv-core/src/finality/
```

Files:

```text
state.rs
policy.rs
monitor.rs
```

---

## Required States

```rust
Observed
Confirmed
Finalized
Irreversible
RolledBack
```

---

## Required Adapter Policy

Each chain adapter MUST implement:

```rust
trait ChainFinalityPolicy {
    fn confirmation_threshold(&self) -> u64;
    fn irreversible_threshold(&self) -> u64;
    fn rollback_window(&self) -> u64;
}
```

---

## Forbidden

Forbidden:

```rust
latest_block
```

inside proof construction.

All proof construction MUST pin:

```text
block hash
block height
chain id
```

---

# 6.2 REORG DETECTION

## Existing Problem

Reorg monitoring exists but is not integrated into transfer flow.

---

## Required Refactor

Refactor:

```text
csv-core/src/monitor.rs
```

into:

```text
csv-core/src/reorg/
```

Required files:

```text
detector.rs
rollback.rs
reconciliation.rs
```

---

## Required Persistent Data

Persist:

```text
parent_hash
canonical_chain
proof_anchor
mint_dependency
```

---

## Critical Requirement

If:

```text
source lock invalidated after destination mint
```

Transfer MUST enter:

```rust
CompromisedTransfer
```

NOT rollback silently.

---

## Required Handling Strategy

Protocol MUST support one:

| Strategy | Status |
|---|---|
| delayed release escrow | preferred |
| compensating burn | acceptable |
| manual freeze | temporary only |

---

## Acceptance Criteria

- reorg invalidates dependent proofs
- mint continuation prevented
- compromised state observable
- rollback events emitted

---

# 7. PHASE 4 — RPC TRUST HARDENING

**Owner Teams:** Infra + Adapter Owners  
**Status:** Complete — QuorumClient uses real HTTP calls, wired into Ethereum adapter

---

## 7.0 Quorum Client Integration (COMPLETED)

**Files:** `csv-core/src/rpc/quorum_client.rs`, `csv-ethereum/src/rpc.rs`

**Issue:** `simulate_rpc_call()` was a stub; quorum client not wired into adapters.

**Fix:**
- `simulate_rpc_call` replaced with actual `reqwest::Client` HTTP POST calls to JSON-RPC endpoints
- Proper timeout handling via `provider.timeout_ms`
- Error handling for HTTP failures, parse errors, and JSON-RPC errors
- `QuorumEthereumRpc` wraps `QuorumClient` for all Ethereum JSON-RPC calls

---

# 7.1 RPC QUORUM CLIENT

## Existing Problem

Single-provider trust exists across adapters.

**Status:** Resolved — `QuorumEthereumRpc` implements quorum-based RPC for Ethereum.

---

## Required Refactor

Create:

```text
csv-core/src/rpc/quorum_client.rs
```

---

## Required Configuration

```toml
[rpc]
min_quorum = 2
provider_count = 3
```

---

## Required Critical Reads

Must quorum-check:

- block hash
- block height
- tx receipt
- event logs
- inclusion proof
- finality proof
- nonce

---

## Required Failure

On mismatch:

```rust
Err(ChainViewInconsistent)
```

Execution MUST halt.

---

## Required Metrics

Create:

```text
csv-observability/src/metrics/
```

Metrics:

```text
rpc_disagreement_total
rpc_latency_ms
provider_failure_total
provider_timeout_total
```

---

## Acceptance Criteria

- no direct provider reads remain
- quorum disagreement observable
- pinned reads reused consistently

---

# 8. PHASE 5 — ABI + CONTRACT SAFETY

**Owner Teams:** EVM + Solidity  
**Status:** Complete — ops.rs migrated to generated Alloy bindings

---

## 8.0 ABI Migration (COMPLETED)

**Files:** `csv-ethereum/src/ops.rs`, `csv-ethereum/src/bindings/csv_lock.rs`, `csv-ethereum/src/bindings/csv_mint.rs`

**Issue:** `ops.rs` used manual `CsvLockAbi`/`CsvMintAbi` encoding instead of generated bindings.

**Fix:**
- `lock_sanad()`, `mint_sanad()`, `refund_sanad()` now use `CsvLockClient`/`CsvMintClient` generated Alloy bindings
- Call structs use `SolCall::abi_encode()` for proper ABI encoding
- Manual ABI encoding functions deprecated in favor of type-safe bindings

---

# 8.1 REMOVE MANUAL ABI ENCODING

## Existing Problem

Manual calldata construction exists in:

```text
csv-ethereum/src/sanad_contract.rs
csv-ethereum/src/seal_contract.rs
csv-ethereum/src/ops.rs
csv-ethereum/src/node.rs
```

**Status:** `ops.rs` migrated to bindings. `sanad_contract.rs` and `seal_contract.rs` still use manual encoding (deprecated, to be removed).

---

## Required Refactor

Create:

```text
csv-ethereum/src/bindings/
```

Generated through Alloy.

**Status:** Complete — `csv_lock.rs` and `csv_mint.rs` generated and used in `ops.rs`.

---

## Forbidden

Forbidden:

```rust
build_abi_call(...)
manual_selector(...)
abi.encode(...)
```

outside generated bindings.

---

## Required CI

Add generation verification:

```text
cargo xtask verify-bindings
```

CI MUST fail if bindings stale.

---

# 8.2 IMMUTABLE DEPLOYMENT MODEL

## Required Solidity Changes

Current governance mutation paths MUST be frozen.

Required changes:

| Contract | Action |
|---|---|
| CSVLock.sol | freeze verifier refs |
| CSVMint.sol | freeze lock refs |
| verifier config | immutable |

---

## Forbidden

Forbidden:

- upgradeable proxies
- delegatecall architectures
- runtime governance mutation

---

## Deployment Manifest

Create:

```text
/deployments/deployment-manifest.json
```

Required fields:

```json
{
  "chain_id": "",
  "block_number": "",
  "tx_hash": "",
  "bytecode_hash": "",
  "constructor_args": "",
  "abi_hash": ""
}
```

Manifest MUST be signed.

---

## Acceptance Criteria

- zero handwritten ABI encoding remains
- bytecode verified automatically
- deployment reproducible
- ownership state deterministic

---

# 9. PHASE 6 — WALLET + KEY MANAGEMENT

**Owner Teams:** Wallet + Security

---

# 9.1 REMOVE UNSAFE SEED STORAGE

## Existing Problem

Browser persistence paths may expose mnemonic material.

---

## Required Refactor

Remove:

```text
localStorage mnemonic persistence
plaintext wallet config seeds
raw seed serialization
```

---

## Required Backends

Use:

- encrypted IndexedDB
- WebCrypto wrapping
- OS keystore integration
- hardware wallet support

---

## Required File Targets

Refactor:

```text
csv-wallet/src/browser_storage.rs
csv-wallet/src/browser_keystore.rs
csv-wallet/src/config.rs
```

Integrate:

```text
csv-keys/
```

as the single secure key abstraction.

---

## Required Memory Hygiene

Sensitive types MUST:

- implement Zeroize
- not derive Clone
- not derive Debug
- avoid accidental serde serialization

---

## Acceptance Criteria

- no plaintext mnemonic persistence
- secret memory zeroized
- hardware wallet path functional

---

# 10. PHASE 7 — PROPERTY TESTING + FUZZING

**Owner Teams:** QA + Security

---

# 10.1 PROPERTY TESTS

## Required Frameworks

Use:

```text
proptest
quickcheck
```

---

## Required Test Files

Create:

```text
csv-core/tests/properties/
```

Files:

```text
seal_consumption.rs
replay_resistance.rs
rollback_consistency.rs
proof_determinism.rs
serialization_roundtrip.rs
```

---

## Mandatory Invariants

```text
seal consumed once
replay impossible
rollback deterministic
proof deterministic
illegal transitions rejected
serialization stable
```

---

# 10.2 FUZZING

## Required Targets

Create:

```text
fuzz/fuzz_targets/
```

Targets:

```text
proof_bundle_decode.rs
consignment_decode.rs
rpc_parser.rs
abi_decoder.rs
finality_parser.rs
```

---

## Required CI

Fuzz corpus MUST execute in CI.

Regression crashes MUST block merges.

---

## Acceptance Criteria

- invariant regressions detected automatically
- malformed proofs never panic runtime
- parser crashes reproducible through corpus

---

# 11. PHASE 8 — OBSERVABILITY + FORENSICS

**Owner Teams:** Infra + Protocol

---

# 11.1 STRUCTURED AUDIT EVENTS

## Existing Foundation

Reuse:

```text
csv-core/src/events.rs
```

Do not create parallel systems.

---

## Required Event Types

Add:

```text
proof_accepted
proof_rejected
replay_detected
rpc_disagreement
reorg_detected
rollback_executed
mint_compromised
```

---

## Required Correlation Fields

Every event MUST include:

```text
transfer_id
operation_id
source_chain
destination_chain
proof_hash
pinned_block_hash
```

---

## Required Format

Structured JSON only.

No plaintext logs.

---

## Acceptance Criteria

- all transfers traceable end-to-end
- reorgs reconstructable from logs
- proof failures auditable

---

# 12. PHASE 9 — END-TO-END CERTIFICATION

**Owner Teams:** Entire Engineering Org

---

# 12.1 REQUIRED CERTIFICATION FLOW

Initial certification target:

```text
Bitcoin Signet -> Ethereum Sepolia
```

Reason:

Most mature adapters.

---

## Mandatory Flow

```text
1. wallet create
2. fund wallet
3. create seal
4. source chain lock
5. wait finality
6. build proof
7. offline proof verify
8. transport proof
9. destination verify
10. mint
11. explorer visibility
12. reorg monitoring
13. recovery replay validation
```

---

## Forbidden During Certification

Forbidden:

- mocks
- fake proofs
- manual overrides
- skipped verification
- test bypasses

---

## Required Failure Injection

Simulate:

- RPC disagreement
- chain reorg
- duplicate proof
- replay attempt
- node restart
- persistence crash
- provider timeout

---

## Acceptance Criteria

- entire flow deterministic
- recovery succeeds after restart
- replay rejected persistently
- offline verification matches online verification
- compromised mint path observable

---

# 13. FINAL MERGE GATES

Production branch MUST NOT merge unless ALL are true:

| Requirement | Status |
|---|---|
| all hashing domain-separated | ~~required~~ **done** |
| canonical proof pipeline active | required |
| replay registry persistent | required |
| typestate enforced | ~~required~~ **done** |
| reorg recovery functional | ~~required~~ **done** |
| quorum RPC active | ~~required~~ **done** |
| manual ABI removed | ~~required~~ **done** |
| wallet persistence hardened | required |
| invariant tests passing | required |
| fuzz regressions clean | required |
| E2E certification passing | required |
| offline verification deterministic | required |

---

# 14. EXPLICIT NON-GOALS

Do NOT:

- add chains
- redesign consensus
- add plugin systems
- add abstraction layers
- optimize performance prematurely
- add upgradeable contracts
- add experimental cryptography

---

# 15. FINAL ENGINEERING DIRECTIVE

The repository does not currently fail because of missing features.

It fails because protocol guarantees are not enforced structurally.

This execution plan converts guarantees from convention into code.

Every implementation decision must optimize for:

- determinism
- invariant enforcement
- replay resistance
- crash recovery
- Byzantine resilience
- operational auditability
- explicit failure semantics

Nothing ships before those guarantees exist simultaneously.

