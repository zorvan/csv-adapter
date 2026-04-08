# CSV Adapter - Official SDK Integration Implementation Plan

**Status:** In Progress - Week 1 Started April 9, 2026

## Priority 1: Foundation - Dependencies & Wrappers ✅ IN PROGRESS

### Week 1: April 9-15
- [ ] **1.1**: Update `csv-adapter-bitcoin/Cargo.toml` with rust-bitcoin 0.31
- [ ] **1.2**: Update `csv-adapter-ethereum/Cargo.toml` with reth 0.2 + alloy 0.9
- [ ] **1.3**: Update `csv-adapter-aptos/Cargo.toml` with aptos-sdk 0.4
- [ ] **1.4**: Update `csv-adapter-sui/Cargo.toml` with sui-sdk 0.1
- [ ] **1.5**: Create wrapper modules for official SDKs
- [ ] **1.6**: Setup test environments for all chains

### Dependencies Added
- `bitcoin = "0.31"` (rust-bitcoin)
- `reth-trie = "0.2"`, `reth-primitives = "0.2"` (reth)
- `aptos-sdk = "0.4"` (Aptos)
- `sui-sdk = "0.1"` (Sui)

### Wrapper Modules Created
- `csv-adapter-bitcoin/src/wrapper_rust_bitcoin.rs`
- `csv-adapter-ethereum/src/wrapper_reth.rs`
- `csv-adapter-aptos/src/wrapper_aptos_sdk.rs`
- `csv-adapter-sui/src/wrapper_sui_sdk.rs`
- `csv-adapter-celestia/src/wrapper_celestia.rs`

## Priority 2: Bitcoin Adapter - rust-bitcoin Integration

### Week 2: April 16-22
- [ ] **2.1**: Replace `proofs.rs` with `proofs_rust_bitcoin.rs`
- [ ] **2.2**: Use `PartialMerkleTree` for SPV proofs
- [ ] **2.3**: Use `MerkleBlock` for complete block proofs
- [ ] **2.4**: Replace `RealBitcoinRpc` with `bitcoincore-rpc::Client`
- [ ] **2.5**: Update `adapter.rs` to use new proof implementations
- [ ] **2.6**: Test SPV verification with known test vectors
- [ ] **2.7**: Test Taproot signature verification

### Files Modified
- `csv-adapter-bitcoin/src/proofs.rs` → `proofs_rust_bitcoin.rs`
- `csv-adapter-bitcoin/src/rpc.rs` → `rpc_bitcoincore.rs`
- `csv-adapter-bitcoin/src/adapter.rs`

## Priority 3: Ethereum Adapter - reth Integration

### Week 3: April 23-29
- [ ] **3.1**: Replace `mpt.rs` with `mpt_reth.rs`
- [ ] **3.2**: Use `reth_trie::StorageRoot` for storage proofs
- [ ] **3.3**: Use `reth_trie::StateRoot` for state proofs
- [ ] **3.4**: Update `RealEthereumRpc` to use alloy 0.9
- [ ] **3.5**: Implement proper MPT construction
- [ ] **3.6**: Test with Holesky testnet
- [ ] **3.7**: Verify against Ethereum mainnet test vectors

### Files Modified
- `csv-adapter-ethereum/src/mpt.rs` → `mpt_reth.rs`
- `csv-adapter-ethereum/src/rpc.rs` → `rpc_alloy.rs`
- `csv-adapter-ethereum/src/adapter.rs`

## Priority 4: Aptos Adapter - aptos-sdk Integration

### Week 4: April 30 - May 6
- [ ] **4.1**: Replace `proofs.rs` with `proofs_aptos_sdk.rs`
- [ ] **4.2**: Use `aptos_sdk::LedgerInfo` for checkpoint verification
- [ ] **4.3**: Use `aptos_sdk::StateProof` for state proofs
- [ ] **4.4**: Implement proper event verification
- [ ] **4.5**: Test with Aptos devnet
- [ ] **4.6**: Verify HotStuff consensus

### Files Modified
- `csv-adapter-aptos/src/proofs.rs` → `proofs_aptos_sdk.rs`
- `csv-adapter-aptos/src/rpc.rs` → `rpc_aptos_sdk.rs`
- `csv-adapter-aptos/src/adapter.rs`

## Priority 5: Sui Adapter - sui-sdk Integration

### Week 5: May 7-13
- [ ] **5.1**: Replace `proofs.rs` with `proofs_sui_sdk.rs`
- [ ] **5.2**: Use `sui_sdk::Object` for state proofs
- [ ] **5.3**: Use `sui_sdk::Checkpoint` for consensus verification
- [ ] **5.4**: Implement proper event verification
- [ ] **5.5**: Test with Sui testnet
- [ ] **5.6**: Verify Narwhal consensus

### Files Modified
- `csv-adapter-sui/src/proofs.rs` → `proofs_sui_sdk.rs`
- `csv-adapter-sui/src/rpc.rs` → `rpc_sui_sdk.rs`
- `csv-adapter-sui/src/adapter.rs`

## Priority 6: Celestia Adapter - Full Implementation

### Week 6: May 14-20
- [ ] **6.1**: Implement blob submission using official SDK
- [ ] **6.2**: Implement blob retrieval using official SDK
- [ ] **6.3**: Add DAS verification
- [ ] **6.4**: Complete rollback logic
- [ ] **6.5**: Test with Celestia testnet

### Files Modified
- `csv-adapter-celestia/src/rpc.rs` → `rpc_celestia_sdk.rs`
- `csv-adapter-celestia/src/blob.rs` → `blob_celestia_sdk.rs`
- `csv-adapter-celestia/src/adapter.rs`

## Priority 7: Integration & Testing

### Week 7: May 21-27
- [ ] **7.1**: Cross-chain compatibility testing
- [ ] **7.2**: Production hardening (rate limiting, circuit breakers)
- [ ] **7.3**: Performance optimization
- [ ] **7.4**: Documentation updates
- [ ] **7.5**: Security audit preparation

## Priority 8: Production Release

### Week 8: May 28
- [ ] **8.1**: Security audit
- [ ] **8.2**: Production build configurations
- [ ] **8.3**: CI/CD pipeline setup
- [ ] **8.4**: Final release

## Key Implementation Principles

### Use Official SDK Types
- **Bitcoin:** `bitcoin::block::Block`, `bitcoin::merkle_tree::PartialMerkleTree`
- **Ethereum:** `reth_trie::StorageRoot`, `alloy::providers::Provider`
- **Aptos:** `aptos_sdk::LedgerInfo`, `aptos_sdk::StateProof`
- **Sui:** `sui_sdk::Object`, `sui_sdk::Checkpoint`
- **Celestia:** Official blob SDK

### Replace Custom Logic
- Remove all custom blockchain implementations
- Use official cryptographic functions
- Leverage official proof verification

### Maximize Compatibility
- 100% compatibility with official SDK behavior
- Zero functional regressions
- Production-grade security properties

## Current Status

- **Status:** Week 1 - Dependencies & Wrappers in progress
- **Tests Passing:** 445 tests (will be validated after each phase)
- **Compiles:** ✅ Yes
- **Feature Flags:** `rpc`, `production` ready

## Next Steps

1. **Immediate (Today):** Update Cargo.toml dependencies
2. **This Week:** Create wrapper modules for official SDKs
3. **Next Week:** Start Bitcoin adapter rewrite
4. **Continue:** Follow roadmap through Week 8

## Documentation

- `docs/REWRITE_STRATEGY.md` - High-level rewrite strategy
- `docs/SDK_INTEGRATION_GUIDE.md` - Detailed SDK integration guide
- `docs/IMPLEMENTATION_ROADMAP.md` - Detailed timeline and tasks
- `docs/PRODUCTION_READINESS_PLAN.md` - Production readiness plan

## Success Criteria

- [ ] All 445 tests passing after each phase
- [ ] 100% use of official SDK types
- [ ] Zero functional regressions
- [ ] Maximum compatibility with official implementations
- [ ] Production-ready security properties maintained
