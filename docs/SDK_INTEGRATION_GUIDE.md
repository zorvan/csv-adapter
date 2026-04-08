# Official SDK Integration Guide

**Purpose:** Replace custom blockchain implementations with official Rust SDKs for maximum compatibility

---

## Bitcoin Integration (rust-bitcoin + bitcoincore-rpc)

### What to Replace

#### Old Implementation (Custom)
```rust
// csv-adapter-bitcoin/src/proofs.rs - Custom SPV
pub fn verify_merkle_proof(txid, merkle_root, proof) -> bool {
    // Custom implementation
}
```

#### New Implementation (rust-bitcoin)
```rust
// csv-adapter-bitcoin/src/proofs_rust_bitcoin.rs
use bitcoin::merkle_tree::PartialMerkleTree;
use bitcoin::util::merkleblock::MerkleBlock;

pub fn verify_merkle_proof_rust_bitcoin(
    txid: &[u8; 32],
    merkle_root: &[u8; 32],
    proof_bytes: &[u8],
    total_txs: u32,
) -> bool {
    let txid = bitcoin::transaction::Txid::from_slice(txid).unwrap();
    let mroot = bitcoin::hashes::sha256d::Hash::from_slice(merkle_root).unwrap();
    
    match PartialMerkleTree::from_bytes(proof_bytes, total_txs) {
        Ok(pmt) => pmt.check_merkle_proof(mroot).is_ok(),
        Err(_) => false,
    }
}
```

### Migration Steps

1. **Update Cargo.toml:**
   ```toml
   [dependencies]
   bitcoin = { version = "0.31", features = ["serde", "rand"] }
   bitcoincore-rpc = "0.18"
   ```

2. **Replace proofs.rs with proofs_rust_bitcoin.rs:**
   - Use `PartialMerkleTree` for SPV proofs
   - Use `MerkleBlock` for complete block proofs
   - Use `bitcoincore-rpc` for RPC calls

3. **Update adapter.rs:**
   ```rust
   use crate::proofs_rust_bitcoin::verify_merkle_proof_rust_bitcoin;
   
   fn verify_inclusion(&self, anchor: Self::AnchorRef) -> CoreResult<Self::InclusionProof> {
       // Use rust-bitcoin implementation
       verify_merkle_proof_rust_bitcoin(...)
   }
   ```

### Key Types Mapping

| Custom Type | rust-bitcoin Type |
|-------------|-------------------|
| `BitcoinInclusionProof` | `MerkleBlock` |
| `merkle_branch` | `PartialMerkleTree` |
| `verify_merkle_proof` | `pmt.check_merkle_proof()` |

---

## Ethereum Integration (reth + alloy)

### What to Replace

#### Old Implementation (Custom)
```rust
// csv-adapter-ethereum/src/mpt.rs - Custom MPT
pub fn compute_mpt_root(entries) -> H256 {
    // Placeholder implementation
    // Just hashes pairs together
}
```

#### New Implementation (reth)
```rust
// csv-adapter-ethereum/src/mpt_reth.rs
use reth_trie::{StorageRoot, StateRoot};
use reth_primitives::{H256, U256};
use alloy::primitives::B256;

pub fn compute_mpt_root_reth(entries: Vec<(H256, Vec<u8>)>) -> H256 {
    // Use reth_trie for proper MPT construction
    let storage_root = StorageRoot::new().into_root(entries);
    storage_root.into()
}
```

### Migration Steps

1. **Update Cargo.toml:**
   ```toml
   [dependencies]
   reth-trie = "0.2"
   reth-primitives = "0.2"
   alloy = { version = "0.9", features = ["full"] }
   ```

2. **Replace mpt.rs with mpt_reth.rs:**
   - Use `reth_trie::StorageRoot` for storage proofs
   - Use `reth_trie::StateRoot` for state proofs
   - Use `alloy` for RPC calls

3. **Update adapter.rs:**
   ```rust
   use crate::mpt_reth::compute_mpt_root_reth;
   
   fn verify_inclusion(&self, anchor: Self::AnchorRef) -> CoreResult<Self::InclusionProof> {
       // Use reth implementation
       compute_mpt_root_reth(...)
   }
   ```

### Key Types Mapping

| Custom Type | reth Type |
|-------------|-----------|
| `MptVerifier` | `reth_trie::StorageRoot` |
| `compute_mpt_root` | `StorageRoot::into_root()` |
| `verify_receipt_proof` | `alloy` transaction receipt verification |

---

## Aptos Integration (aptos-sdk)

### What to Replace

#### Old Implementation (Custom)
```rust
// csv-adapter-aptos/src/proofs.rs - Custom proofs
pub fn verify_event_in_tx(...) -> bool {
    // Stubbed implementation
    Ok(true) // Always returns true
}
```

#### New Implementation (aptos-sdk)
```rust
// csv-adapter-aptos/src/proofs_aptos_sdk.rs
use aptos_sdk::types::{AccountAddress, EventAccumulatorProof};
use aptos_sdk::ledger_info::LedgerInfo;
use aptos_sdk::transaction::TransactionInfo;

pub fn verify_event_in_tx_aptos_sdk(
    tx_version: u64,
    event_data: &[u8],
    client: &aptos_sdk::Client,
) -> Result<bool, AptosError> {
    // Use aptos-sdk for proper event verification
    let proof = client.get_state_proof(vec![tx_version]).await?;
    
    // Verify event in transaction using Move prover
    let events = client.get_events(tx_version).await?;
    
    Ok(events.iter().any(|e| e.data == event_data))
}
```

### Migration Steps

1. **Update Cargo.toml:**
   ```toml
   [dependencies]
   aptos-sdk = "0.4"
   ```

2. **Replace proofs.rs with proofs_aptos_sdk.rs:**
   - Use `aptos_sdk::LedgerInfo` for checkpoint verification
   - Use `aptos_sdk::StateProof` for state proofs
   - Use `aptos_sdk::EventAccumulatorProof` for event proofs

3. **Update adapter.rs:**
   ```rust
   use crate::proofs_aptos_sdk::verify_event_in_tx_aptos_sdk;
   ```

---

## Sui Integration (sui-sdk)

### What to Replace

#### Old Implementation (Custom)
```rust
// csv-adapter-sui/src/proofs.rs - Custom proofs
pub fn verify_event_in_tx(...) -> bool {
    // Stubbed implementation
    Ok(true) // Always returns true
}
```

#### New Implementation (sui-sdk)
```rust
// csv-adapter-sui/src/proofs_sui_sdk.rs
use sui_sdk::SuiClient;
use sui_sdk::types::Object;
use sui_sdk::types::Checkpoint;

pub fn verify_event_in_tx_sui_sdk(
    tx_digest: &[u8; 32],
    event_data: &[u8],
    client: &SuiClient,
) -> Result<bool, SuiError> {
    // Use sui-sdk for proper event verification
    let effects = client.transaction_block(tx_digest).await?;
    let events = effects.events()?;
    
    Ok(events.iter().any(|e| e.data == event_data))
}
```

### Migration Steps

1. **Update Cargo.toml:**
   ```toml
   [dependencies]
   sui-sdk = "0.1"
   ```

2. **Replace proofs.rs with proofs_sui_sdk.rs:**
   - Use `sui_sdk::Object` for state proofs
   - Use `sui_sdk::Checkpoint` for consensus verification
   - Use `sui_sdk::TransactionEffects` for event verification

---

## RPC Client Migration

### Bitcoin RPC

**Old:** Custom `RealBitcoinRpc`
**New:** `bitcoincore-rpc::Client`

```rust
use bitcoincore_rpc::{Client, Rpc};

// Old way
pub struct RealBitcoinRpc {
    url: String,
}

// New way
pub struct RealBitcoinRpc {
    client: Client,
}

impl RealBitcoinRpc {
    pub fn new(url: &str) -> Self {
        let client = Client::new(url, bitcoincore_rpc::Auth::None)
            .expect("Failed to create Bitcoin client");
        Self { client }
    }
    
    pub fn get_block_count(&self) -> Result<u64, BitcoinError> {
        Ok(self.client.get_block_count()? as u64)
    }
}
```

### Ethereum RPC

**Old:** Custom `RealEthereumRpc` with Alloy
**New:** Alloy (already using it)

```rust
use alloy::providers::Provider;
use alloy::transports::http::{Http, Client as Httpclient};

pub struct RealEthereumRpc {
    provider: Provider<Http>,
}

impl RealEthereumRpc {
    pub fn new(url: &str) -> Result<Self, AlloyRpcError> {
        let http = Http::new(url.parse()?);
        let client = Httpclient::new();
        let provider = Provider::new(client, http);
        
        Ok(Self { provider })
    }
    
    pub async fn block_number(&self) -> Result<u64, AlloyRpcError> {
        Ok(self.provider.get_block_number().await?)
    }
}
```

---

## Testing Strategy

### Unit Tests
- Verify all new implementations match official SDK behavior
- Compare results with known test vectors
- Ensure backward compatibility

### Integration Tests
- Connect to actual testnets
- Verify real proofs from chain data
- Test with production RPC endpoints

### Fuzz Tests
- Test with malformed inputs
- Verify error handling
- Check for edge cases

---

## Rollout Plan

1. **Week 1:** Bitcoin adapter with rust-bitcoin
2. **Week 2:** Ethereum adapter with reth
3. **Week 3:** Aptos adapter with aptos-sdk
4. **Week 4:** Sui adapter with sui-sdk
5. **Week 5:** Celestia adapter with official SDK
6. **Week 6:** Integration testing
7. **Week 7:** Documentation and security audit

---

## Success Criteria

- [ ] All official SDK types are used directly
- [ ] No custom blockchain logic remains
- [ ] 100% compatibility with official implementations
- [ ] All 445 tests pass
- [ ] Zero functional regressions

---

*This guide will be updated weekly during implementation.*