# Raw HTTP Fallback Usage Documentation

## Overview

This document tracks locations in the codebase where raw HTTP requests are used as fallbacks when native SDK methods are not available. All such usage must follow these rules:

1. **Strongly Typed**: Request/response structures must be properly typed
2. **Error Handling**: Must return typed errors with context
3. **Documented**: Must have documentation explaining why native SDK isn't used
4. **Tested**: Must have tests for both success and failure paths

## Current Raw HTTP Usage

### 1. Wallet Service - Nonce Queries (Legacy)

**Location**: `csv-wallet/src/services/blockchain/service.rs`
**Methods**: `get_nonce()`, `get_gas_price()`

**Current Status**: These methods use manual JSON-RPC calls because the core `ChainQuery` trait doesn't expose nonce/gas price specific methods.

**Justification**: The `ChainQuery` trait focuses on higher-level queries (balance, transactions, finality). Nonce and gas price are EVM-specific concepts that don't map cleanly to all chains.

**Plan**: Add `get_transaction_count` and `get_fee_estimate` to `ChainQuery` trait and implement in all adapters.

**Error Handling**: Currently returns `BlockchainError` with chain context.

**Example**:

```rust
// Line ~260-298: Ethereum nonce query via raw HTTP
async fn get_nonce(&self, chain: Chain, address: &str) -> Result<u64, BlockchainError> {
    // Uses eth_getTransactionCount JSON-RPC
    // Returns typed BlockchainError on failure
}
```

### 2. CLI Chain Commands (Legacy)

**Location**: `csv-cli/src/commands/chain.rs`
**Method**: `cmd_info()`

**Current Status**: Uses `reqwest::blocking::get` for quick chain connectivity checks.

**Justification**: Simple health check that doesn't require full RPC call structure.

**Plan**: Replace with `ChainQuery::get_chain_info()` via facade.

## Migration Path

### Phase 1: Extend Core Traits

Add these methods to `ChainQuery` trait in `csv-adapter-core`:

```rust
#[async_trait]
pub trait ChainQuery: Send + Sync {
    // Existing methods...
    
    /// Get transaction count (nonce) for an address
    async fn get_transaction_count(&self, address: &str) -> ChainOpResult<u64> {
        // Default implementation returns CapabilityUnavailable
        Err(ChainOpError::CapabilityUnavailable(
            "Transaction count not supported on this chain".to_string()
        ))
    }
    
    /// Get fee estimate for transactions
    async fn get_fee_estimate(&self) -> ChainOpResult<u64> {
        // Default implementation returns CapabilityUnavailable
        Err(ChainOpError::CapabilityUnavailable(
            "Fee estimation not supported on this chain".to_string()
        ))
    }
}
```

### Phase 2: Implement in Adapters

- Bitcoin: Use `estimatesmartfee` RPC via `BitcoinRpc` trait
- Ethereum: Use `eth_gasPrice` and `eth_getTransactionCount` via Alloy
- Sui: Use `sui_getReferenceGasPrice` API
- Aptos: Use gas estimation API
- Solana: Use `getRecentBlockhash` for fee estimation

### Phase 3: Update Wallet Service

Replace manual HTTP calls in `csv-wallet` with trait methods:

```rust
// Instead of:
let body = serde_json::json!({
    "jsonrpc": "2.0",
    "method": "eth_getTransactionCount",
    // ...
});

// Use:
let nonce = adapter.get_transaction_count(address).await?;
```

## Error Handling Standards

All HTTP fallback code must use this error pattern:

```rust
let response = self.client.post(&rpc_url).json(&body).send().await
    .map_err(|e| ChainOpError::RpcError(format!(
        "Failed to call {}: {}", 
        method_name, 
        e
    )))?;
```

## Testing Requirements

Each HTTP fallback must have:

1. Unit test with mocked HTTP response
2. Integration test against real testnet (marked `#[ignore]`)
3. Error case tests for:
   - Network timeout
   - Invalid response format
   - RPC error response
   - HTTP error status codes

## Compliance Checklist

- [ ] All raw HTTP is behind typed interfaces
- [ ] All errors use ChainOpError variants
- [ ] All fallbacks are documented with justification
- [ ] All fallbacks have unit tests
- [ ] All fallbacks have integration tests
- [ ] Migration plan exists for each fallback

## Last Updated

May 2, 2026
