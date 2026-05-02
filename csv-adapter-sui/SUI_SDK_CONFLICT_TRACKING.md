# Sui SDK Core2 Dependency Conflict Tracking

**Status**: **RESOLVED** - Migration to sui-rust-sdk complete  
**Last Updated**: May 2, 2026  
**Priority**: Complete

---

## Problem Summary (Resolved)

~~The `csv-adapter-sui` crate cannot use the official `sui-sdk` due to dependency conflicts. The current workaround uses raw JSON-RPC via `reqwest`, which violates the Production Guarantee Plan's "Native SDK usage" requirement for Sui.~~

**Resolution**: Migrated to `sui-rust-sdk` from crates.io, resolving all dependency conflicts.

### Original Error

When attempting to add `sui-sdk` from the Sui monorepo:

```
error: failed to select a version for `tokio`.
    ... required by package `sui-keys v0.0.0 (https://github.com/MystenLabs/sui?tag=testnet-v1.70.2)`
    ... which satisfies git dependency `sui-keys` of package `sui-sdk v1.70.2`
versions that meet the requirements `=1.49.0` are: 1.49.0

all possible versions conflict with previously selected packages.

  previously selected package `tokio v1.52.1`
    ... which satisfies dependency `tokio = "^1"` of package `csv-cli v0.4.0`
```

**Note**: The `core2` dependency mentioned in issue #22547 appears to be incorrect or outdated. The actual conflict is with `tokio` version pinning in the Sui monorepo.

---

## Investigation Results

### Sui SDK Options

| SDK | Status | Dependencies | Recommendation |
|-----|--------|--------------|----------------|
| `sui-sdk` (monorepo) | Legacy | Pins `tokio = "=1.49.0"` | **Avoid** - Version conflicts |
| `sui-rust-sdk` | **Active** | Modular, minimal deps | **Use this** |

### sui-rust-sdk Architecture

The new [sui-rust-sdk](https://github.com/MystenLabs/sui-rust-sdk) provides modular crates:

- **`sui-sdk-types`** - Core types (no_std compatible)
- **`sui-crypto`** - Ed25519, BLS, hashing
- **`sui-rpc`** - JSON-RPC client
- **`sui-transaction-builder`** - Transaction construction

**Advantages**:
- Modular - pay only for what you use
- Light dependency footprint
- WASM-compatible
- No tokio version conflicts (uses `^1`)

---

## Resolution Path

### Option A: Migrate to sui-rust-sdk (Recommended)

**Steps**:
1. Replace `sui-sdk` dependency with `sui-rust-sdk` crates
2. Use `sui-sdk-types` for core types
3. Use `sui-rpc` for JSON-RPC client (replaces raw `reqwest`)
4. Use `sui-transaction-builder` for transaction construction
5. Use `sui-crypto` for signing (alternative to `ed25519-dalek`)

**Confirmed Crates.io Versions**:
```toml
[dependencies]
# Replace raw reqwest with sui-rpc
sui-rpc = "0.3.1"                # RPC client (replaces raw reqwest)
sui-sdk-types = "0.3.1"          # Core types
sui-transaction-builder = "0.3.1" # Transaction construction
sui-crypto = "0.3.0"             # Ed25519 signing (optional, can keep ed25519-dalek)
```

**Verified**: All crates available on crates.io as of May 2026.

### Option B: Wait for sui-sdk crates.io Release

**Risk**: Unknown timeline, may still have dependency conflicts

### Option C: Continue with JSON-RPC Fallback

**Status**: Current implementation  
**Risk**: Violates Production Guarantee Plan Phase 3

---

## Implementation Tasks

- [x] Verify `sui-rust-sdk` crates availability on crates.io
- [x] Test `sui-rpc` integration with existing `SuiRpc` trait
- [x] Replace raw BCS serialization with `sui-transaction-builder`
- [x] Update `NATIVE_SDK_COMPLIANCE.md` with new SDK status
- [x] Remove workaround code in `deploy.rs` and `chain_operations.rs`
- [x] Enable `sui-sdk-deploy` feature with real implementation

---

## References

- [Sui Legacy Rust SDK Docs](https://docs.sui.io/references/rust-sdk) - Documents the legacy `sui-sdk` as "Legacy"
- [sui-rust-sdk Repository](https://github.com/MystenLabs/sui-rust-sdk) - New modular SDK
- [sui-sdk-types Docs](https://mystenlabs.github.io/sui-rust-sdk/sui_sdk_types/sui-sdk-types) - Core types

---

## Next Actions

1. ~~**Immediate**: Verify `sui-rust-sdk` crate versions on crates.io~~ ✓ Complete
2. ~~**Short-term**: Create proof-of-concept with `sui-rpc` and `sui-transaction-builder`~~ ✓ Complete
3. ~~**Medium-term**: Full migration to sui-rust-sdk~~ ✓ Complete
4. ~~**Update**: Fix incorrect issue #22547 reference in `NATIVE_SDK_COMPLIANCE.md`~~ ✓ Complete
5. **Ongoing**: Monitor `sui-rust-sdk` for updates and new features

---

## Notes

- The issue #22547 in `NATIVE_SDK_COMPLIANCE.md` is incorrect - it's a PR about framework annotations
- The actual conflict is `tokio` version pinning, not `core2`
- The `sui-rust-sdk` appears to be Sui Labs' recommended path forward
