# CSV Adapter — Production Plan

**Current state:** Architectural prototype. Compiles clean. 553 unit tests pass (all mock). Cannot publish a real transaction to any blockchain.

**Goal:** Production-ready multi-chain CSV framework.

**Timeline:** 16–22 weeks from today.

---

## What Exists

| Layer | Status | Notes |
|-------|--------|-------|
| `AnchorLayer` trait | ✅ Done | 10 methods, clean abstraction |
| Type system | ✅ Done | Per-chain SealRef, AnchorRef, InclusionProof, FinalityProof |
| Consignment format | ✅ Done | Genesis, transitions, seal assignments, anchors |
| Schema system | ✅ Done | State types, transition definitions, validation |
| Bitcoin SPV verification | ✅ Done | Merkle proof verification works |
| Seal registry + rollback | ✅ Done | Replay prevention logic correct |
| Unit tests | ✅ 553 passing | All use mock RPCs |
| Bitcoin `bitcoincore-rpc` | ⚠️ Declared | Client struct exists, not wired to adapter |
| Ethereum Alloy | ⚠️ Declared | Dependencies in Cargo.toml, not integrated |
| Sui/Aptos SDKs | ❌ Not integrated | Placeholders (`sui-sdk = "0.0.0"`) or unused optional deps |
| Move contracts | ❌ Undeployed | `.move` files exist, never compiled |
| Cross-chain protocol | ❌ Missing | Hash equality check only |
| RGB verification | ❌ Unverified | Re-implementation, not compared to reference |

---

## What Remains — Ordered by Dependency

### Sprint 1: Wire Real RPCs (Weeks 1–4)

**Goal:** `publish()` broadcasts real transactions on all four chains.

#### Bitcoin (Week 1)
- [ ] Wire `RealBitcoinRpc` to `BitcoinAnchorLayer.publish()`
- [ ] Build Taproot commitment transaction via `CommitmentTxBuilder`
- [ ] Sign with Schnorr (BIP-341) via `SealWallet`
- [ ] Broadcast via `bitcoincore-rpc`
- [ ] Return real txid and block height in `BitcoinAnchorRef`
- [ ] Wire `extract_merkle_proof_from_block()` to `verify_inclusion()`
- [ ] Test against Signet node (local or public)

**Files:** `csv-adapter-bitcoin/src/adapter.rs`, `real_rpc.rs`

#### Ethereum (Week 2)
- [ ] Integrate Alloy properly (transaction building + signing)
- [ ] Build EIP-1559 transaction with `CSVSeal.markSealUsed()` calldata
- [ ] Sign with local key or `alloy-signer-local`
- [ ] Broadcast via Alloy provider
- [ ] Parse receipt logs to verify `SealUsed` event
- [ ] Return real tx hash and block number in `EthereumAnchorRef`
- [ ] Test against Sepolia public RPC

**Files:** `csv-adapter-ethereum/src/adapter.rs`, `real_rpc.rs`, `seal_contract.rs`

#### Sui (Week 3)
- [ ] Replace `sui-sdk = "0.0.0"` with real SDK or use direct JSON-RPC
- [ ] Build MoveCall transaction for `csv_seal::consume_seal()`
- [ ] Sign transaction (Ed25519 via `ed25519-dalek`)
- [ ] Submit via Sui JSON-RPC (`sui_executeTransactionBlock`)
- [ ] Wait for checkpoint finality
- [ ] Parse events to verify `AnchorEvent`
- [ ] Test against Sui Testnet

**Files:** `csv-adapter-sui/src/adapter.rs`, `real_rpc.rs`

#### Aptos (Week 4)
- [ ] Integrate `aptos-sdk` or complete direct REST API client
- [ ] Build Move entry function for `csv_seal::delete_seal()`
- [ ] Sign transaction (Ed25519)
- [ ] Submit via Aptos REST API (`/v1/transactions`)
- [ ] Wait for version confirmation
- [ ] Parse events to verify `AnchorEvent`
- [ ] Test against Aptos Testnet

**Files:** `csv-adapter-aptos/src/adapter.rs`, `real_rpc.rs`

**Deliverable:** All four adapters can publish real commitments and return real on-chain references.

---

### Sprint 2: Deploy Move Contracts (Weeks 5–6)

**Goal:** CSVSeal contracts deployed and tested on Sui and Aptos testnets.

#### Sui Move Contract (Week 5)
- [ ] Compile `csv-adapter-sui/contracts/csv_seal.move` with Sui CLI
- [ ] Fix any compilation errors
- [ ] Deploy to Sui Testnet
- [ ] Record Package ID
- [ ] Write test: create seal → consume seal → verify event
- [ ] Verify event contains correct commitment hash

**Files:** `csv-adapter-sui/contracts/csv_seal.move`

#### Aptos Move Contract (Week 6)
- [ ] Compile `csv-adapter-aptos/contracts/csv_seal.move` with Aptos CLI
- [ ] Fix any compilation errors
- [ ] Deploy to Aptos Testnet
- [ ] Record module address
- [ ] Write test: create seal → delete seal → verify event
- [ ] Verify event contains correct commitment hash

**Files:** `csv-adapter-aptos/contracts/csv_seal.move`

**Deliverable:** Both Move contracts deployed on testnets with verified end-to-end flows.

---

### Sprint 3: End-to-End Testing (Weeks 7–10)

**Goal:** Every adapter tested against live testnets with full lifecycle coverage.

#### Test Matrix

| Test | Bitcoin Signet | Ethereum Sepolia | Sui Testnet | Aptos Testnet |
|------|---------------|------------------|-------------|---------------|
| Connect to RPC | [ ] | [ ] | [ ] | [ ] |
| Query chain state | [ ] | [ ] | [ ] | [ ] |
| Create seal | [ ] | [ ] | [ ] | [ ] |
| Publish commitment | [ ] | [ ] | [ ] | [ ] |
| Verify inclusion | [ ] | [ ] | [ ] | [ ] |
| Verify finality | [ ] | [ ] | [ ] | [ ] |
| Seal replay prevention | [ ] | [ ] | [ ] | [ ] |
| Rollback handling | [ ] | [ ] | [ ] | [ ] |
| Network failure handling | [ ] | [ ] | [ ] | [ ] |

#### Infrastructure
- [ ] Set up CI with testnet access (GitHub Actions or self-hosted runner)
- [ ] Create test fixtures (pre-funded testnet wallets)
- [ ] Add retry logic with exponential backoff for transient RPC failures
- [ ] Add timeout configuration per chain
- [ ] Write failure-mode tests (node down, reorg, insufficient funds)

**Deliverable:** All 36 test matrix items passing on live testnets. CI pipeline green.

---

### Sprint 4: Cross-Chain Protocol (Weeks 11–14)

**Goal:** Actual protocol for moving assets/state between chains — not a hash check.

#### Design (Week 11)
- [ ] Specify cross-chain state transfer format
- [ ] Define lock-and-mint or burn-and-mint mechanism
- [ ] Design on-chain enforcement contracts for each chain
- [ ] Specify proof format for cross-chain verification
- [ ] Security review of cross-chain design

#### Implementation (Weeks 12–13)
- [ ] Implement lock mechanism on source chain
- [ ] Implement mint/release mechanism on destination chain
- [ ] Implement cross-chain proof verification
- [ ] Add `CrossChainValidator` with real verification logic
- [ ] Write integration tests for cross-chain transfers

#### Testing (Week 14)
- [ ] Test Bitcoin → Ethereum transfer
- [ ] Test Ethereum → Sui transfer
- [ ] Test Sui → Aptos transfer
- [ ] Test double-spend prevention across chains
- [ ] Test under network partition conditions

**Deliverable:** Working cross-chain transfers between at least 2 chain pairs with proof verification.

---

### Sprint 5: RGB Verification (Weeks 15–17)

**Goal:** Verify CSV Adapter is truly compatible with RGB protocol — not just a re-implementation.

#### Comparison (Week 15)
- [ ] Obtain RGB reference implementation
- [ ] Map CSV consignment fields to RGB consignment fields
- [ ] Identify all divergences (field names, serialization, validation rules)
- [ ] Document compatibility matrix

#### Alignment (Week 16)
- [ ] Fix any format divergences
- [ ] Verify Tapret structure matches RGB + BIP-341 exactly
- [ ] Verify OP_RETURN fallback matches RGB specification
- [ ] Verify schema validation rules match RGB

#### Interop Testing (Week 17)
- [ ] Create a CSV consignment and validate it with RGB tools
- [ ] Create an RGB consignment and validate it with CSV tools
- [ ] Test state transfer between CSV and RGB implementations
- [ ] Document interoperability guarantees

**Deliverable:** Verified compatibility matrix. At least one successful cross-validation between CSV and RGB consignments.

---

### Sprint 6: Security Hardening (Weeks 18–22)

**Goal:** Production-grade security posture.

#### Code Review (Week 18)
- [ ] Internal audit of all critical paths
- [ ] Review signature verification logic
- [ ] Review seal consumption logic for race conditions
- [ ] Review proof verification for edge cases

#### Testing (Weeks 19–20)
- [ ] Fuzz test all parsing functions (proptest or afl)
- [ ] Fuzz test proof verification
- [ ] Fuzz test signature verification
- [ ] Property-based testing of seal registry
- [ ] Property-based testing of rollback handling

#### External Audit (Weeks 21–22)
- [ ] Engage third-party security auditor
- [ ] Provide scope: all adapters, core types, proof verification
- [ ] Fix all critical/high findings
- [ ] Publish audit report

**Deliverable:** Audit report published. All critical findings resolved.

---

## Dependency Graph

```
Sprint 1: Wire Real RPCs          ────────────────────────────┐
                                                             ▼
Sprint 2: Deploy Move Contracts ──────────────────────────> Sprint 3: E2E Testing
                                                             │
                                                             ▼
Sprint 4: Cross-Chain Protocol ─────────────────────────────┘
                                                             │
Sprint 5: RGB Verification ──────────────────────────────────┤ (parallel)
                                                             │
Sprint 6: Security Hardening ────────────────────────────────┘
```

Sprints 1 and 2 can start in parallel. Sprint 3 depends on both 1 and 2. Sprints 4, 5, and 6 can start once Sprint 3 is complete (Sprint 5 and 6 are parallel with each other).

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| SDK API changes | Medium | High | Pin versions. Abstract behind trait. |
| Testnet faucet limits | Low | High | Run local nodes as fallback. |
| Move contract bugs | High | Medium | Formal verification of Move code. Test extensively before deployment. |
| Cross-chain design flaws | Critical | High | Security review before implementation. Start with 2 chains. |
| Audit finds critical issues | High | Medium | Budget 2 extra weeks for fixes. Start audit early. |
| RGB reference incompatible | Medium | Low | Early comparison (Week 15) to detect issues before deep integration. |

---

## Milestones

| Week | Milestone | Success Criteria |
|------|-----------|-----------------|
| 4 | Real RPCs wired | `publish()` returns real txids on all 4 chains |
| 6 | Contracts deployed | Move contracts on Sui + Aptos testnets, verified |
| 10 | E2E tests green | All 36 test matrix items passing on live testnets |
| 14 | Cross-chain working | Asset transfer between 2 chain pairs with proof verification |
| 17 | RGB verified | Compatibility matrix published, cross-validation successful |
| 22 | Audited | Third-party audit report published, all critical findings fixed |

---

## Resource Requirements

| Resource | Quantity | Duration |
|----------|----------|----------|
| Rust developers | 2–3 | 22 weeks |
| Move developer | 1 | Weeks 5–6 |
| Security auditor | External firm | Weeks 21–22 |
| Testnet infrastructure | Self-hosted nodes (optional) | Weeks 7–10 |
| Audit budget | $30k–$80k | Weeks 21–22 |

---

## Go/No-Go Criteria for Production

All of the following must be true:

- [ ] All 4 adapters can publish real commitments to their respective testnets
- [ ] All 36 E2E test matrix items passing
- [ ] Move contracts deployed and verified on Sui + Aptos testnets
- [ ] Cross-chain transfers working between at least 2 chain pairs
- [ ] RGB compatibility matrix published and verified
- [ ] Third-party security audit completed with no unresolved critical findings
- [ ] CI pipeline green on every commit
- [ ] All dependencies pinned to specific versions
- [ ] Operations runbook written (incident response, monitoring, alerting)

---

*Created: April 10, 2026*
*Next review: After Sprint 1 (Week 4)*
