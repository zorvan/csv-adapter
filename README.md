# CSV Adapter — Multi-Chain Client-Side Validation Framework

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-556%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)]()

**CSV Adapter** is an architectural prototype for extending Client-Side Validation beyond Bitcoin. It provides a chain-agnostic trait (`AnchorLayer`) and implements it for Bitcoin, Ethereum, Sui, and Aptos — using each chain's native primitives (rust-bitcoin, Alloy, ed25519-dalek).

**Status: Architectural prototype.** The trait design is sound, the code compiles, and 556 tests pass. But **the adapters cannot yet publish real transactions to any live blockchain**, Move contracts are undeployed, and RGB compatibility is unverified against the reference implementation. See [Reality Check](#reality-check) below.

---

## Table of Contents

- [Reality Check](#reality-check)
- [What This Is](#what-this-is)
- [What This Is Not](#what-this-is-not)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Philosophy](#philosophy)
- [What Problem It Solves](#what-problem-it-solves)
- [How It Solves It](#how-it-solves-it)
- [Design Decisions](#design-decisions)
- [Network Support](#network-support)
- [Project Structure](#project-structure)
- [Test Results](#test-results)
- [What Remains](#what-remains)
- [Key Dependencies](#key-dependencies)
- [License](#license)

---

## Reality Check

**Honest assessment of what this codebase can and cannot do today:**

| Claim | Status | Truth |
|-------|--------|-------|
| "Publish commitments to Bitcoin" | ❌ Not working | `publish()` generates fake txids (`b"sim-commit"`) without RPC feature. With RPC, wiring is incomplete. |
| "Publish commitments to Ethereum" | ❌ Not working | Alloy declared but not wired. MPT verification is custom, not tested against real proofs. |
| "Publish commitments to Sui" | ❌ Not working | `sui-sdk = "0.0.0"` is a placeholder. `real_rpc.rs` exists but uses direct HTTP, not the SDK. |
| "Publish commitments to Aptos" | ❌ Not working | `aptos-sdk` is optional and unused. REST API client exists but not wired to adapter. |
| "Cross-chain asset transfer" | ❌ Not implemented | `CrossChainValidator` checks hash equality on in-memory structs. No swap protocol, no bridge, no atomic mechanism. |
| "RGB compatible" | ⚠️ Partial | `rgb_compat` re-implements validation logic. Not verified against RGB reference implementation. |
| "Move contracts deployed" | ❌ Not deployed | `csv_seal.move` files exist for Sui and Aptos. Never compiled, never deployed, never tested on-chain. |
| "553 passing tests" | ⚠️ Misleading | Tests pass, but ~100% use mock RPCs returning hardcoded values (`[0xAB; 32]`). Zero tests run against live nodes. |
| "Production ready ~95%" | ❌ False | ~15% is more accurate. The hardest 85% (live RPC, contract deployment, security audit, end-to-end testing) is ahead. |

**This is a well-structured Rust skeleton with correct abstractions. It is not a deployable system.**

---

## What This Is

A trait-based framework that models the right abstractions for multi-chain Client-Side Validation:

- **`AnchorLayer` trait** — 10 methods every chain adapter must implement (`publish`, `verify_inclusion`, `verify_finality`, `enforce_seal`, `create_seal`, `hash_commitment`, `build_proof_bundle`, `rollback`, `domain_separator`, `signature_scheme`)
- **Chain-specific type systems** — Each adapter defines its own `SealRef`, `AnchorRef`, `InclusionProof`, `FinalityProof`
- **Consignment wire format** — Complete provable contract history with genesis, transitions, seal assignments, and anchors
- **Schema system** — Defines valid state types and transition rules
- **Deterministic VM abstraction** — For validating transition logic
- **RGB compatibility layer** — Re-implementation of RGB validation rules (unverified against reference)

The code compiles cleanly, follows Rust best practices, and the trait design would support adding new chains with ~500 lines of implementation.

---

## What This Is Not

- ❌ **Not a production system** — Cannot publish a single real transaction to any blockchain
- ❌ **Not cross-chain** — No protocol for moving assets between chains; just a hash equality check
- ❌ **Not RGB compatible** — Re-implements validation logic without verifying against the RGB reference implementation
- ❌ **Not tested on live networks** — All tests use mock RPCs; zero integration tests against real nodes
- ❌ **Not audited** — No security review of any kind

---

## Architecture

```
┌──────────────────────────────────────────┐
│          csv-adapter-core                 │
│  AnchorLayer trait + shared types        │
│  Consignment format + RGB compat layer   │
└──────────────────────────────────────────┘
         │         │         │         │
    ┌────┴───┐ ┌───┴────┐ ┌─┴────┐ ┌┴─────┐
    │Bitcoin │ │Ethereum│ │ Sui  │ │Aptos │
    │(0.30)  │ │(Alloy) │ │HTTP  │ │HTTP  │
    └────────┘ └────────┘ └──────┘ └──────┘
```

Every adapter implements the `AnchorLayer` trait. Without the `rpc` feature, all adapters return simulated results. With the `rpc` feature, Bitcoin uses `bitcoincore-rpc` and Ethereum uses `alloy`, but the wiring to `publish()` is incomplete. Sui and Aptos use direct HTTP calls (no official SDKs integrated).

---

## Quick Start

```bash
git clone https://github.com/your-org/csv-adapter.git
cd csv-adapter
cargo build --workspace
cargo test --workspace
```

```rust
use csv_adapter_bitcoin::BitcoinAnchorLayer;
use csv_adapter_core::{Hash, AnchorLayer};

// This works — but publish() returns a simulated txid, not a real transaction
let adapter = BitcoinAnchorLayer::signet()?;
let seal = adapter.create_seal(Some(100_000))?;
let anchor = adapter.publish(Hash::new([0xAB; 32]), seal)?; // FAKE txid
```

---

## Philosophy

Client-Side Validation flips the blockchain paradigm: instead of forcing every node to validate every transaction, validation is pushed to the edges. Only the participants in a specific contract need to verify its state transitions. The blockchain serves as an immutable commitment layer and single-use seal system — not a global validation engine.

CSV Adapter embraces this philosophy and extends it beyond Bitcoin. The core insight is that **CSV is chain-agnostic**: any blockchain that can provide (1) single-use seals and (2) deterministic commitment anchoring can support client-side validated state machines. By extracting the CSV abstraction into a reusable framework, we enable digital assets, smart contracts, and decentralized state machines on any chain — without forcing every validator to process every contract.

---

## What Problem It Solves

### 1. Blockchain Scalability
Every node validating every transaction is the fundamental scalability bottleneck of global consensus blockchains. CSV moves validation off-chain, reducing the blockchain's role to commitment anchoring and seal consumption.

### 2. Multi-Chain Asset Portability
RGB proved CSV works on Bitcoin. But assets and contracts need to exist across chains. CSV Adapter provides a unified interface for anchoring state transitions to Bitcoin, Ethereum, Sui, and Aptos — using each chain's native primitives.

### 3. Privacy by Default
Since validation happens client-side, contract details are only shared between participants. The blockchain only sees commitment hashes and consumed seals — not the full state machine.

### 4. Protocol Fragmentation
Without a standard interface, each chain reinvents the wheel for seal management, commitment anchoring, and proof verification. CSV Adapter provides a single `AnchorLayer` trait that any chain can implement.

---

## How It Solves It

CSV Adapter implements a **three-layer architecture**:

1. **Core Layer** (`csv-adapter-core`): Chain-agnostic traits and types.
   - `AnchorLayer` trait — the interface every chain adapter implements.
   - Consignment wire format — the complete provable history of a contract.
   - Schema system — defines valid state types and transitions.
   - Deterministic VM abstraction — for validating transition logic.
   - RGB compatibility layer — validates consignments against RGB protocol rules.

2. **Chain Adapter Layer** (one per blockchain):
   - **Bitcoin**: UTXO seals, Tapret/OP_RETURN anchoring, SPV inclusion proofs.
   - **Ethereum**: Storage slot seals, Merkle-Patricia Trie proofs, Alloy RPC.
   - **Sui**: Object seals, certified checkpoint finality, Ed25519 signatures.
   - **Aptos**: Resource seals, HotStuff 2f+1 finality, Ed25519 signatures.

3. **Persistence Layer** (`csv-adapter-store`):
   - SQLite-based local storage for seals, anchors, and consignments.

Each adapter translates the chain's native primitives into the core's generic types, enabling a single contract to be anchored to multiple chains with a unified verification pipeline.

---

## Design Decisions

### 1. Trait-Based Abstraction Over Inheritance
We chose a trait (`AnchorLayer`) rather than a base class because Rust's trait system enables zero-cost abstraction, multiple trait bounds, and cleaner composition. Each adapter owns its types (`SealRef`, `AnchorRef`, `InclusionProof`, `FinalityProof`) and maps them to core types at the boundary.

### 2. Official Blockchain Libraries
We use `rust-bitcoin`, `alloy`, and `ed25519-dalek` rather than re-implementing cryptographic primitives. This ensures maximum compatibility with each chain's consensus rules and reduces the attack surface. Sui and Aptos SDKs are NOT yet integrated — they use direct HTTP calls.

### 3. Feature-Gated RPC
Real blockchain communication is optional. All adapters work with mock RPCs by default, enabling offline testing, CI, and deterministic unit tests. The `rpc` feature gate switches to real HTTP/JSON-RPC clients. **But the wiring is incomplete — even with `rpc` enabled, `publish()` returns fake txids.**

### 4. RGB Compatibility as a Layer, Not a Fork
Rather than forking RGB, we implemented a compatibility layer (`rgb_compat`) that validates CSV consignments against RGB protocol rules. This includes seal double-spend detection, Tapret verification, topological ordering validation, and cross-chain consistency checks. **This layer is NOT verified against the RGB reference implementation.**

### 5. SQLite for Local State
The persistence layer uses SQLite (via `rusqlite`) because it's embedded, zero-config, and battle-tested. It stores seal registries, anchor histories, and consignment caches — enabling fast local queries without a running database server.

### 6. Signature Scheme Per Chain
Bitcoin and Ethereum use `Secp256k1`. Sui and Aptos use `Ed25519`. The `signature_scheme()` method on `AnchorLayer` returns the correct scheme so the verification pipeline selects the right cryptographic algorithm automatically.

### 7. Rollback as Seal Clearing
When a chain reorg invalidates an anchor, the adapter clears the seal from the registry so it can be reused. This is implemented via `clear_seal()` on each chain's seal registry, with graceful handling for seals not yet recorded.

---

## Network Support

| Chain | Networks | Default | Finality |
|-------|----------|---------|----------|
| **Bitcoin** | Mainnet, Testnet3, Signet, Regtest | Signet | 6 blocks |
| **Ethereum** | Mainnet, Sepolia, Holesky, Dev | Sepolia | 15 blocks |
| **Sui** | Mainnet, Testnet, Devnet, Local | Testnet | Certified checkpoint |
| **Aptos** | Mainnet, Testnet, Devnet | Testnet | HotStuff 2f+1 |

Configurations exist for all networks. **None have been tested against live nodes.**

---

## Project Structure

```
csv-adapter/
├── csv-adapter-core/          # Trait definitions, type system, state machine, RGB compatibility
├── csv-adapter-bitcoin/       # UTXO seals, Tapret anchoring, SPV proofs, rust-bitcoin integration
├── csv-adapter-ethereum/      # Storage slot seals, MPT proofs, Alloy integration
├── csv-adapter-sui/           # Object seals, checkpoint finality, Ed25519, HTTP client
├── csv-adapter-aptos/         # Resource seals, HotStuff finality, Ed25519, HTTP client
├── csv-adapter-store/         # SQLite persistence for seals and anchors
└── docs/
    └── PRODUCTION_PLAN.md          # 22-week plan to production
```

---

## Test Results

```
556 tests passing across all crates

  csv-adapter-core:        230  (includes 9 RGB compatibility tests)
  csv-adapter-bitcoin:      95  (unit tests only — all use mock RPCs)
  csv-adapter-ethereum:     60  (unit tests only — all use mock RPCs)
  csv-adapter-sui:          48  (unit tests only — all use mock RPCs)
  csv-adapter-aptos:        10  (unit tests only — all use mock RPCs)
  csv-adapter-store:         3
  Live network tests:       10  (all #[ignore] — have never been run)
```

**Important: 100% of the 546 non-ignored tests use mock RPCs that return hardcoded values.** No test has ever connected to a real Bitcoin node, Ethereum node, Sui fullnode, or Aptos fullnode. The mock returns `[0xAB; 32]` for txids, constructs block hashes from `height.to_le_bytes()`, and generates MPT proof nodes from `vec![0xAB; 32]`.

Run all tests:
```bash
cargo test --workspace
```

Run live network tests (will fail — not implemented):
```bash
cargo test --test live_network -- --ignored
```

---

## What Remains

**This is an incomplete prototype.** The full 22-week production plan with milestones, dependencies, and resource requirements is in [docs/PRODUCTION_PLAN.md](docs/PRODUCTION_PLAN.md).

**Summary of what needs to happen:**

| Sprint | Duration | Goal |
|--------|----------|------|
| 1. Wire Real RPCs | 4 weeks | `publish()` broadcasts real transactions on all 4 chains |
| 2. Deploy Contracts | 2 weeks | Move contracts deployed on Sui + Aptos testnets |
| 3. E2E Testing | 4 weeks | All adapters tested against live testnets (36 test cases) |
| 4. Cross-Chain Protocol | 4 weeks | Actual asset transfer between chains, not a hash check |
| 5. RGB Verification | 3 weeks | Compare against RGB reference, publish compatibility matrix |
| 6. Security Hardening | 5 weeks | Fuzz testing, property tests, third-party audit |

**Total: 22 weeks to production.** See the plan for details.

---

## Key Dependencies

| Chain | Library | Version | Purpose | Status |
|-------|---------|---------|---------|--------|
| Bitcoin | `bitcoin` | 0.30 | Block/tx parsing, Merkle trees, Taproot | ✅ Used |
| Bitcoin | `bitcoincore-rpc` | 0.17 | Node RPC | ⚠️ Declared, not wired |
| Ethereum | `alloy` | 0.9 | Transaction building, signing | ⚠️ Declared, not wired |
| Ethereum | `alloy-sol-types` | 0.8 | ABI encoding | ⚠️ Declared, not wired |
| Sui/Aptos | `ed25519-dalek` | 2.0 | Ed25519 signature verification | ✅ Used |
| Sui | `sui-sdk` | 0.0.0 | **Placeholder — not a real crate** | ❌ Not integrated |
| Aptos | `aptos-sdk` | 0.4 | Optional, unused | ❌ Not integrated |
| All | `rusqlite` | 0.30 | SQLite persistence | ✅ Used |

---

## License

MIT or Apache-2.0 — choose the license that best fits your use case.
