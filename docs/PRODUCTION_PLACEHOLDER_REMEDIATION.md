# Production Placeholder Remediation Plan

**Document:** PRODUCTION_PLACEHOLDER_REMEDIATION.md  
**Date:** May 2, 2026  
**Status:** Critical - Blocking Production  
**Total Items:** 56+ placeholders/unimplemented features

---

## Executive Summary

The codebase contains 56+ placeholder implementations and unimplemented features marked for production. These span critical security, cross-chain, and operational functionality. **All must be resolved before external audit**.

---

## Critical Items by Category

### 1. **PROOF VERIFICATION & GENERATION (CRITICAL - Security)**

| # | File | Line | Issue | Risk | Remediation |
|---|------|------|-------|------|-------------|
| 1 | `csv-adapter/src/facade.rs:468-477` | 468 | `generate_proof()` placeholder | Cannot generate proofs for cross-chain transfers | Implement proof query from chain state |
| 2 | `csv-adapter/src/facade.rs:485-496` | 485 | `verify_proof_bundle()` placeholder | Cannot verify incoming proofs - **breaks cross-chain security** | Implement `verify_proof()` call from core |
| 3 | `csv-wallet/src/services/blockchain/service.rs:495-504` | 495 | Proof verification stub | Wallet accepts proofs without verification | Wire up to `csv-adapter-core::proof_verify` |
| 4 | `csv-cli/src/commands/proofs.rs` | - | CLI proof commands stubbed | No proof CLI tooling | Implement proof generation/verification CLI |
| 5 | `csv-adapter-sui/src/chain_adapter_impl.rs:127` | 127 | Balance extraction placeholder | Cannot verify SUI balances | Implement BCS parsing for coin objects |

**Security Impact:** Items #2 and #3 are **critical security vulnerabilities** - cross-chain transfers will accept unverified proofs.

---

### 2. **TRANSACTION SUBMISSION (CRITICAL - Operational)**

| # | File | Line | Issue | Risk | Remediation |
|---|------|------|-------|------|-------------|
| 6 | `csv-wallet/src/services/blockchain/submitter.rs:54` | 54 | Bitcoin submission not implemented | Cannot submit Bitcoin transactions | Implement `bitcoin_rpc::send_raw_transaction` |
| 7 | `csv-wallet/src/services/blockchain/submitter.rs:70` | 70 | Ethereum submission not implemented | Cannot submit Ethereum transactions | Implement `eth_sendRawTransaction` |
| 8 | `csv-wallet/src/services/blockchain/submitter.rs:86` | 86 | Sui submission not implemented | Cannot submit Sui transactions | Implement `sui_executeTransactionBlock` |
| 9 | `csv-wallet/src/services/blockchain/submitter.rs:102` | 102 | Aptos submission not implemented | Cannot submit Aptos transactions | Implement `aptos transactions submit` |
| 10 | `csv-wallet/src/services/blockchain/submitter.rs:118` | 118 | Solana submission not implemented | Cannot submit Solana transactions | Implement `solana sendTransaction` |
| 11 | `csv-wallet/src/services/transaction_builder.rs:120` | 120 | Bitcoin tx building placeholder | Invalid Bitcoin transactions built | Implement proper UTXO selection |
| 12 | `csv-wallet/src/services/transaction_builder.rs:236-237` | 236 | Solana blockhash placeholder | Invalid Solana transactions | Query recent blockhash from RPC |

**Operational Impact:** Wallet cannot actually submit transactions to **any chain** - completely non-functional for transfers.

---

### 3. **DEPLOYMENT OPERATIONS (HIGH - Feature)**

| # | File | Line | Issue | Risk | Remediation |
|---|------|------|-------|------|-------------|
| 13 | `csv-adapter/src/deploy.rs:181-189` | 181 | Ethereum deploy placeholder (no feature) | Cannot deploy contracts | Enable `deploy-ethereum` feature |
| 14 | `csv-adapter/src/deploy.rs:193-200` | 193 | Ethereum contract deploy stub | Cannot deploy CSV contracts | Implement contract deployment |
| 15 | `csv-adapter/src/deploy.rs:261-270` | 261 | Sui deploy placeholder (no feature) | Cannot deploy Sui packages | Enable `deploy-sui` feature |
| 16 | `csv-adapter/src/deploy.rs:332-340` | 332 | Aptos deploy placeholder (no feature) | Cannot deploy Aptos modules | Enable `deploy-aptos` feature |
| 17 | `csv-adapter/src/deploy.rs:403-410` | 403 | Solana deploy placeholder (no feature) | Cannot deploy Solana programs | Enable `deploy-solana` feature |
| 18 | `csv-adapter-ethereum/src/deploy.rs:236-238` | 236 | Ethereum RPC deploy not implemented | Cannot deploy with RPC | Enable `rpc` feature |
| 19 | `csv-adapter-sui/src/deploy.rs:105-115` | 105 | Sui transaction submission not implemented | Cannot submit Sui deployments | Implement gRPC transaction submission |
| 20 | `csv-adapter-sui/src/deploy.rs:126-131` | 126 | Sui gRPC execution not implemented | Cannot execute Sui transactions | Implement gRPC service calls |
| 21 | `csv-adapter-sui/src/deploy.rs:191-192` | 191 | Sui package upgrade not implemented | Cannot upgrade packages | Implement upgrade transactions |
| 22 | `csv-adapter-sui/src/deploy.rs:230-232` | 223 | Sui BCS building not implemented | Cannot build Sui transactions | Implement BCS serialization |

---

### 4. **WALLET & KEY MANAGEMENT (HIGH - Security/Feature)**

| # | File | Line | Issue | Risk | Remediation |
|---|------|------|-------|------|-------------|
| 23 | `csv-wallet/src/services/blockchain/wallet.rs:130` | 130 | Signing not implemented for chains | Cannot sign transactions | Implement per-chain signing |
| 24 | `csv-wallet/src/assets/tracker.rs:42-43` | 42 | IndexedDB placeholder | Asset tracking non-functional | Implement IndexedDB integration |
| 25 | `csv-adapter-ethereum/src/chain_adapter_impl.rs:223-225` | 223 | Ethereum key import not implemented | Cannot import keys | Implement key import |
| 26 | `csv-adapter-ethereum/src/chain_adapter_impl.rs:264-267` | 264 | Ethereum client from config stub | Cannot create client | Implement `create_client` |
| 27 | `csv-adapter-ethereum/src/chain_adapter_impl.rs:334-336` | 334 | Ethereum RPC without feature | Falls back to stub | Enable `rpc` feature |
| 28 | `csv-adapter-sui/src/chain_adapter_impl.rs:239-241` | 239 | Sui key import not implemented | Cannot import keys | Implement key import |
| 29 | `csv-adapter-sui/src/chain_adapter_impl.rs:278-280` | 278 | Sui client from config stub | Cannot create client | Implement `create_client` |
| 30 | `csv-adapter-sui/src/chain_adapter_impl.rs:349-351` | 349 | Sui RPC without feature | Falls back to stub | Enable `rpc` feature |
| 31 | `csv-wallet/src/services/transaction_builder.rs:373-375` | 373 | Contract discovery placeholder | Cannot discover user contracts | Implement chain query for contracts |
| 32 | `csv-wallet/src/services/blockchain/service.rs:772-775` | 772 | Local transfer not implemented | Cannot do local transfers | Implement local transfer |

---

### 5. **ERROR HANDLING & CODES (MEDIUM - Quality)**

| # | File | Line | Issue | Risk | Remediation |
|---|------|------|-------|------|-------------|
| 33 | `csv-adapter-core/src/chain_adapter.rs:39-41` | 39 | `NotImplemented` error variant | API allows unimplemented errors | Remove after implementing all features |
| 34 | `csv-adapter-core/src/agent_types.rs:77` | 77 | `NOT_IMPLEMENTED` error code | Production code returns placeholder errors | Remove after implementation |
| 35 | `csv-adapter-ethereum/src/error.rs:42-43` | 42 | `NotImplemented` error variant | Ethereum allows unimplemented | Remove after implementation |
| 36 | `csv-adapter-solana/src/error.rs` | - | `NotImplemented` error variant | Solana allows unimplemented | Remove after implementation |
| 37 | `csv-adapter-solana/src/agent_types.rs:193` | 193 | `SOL_NOT_IMPLEMENTED` code | Solana placeholder error | Remove after implementation |

---

### 6. **ADVANCED FEATURES (MEDIUM - Future)**

| # | File | Line | Issue | Risk | Remediation |
|---|------|------|-------|------|-------------|
| 38 | `csv-adapter-core/src/advanced_commitments.rs:8` | 8 | ZK-proof verification not implemented | Future feature | Document as roadmap item |
| 39 | `csv-adapter-bitcoin/src/adapter.rs` | - | Bitcoin anchor layer methods | Partial implementation | Complete Bitcoin adapter |
| 40 | `csv-adapter-solana/src/chain_operations.rs` | - | Solana operations placeholder | Partial implementation | Complete Solana adapter |
| 41 | `csv-adapter-solana/src/rpc.rs` | - | Solana RPC stub | Cannot query Solana | Implement Solana RPC |
| 42 | `csv-adapter/tests/facade_security_tests.rs` | - | Security tests stubbed | No security validation | Implement security test suite |
| 43 | `csv-adapter-keystore/tests/security_tests.rs` | - | Keystore security tests stubbed | No keystore validation | Implement keystore tests |
| 44 | `csv-wallet/src/assets/tracker.rs:42` | 42 | Zeroed store pointer | Unsafe placeholder | Implement proper IndexedDB |

---

## Remediation Priority Matrix

```
                    IMPACT
              Low    Medium    High    Critical
         ┌─────────┬─────────┬─────────┬─────────┐
   High  │   P3    │   P2    │   P1    │  P0     │  ┐
EFFORT   │         │         │         │         │  │
         ├─────────┼─────────┼─────────┼─────────┤  │
   Med   │   P4    │   P3    │   P2    │  P1     │  ├ Priorities
         │         │         │         │         │  │
         ├─────────┼─────────┼─────────┼─────────┤  │
   Low   │   P5    │   P4    │   P3    │  P2     │  ┘
         └─────────┴─────────┴─────────┴─────────┘
```

### Priority Definitions

- **P0 - CRITICAL/CRITICAL:** Fix immediately. Blocks production and audit.
- **P1 - HIGH/CRITICAL:** Fix before audit. Security or core feature gaps.
- **P2 - HIGH/HIGH or MED/CRITICAL:** Fix in first remediation sprint.
- **P3 - MED/HIGH or HIGH/LOW:** Fix in second sprint.
- **P4 - LOW/MED:** Fix after core functionality.
- **P5 - LOW/LOW:** Document for future releases.

---

## Remediation Sprints

### Sprint 1: Security Critical (P0-P1) - Week 1-2

**Goal:** Resolve all security-critical items blocking audit.

**Items:**

1. #2 - `verify_proof_bundle()` in facade
2. #3 - Wallet proof verification
3. #4 - CLI proof commands
4. #33-37 - Remove `NotImplemented` error variants (require implementations first)

**Deliverables:**

- [ ] Cross-chain proof verification works end-to-end
- [ ] No `NotImplemented` errors in production paths
- [ ] Security tests passing

### Sprint 2: Core Operations (P1-P2) - Week 3-4

**Goal:** Make wallet operational for basic transfers.

**Items:**
5. #6-10 - Transaction submission for all chains
6. #11-12 - Proper transaction building
7. #23 - Wallet signing implementation
8. #25-30 - Key import and client creation

**Deliverables:**

- [ ] Wallet can submit transactions on all chains
- [ ] Proper transaction construction (no placeholders)
- [ ] Key import/export functional

### Sprint 3: Deployment Features (P2-P3) - Week 5-6

**Goal:** Enable contract deployment from CLI.

**Items:**
9. #13-22 - All deployment operations
10. #18-22 - Sui-specific deployment features

**Deliverables:**

- [ ] `csv-cli deploy` works for all chains
- [ ] Feature flags properly configured
- [ ] Deployment tested on testnets

### Sprint 4: Polish & Remaining (P3-P5) - Week 7-8

**Goal:** Complete remaining items and documentation.

**Items:**
11. #31 - Contract discovery
12. #32 - Local transfers
13. #38 - ZK-proof documentation
14. #39-44 - Remaining chain adapters and tests

**Deliverables:**

- [ ] All placeholder code removed
- [ ] Complete test coverage
- [ ] Documentation updated

---

## Implementation Details

### 1. Proof Verification Implementation

```rust
// In csv-adapter/src/facade.rs
pub async fn verify_proof_bundle(
    &self,
    chain: Chain,
    bundle: &ProofBundle,
) -> Result<bool, CsvError> {
    let adapter = self.get_adapter(chain)?;
    
    // Use core proof verification
    match csv_adapter_core::proof_verify::verify_proof(
        bundle,
        |seal_id| self.seal_registry.contains(seal_id),
        adapter.signature_scheme(),
    ) {
        Ok(()) => Ok(true),
        Err(e) => {
            log::warn!("Proof verification failed: {}", e);
            Ok(false)
        }
    }
}
```

### 2. Transaction Submission Pattern

For each chain in `submitter.rs`:

```rust
async fn submit_bitcoin(&self, tx: SignedTransaction) -> Result<TxHash, BlockchainError> {
    let client = self.get_bitcoin_rpc()?;
    let tx_hex = hex::encode(&tx.raw_bytes);
    
    let txid = client
        .send_raw_transaction(&tx_hex)
        .await
        .map_err(|e| BlockchainError {
            message: format!("Bitcoin submission failed: {}", e),
            chain: Some(Chain::Bitcoin),
            code: Some(500),
        })?;
    
    Ok(TxHash::from_str(&txid)?)
}
```

### 3. Feature Flag Strategy

Enable features in workspace `Cargo.toml`:

```toml
[features]
default = ["wallet", "cli"]
wallet = [
    "csv-adapter/ethereum-rpc",
    "csv-adapter/sui-rpc",
    "csv-adapter/bitcoin-rpc",
    "csv-adapter/aptos-rpc",
    "csv-adapter/solana-rpc",
]
deploy = [
    "csv-adapter/deploy-ethereum",
    "csv-adapter/deploy-sui",
    "csv-adapter/deploy-aptos",
    "csv-adapter/deploy-solana",
]
```

---

## Testing Requirements

Each remediation must include:

1. **Unit tests** - Test the specific function/method
2. **Integration tests** - Test with real RPC (testnet)
3. **Security tests** - Verify security properties (for P0/P1)
4. **End-to-end tests** - Full workflow validation

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[cfg_attr(not(feature = "testnet"), ignore)]
    async fn test_proof_verification_e2e() {
        // Test with real testnet data
    }
    
    #[test]
    fn test_proof_verification_mock() {
        // Test with mock data (always runs)
    }
}
```

---

## Audit Readiness Checklist

Before external audit:

- [ ] All P0 items resolved
- [ ] All P1 items resolved
- [ ] No `NotImplemented` errors in production paths
- [ ] No `placeholder` comments in security-critical code
- [ ] All `TODO` items triaged (removed or converted to issues)
- [ ] Integration tests pass on all chains
- [ ] Security tests pass
- [ ] Documentation reflects actual implementation

---

## Appendix: Full Item List by File

### csv-adapter/src/facade.rs (4 items)

- [ ] Line 468-477: `generate_proof()` placeholder
- [ ] Line 485-496: `verify_proof_bundle()` placeholder

### csv-adapter/src/deploy.rs (5 items)

- [ ] Line 181-189: Ethereum deploy placeholder (no feature)
- [ ] Line 193-200: Ethereum contract deploy stub
- [ ] Line 261-270: Sui deploy placeholder (no feature)
- [ ] Line 332-340: Aptos deploy placeholder (no feature)
- [ ] Line 403-410: Solana deploy placeholder (no feature)

### csv-wallet/src/services/transaction_builder.rs (5 items)

- [ ] Line 120: Bitcoin tx data placeholder
- [ ] Line 123-124: Input/output count placeholders
- [ ] Line 236-237: Solana blockhash placeholder
- [ ] Line 373-375: Contract discovery placeholder

### csv-wallet/src/services/blockchain/submitter.rs (5 items)

- [ ] Line 54: Bitcoin submission not implemented
- [ ] Line 70: Ethereum submission not implemented
- [ ] Line 86: Sui submission not implemented
- [ ] Line 102: Aptos submission not implemented
- [ ] Line 118: Solana submission not implemented

### csv-wallet/src/services/blockchain/wallet.rs (1 item)

- [ ] Line 130: Signing not implemented for some chains

### csv-wallet/src/services/blockchain/service.rs (2 items)

- [ ] Line 495-504: Proof verification placeholder
- [ ] Line 772-775: Local transfer not implemented

### csv-wallet/src/assets/tracker.rs (1 item)

- [ ] Line 42-43: IndexedDB placeholder with unsafe zeroed

### csv-adapter-ethereum/src/deploy.rs (1 item)

- [ ] Line 236-238: Ethereum RPC deploy not implemented

### csv-adapter-ethereum/src/chain_adapter_impl.rs (3 items)

- [ ] Line 223-225: Key import not implemented
- [ ] Line 264-267: Client from config stub
- [ ] Line 334-336: RPC without feature stub

### csv-adapter-sui/src/deploy.rs (4 items)

- [ ] Line 105-115: Transaction submission not implemented
- [ ] Line 126-131: gRPC execution not implemented
- [ ] Line 191-192: Package upgrade not implemented
- [ ] Line 230-232: BCS building not implemented

### csv-adapter-sui/src/chain_adapter_impl.rs (4 items)

- [ ] Line 127: Balance extraction placeholder
- [ ] Line 239-241: Key import not implemented
- [ ] Line 278-280: Client from config stub
- [ ] Line 349-351: RPC without feature stub

### csv-cli/src/commands/proofs.rs (1 item)

- [ ] Proof CLI commands stubbed

### csv-adapter-core/src/chain_adapter.rs (1 item)

- [ ] Line 39-41: `NotImplemented` error variant

### csv-adapter-core/src/agent_types.rs (1 item)

- [ ] Line 77: `NOT_IMPLEMENTED` error code

### csv-adapter-ethereum/src/error.rs (1 item)

- [ ] Line 42-43: `NotImplemented` error variant

### csv-adapter-solana/src/error.rs (2 items)

- [ ] `NotImplemented` error variant
- [ ] Line 193: `SOL_NOT_IMPLEMENTED` code

### csv-adapter/tests/facade_security_tests.rs (3 items)

- [ ] Security tests stubbed

### csv-adapter-keystore/tests/security_tests.rs (2 items)

- [ ] Keystore security tests stubbed

### csv-adapter-core/src/advanced_commitments.rs (1 item)

- [ ] Line 8: ZK-proof verification not implemented

---

**Next Steps:**

1. Review and approve this plan
2. Create GitHub issues for each P0/P1 item
3. Begin Sprint 1 implementation
4. Run pre-audit security scan after Sprint 2
