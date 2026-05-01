# Sui SDK Integration Justification

## Phase 3 Compliance: Native SDK Integration

### Status: Raw HTTP with Documented Justification

Per Phase 3 of the PRODUCTION_GUARANTEE_PLAN.md, all chain adapters **must** use their official native SDK. However, Sui currently uses direct JSON-RPC over HTTP via `reqwest` instead of the `sui-sdk` crate.

## Justification for Raw HTTP

### 1. Known Dependency Conflict

The `sui-sdk` crate (as of testnet-v1.70.2) has a transitive dependency on `core2` via the `sui-types` crate. This causes a build conflict with other crates in the workspace that depend on different versions of `core2` or related cryptographic libraries.

**Error Pattern:**
```
error: failed to select a version for `core2`.
    ... required by package `sui-types v1.70.2`
    ... which satisfies git dependency `sui-types` of package `sui-sdk v1.70.2`
```

### 2. Root Cause

The `sui-types` crate uses `core2` for no-std compatibility, which conflicts with:
- `bitcoin` crate's use of `core2` (different version requirements)
- `secp256k1` crate's feature flags

This is a known issue in the Sui ecosystem tracked at:
- https://github.com/MystenLabs/sui/issues/18443

### 3. Current Implementation

**Location:** `csv-adapter-sui/src/real_rpc.rs` (when `rpc` feature enabled)

The current implementation uses:
- `reqwest` for HTTP transport
- Direct JSON-RPC 2.0 protocol
- BCS serialization for transaction data

### 4. Compliance Argument

Per PRODUCTION_GUARANTEE_PLAN.md Phase 3:

> "No raw HTTP RPC calls unless explicitly justified and documented"

This document provides the explicit justification. The Sui adapter:

1. **Uses official serialization format**: BCS (Binary Canonical Serialization) is the official Sui serialization format
2. **Uses official RPC protocol**: JSON-RPC 2.0 is the official Sui node protocol
3. **Minimal abstraction layer**: The `SuiRpc` trait abstracts the transport, allowing SDK migration when the dependency issue is resolved

### 5. Migration Path

When the `core2` dependency conflict is resolved:

1. Uncomment the `sui-sdk` dependency in `csv-adapter-sui/Cargo.toml`
2. Implement a `SuiSdkRpc` struct that wraps `sui-sdk` types
3. Make it the default implementation, keeping `reqwest` as a fallback

### 6. Security Considerations

The raw HTTP implementation:
- Does NOT implement its own cryptography
- Uses standard `ed25519-dalek` for signing (same as Sui SDK)
- Uses standard BCS serialization (official Sui format)
- Relies on the Sui node for transaction validation

## Conclusion

The Sui adapter is **conditionally compliant** with Phase 3:
- ✅ Uses official serialization (BCS)
- ✅ Uses official RPC protocol (JSON-RPC 2.0)
- ⚠️ Uses raw HTTP instead of SDK due to dependency conflict
- ✅ Documented justification provided (this document)

**Recommendation:** Accept as production-ready with documented exception. Monitor Sui SDK releases for dependency resolution.

---
*Document Version: 1.0*
*Date: 2026-05-01*
*Review on: Sui SDK v1.80.0+ or when core2 conflict resolved*
