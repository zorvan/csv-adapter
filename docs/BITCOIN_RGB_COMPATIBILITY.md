# Bitcoin CSV Adapter - RGB Compatibility Guide

**Date:** April 10, 2026  
**Goal:** Ensure Bitcoin adapter follows RGB protocol patterns exactly  
**Reference:** RGB specification, LNP/BP standards

---

## Executive Summary

The Bitcoin CSV adapter is a **generalization of RGB's Bitcoin implementation**. While RGB implements Client-Side Validation exclusively on Bitcoin using specific patterns, this adapter extends those patterns while maintaining exact compatibility. This document details what must match RGB for interoperability.

---

## RGB Core Requirements

### 1. Taproot Commitment Structure

RGB uses a specific taproot commitment structure that MUST be matched exactly.

#### RGB Tapret Pattern
```rust
// RGB Reference Implementation (pseudocode)
pub struct TapretCommitment {
    pub protocol_id: ProtocolId,    // 32 bytes - identifies RGB
    pub commitment: CommitmentHash, // 32 bytes - the actual commitment
    pub merkle_root: TapNodeHash,   // 32 bytes - taproot merkle root
    pub control_block: ControlBlock, // variable - proves path in taproot tree
}
```

#### Current CSV Adapter Status
```rust
// csv-adapter-bitcoin/src/tapret.rs - EXISTS but not RGB-verified
pub struct TapretCommitment {
    // ✅ Fields exist
    // ❌ Must verify exact structure matches RGB
    // ❌ Must verify control block construction matches RGB
}
```

#### What Must Match RGB

| Component | RGB Spec | CSV Adapter | Status |
|-----------|----------|-------------|--------|
| Protocol ID | 32 bytes | 32 bytes | ✅ Matches |
| Commitment hash | SHA256 double | SHA256 double | ✅ Matches |
| Merkle root | TapNodeHash | Not verified | ❌ Need verification |
| Control block | BIP-341 compliant | BIP-341 compliant | ⚠️ Need testing |
| Script structure | Specific to RGB | Similar | ⚠️ Need verification |

---

### 2. SPV Proof Format

RGB expects SPV proofs in a specific format for peer verification.

#### RGB SPV Proof Pattern
```rust
// RGB expects this exact structure
pub struct SpvProof {
    pub txid: Txid,                    // Transaction ID
    pub merkle_proof: MerkleBranch,    // Merkle branch hashes
    pub block_header: BlockHeader,     // Full 80-byte header
    pub tx_index: u32,                 // Position in block
    pub block_height: u64,             // Block number
}
```

#### Current CSV Adapter Status
```rust
// csv-adapter-bitcoin/src/types.rs - EXISTS
pub struct BitcoinInclusionProof {
    pub merkle_branch: Vec<[u8; 32]>,  // ✅ Matches MerkleBranch
    pub block_hash: [u8; 32],          // ✅ Can derive from header
    pub tx_index: u32,                 // ✅ Matches
    pub block_height: u64,             // ✅ Matches
}
```

#### What Must Match RGB

| Component | RGB Expects | CSV Provides | Status |
|-----------|-------------|--------------|--------|
| Merkle branch | Array of 32-byte hashes | `Vec<[u8; 32]>` | ✅ Matches |
| Block identification | Full header or hash | Block hash | ⚠️ RGB uses full header |
| Transaction index | u32 | u32 | ✅ Matches |
| Block height | u64 | u64 | ✅ Matches |
| Proof serialization | rust-bitcoin PMT | Custom serialization | ❌ Must match |

---

### 3. Transaction Structure

RGB creates specific transaction structures for commitments.

#### RGB Transaction Pattern
```
Inputs:
  - Seal UTXO (Taproot output)
  
Outputs:
  - Change output (if any)
  - OP_RETURN with protocol ID + commitment (fallback)
  - Taproot output with embedded commitment (primary)
```

#### Current CSV Adapter Status
```rust
// csv-adapter-bitcoin/src/tx_builder.rs - EXISTS
pub struct CommitmentTxBuilder {
    // ✅ Builds commitment transactions
    // ✅ Supports Taproot outputs
    // ✅ Supports OP_RETURN fallback
    // ❓ Must verify exact structure matches RGB
}
```

---

## Network Support - RGB Compatibility

### Bitcoin Networks

| Network | RGB Support | CSV Adapter | Compatibility |
|---------|-------------|-------------|---------------|
| **Mainnet** | ✅ Full | ✅ Ready | Need Tapret verification |
| **Testnet3** | ✅ Full | ✅ Ready | Need testing |
| **Signet** | ✅ Preferred | ✅ Default | ✅ Best for dev |
| **Regtest** | ✅ Testing | ✅ Ready | ✅ Best for testing |

### RGB Default Configuration

```rust
// What RGB uses as defaults
impl BitcoinConfig {
    /// RGB mainnet configuration
    pub fn rgb_mainnet() -> Self {
        Self {
            network: Network::Mainnet,
            finality_depth: 6,  // RGB standard confirmation depth
            publication_timeout_seconds: 3600,  // 1 hour censorship detection
            rpc_url: "http://127.0.0.1:8332",  // Local node required
        }
    }
    
    /// RGB development configuration (signet)
    pub fn rgb_dev() -> Self {
        Self {
            network: Network::Signet,
            finality_depth: 1,  // Fast for development
            publication_timeout_seconds: 600,
            rpc_url: "http://127.0.0.1:38332",
        }
    }
    
    /// RGB testing configuration (regtest)
    pub fn rgb_test() -> Self {
        Self {
            network: Network::Regtest,
            finality_depth: 1,  // Instant finality
            publication_timeout_seconds: 60,
            rpc_url: "http://127.0.0.1:18443",
        }
    }
}
```

---

## Critical Implementation Gaps

### Gap 1: Proof Extraction from Real Blocks 🔴

**Problem:** `generate_spv_proof()` returns stub, not real proofs

**RGB Impact:** Cannot produce proofs that RGB peers can verify

**Fix Required:**
```rust
/// Extract Merkle proof from real Bitcoin block (RGB-compatible)
pub fn extract_merkle_proof_from_block(
    txid: Txid,
    block: &Block,
    block_height: u64,
) -> Result<BitcoinInclusionProof, BitcoinError> {
    // 1. Find txid position (RGB does this)
    let tx_index = block.txdata.iter()
        .position(|t| t.compute_txid() == txid)
        .ok_or(BitcoinError::TxNotFound)?;
    
    // 2. Build PMT with match flags (RGB uses rust-bitcoin)
    let matches: Vec<bool> = block.txdata.iter()
        .map(|t| t.compute_txid() == txid)
        .collect();
    let pmt = PartialMerkleTree::from_txids(
        &block.txdata.iter().map(|t| t.compute_txid()).collect::<Vec<_>>(),
        &matches,
    );
    
    // 3. Serialize for wire format (RGB expects rust-bitcoin serialization)
    let merkle_branch = serialize_merkle_branch(&pmt);
    
    Ok(BitcoinInclusionProof {
        merkle_branch,
        block_hash: block.block_hash().to_byte_array(),
        tx_index: tx_index as u32,
        block_height,
    })
}
```

---

### Gap 2: Real RPC Integration 🔴

**Problem:** `publish()` generates fake txids without RPC

**RGB Impact:** Cannot create real anchors that RGB tools can verify

**Fix Required:**
```rust
/// Real Bitcoin RPC (RGB-compatible)
pub struct RealBitcoinRpc {
    client: bitcoincore_rpc::Client,
    network: Network,
}

impl RealBitcoinRpc {
    /// Publish commitment (RGB-style)
    pub fn publish_commitment(
        &self,
        outpoint: OutPoint,
        commitment: Hash,
    ) -> Result<Txid, BitcoinError> {
        // 1. Build RGB-compatible Taproot transaction
        let tx = self.build_rgb_taproot_tx(outpoint, commitment)?;
        
        // 2. Sign with BIP-341 (RGB uses Schnorr)
        let signed = self.sign_taproot_rgb(tx)?;
        
        // 3. Broadcast
        let txid = self.client.send_raw_transaction(&signed)?;
        
        // 4. Wait for confirmation (RGB-style)
        self.wait_for_confirmation_rgb(txid)?;
        
        Ok(txid)
    }
    
    /// Get block for proof extraction
    pub fn get_block(&self, hash: &BlockHash) -> Result<Block> {
        Ok(self.client.get_block(hash)?)
    }
    
    /// Get block height
    pub fn get_block_count(&self) -> Result<u64> {
        Ok(self.client.get_block_count()?)
    }
}
```

---

### Gap 3: Rollback Handling 🟡

**Problem:** `rollback()` doesn't actually unmark seals

**RGB Impact:** Reorgs leave seals permanently marked as used

**Fix Required:**
```rust
fn rollback(&self, anchor: Self::AnchorRef) -> CoreResult<()> {
    let current_height = self.get_current_height();
    
    if anchor.block_height > current_height {
        // 1. Unmark seal (allow reuse)
        let seal_ref = self.anchor_to_seal_ref(&anchor)?;
        self.seal_registry
            .lock()
            .unwrap()
            .unmark_seal(&seal_ref)
            .map_err(|e| AdapterError::Generic(e.to_string()))?;
        
        // 2. Log for auditing (RGB requires auditability)
        log::warn!(
            "Rolled back anchor {} at height {} (current: {})",
            hex::encode(anchor.txid),
            anchor.block_height,
            current_height
        );
        
        // 3. Update local state
        self.update_state_after_rollback(&anchor)?;
    }
    
    Ok(())
}
```

---

### Gap 4: Schema Validation 🟡

**Problem:** No RGB schema validation

**RGB Impact:** Cannot verify consignments against RGB schemas

**Fix Required:**
```rust
/// RGB Schema validation
pub struct RgbSchemaValidator;

impl RgbSchemaValidator {
    /// Validate consignment against RGB schema
    pub fn validate_schema(
        &self,
        consignment: &Consignment,
        schema_id: Hash,
    ) -> Result<(), SchemaError> {
        // 1. Verify schema ID is valid RGB schema
        // 2. Verify state types match schema
        // 3. Verify transition rules match schema
        // 4. Verify seal types are compatible
        // 5. Return validation result
    }
}
```

---

## Testing Requirements

### Unit Tests
- [x] Merkle proof verification (existing)
- [x] Tapret commitment creation (existing)
- [x] Transaction building (existing)
- [ ] **NEW:** Proof extraction from real blocks
- [ ] **NEW:** RGB schema validation
- [ ] **NEW:** Rollback handling

### Integration Tests
- [ ] Create seal → publish → verify inclusion → verify finality (signet)
- [ ] Create seal → publish → verify inclusion → verify finality (regtest)
- [ ] Test rollback handling with simulated reorg
- [ ] Test proof verification by RGB peer

### Network Tests
- [ ] Mainnet: Verify against real Bitcoin blocks
- [ ] Testnet3: Full lifecycle on testnet
- [ ] Signet: Full lifecycle on signet (RGB preferred)
- [ ] Regtest: Full lifecycle on regtest (fast testing)

---

## Verification Checklist

### Before Mainnet Deployment

- [ ] Tapret structure matches RGB specification exactly
- [ ] SPV proof format is RGB-compatible
- [ ] Transaction structure matches RGB patterns
- [ ] Schema validation implemented and tested
- [ ] Rollback handling tested with simulated reorgs
- [ ] All networks (mainnet/testnet/signet/regtest) tested
- [ ] Proof extraction produces valid, verifiable proofs
- [ ] Real RPC integration tested with real Bitcoin node
- [ ] Cross-verified with RGB reference implementation

### RGB Compatibility Verification

- [ ] Can create consignment that RGB tools can understand
- [ ] Can verify consignment created by RGB tools
- [ ] Tapret commitments verifiable by RGB verification
- [ ] SPV proofs verifiable by RGB SPV verifier
- [ ] Schema validation matches RGB validator

---

## Implementation Priority

### Week 1: Critical
1. Fix proof extraction from real blocks (2 days)
2. Test Tapret against RGB specification (1 day)
3. Implement rollback handling (1 day)

### Week 2: Integration
1. Wire real RPC client (2 days)
2. Test on signet (2 days)
3. Test on regtest (1 day)

### Week 3: Verification
1. Schema validation (2 days)
2. RGB compatibility testing (2 days)
3. Integration tests (1 day)

---

## References

- [RGB Specification](https://github.com/RGB-Tools/rgb-spec)
- [LNP/BP Standards](https://github.com/LNP-BP)
- [BIP-341 Taproot](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki)
- [rust-bitcoin Documentation](https://docs.rs/bitcoin)

---

*This document should be reviewed against the latest RGB specification and updated accordingly.*
