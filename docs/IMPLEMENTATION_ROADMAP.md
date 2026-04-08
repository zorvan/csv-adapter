# CSV Adapter - Official SDK Integration Implementation Roadmap

**Start Date:** April 9, 2026  
**Target Completion:** May 28, 2026 (8 weeks)
**Current Status:** 445 tests passing, workspace compiles

---

## Phase 1: Foundation (Week 1) - April 9-15

### Goals
- Update all Cargo.toml files with official SDK dependencies
- Create wrapper modules for official SDKs
- Setup test environments

### Tasks

#### Day 1-2: Dependency Updates
- [ ] Update `csv-adapter-bitcoin/Cargo.toml` with rust-bitcoin 0.31
- [ ] Update `csv-adapter-ethereum/Cargo.toml` with reth 0.2 + alloy 0.9
- [ ] Update `csv-adapter-aptos/Cargo.toml` with aptos-sdk 0.4
- [ ] Update `csv-adapter-sui/Cargo.toml` with sui-sdk 0.1
- [ ] Update `csv-adapter-celestia/Cargo.toml` with official SDK
- [ ] Run `cargo update` to pull new versions

#### Day 3-4: Wrapper Modules
- [ ] Create `csv-adapter-bitcoin/src/wrapper_rust_bitcoin.rs`
- [ ] Create `csv-adapter-ethereum/src/wrapper_reth.rs`
- [ ] Create `csv-adapter-aptos/src/wrapper_aptos_sdk.rs`
- [ ] Create `csv-adapter-sui/src/wrapper_sui_sdk.rs`
- [ ] Create `csv-adapter-celestia/src/wrapper_celestia.rs`

#### Day 5: Test Environment Setup
- [ ] Setup Bitcoin testnet (Signet) node
- [ ] Setup Ethereum testnet (Holesky) connection
- [ ] Setup Aptos devnet connection
- [ ] Setup Sui testnet connection
- [ ] Setup Celestia testnet connection

### Deliverables
- [ ] All dependencies updated and compiling
- [ ] Wrapper modules created with basic functionality
- [ ] Test environments operational

---

## Phase 2: Bitcoin Adapter Rewrite (Week 2) - April 16-22

### Goals
- Replace custom SPV with rust-bitcoin
- Use bitcoincore-rpc for all RPC calls
- Implement proper Taproot support

### Tasks

#### Day 1-2: SPV Implementation
- [ ] Replace `proofs.rs` with `proofs_rust_bitcoin.rs`
- [ ] Use `PartialMerkleTree` for transaction inclusion proofs
- [ ] Use `MerkleBlock` for complete block proofs
- [ ] Implement `verify_merkle_proof_rust_bitcoin()`
- [ ] Implement `verify_full_spv_proof_rust_bitcoin()`

#### Day 3-4: RPC Client
- [ ] Replace `RealBitcoinRpc` with `bitcoincore-rpc::Client`
- [ ] Implement all RPC methods using official client
- [ ] Add proper error handling
- [ ] Implement caching for frequently accessed data

#### Day 5-6: Adapter Integration
- [ ] Update `adapter.rs` to use new proof implementations
- [ ] Update `wallet.rs` for Taproot key derivation
- [ ] Update `tx_builder.rs` for Tapret commitments
- [ ] Run all Bitcoin-specific tests

#### Day 7: Testing & Validation
- [ ] Test SPV verification with known test vectors
- [ ] Test Taproot signature verification
- [ ] Test UTXO selection and commitment building
- [ ] Fix any compatibility issues

### Deliverables
- [ ] Bitcoin adapter using rust-bitcoin 100%
- [ ] All Bitcoin tests passing
- [ ] SPV proofs using official implementation

---

## Phase 3: Ethereum Adapter Rewrite (Week 3) - April 23-29

### Goals
- Replace custom MPT with reth_trie
- Use alloy for RPC and transaction building
- Implement proper storage proofs

### Tasks

#### Day 1-2: MPT Implementation
- [ ] Replace `mpt.rs` with `mpt_reth.rs`
- [ ] Use `reth_trie::StorageRoot` for storage proofs
- [ ] Use `reth_trie::StateRoot` for state proofs
- [ ] Implement `compute_mpt_root_reth()`
- [ ] Implement `verify_receipt_proof_reth()`

#### Day 3-4: RPC Integration
- [ ] Update `RealEthereumRpc` to use alloy 0.9
- [ ] Implement proper transaction building with alloy
- [ ] Implement proper receipt verification
- [ ] Add proper error handling

#### Day 5-6: Contract Integration
- [ ] Use `alloy-contract` for CSVSeal interaction
- [ ] Implement proper event parsing with alloy-sol-types
- [ ] Update seal verification logic
- [ ] Test with local Hardhat node

#### Day 7: Testing & Validation
- [ ] Test MPT construction with known Ethereum test vectors
- [ ] Test storage proofs against Holesky testnet
- [ ] Test event verification with real transactions
- [ ] Fix any compatibility issues

### Deliverables
- [ ] Ethereum adapter using reth_trie 100%
- [ ] All Ethereum tests passing
- [ ] MPT proofs using official implementation

---

## Phase 4: Aptos Adapter Rewrite (Week 4) - April 30 - May 6

### Goals
- Replace custom proofs with aptos-sdk
- Use official Move prover for event verification
- Implement HotStuff consensus verification

### Tasks

#### Day 1-2: State Proofs
- [ ] Replace `proofs.rs` with `proofs_aptos_sdk.rs`
- [ ] Use `aptos_sdk::LedgerInfo` for checkpoint verification
- [ ] Use `aptos_sdk::StateProof` for state proofs
- [ ] Implement `verify_resource_proof_aptos_sdk()`
- [ ] Implement `verify_event_proof_aptos_sdk()`

#### Day 3-4: Transaction Verification
- [ ] Use `aptos_sdk::TransactionInfo` for transaction proofs
- [ ] Use `aptos_sdk::EventAccumulatorProof` for event proofs
- [ ] Implement proper version finality verification
- [ ] Add proper error handling

#### Day 5-6: Adapter Integration
- [ ] Update `adapter.rs` to use new proof implementations
- [ ] Update seal verification logic
- [ ] Update checkpoint verification
- [ ] Run all Aptos-specific tests

#### Day 7: Testing & Validation
- [ ] Test state proofs with Aptos devnet
- [ ] Test event verification with real transactions
- [ ] Test HotStuff consensus verification
- [ ] Fix any compatibility issues

### Deliverables
- [ ] Aptos adapter using aptos-sdk 100%
- [ ] All Aptos tests passing
- [ ] State proofs using official implementation

---

## Phase 5: Sui Adapter Rewrite (Week 5) - May 7-13

### Goals
- Replace custom object proofs with sui-sdk
- Use Sui checkpoint verification
- Implement proper event verification

### Tasks

#### Day 1-2: Object Proofs
- [ ] Replace `proofs.rs` with `proofs_sui_sdk.rs`
- [ ] Use `sui_sdk::Object` for state proofs
- [ ] Use `sui_sdk::GlobalStateHash` for object verification
- [ ] Implement `verify_object_proof_sui_sdk()`
- [ ] Implement `verify_dynamic_field_proof_sui_sdk()`

#### Day 3-4: Checkpoint Verification
- [ ] Use `sui_sdk::Checkpoint` for consensus verification
- [ ] Implement proper checkpoint certification
- [ ] Add proper error handling

#### Day 5-6: Event Verification
- [ ] Use `sui_sdk::TransactionEffects` for event verification
- [ ] Implement `verify_event_in_tx_sui_sdk()`
- [ ] Test with real Sui testnet data

#### Day 7: Testing & Validation
- [ ] Test object proofs with Sui testnet
- [ ] Test checkpoint verification
- [ ] Test event verification with real transactions
- [ ] Fix any compatibility issues

### Deliverables
- [ ] Sui adapter using sui-sdk 100%
- [ ] All Sui tests passing
- [ ] Object proofs using official implementation

---

## Phase 6: Celestia Adapter Implementation (Week 6) - May 14-20

### Goals
- Implement full blob submission/retrieval
- Add DAS verification
- Complete rollback logic

### Tasks

#### Day 1-2: Blob Operations
- [ ] Implement blob submission using official SDK
- [ ] Implement blob retrieval using official SDK
- [ ] Add namespace handling
- [ ] Implement proper error handling

#### Day 3-4: DAS Verification
- [ ] Implement Data Availability Sampling
- [ ] Add statistical verification
- [ ] Test with Celestia testnet

#### Day 5-6: Final Integration
- [ ] Update adapter for full functionality
- [ ] Test with real Celestia nodes
- [ ] Fix any compatibility issues

#### Day 7: Testing & Validation
- [ ] Test blob submission with testnet
- [ ] Test DAS verification
- [ ] Test rollback functionality
- [ ] Fix any remaining issues

### Deliverables
- [ ] Celestia adapter fully functional
- [ ] All Celestia tests passing
- [ ] Blob operations working with official SDK

---

## Phase 7: Integration Testing (Week 7) - May 21-27

### Goals
- Cross-chain compatibility testing
- Production hardening
- Documentation updates

### Tasks

#### Day 1-2: Cross-Chain Testing
- [ ] Test multi-chain anchor creation
- [ ] Test cross-chain proof verification
- [ ] Test state transition between chains
- [ ] Fix any compatibility issues

#### Day 3-4: Production Hardening
- [ ] Add rate limiting
- [ ] Add circuit breakers
- [ ] Add timeout configurations
- [ ] Add memory limits

#### Day 5-6: Documentation
- [ ] Update README.md with official SDK info
- [ ] Update per-adapter documentation
- [ ] Add migration guide
- [ ] Update API documentation

#### Day 7: Final Testing
- [ ] Run all 445 tests
- [ ] Verify no functional regressions
- [ ] Performance testing
- [ ] Security review

### Deliverables
- [ ] All cross-chain tests passing
- [ ] Production-ready code
- [ ] Complete documentation

---

## Phase 8: Security Audit & Release (Week 8) - May 28

### Goals
- Security audit
- Production release

### Tasks

#### Day 1-3: Security Audit
- [ ] Third-party security audit
- [ ] Code review for critical paths
- [ ] Penetration testing
- [ ] Cryptographic review

#### Day 4-5: Production Release
- [ ] Production build configurations
- [ ] CI/CD pipeline setup
- [ ] Monitoring and alerting
- [ ] Disaster recovery procedures

#### Day 6-7: Final Delivery
- [ ] Version tagging
- [ ] Release notes
- [ ] Deployment procedures
- [ ] Post-release support plan

### Deliverables
- [ ] Security audit complete
- [ ] Production release ready
- [ ] Full deployment documentation

---

## Risk Management

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes in official SDKs | HIGH | Pin to specific versions, test extensively |
| Performance degradation | MEDIUM | Profile before and after, optimize hot paths |
| Compatibility issues | HIGH | Use official type conversions, test with real data |

### Timeline Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Delays in one chain implementation | HIGH | Cross-train team members, parallelize where possible |
| Unexpected complexity | MEDIUM | Allocate buffer time, escalate early |
| Resource availability | MEDIUM | Maintain regular check-ins, adjust priorities |

---

## Success Criteria

### Technical Success
- [ ] All 445 tests passing
- [ ] Zero functional regressions
- [ ] 100% use of official SDK types
- [ ] All official SDK methods properly integrated
- [ ] No custom blockchain logic remains

### Business Success
- [ ] Maximum compatibility with official implementations
- [ ] Production-ready security properties
- [ ] Complete documentation
- [ ] Security audit passed

---

## Weekly Checkpoints

### Week 1 Checkpoint (April 15)
- All dependencies updated
- Wrapper modules created
- Test environments operational

### Week 2 Checkpoint (April 22)
- Bitcoin adapter using rust-bitcoin 100%
- All Bitcoin tests passing

### Week 3 Checkpoint (April 29)
- Ethereum adapter using reth 100%
- All Ethereum tests passing

### Week 4 Checkpoint (May 6)
- Aptos adapter using aptos-sdk 100%
- All Aptos tests passing

### Week 5 Checkpoint (May 13)
- Sui adapter using sui-sdk 100%
- All Sui tests passing

### Week 6 Checkpoint (May 20)
- Celestia adapter fully functional
- All Celestia tests passing

### Week 7 Checkpoint (May 27)
- Cross-chain tests passing
- Production hardening complete
- Documentation updated

### Week 8 Checkpoint (May 28)
- Security audit complete
- Production release ready

---

## Notes

- This roadmap is aggressive but achievable with focused effort
- Weekly checkpoints ensure we stay on track
- Flexibility built in for unexpected challenges
- Security and compatibility are top priorities

---

*Last Updated: April 9, 2026*
*Next Review: April 15, 2026 (Week 1 Checkpoint)*