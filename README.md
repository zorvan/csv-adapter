# CSV Adapter — Client-Side Validation via Universal Seal Primitive

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-630%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)]()

**CSV Adapter** is a **client-side validation system** built on the **Universal Seal Primitive (USP)**. Rights are anchored to single-use seals on any chain. To transfer a Right, the seal is consumed on-chain and the new owner verifies the consumption proof locally—no bridges, no minting, no cross-chain messaging.

> We are not building a bridge. We are building a validation system where each chain enforces single-use at its strongest available guarantee, and clients verify everything else.

---

## Quick Start

```bash
git clone https://github.com/zorvan/csv-adapter.git
cd csv-adapter
cargo build --workspace
cargo test --workspace
```

### CLI

```bash
# Build CLI
cargo build -p csv-cli --release

# Generate wallet
csv wallet generate bitcoin test

# Check balance
csv wallet balance bitcoin

# Create a Right on Bitcoin
csv right create --chain bitcoin --value 100000

# Transfer cross-chain to Sui
csv cross-chain transfer --from bitcoin --to sui --right-id 0x...

# Verify the proof
csv proof verify-cross-chain --source bitcoin --dest sui --proof proof.json
```

Full CLI guide: [CLI Documentation](docs/DEVELOPER_GUIDE.md)

### Code

```rust
use csv_adapter_bitcoin::BitcoinAnchorLayer;
use csv_adapter_core::{Hash, AnchorLayer};

// Create a Right anchored to a Bitcoin UTXO
let adapter = BitcoinAnchorLayer::signet()?;
let seal = adapter.create_seal(Some(100_000))?;
let commitment = Hash::new([0xAB; 32]);

// Publish commitment (anchors Right to chain)
let anchor = adapter.publish(commitment, seal)?;

// To transfer: spend the UTXO, give the receiver the proof
// Receiver verifies locally, no destination chain needed
```

---

## How It Works

### Philosophy

Client-Side Validation flips the blockchain paradigm: validation is pushed to the edges. Only contract participants verify state transitions. The blockchain provides commitment anchoring and single-use enforcement, not global validation.

**The USP insight:** different chains enforce single-use at different levels. Bitcoin does it structurally (UTXOs). Sui does it structurally (Objects). Aptos does it via type system (Move resources). Ethereum does it cryptographically (nullifier contracts). Rather than pretending these are equivalent, we model the degradation explicitly and let each chain enforce at its strongest available guarantee.

**Cross-chain portability:** a Right doesn't "move" between chains. It exists in the client's state, anchored to whichever chain's seal enforced its single-use. Any client can verify any seal's consumption proof. The Right is portable because the proof is verifiable, not because a bridge transferred anything.

Full specification: [docs/Blueprint.md](docs/Blueprint.md)


### The Right Is Portable. The Seal Is Not

```
Alice owns Right A, anchored to Bitcoin UTXO X

Transfer to Bob:
1. Alice spends UTXO X          ← Bitcoin enforces single-use (structural)
2. Alice sends Bob: txid + Merkle proof
3. Bob's client verifies:
   ✓ UTXO X was spent           (chain-enforced)
   ✓ Merkle proof is valid      (client-verified)
   ✓ Right A's owner is now Bob (state transition)
```

**No destination chain.** No "minting" on another chain. No bridge tokens. Bob's client verified everything locally.

### Cross-Chain Portability

A Right created on Bitcoin can be verified by a client that only knows about Ethereum. The client doesn't care which chain enforced the seal—it verifies the proof and accepts the state transition.

**This is the USP:** the `Right` type is chain-agnostic. Each chain maps its native primitive (UTXO, Object, Resource, Nullifier) to the same `Right`. Clients verify proofs uniformly regardless of which chain enforced the seal.

---

## Supported Chains

| Chain | Level | Mechanism | Status |
|-------|-------|-----------|--------|
| **Bitcoin** | L1 Structural | UTXO spend | ✅ Complete |
| **Sui** | L1 Structural | Object deletion | ✅ Complete |
| **Aptos** | L2 Type-Enforced | Resource destruction | ✅ Complete |
| **Ethereum** | L3 Cryptographic | Nullifier registration | ✅ Complete |

---

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System architecture and design |
| [Developer Guide](docs/DEVELOPER_GUIDE.md) | How to build on CSV |
| [Blueprint](docs/BLUEPRINT.md) | Full specification and roadmap |
| [Cross-Chain Spec](docs/CROSS_CHAIN_SPEC.md) | Cross-chain protocol specification |
| [Cross-Chain Implementation](docs/CROSS_CHAIN_IMPLEMENTATION.md) | Implementation details |
| [E2E Testnet Manual](docs/E2E_TESTNET_MANUAL.md) | End-to-end testing guide |
| [Testnet Report](docs/TESTNET_E2E_REPORT.md) | Testnet results |
| [Audit Report](docs/Audit/csv-adapter-audit-report-10-april-2026.html) | Security audit findings |

---

## Key Concepts

### Right
A **Right** is the core portable primitive. It represents a transferrable claim that can be exercised at most once. Exists in **client state**, not on any chain.

### Seal
A **Seal** is the on-chain mechanism that enforces a Right's single-use. Chain-specific and exists on one chain only.

### Client-Side Validation (CSV)
The client does the verification, not the blockchain. The chain only records commitments and enforces single-use of seals.

---

## Test Results

```
630 tests passing across all crates

  csv-adapter-core:        296
  csv-adapter-bitcoin:      99
  csv-adapter-ethereum:     57
  csv-adapter-sui:          48
  csv-adapter-aptos:        10
  csv-adapter-store:         3
  Integration tests:        10
  Signature integration:     8
```

Run all tests:

```bash
cargo test --workspace
```

---

## Project Structure

```
csv-adapter/
├── csv-adapter-core/          # Right type, AnchorLayer trait, cross-chain registry
├── csv-adapter-bitcoin/       # L1 Structural: UTXO seals, Tapret anchoring
├── csv-adapter-ethereum/      # L3 Cryptographic: nullifier registry, MPT proofs
├── csv-adapter-sui/           # L1 Structural: object seals, checkpoint finality
├── csv-adapter-aptos/         # L2 Type-Enforced: resource seals, HotStuff finality
├── csv-adapter-store/         # SQLite persistence
└── csv-cli/                   # Command-line interface
```

---

## License

MIT or Apache-2.0 — choose the license that best fits your use case.
