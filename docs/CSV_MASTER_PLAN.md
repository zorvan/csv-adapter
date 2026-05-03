# CSV Protocol — Master Engineering Plan

> **Vision**: Trustless cross-chain rights transfer via single-use seals and client-side validation.
> **North Star**: No bridge. No validator. Just cryptographic proof, chain-native guarantees, and client-verified state.

---

## WHERE YOU ARE RIGHT NOW

### What is working
- Core protocol types fully defined in `csv-adapter-core/src/` — `Right`, `Seal`, `Commitment`, `ProofBundle`, `AnchorLayer` trait
- Bitcoin, Sui, Aptos adapters are the most complete; each has SPV/checkpoint/ledger proof logic
- Ethereum adapter has nullifier contract (`CSVLock.sol`, `CSVMint.sol`), MPT proofs, finality logic
- Wallet (`csv-wallet/`) has HD key derivation, per-chain addresses, AES-256-GCM encryption, design token system
- CLI (`csv-cli/`) supports full lifecycle: generate, import, fund, transfer, validate
- Explorer (`csv-explorer/`) has Docker setup, WebSocket API, Dioxus UI, storage schema
- CI running in `.github/workflows/`
- Fuzz targets in `csv-adapter-core/fuzz/`

### What is broken / incomplete
| Problem | Location | Severity |
|---|---|---|
| TWO builder versions coexist | `csv-adapter/src/scalable_builder.rs` + `scalable_builder_v2.rs` | HIGH — confuses ownership |
| TWO Bitcoin proof modules | `csv-adapter-bitcoin/src/proofs.rs` + `proofs_new.rs` | HIGH — dead code risk |
| VM is a stub | `csv-adapter-core/src/vm.rs` — only `PassthroughVM` | HIGH — no real contract execution |
| THREE registry types overlap | `ChainRegistry`, `SimpleChainRegistry`, `ChainDiscovery` | MEDIUM — chain plug-and-play broken |
| Wallet pages ignore design tokens | `csv-wallet/src/pages/*/` | MEDIUM — inconsistent UI |
| Solana adapter skeleton only | `csv-adapter-solana/src/` — no `proofs.rs`, no SPV | MEDIUM — chain incomplete |
| Store not wired into wallet | `csv-adapter-store/` state never consumed by `csv-wallet/` | MEDIUM — persistence gap |
| MPC is untested theory | `csv-adapter-core/src/mpc.rs` | LOW — feature in isolation |
| ZK not started | — | LOW — but perfectly aligned with vision |

---

## THE PLAN — 6 PHASES

### PHASE 0 — DEAD CODE BURIAL (1 week)
*Make the codebase tell the truth. No navigation without a map.*

**Step 0.1 — Delete the old builder**
- DELETE `csv-adapter/src/scalable_builder.rs`
- RENAME `csv-adapter/src/scalable_builder_v2.rs` → `csv-adapter/src/scalable_builder.rs`
- Update `csv-adapter/src/lib.rs` exports accordingly
- Update `csv-adapter/src/prelude.rs` re-exports

**Step 0.2 — Delete the old Bitcoin proofs**
- DELETE `csv-adapter-bitcoin/src/proofs.rs`
- RENAME `csv-adapter-bitcoin/src/proofs_new.rs` → `csv-adapter-bitcoin/src/proofs.rs`
- Update `csv-adapter-bitcoin/src/lib.rs` to point to renamed module

**Step 0.3 — Collapse the three chain registries into one**

The current situation:
```
csv-adapter-core/src/chain_registry.rs     ← HashMap<String, Box<dyn ChainAdapter>>
csv-adapter-core/src/chain_system.rs       ← SimpleChainRegistry (different interface)
csv-adapter-core/src/chain_discovery.rs    ← ChainDiscovery (wraps both + plugin system)
```

The correct model: ONE public API, `ChainDiscovery` is the only entry point. Do this:
- KEEP `chain_discovery.rs` as the canonical public type
- DELETE `chain_registry.rs` (its role is absorbed by discovery)
- DELETE `chain_system.rs` (its `SimpleChainRegistry` is internal to discovery only)
- Update all `use` sites in `csv-cli/src/chain_management.rs` and `csv-adapter/src/lib.rs`

**Step 0.4 — Write `CODEBASE_MAP.md`**

Create `docs/CODEBASE_MAP.md` with one-line purpose for every `src/` file across all crates. This file is the orientation document for new contributors and future AI agents. Format:

```
csv-adapter-core/src/vm.rs         — DeterministicVM trait + PassthroughVM stub
csv-adapter-core/src/mpc.rs        — Multi-party computation threshold sigs (experimental)
csv-adapter-core/src/rgb_compat.rs — RGB protocol compatibility bridge (experimental)
...
```

Maintain this file on every PR as a mandatory checklist item in `.github/workflows/ci.yml`.

---

### PHASE 1 — PROTOCOL CORE HARDENING (3 weeks)
*The engine has to be real before the car matters.*

**Step 1.1 — Real VM in `csv-adapter-core/src/vm.rs`**

The `PassthroughVM` is blocking real contract transitions. Replace it with an AluVM integration:

```toml
# csv-adapter-core/Cargo.toml — add:
aluvm = { version = "0.11", default-features = false }
```

Create `csv-adapter-core/src/vm/` as a module directory:
```
csv-adapter-core/src/vm/
├── mod.rs          ← public trait re-export (DeterministicVM, VMInputs, VMOutputs, VMError)
├── aluvm.rs        ← AluVmAdapter implements DeterministicVM via aluvm crate
├── passthrough.rs  ← move PassthroughVM here (testing only, #[cfg(test)] or feature-gated)
└── metered.rs      ← MeteredVMAdapter wraps any DeterministicVM, tracks step counts for gas
```

The `AluVmAdapter` in `aluvm.rs`:
```rust
pub struct AluVmAdapter {
    max_cycles: u64,
}
impl DeterministicVM for AluVmAdapter {
    fn execute(&self, bytecode: &[u8], inputs: VMInputs, sigs: &[Vec<u8>]) -> Result<VMOutputs, VMError> {
        // load bytecode into aluvm::Program
        // bind VMInputs as registers
        // execute with cycle limit self.max_cycles
        // extract VMOutputs from registers
    }
}
```

This is the single highest-leverage engineering item in the entire codebase. Until the VM is real, CSV is a proof-of-concept.

**Step 1.2 — Harden `csv-adapter-core/src/validator.rs`**

Current validator likely has placeholder paths. Enforce:
- Every `ProofBundle` verification MUST run through `csv-adapter-core/src/proof_verify.rs` without shortcuts
- Add property test (proptest) in `csv-adapter-core/src/validator.rs` covering:
  - Tampered commitment hash → rejected
  - Replayed seal → rejected
  - Wrong chain ID → rejected
  - Truncated inclusion proof → rejected

Add tests to CI in `.github/workflows/production-guarantee.yml`.

**Step 1.3 — Tapret commitment enforcement**

`csv-adapter-core/src/tapret_verify.rs` exists. Audit it against BIP-341 script path spend semantics. Align with `csv-adapter-bitcoin/src/bip341.rs`. The two must agree on:
- Tagged hash domain separation (see `csv-adapter-core/src/tagged_hash.rs`)
- Internal key commitment structure
- Merkle branch format

Add a round-trip test: construct tapret commitment → verify via tapret_verify → assert passes.

**Step 1.4 — Commitment chain integrity**

`csv-adapter-core/src/commitment_chain.rs` and `csv-adapter-core/src/dag.rs` implement a DAG of commitment chains. Wire this into:
- `csv-adapter/src/transfers.rs` — every transfer must produce a new commitment chain node
- `csv-adapter-core/src/consignment.rs` — consignments must carry the full DAG from genesis to tip
- `csv-cli/src/commands/validate.rs` — validate command must walk the full DAG

---

### PHASE 2 — CHAIN ADAPTER COMPLETION (2 weeks)
*Plug-and-play must mean something.*

**Step 2.1 — Complete the Solana adapter**

`csv-adapter-solana/src/` is missing what every other adapter has:
- `proofs.rs` — implement SPV equivalent via Solana `SlotProof` + account proof
- `chain_operations.rs` exists — ensure it implements `AnchorLayer` fully (currently skeleton)
- `seal.rs` exists but compare with `csv-adapter-bitcoin/src/seal.rs` for completeness
- Contract `csv-adapter-solana/contracts/programs/csv-seal/src/lib.rs` has only `initialize` instruction — add `consume_seal`, `verify_proof`, `emit_seal_event`

Reference: `csv-adapter-aptos/src/chain_adapter_impl.rs` is the most complete. Mirror its pattern.

**Step 2.2 — Wire `chains/*.toml` to runtime adapter creation**

Currently `chains/bitcoin.toml`, `chains/ethereum.toml`, etc. exist but the `ChainDiscovery` plugin mechanism is not wired to adapter construction. The path to plug-and-play:

1. Each chain crate implements `ChainPlugin` trait from `csv-adapter-core/src/chain_plugin.rs`
2. `csv-adapter/src/lib.rs` registers all known plugins at startup
3. `ChainDiscovery::discover_chains("./chains/")` reads TOML files
4. `ChainDiscovery::create_adapter("bitcoin")` instantiates the right adapter from the plugin

The TOML files for a new chain become the ONLY thing needed to add chain support. This is the plug-and-play vision.

In `csv-adapter/src/lib.rs`, add:
```rust
pub fn build_discovery() -> ChainDiscovery {
    let mut d = ChainDiscovery::new();
    d.register_plugin(Arc::new(BitcoinPlugin::new()));
    d.register_plugin(Arc::new(EthereumPlugin::new()));
    d.register_plugin(Arc::new(SuiPlugin::new()));
    d.register_plugin(Arc::new(AptosPlugin::new()));
    d.register_plugin(Arc::new(SolanaPlugin::new()));
    d.load_default_chains().expect("chains/ directory required");
    d
}
```

**Step 2.3 — Harden the Ethereum adapter**

`csv-adapter-ethereum/src/mpt.rs` (Merkle Patricia Trie proofs) is the weakest link in the Ethereum proof pipeline. Audit against EIP-1186 (`eth_getProof`):
- Receipt trie must use log bloom + status
- `csv-adapter-ethereum/src/finality.rs` must correctly handle PoS checkpoint finality (not just confirmation count)
- `csv-adapter-ethereum/src/seal_contract.rs` must enforce nullifier uniqueness at contract level AND in `proof_verify.rs`

---

### PHASE 3 — STORE AND PERSISTENCE (1 week)
*Client-side validation means the CLIENT stores the proofs.*

The `csv-adapter-store/` crate has the right domain model (`state/core.rs`, `state/domain.rs`, `state/wallet.rs`) but it is not consumed by `csv-wallet/`. The wallet has its own ad-hoc storage in `csv-wallet/src/core/storage.rs`.

**Step 3.1 — Merge wallet storage into `csv-adapter-store`**

- `csv-wallet/src/core/storage.rs` → becomes a thin wrapper over `csv-adapter-store`
- `csv-adapter-store/src/state/backend.rs` already defines `StorageBackend` trait
- Add `WasmBackend` implementing `StorageBackend` using browser `IndexedDB` (via `rexie` crate) for WASM target
- Add `NativeBackend` using `sled` or `rocksdb` for CLI and desktop targets

Feature-gate by target:
```toml
[features]
wasm = ["rexie"]
native = ["sled"]
```

**Step 3.2 — Proof archive discipline**

Every `ProofBundle` produced must be written to the store before the chain transaction is broadcast. The proof IS the receipt. If you broadcast without archiving, you lose the proof.

Enforce this contract in `csv-adapter/src/transfers.rs`:
```rust
// REQUIRED ORDER — never invert this
store.archive_proof(&bundle)?;   // 1. Store proof locally
chain.broadcast(tx)?;            // 2. Only then broadcast
```

Add a test that verifies the store write fails before broadcast in `csv-adapter/src/transfers.rs`.

---

### PHASE 4 — WALLET DESIGN SYSTEM (2 weeks)
*You have a design token file. Use it everywhere.*

The design token file `csv-wallet/src/components/design_tokens.rs` is excellent — complete color palette, seal state colors, spacing, typography, radii, shadows. The problem: pages don't use it.

**Step 4.1 — Audit every page for raw CSS**

Pages that need the design token pass:
- `csv-wallet/src/pages/transactions/card.rs` — uses raw hex colors in inline styles
- `csv-wallet/src/pages/cross_chain/detail.rs` — ad-hoc layout
- `csv-wallet/src/pages/rights/show.rs` — no visual hierarchy
- `csv-wallet/src/pages/seals/mod.rs` — seal status not using `SealState` enum from design tokens
- `csv-wallet/src/layout.rs` — sidebar and header do not reference token variables

Replace every `color: "#3b82f6"` with `color: var(--color-primary-500)`.
Replace every `border-radius: 8px` with `border-radius: var(--radius-lg)`.
Replace every `font-family: monospace` with `font-family: var(--font-mono)`.

**Step 4.2 — Visual identity direction**

The wallet is a cryptographic security tool, not a bank app. Recommended aesthetic:

- **Dark theme by default** — seals are cryptographic vaults, not bank accounts
- **Monochrome base** with two accent colors: Bitcoin orange (`#f7931a`) for active seals, electric blue (`#0ea5e9`) for cross-chain operations
- **Monospace typography for all addresses and hashes** — already exists as `--font-mono`, enforce it
- **Seal status as first-class visual** — the `SealState` enum in design tokens already defines 5 states; each page showing seals must use `seal_state_class()` from that module
- **Zero decorative imagery** — no stock photos, no gradients for decoration; visual weight comes from data density and typography

**Step 4.3 — Component audit**

Build or fix these components in `csv-wallet/src/components/`:
- `seal_status.rs` — exists, audit against design token seal state colors
- `card.rs` — exists, ensure it uses `--shadow-md`, `--radius-xl`, `--color-gray-800`
- ADD `hash_display.rs` — truncated hash with copy button, hover to expand, uses `--hash-font`, `--hash-color`
- ADD `proof_badge.rs` — visual indicator of proof type (SPV, checkpoint, ledger), with chain logo
- ADD `chain_selector.rs` — unified chain/network selector for all pages that need it (currently duplicated per page)

**Step 4.4 — Page UX priorities**

In order of user-facing importance:
1. **Seal creation flow** (`csv-wallet/src/pages/seals/mod.rs`) — clearest single action in the protocol
2. **Proof verification** (`csv-wallet/src/pages/validate/`) — must show pass/fail clearly, with proof DAG visualization
3. **Transfer flow** (`csv-wallet/src/pages/rights/consume.rs`) — step-by-step with progress indicator
4. **Cross-chain status** (`csv-wallet/src/pages/cross_chain/status.rs`) — timeline visualization of lock → prove → verify
5. **Dashboard** — portfolio of active seals with chain badges

---

### PHASE 5 — ZK PROOF INTEGRATION (4 weeks)
*This is not premature optimization. ZK proofs are the endgame of client-side validation.*

ZK fits the CSV vision perfectly because:
- Current `InclusionProof` requires the verifier to trust the chain's RPC response
- A ZK proof of seal consumption lets the verifier check the proof WITHOUT trusting ANY RPC

**Architecture for ZK in CSV:**

```
Prover (sender):
  Bitcoin UTXO spend data
  + Merkle branch
  → SP1/Risc0 proof that: "UTXO X was spent in block Y"
  → ZkSealProof { proof_bytes, public_inputs: { seal_ref, block_hash } }

Verifier (receiver):
  ZkSealProof
  → verify proof_bytes against known verifier key
  → extract seal_ref and block_hash as trusted outputs
  → no RPC call required
```

**Step 5.1 — Add ZK proof module**

Create `csv-adapter-core/src/zk_proof.rs`:
```rust
pub struct ZkSealProof {
    pub proof_bytes: Vec<u8>,
    pub verifier_key: VerifierKey,
    pub public_inputs: ZkPublicInputs,
}

pub struct ZkPublicInputs {
    pub seal_ref: SealRef,
    pub block_hash: Hash,
    pub commitment: Commitment,
}

pub trait ZkProver {
    fn prove_seal_consumption(
        &self,
        seal: &SealRef,
        witness: &ChainWitness,
    ) -> Result<ZkSealProof, ZkError>;
}

pub trait ZkVerifier {
    fn verify(&self, proof: &ZkSealProof) -> Result<ZkPublicInputs, ZkError>;
}
```

**Step 5.2 — SP1 prover for Bitcoin**

In `csv-adapter-bitcoin/src/` add `zk_prover.rs`:
- Use `sp1-sdk` (Succinct Labs SP1) — best Rust support, fastest WASM-compatible proving
- Guest program: given UTXO spend transaction + Merkle branch, verify SPV inclusion
- Output: `ZkSealProof` wrapping the SP1 proof

Why SP1 over Risc0:
- SP1 supports `no_std` guest programs, compatible with WASM wallet
- Risc0 requires more ceremony for custom guest programs

```toml
# csv-adapter-bitcoin/Cargo.toml
[features]
zk = ["sp1-sdk"]

[dependencies]
sp1-sdk = { version = "2.0", optional = true }
```

**Step 5.3 — Add ZK variant to `ProofBundle`**

Extend `csv-adapter-core/src/proof.rs`:
```rust
pub enum InclusionProof {
    SpvBranch(MerkleBranch),         // existing Bitcoin SPV
    CheckpointCertified(Checkpoint), // existing Sui
    LedgerProof(LedgerInfo),         // existing Aptos
    MptReceipt(MptProof),            // existing Ethereum
    ZkSeal(ZkSealProof),             // NEW — chain-agnostic ZK
}
```

The `ZkSeal` variant bypasses chain-specific verification logic. A verifier that trusts the verifier key needs NO chain RPC. This is the privacy-preserving, trustless endpoint of the CSV vision.

**Step 5.4 — ZK verification in validator**

Update `csv-adapter-core/src/validator.rs` to dispatch on proof variant:
```rust
match &bundle.inclusion_proof {
    InclusionProof::ZkSeal(zk) => {
        let verifier = ZkVerifierRegistry::get(bundle.source_chain)?;
        verifier.verify(zk)?
    }
    // ... existing variants
}
```

---

### PHASE 6 — SCALABILITY AND ADVANCED FEATURES (ongoing)
*These are real next steps, not premature optimization — but only after Phases 0–4 are solid.*

**6.1 — Commitment chain batching**

`csv-adapter-core/src/commitment_chain.rs` + `dag.rs` already model multi-commitment chains. Add:
- Batch commitment builder in `csv-adapter/src/scalable_builder.rs` (after Phase 0 rename)
- Single on-chain anchor for N parallel commitments using MPC-style aggregation
- Benefit: N transfers for the cost of 1 chain transaction

**6.2 — MPC threshold signatures**

`csv-adapter-core/src/mpc.rs` exists. Complete it:
- Use `frost-secp256k1` crate for threshold Schnorr signatures
- Multi-party seal creation: a seal can require M-of-N signers to consume
- This unlocks multi-sig rights, DAO-controlled seals, and protocol-governed assets

**6.3 — Cross-chain atomic swaps**

Currently `csv-adapter-core/src/cross_chain.rs` and `csv-adapter/src/cross_chain.rs` implement lock-and-prove. Extend to atomic swap:
- Party A locks seal on Bitcoin
- Party B observes lock proof, locks seal on Ethereum
- A reveals secret → B's seal is now consumable
- Implemented as a commitment chain where each step is a new DAG node

**6.4 — RGB protocol compatibility**

`csv-adapter-core/src/rgb_compat.rs` already exists. This is powerful — RGB assets can be treated as CSV rights. Complete the mapping:
- RGB asset genesis → CSV `Right` genesis
- RGB state transition → CSV `Transition`
- RGB seal → CSV `SealRef` with Bitcoin UTXO backend

This means CSV becomes a superset of RGB, allowing RGB asset holders to use the CSV cross-chain protocol.

**6.5 — TypeScript SDK and MCP server**

`typescript-sdk/` and `csv-mcp-server/` are listed in ARCHITECTURE.md but excluded from the repomix output. These are the ecosystem surface for non-Rust developers. Prioritize after the core Rust codebase is stable.

---

## FILE-BY-FILE PRIORITIES TABLE

| File | Action | Phase |
|---|---|---|
| `csv-adapter/src/scalable_builder.rs` | DELETE | 0 |
| `csv-adapter/src/scalable_builder_v2.rs` | RENAME to `scalable_builder.rs` | 0 |
| `csv-adapter-bitcoin/src/proofs.rs` | DELETE | 0 |
| `csv-adapter-bitcoin/src/proofs_new.rs` | RENAME to `proofs.rs` | 0 |
| `csv-adapter-core/src/chain_registry.rs` | DELETE (absorbed by discovery) | 0 |
| `csv-adapter-core/src/vm.rs` | CONVERT to module dir, add AluVM | 1 |
| `csv-adapter-core/src/tapret_verify.rs` | AUDIT + add round-trip test | 1 |
| `csv-adapter-core/src/commitment_chain.rs` | WIRE into transfers and consignment | 1 |
| `csv-adapter-core/src/validator.rs` | ADD property tests | 1 |
| `csv-adapter-solana/src/` | ADD `proofs.rs`, complete `chain_operations.rs` | 2 |
| `csv-adapter/src/lib.rs` | ADD `build_discovery()` function | 2 |
| `csv-adapter-ethereum/src/mpt.rs` | AUDIT against EIP-1186 | 2 |
| `csv-adapter-store/src/state/backend.rs` | ADD `WasmBackend` + `NativeBackend` | 3 |
| `csv-adapter/src/transfers.rs` | ENFORCE proof-before-broadcast ordering | 3 |
| `csv-wallet/src/components/design_tokens.rs` | REFERENCE from every page | 4 |
| `csv-wallet/src/layout.rs` | APPLY design tokens | 4 |
| `csv-wallet/src/pages/**/*.rs` | AUDIT + apply design tokens | 4 |
| `csv-wallet/src/components/` | ADD `hash_display.rs`, `proof_badge.rs`, `chain_selector.rs` | 4 |
| `csv-adapter-core/src/zk_proof.rs` | CREATE | 5 |
| `csv-adapter-bitcoin/src/zk_prover.rs` | CREATE (SP1 guest) | 5 |
| `csv-adapter-core/src/proof.rs` | ADD `ZkSeal` variant to `InclusionProof` | 5 |
| `csv-adapter-core/src/mpc.rs` | COMPLETE with frost-secp256k1 | 6 |
| `csv-adapter-core/src/rgb_compat.rs` | COMPLETE asset genesis mapping | 6 |

---

## WHAT IS PREMATURE OPTIMIZATION (avoid for now)

These are real advanced techniques but wrong priority now:

- **Recursive ZK proofs** (proof of proofs) — needs working single-level ZK first (Phase 5)
- **More chain adapters** (Cosmos, Near, TON) — Solana not complete yet (Phase 2)
- **Multi-hop cross-chain** (A→B→C) — single hop not production-solid yet
- **On-chain verifier contracts for ZK** (Groth16 on Ethereum) — proof generation first
- **DAG sharding / parallel commitment chains at scale** — commitment chain is not even wired yet
- **Hardware wallet integration** — wallet UX not stable yet

---

## MAINTAINABILITY RULES

These rules apply immediately, not in a future phase:

1. **One purpose per file, stated in the module doc comment.** If a file has two purposes, split it.
2. **No `_v2` suffixes in production code.** Versioning is for crates, not files. Use `git` for history.
3. **`CODEBASE_MAP.md` is a required update on every PR** that adds or removes a file.
4. **Every public trait must have at least one integration test** that exercises the full call chain, not just unit behavior.
5. **Dead code gets a `// TODO: remove by [date]` comment** with a GitHub issue link. If neither exists, delete it immediately.
6. **Chain adapters are complete or absent.** A skeleton adapter that compiles but returns `unimplemented!()` is worse than no adapter — it passes CI and silently fails at runtime. Use `#[cfg(feature = "...")]` to gate incomplete adapters.

---

## WALLET DESIGN DIRECTION — FINAL RECOMMENDATION

The wallet has evolved into a technical dashboard. The design should embrace that, not fight it.

**Direction: Cryptographic Terminal**

- Dark background (`--color-gray-900`: `#111827`)
- Two accent colors only: operational blue (`--color-info-500`) and seal-active green (`--seal-active-dot`)
- All addresses and hashes in monospace with truncation and expand-on-hover
- Seal lifecycle as the primary visual metaphor — not wallets, not balances
- Status indicators using the existing `SealState` enum colors (already defined in design tokens)
- Remove decorative elements; visual weight from data, typography, status colors

**Three screens that must be excellent before anything else:**
1. **Seal dashboard** — all your seals, their states, one-click actions
2. **Proof viewer** — the proof bundle rendered as a readable audit trail
3. **Cross-chain transfer** — step-by-step with clear progress indicators using the `--seal-locked-*` color family

The wallet is a proof vault, not a bank. Design for that.

---

## SUMMARY: WHAT TO DO THIS WEEK

1. Delete `scalable_builder.rs` and old `proofs.rs` (30 min)
2. Collapse three registries into `ChainDiscovery` (2–3 hours)
3. Write `docs/CODEBASE_MAP.md` (2 hours)
4. Add `AluVmAdapter` stub to `csv-adapter-core/src/vm/` (not full implementation, but real structure replacing PassthroughVM) (half day)
5. Apply design tokens to `csv-wallet/src/layout.rs` as proof of concept (1 hour)

These five actions will immediately reduce confusion, signal architectural intent, and stop the dead-code accumulation without touching any protocol logic.
