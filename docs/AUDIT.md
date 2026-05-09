# CSV Protocol — AI Agent Developer Masterplan
**Version**: 0.5.0-dev  
**Analyzed codebase**: v0.4.0  
**Last updated**: 2026-05-09  
**Audience**: AI coding agents acting as primary developers

---

## 1. What This Codebase Is

**CSV (Client-Side Validation) Protocol** — a multichain single-use seal platform.
A *seal* is a one-time cryptographic commitment anchor on any supported chain.
A *Sanad* is the cross-chain transferable asset whose state is validated client-side.

### Supported chains
| Chain | Contract | Status |
|---|---|---|
| Bitcoin | Tapret/OP_RETURN | ✅ Working |
| Ethereum | CSVLock.sol / CSVMint.sol | ⚠️ Deploy stub |
| Solana | Anchor PDA SanadAccount | ✅ Working |
| Aptos | Move CSVSealV2 | ✅ Working |
| Sui | Move csv_seal | ✅ Working |
| Celestia | DA blob layer | ✅ Working |

### Ultimate roadmap goals
1. **Atomic Seal Swap** — cross-chain swap without escrow
2. **ZK Seal Consumption** — Pedersen commitments + stealth addresses
3. **IoT STARK streams** — batch verify 1,000+ sensor readings via STARK

---

## 2. Codebase Architecture

```
csv-adapter/
├── csv-core/          # Protocol types, traits, DAG, validators, ZK interfaces
├── csv-bitcoin/       # Bitcoin adapter: Tapret, BIP341, SPV, MPC batch
├── csv-ethereum/      # Ethereum adapter: EVM RPC, MPT proofs, ERC contracts
├── csv-solana/        # Solana Anchor program + Rust adapter
├── csv-aptos/         # Aptos Move V2 + Rust SDK adapter
├── csv-sui/           # Sui Move + Rust SDK adapter
├── csv-celestia/      # Celestia DA layer adapter + IPFS hybrid
├── csv-p2p/           # P2P proof delivery via Nostr
├── csv-sdk/           # High-level SDK (builder pattern)
├── csv-cli/           # CLI tool
├── csv-keys/          # BIP39/BIP44 key management
├── csv-store/         # State persistence (browser + file + encrypted)
├── csv-wallet/        # Dioxus WASM wallet UI
├── csv-explorer/      # Block explorer (API + indexer + Dioxus UI)
├── csv-mcp-server/    # TypeScript MCP server for AI tool use
├── typescript-sdk/    # TS client SDK + WASM bindings (ML-DSA-65)
└── docs/              # This file + MOTIVATION.md
```

### Core trait hierarchy (csv-core)
```
ChainBackend
  ├── ChainOps (lock_seal, consume_seal, get_seal_status)
  ├── ChainDeployer (deploy_lock_contract, verify_deployment)
  ├── ChainProofProvider (build_inclusion_proof, verify_inclusion_proof, build_finality_proof)
  └── SealProtocol (create_seal, verify_seal, build_proof_bundle)
```

Every chain adapter (`csv-bitcoin`, `csv-ethereum`, etc.) implements this trait set on a `*Backend` struct.

---

## 3. Current State Analysis

### 3.1 Functionality Inventory

| Component | Status | Notes |
|---|---|---|
| Seal lifecycle (create/consume/verify) | ✅ Complete | All 5 chains |
| Aptos Move V2 contract | ✅ Complete | Cross-chain events, safe 2-phase transfer |
| Solana Anchor program | ✅ Complete | PDA-based, LockRegistry with refund timeout |
| Bitcoin Tapret encoding | ✅ Complete | BIP341, SPV proof, signet demo |
| Ethereum EVM contracts | ✅ Complete | CSVLock.sol / CSVMint.sol deployed on Sepolia |
| Sui Move contract | ✅ Complete | Object-based seals |
| Cross-chain state machine | ✅ Complete | TransferState enum, 7 states |
| Merkle proofs (all chains) | ✅ Complete | Bitcoin Merkle, Ethereum MPT, Aptos accumulator |
| ML-DSA-65 WASM bindings | ✅ Complete | pqcrypto-dilithium, keygen/sign/verify |
| BIP39/BIP44 key derivation | ✅ Complete | csv-keys |
| Explorer API (REST + GraphQL + WS) | ✅ Complete | axum-based |
| Ethereum contract **deployment** | ❌ Stub | Returns `CapabilityUnavailable` |
| Sui package deployment | ❌ Stub | `execute_with_client` errors; BCS TX builder missing |
| ZK Pedersen commitments | ❌ Not started | File exists, no implementation |
| ZK Stealth addresses | ❌ Not started | Planned in csv-core/zk_proof.rs |
| STARK IoT batch verification | ❌ Not started | No code |
| Atomic Seal Swap protocol | ❌ Partial | State machine defined, swap logic missing |
| P2P proof delivery | ❌ Skeleton | csv-p2p/nostr.rs is hollow |
| NFT page | ❌ Stub | `has_nfts = false` hardcoded |
| Desktop filesystem keystore | ❌ TODO | WASM keystore works; native is `// TODO` |
| Transaction validation (Ethereum) | ❌ Stub | `validate_transaction` returns `Ok(())` always |

---

### 3.2 Security Vulnerabilities (ordered by severity)

#### CRITICAL

**SV-01: Unconditional proof acceptance in non-RPC builds**
- **File**: `csv-ethereum/src/backend.rs` → `verify_inclusion_proof`
- **Code**: `#[cfg(not(feature = "rpc"))] { Ok(true) }` — no-feature build accepts any proof
- **Fix**: Return `Err(ChainOpError::FeatureRequired("rpc"))` instead of `Ok(true)` when proof verification infrastructure is unavailable

**SV-02: `validate_transaction` is a no-op**
- **File**: `csv-ethereum/src/backend.rs` → `validate_transaction`
- **Code**: Returns `Ok(())` after a comment listing 5 validations that are not implemented
- **Fix**: Implement full validation: RLP decode, nonce check, gas price ≥ min, sender balance ≥ gas × price + value

#### HIGH

**SV-03: `is_seal_used_by_path` is semantically wrong**
- **File**: `csv-bitcoin/src/seal.rs` → `is_seal_used_by_path`
- **Code**: Returns `any(|seal_bytes| seal_bytes.len() > 32)` — checks byte length, not path
- **Fix**: Derive the key at `path`, compute the expected txid from it, then check `used_seals.contains(txid)`

**SV-04: Hardcoded `"default"` chain_id in WASM commitment**
- **File**: `typescript-sdk/wasm/src/lib.rs` → `build_commitment`, `build_proof_bundle`
- **Code**: `let chain_id = "default";` — produces identical commitments for different chains
- **Fix**: Remove the internal default; require caller to pass chain_id; `build_proof_bundle` must accept `chain_id: &str` parameter

**SV-05: Solana LockRegistry unbounded growth + linear scan**
- **File**: `csv-solana/contracts/programs/csv-seal/src/state.rs` → `LockRegistry`
- **Code**: `Vec<LockRecord>` stored inline; cap is 1000 but the `SIZE` constant = `BASE_SIZE` (no space for actual records)
- **Fix**: Replace with a separate PDA per lock (use `sanad_id` as seed); remove `locks: Vec<LockRecord>` from the registry account

#### MEDIUM

**SV-06: RPC endpoint health check is URL-string-only**
- **File**: `csv-explorer/indexer/src/rpc_manager.rs` → `get_healthy_endpoint`
- **Code**: `if !endpoint.url.is_empty()` — no HTTP ping, no latency check
- **Fix**: Send `GET /health` or a minimal JSON-RPC call; skip endpoints that time out > 2s

**SV-07: `CommitAnchor::new_unchecked` skips size validation**
- **File**: `csv-core/src/seal.rs`
- **Fix**: Add an `unsafe` block with a debug-assert on anchor_id size, or deprecate the method; callers in `tapret_verify.rs` and `ops.rs` should use `new()` with proper error propagation

**SV-08: Merkle tree lacks domain separation**
- **File**: `csv-aptos/src/merkle.rs` → `compute_internal_hash`
- **Code**: `SHA256(left || right)` — vulnerable to second-preimage if attacker controls leaf content
- **Fix**: Prefix leaf hashes with `0x00` and internal nodes with `0x01` (RFC 6962 style)

#### LOW

**SV-09: Aptos V1 `transfer_seal` takes `address` not `signer`**
- **File**: `csv-aptos/contracts/csv_seal.move` (V1 module) → `transfer_seal`
- **Code**: `move_to(&to, seal_res)` where `to: address` — bypasses recipient consent
- **Status**: V2 has the correct 2-phase `initiate_transfer` / `accept_transfer` pattern
- **Fix**: Remove V1 `transfer_seal`; enforce V2 module usage only; add `#[deprecated]` annotation

---

### 3.3 Performance Issues

**PF-01: SealRegistry is in-memory only**  
`csv-bitcoin/src/seal.rs`: `used_seals: HashSet<Vec<u8>>` resets on restart. Any restart enables seal replay.  
**Fix**: Persist to RocksDB or SQLite; add startup hydration from chain state.

**PF-02: Solana lock lookups are O(n)**  
`LockRegistry.locks: Vec<LockRecord>` requires full scan for refund eligibility.  
**Fix**: Use per-lock PDAs (see SV-05).

**PF-03: Ethereum finality polling uses blocking sleep**  
`csv-ethereum/src/finality.rs`: `std::thread::sleep(poll_interval)` inside `#[cfg(not(feature="rpc"))]`.  
**Fix**: Gate the entire polling function behind `feature = "rpc"`; non-RPC builds should not wait.

**PF-04: Merkle tree clones entire node tree per level**  
`csv-aptos/src/merkle.rs` → `build_tree`: each level does `.clone()` on all nodes.  
**Fix**: Build iteratively with index-based computation on a `Vec<[u8;32]>`; no boxing needed.

**PF-05: Explorer indexer state is in-memory HashMap**  
Each indexer (`bitcoin.rs`, `ethereum.rs`, etc.) tracks chain tip in a local field.  
**Fix**: Persist tips to the SQLite `sync` table; read on startup.

---

### 3.4 Scalability Issues

**SC-01: Solana LockRegistry max 1000 locks per registry**  
Hard cap prevents high-throughput usage.  
**Fix**: Per-lock PDA (SV-05 fix eliminates this cap entirely).

**SC-02: Cross-chain registry is in-memory**  
`csv-core/src/cross_chain.rs`: `CrossChainRegistryEntry` has no persistence layer.  
**Fix**: Wire to `csv-explorer/storage` SQLite via the `transfers` repository.

**SC-03: Explorer SQLite has no sharding plan**  
`csv-explorer/storage/src/schema.sql`: Single SQLite file for all chains.  
**Fix**: Partition by chain_id; use WAL mode (`PRAGMA journal_mode=WAL`); add read replicas via SQLite backup API for explorer.

**SC-04: No proof batching**  
Each seal proof is built and transmitted individually.  
**Fix prerequisite for IoT STARK**: Add `batch_build_proofs(seals: &[SealPoint]) -> Vec<ProofBundle>` to `ChainProofProvider`; implement STARK batch verifier circuit.

---

## 4. Development Roadmap for AI Agents

Tasks are ordered: fix critical bugs first, then implement missing protocol features, then scale.

---

### Phase 1 — Critical Bug Fixes (do first, in this order)

#### Task 1.1 — Fix unconditional proof acceptance (SV-01)
**Crate**: `csv-ethereum`  
**File**: `src/backend.rs`  
**Action**:
```rust
// Replace:
#[cfg(not(feature = "rpc"))]
{ let _ = (proof, commitment); Ok(true) }
// With:
#[cfg(not(feature = "rpc"))]
{ Err(ChainOpError::FeatureRequired("rpc feature required for proof verification".to_string())) }
```
Apply same pattern to all other chain backends containing `Ok(true)` stubs in `verify_inclusion_proof`.

#### Task 1.2 — Implement Ethereum transaction validation (SV-02)
**Crate**: `csv-ethereum`  
**File**: `src/backend.rs` → `validate_transaction`  
**Action**: Add dependency `rlp = "0.5"`. RLP-decode the tx bytes. Validate:
1. `nonce >= sender_nonce` (fetch from RPC)
2. `gas_price >= min_gas_price` (fetch from RPC `eth_gasPrice`)
3. `gas_limit <= block_gas_limit`
4. `sender_balance >= gas_limit * gas_price + value`

Return `Err(ChainOpError::InvalidInput(...))` on any failure.

#### Task 1.3 — Fix Bitcoin path-based seal tracking (SV-03)
**Crate**: `csv-bitcoin`  
**File**: `src/seal.rs` → `is_seal_used_by_path`  
**Action**: Derive the BIP86 key at `path`, construct the P2TR address, look up whether any used seal in `used_seals` was funded by that address. Remove the `len() > 32` heuristic entirely.

#### Task 1.4 — Fix WASM commitment chain_id (SV-04)
**Crate**: `typescript-sdk/wasm`  
**File**: `src/lib.rs`  
**Action**: Change `build_proof_bundle(seal_id, block_height, commitment)` → `build_proof_bundle(seal_id, block_height, commitment, chain_id)`. Pass `chain_id` to `build_commitment`. Update TypeScript bindings in `typescript-sdk/src/proof.ts`.

#### Task 1.5 — Fix Merkle domain separation (SV-08)
**Crate**: `csv-aptos`  
**File**: `src/merkle.rs`  
**Action**:
```rust
// Leaf hash: SHA256(0x00 || data)
// Internal hash: SHA256(0x01 || left || right)
```
Update `Leaf::hash` computation and `MerkleNode::compute_internal_hash`. Update all test vectors.

---

### Phase 2 — Structural Fixes

#### Task 2.1 — Persist SealRegistry to storage
**Crate**: `csv-bitcoin`  
**Files**: `src/seal.rs`, `src/backend.rs`  
**Action**:
1. Add `sled` or `rusqlite` as optional dependency under `feature = "persist"`
2. On `mark_seal_used`: write `(seal_vec, timestamp)` to the DB
3. On `SealRegistry::new`: load existing entries from DB into `used_seals`
4. Test: seal survives process restart

#### Task 2.2 — Replace Solana LockRegistry Vec with per-lock PDAs
**Crate**: `csv-solana`  
**Files**: `contracts/programs/csv-seal/src/state.rs`, `instructions.rs`  
**Action**:
1. Define `#[account] struct LockAccount { ... }` with seeds `[b"lock", sanad_id]`
2. Remove `locks: Vec<LockRecord>` from `LockRegistry`; keep only `authority`, `lock_count`, `refund_timeout`, `bump`
3. Update `lock_sanad` instruction: init a `LockAccount` PDA
4. Update `refund_sanad` instruction: close the `LockAccount` PDA
5. Remove `MAX_LOCKS = 1000` constant

#### Task 2.3 — Implement Ethereum contract deployment
**Crate**: `csv-ethereum`  
**File**: `src/backend.rs` → `deploy_lock_contract`, `deploy_mint_contract`  
**Action**:
1. Include compiled bytecode via `include_bytes!("../contracts/out/CSVLock.sol/CSVLock.json")`
2. Extract `bytecode.object` field
3. Build deployment calldata: `bytecode || abi.encode(constructor_args)`
4. Sign and send via `eth_sendRawTransaction`
5. Poll for receipt; return deployed address

#### Task 2.4 — Implement Sui package deployment
**Crate**: `csv-sui`  
**File**: `src/deploy.rs`  
**Action**:
1. Use `sui_sdk::TransactionBuilder` (feature-gate with `sui-sdk-deploy`)
2. Implement `build_publish_transaction_data` using proper BCS encoding via `bcs::to_bytes`
3. Implement `execute_with_client` using `sui_sdk::SuiClient::quorum_driver_api().execute_transaction_block`
4. Test against Sui devnet

#### Task 2.5 — Implement desktop filesystem keystore
**Crate**: `csv-wallet`  
**File**: `src/core/key_manager.rs` → `#[cfg(not(target_arch = "wasm32"))]` block  
**Action**:
1. Use `csv-keys/src/file_keystore.rs`
2. Store to `~/.csv/keys/{chain}/{keystore_id}.enc` using AES-256-GCM
3. Derive encryption key from passphrase via Argon2id (already a dependency)

#### Task 2.6 — Fix RPC health check
**Crate**: `csv-explorer`  
**File**: `indexer/src/rpc_manager.rs` → `get_healthy_endpoint`  
**Action**: Send a minimal JSON-RPC call (e.g. `eth_blockNumber`, `getSlot`) with `timeout(2s)`. Skip and try next endpoint on failure. Cache healthy endpoint for 30 seconds.

---

### Phase 3 — Missing Protocol Features

#### Task 3.1 — Implement Atomic Seal Swap
**Crate**: `csv-core`  
**File**: `src/cross_chain.rs`  
**Design**: Hash Time Locked Seal Exchange (HTLSE) — escrow-free variant:
```
Alice (Chain A) locks Seal_A with H(secret)
Bob (Chain B) locks Seal_B with H(secret)
Alice reveals secret → consumes Seal_B on Chain B
Bob reads secret from on-chain event → consumes Seal_A on Chain A
```
**Action**:
1. Add `AtomicSwapOffer { seal_a, seal_b, hash_lock: [u8;32], timeout_blocks: u64 }` to `cross_chain.rs`
2. Add `initiate_swap` / `complete_swap` / `refund_swap` to each chain contract
3. Add swap coordination logic to `csv-sdk/src/cross_chain.rs`
4. Bitcoin: encode hash-lock in Tapscript leaf; Ethereum: add to CSVLock.sol; Solana: new instruction; Aptos: new entry fun; Sui: new Move function

#### Task 3.2 — ZK Seal Consumption: Pedersen Commitments
**Crate**: `csv-core`  
**File**: `src/zk_proof.rs` (currently a skeleton)  
**Dependencies**: Add `bulletproofs = "4"` and `curve25519-dalek = "4"` to `csv-core/Cargo.toml`  
**Action**:
1. Implement `PedersenCommitment::commit(value: u64, blinding: Scalar) -> CompressedRistretto`
2. Implement `PedersenCommitment::verify(commitment, value, blinding) -> bool`
3. Wrap into `ZkSealProof::Pedersen { commitment, range_proof: RangeProof }`
4. Add `prove_seal_value` and `verify_seal_value` to `ChainProofProvider` trait
5. Wire to wallet ZK proof pages (`csv-wallet/src/pages/zk_proofs/`)

#### Task 3.3 — ZK Seal Consumption: Stealth Addresses
**Crate**: `csv-core`  
**File**: `src/zk_proof.rs`  
**Action**:
1. Implement dual-key stealth address scheme: `(scan_key, spend_key)` → ephemeral address
2. `StealthAddress::generate(recipient_scan_pk, recipient_spend_pk) -> (ephemeral_pk, stealth_addr)`
3. `StealthAddress::scan(scan_sk, ephemeral_pk) -> Option<stealth_addr>`
4. Integrate into seal creation: seal owner address = stealth_addr; only recipient with scan_sk can detect it
5. Add stealth scanning loop to `csv-wallet/src/seals/monitor.rs`

#### Task 3.4 — STARK Batch Verification for IoT Streams
**Crate**: Create new `csv-stark` crate  
**Dependencies**: `winterfell = "0.9"` (STARK prover) or `stone-prover` FFI bindings  
**Action**:
1. Define `IoTReading { device_id: [u8;32], value: u64, timestamp: u64, signature: [u8;64] }`
2. Implement `IoTBatchProver::prove(readings: &[IoTReading]) -> StarkProof`
   - AIR (Algebraic Intermediate Representation): enforce `sig_valid(device_id, value || timestamp)`
   - Batch size target: 1024 readings per proof
3. Implement `IoTBatchVerifier::verify(proof: &StarkProof, batch_commitment: [u8;32]) -> bool`
4. Add `batch_verify_iot` to `ChainProofProvider` trait
5. Wire to Celestia DA layer (`csv-celestia`) for proof posting

#### Task 3.5 — P2P Proof Delivery via Nostr
**Crate**: `csv-p2p`  
**File**: `src/nostr.rs`, `src/proof_delivery.rs`  
**Dependencies**: `nostr-sdk = "0.29"`  
**Action**:
1. Implement `NostrProofRelayer::publish(proof: &ProofBundle, recipient_pubkey: XOnlyPublicKey)`
   - Encrypt proof bytes with NIP-04 or NIP-44
   - Post as Nostr event kind 4 (DM) or custom kind 30078
2. Implement `NostrProofRelayer::subscribe(my_sk: SecretKey, handler: impl Fn(ProofBundle))`
3. Add relay selection: default to `wss://relay.damus.io`, `wss://nos.lol`
4. Wire to `csv-sdk/src/proofs.rs` as the delivery transport for cross-chain mints

---

### Phase 4 — Scalability & Performance

#### Task 4.1 — Proof batching API
**Crate**: `csv-core`  
**File**: `src/backend.rs`  
**Action**: Add to `ChainProofProvider`:
```rust
async fn build_batch_proofs(
    &self,
    commitments: &[Hash],
    block_height: u64,
) -> ChainOpResult<BatchProofBundle>;

fn verify_batch_proofs(
    &self,
    bundle: &BatchProofBundle,
    commitments: &[Hash],
) -> ChainOpResult<bool>;
```
Implement using Merkle aggregation: build a Merkle tree over the commitments, post the root on-chain, include per-leaf paths in `BatchProofBundle`.

#### Task 4.2 — Explorer SQLite optimizations
**Crate**: `csv-explorer/storage`  
**File**: `src/schema.sql`  
**Action**:
1. Add `PRAGMA journal_mode=WAL;` to `db.rs` on connection open
2. Add composite indexes: `CREATE INDEX idx_seals_chain_status ON seals(chain, status);`
3. Add pagination cursor (keyset-based, not offset-based): replace `OFFSET ?` with `WHERE id > ?`
4. Add `chain_id` column to all tables; add filtered queries per chain

#### Task 4.3 — Cross-chain registry persistence
**Crate**: `csv-sdk`  
**File**: `src/cross_chain.rs`  
**Action**:
1. Add `CrossChainRegistry` struct backed by SQLite (reuse `csv-explorer/storage`)
2. On `complete_transfer`: write `CrossChainRegistryEntry` to DB
3. On startup: load entries from DB
4. Expose `query_transfer_by_sanad_id` and `query_transfers_by_chain` methods

#### Task 4.4 — Rate limiting and anti-DoS
**Crate**: `csv-core`  
**File**: `src/hardening.rs`  
**Action**:
1. Add `SealRateLimiter { per_address: HashMap<Vec<u8>, TokenBucket> }`
2. Enforce max 10 seal creations per address per minute using token bucket algorithm
3. Wire to all chain backends' `create_seal` implementations
4. Add `BoundedCache<K, V>` with LRU eviction for proof verification results (avoid redundant RPC calls)

---

## 5. Key Invariants (Never Break)

These invariants are documented in `csv-core/src/PROTOCOL_INVARIANTS.md`. Agents must preserve them:

1. **Single-use**: A seal consumed on chain A must never be re-consumable on chain A. `nullifier` must be stored permanently.
2. **Commitment binding**: `seal_id` uniquely determines the commitment; two seals with identical IDs on different chains must produce identical commitments.
3. **Sanad ID preservation**: `sanad_id` is preserved across chains. A cross-chain transfer must not change it.
4. **Finality before mint**: No destination chain mint may occur until source chain finality is confirmed (`is_finalized == true` in `CrossChainFinalityProof`).
5. **No escrow in swaps**: Atomic swaps must not hold funds in a third-party contract. Hash-lock pattern only.

---

## 6. Testing Requirements

Each task above must include tests. Required test coverage per task type:

| Task type | Required tests |
|---|---|
| Security fix | Regression test proving the vulnerability no longer exists |
| New protocol feature | Unit + integration test against chain devnet/testnet |
| Smart contract change | Anchor/Move unit tests + cross-chain simulation test |
| Performance fix | Benchmark showing improvement (use `criterion`) |
| Cryptographic primitive | Known-answer test (KAT) vectors + property-based test |

### How to run tests
```bash
# All workspace tests (excluding integration)
cargo test --workspace

# Single crate
cargo test -p csv-bitcoin

# Integration tests (requires chain nodes)
cargo test -p csv-bitcoin --features integration

# Benchmarks
cargo bench -p csv-core
```

---

## 7. Dependency Notes

- `aws-lc-sys = "0.39"` is pinned workspace-wide to fix RUSTSEC-2026-0044/0048. Do not downgrade.
- `pqcrypto-dilithium = "0.5"` is feature-gated (`pq`). WASM build requires it; CLI/server builds are optional.
- Solana Anchor version must stay at `0.29.x`; do not upgrade without checking IDL compatibility.
- Sui SDK: `sui-rpc` is used (not the monorepo SDK) due to compile-time constraints. `sui-sdk-deploy` feature unlocks full deployment.
- Bitcoin: `bitcoin = "0.32"` and `rust-bitcoin = "0.32"` must match. Do not mix versions.
- Dioxus (wallet + explorer UI): `0.5.x`. Do not upgrade to `0.6.x` until router API stabilizes.

---

## 8. File Creation Checklist for New Chain Adapter

When adding a new chain (e.g., TON, StarkNet), create these files:
```
csv-{chain}/
├── Cargo.toml          # dependencies, features: ["rpc", "persist"]
├── build.rs            # optional codegen
├── contracts/          # on-chain program/contract code
│   └── ...
└── src/
    ├── lib.rs          # pub use all
    ├── backend.rs      # impl ChainBackend, ChainOps, ChainDeployer, ChainProofProvider
    ├── config.rs       # ChainConfig, NetworkConfig
    ├── node.rs         # ChainNode wrapper
    ├── rpc.rs          # trait ChainRpc + mock for tests
    ├── seal.rs         # SealRegistry, SealRecord, SealStore
    ├── seal_protocol.rs# impl SealProtocol
    ├── proofs.rs       # InclusionProof, FinalityProof builders
    ├── ops.rs          # high-level operations
    ├── types.rs        # chain-specific CommitAnchor, SealPoint newtype
    ├── error.rs        # ChainError enum, ChainResult<T>
    └── deploy.rs       # contract deployment logic
```
Register the adapter in `csv-core/src/driver_registry.rs` and `csv-sdk/src/config.rs`.

---

## 9. Open Questions (Unresolved Design Decisions)

These require human input before agents should implement:

1. **ZK proof system choice for IoT STARK**: Winterfell (Rust-native) vs. Stone Prover (Cairo-compatible) vs. SP1 (already partially integrated in `csv-bitcoin/src/sp1_guest`). SP1 seems preferred given existing code.
2. **Stealth address scanning loop performance**: Full chain scan is O(n blocks). Should scanning use a Bloom filter per block or a dedicated scanning server?
3. **Explorer sharding strategy**: Separate SQLite per chain or partitioned tables in one DB? Affects `csv-explorer/storage/src/db.rs` significantly.
4. **Cross-chain fee model**: Who pays for the destination mint transaction? Protocol-level fee escrow or out-of-band agreement?
5. **Sanad versioning**: Protocol version is tracked in `csv-core/src/protocol_version.rs`. Is there a migration path for V1 seals to V2 Sanad format?

---

## 10. Quick Reference: Critical File Locations

| What | Where |
|---|---|
| Core seal type | `csv-core/src/seal.rs` |
| Cross-chain state machine | `csv-core/src/cross_chain.rs` |
| ZK proof stubs | `csv-core/src/zk_proof.rs` |
| Protocol invariants doc | `csv-core/src/PROTOCOL_INVARIANTS.md` |
| Ethereum proof verification bug | `csv-ethereum/src/backend.rs:verify_inclusion_proof` |
| Bitcoin path tracking bug | `csv-bitcoin/src/seal.rs:is_seal_used_by_path` |
| WASM chain_id bug | `typescript-sdk/wasm/src/lib.rs:build_proof_bundle` |
| Solana LockRegistry bloat | `csv-solana/contracts/programs/csv-seal/src/state.rs` |
| Aptos V2 contract (reference) | `csv-aptos/contracts/sources/csv_seal.move` |
| Explorer DB schema | `csv-explorer/storage/src/schema.sql` |
| Chain adapter trait | `csv-core/src/backend.rs` |
| Driver registry | `csv-core/src/driver_registry.rs` |
