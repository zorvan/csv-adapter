# CSV Adapter - Official SDK Integration Rewrite Strategy

**Date:** April 9, 2026  
**Goal:** Maximize compatibility by using official Rust implementations

---

## Current State Analysis

### What Works
- Core trait definitions (`AnchorLayer`) are well-designed
- Multi-chain architecture with adapter pattern is solid
- Basic seal management and proof generation logic is implemented
- 445 tests passing across all crates

### What Needs Rewriting
- Chain-specific blockchain logic (SPV, MPT, state proofs)
- RPC client implementations (replace custom with official SDKs)
- Proof verification (use official cryptographic implementations)

---

## Strategy: Official SDK Integration

### Bitcoin Adapter
**Current:** Custom SPV implementation with `bitcoin` crate  
**Target:** `rust-bitcoin` + `bitcoincore-rpc`

**Key Components:**
- Use `bitcoin::block::Block` for block parsing
- Use `bitcoin::merkle_tree::PartialMerkleTree` for SPV proofs
- Use `bitcoincore-rpc` for RPC calls (replaces custom `RealBitcoinRpc`)

**Benefits:**
- Maximum compatibility with Bitcoin protocol
- Tested SPV implementation
- Official support for Taproot/Schnorr

### Ethereum Adapter  
**Current:** Custom MPT with placeholder implementation  
**Target:** `reth` + `alloy`

**Key Components:**
- Use `reth_trie` for proper Merkle Patricia Trie
- Use `reth_primitives` for Ethereum types
- Use `alloy` for RPC calls and transaction building
- Use `alloy-contract` for smart contract interactions

**Benefits:**
- Production-grade MPT implementation
- Ethereum mainnet compatibility
- Active development by Anvil team

### Aptos Adapter
**Current:** Custom state proof and event verification  
**Target:** `aptos-sdk`

**Key Components:**
- Use `aptos-sdk::LedgerInfo` for checkpoint verification
- Use `aptos-sdk::StateProof` for state proofs
- Use `aptos-sdk::EventAccumulatorProof` for event proofs
- Use official Move prover for event parsing

**Benefits:**
- Full HotStuff consensus compatibility
- Proper ledger version verification
- Move event verification via official prover

### Sui Adapter
**Current:** Custom object and checkpoint verification  
**Target:** `sui-sdk`

**Key Components:**
- Use `sui-sdk::Object` for state proofs
- Use `sui-sdk::Checkpoint` for consensus verification
- Use `sui-sdk::TransactionEffects` for event verification
- Leverage Sui's Global State Hash

**Benefits:**
- Full Narwhal consensus compatibility  
- Proper object state verification
- Checkpoint certification via official SDK

---

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)
- [ ] Update all Cargo.toml dependencies to use official SDKs
- [ ] Create wrapper modules for each official SDK
- [ ] Implement trait adapters for official types
- [ ] Setup test environments for each chain

### Phase 2: Bitcoin Adapter Rewrite (Week 2)
- [ ] Replace custom SPV with `rust-bitcoin` implementations
- [ ] Use `bitcoincore-rpc` for all RPC calls
- [ ] Implement proper Taproot support
- [ ] Add comprehensive SPV test vectors

### Phase 3: Ethereum Adapter Rewrite (Week 3)
- [ ] Replace custom MPT with `reth_trie`
- [ ] Use `alloy` for RPC and transaction building
- [ ] Implement proper storage proofs
- [ ] Add mainnet test vector compatibility

### Phase 4: Aptos Adapter Rewrite (Week 4)
- [ ] Replace custom proofs with `aptos-sdk`
- [ ] Use official Move prover for event verification
- [ ] Implement HotStuff consensus verification
- [ ] Add devnet integration tests

### Phase 5: Sui Adapter Rewrite (Week 5)
- [ ] Replace custom object proofs with `sui-sdk`
- [ ] Use Sui checkpoint verification
- [ ] Implement proper event verification
- [ ] Add testnet integration tests

### Phase 6: Celestia Adapter Rewrite (Week 6)
- [ ] Implement blob submission via official SDK
- [ ] Add DAS verification
- [ ] Complete rollback logic

### Phase 7: Integration & Testing (Week 7)
- [ ] Cross-chain compatibility testing
- [ ] Production hardening
- [ ] Documentation updates
- [ ] Security audit preparation

---

## Code Structure Changes

### Before (Current)
```rust
// Custom SPV implementation
pub mod proofs;
pub fn verify_merkle_proof(...) -> bool { ... }

// Custom MPT implementation  
pub mod mpt;
pub fn compute_mpt_root(...) -> H256 { ... }

// Custom RPC client
pub struct RealBitcoinRpc { ... }
```

### After (New)
```rust
// Use rust-bitcoin for SPV
use bitcoin::merkle_tree::PartialMerkleTree;

// Use reth for MPT
use reth_trie::StorageRoot;

// Use official RPC clients
use bitcoincore_rpc::Rpc;
use alloy::providers::Provider;
```

---

## Compatibility Goals

1. **Bitcoin:** 100% compatibility with `rust-bitcoin` SPV
2. **Ethereum:** Full MPT compatibility with `reth`
3. **Aptos:** Full state proof compatibility with `aptos-sdk`
4. **Sui:** Full object state compatibility with `sui-sdk`
5. **Celestia:** Full blob compatibility with official SDK

---

## Risk Mitigation

- **Incremental migration:** Keep existing code paths while adding new ones
- **Feature flags:** Use `old-impl` vs `new-impl` feature flags
- **Test coverage:** Ensure 100% test coverage before removing old implementations
- **Documentation:** Document breaking changes and migration paths

---

## Success Metrics

- [ ] All 445 existing tests pass with new implementations
- [ ] Zero functional regressions
- [ ] 100% compatibility with official SDK types
- [ ] Production-ready security properties maintained

---

*This strategy document will be updated weekly during implementation.*