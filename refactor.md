# CSV Protocol — Refactoring Plan

**Status:** Phase 0-4 complete. D4 (csv-wallet canonical types) **COMPLETE**.  
**Design Principle:** Canonical primitives, types, and traits. No duplication.

---

## Completed

- **D5:** csv-cli filesystem keystore (AES-256-GCM + Scrypt)
- **D1:** Mnemonic export/import (same mnemonic → same keys across devices)
- **D3:** Removed mock deployment UI from csv-wallet
- **D7:** Chain enum → string-based `ChainId` (csv_core::ChainId)
- **D8:** No dead code found
- **refactor.md:** Session log added
- **csv-cli compilation:** All type mismatch errors fixed (ChainId vs string literals)
- **csv-core cleanup:** Removed unused imports (Deserialize, String, HashMap)
- **Missing example:** Created csv-core/examples/basic_sanad.rs
- **Test fixes:** Fixed csv-core client.rs tests, csv-keys security tests

---

## D4: csv-wallet Type Migration (COMPLETE)

**Canonical types:** csv-store types are canonical. csv-wallet adapts to csv-store.

### What's Done

- `types.rs` re-exports csv-store domain types (SanadRecord, TransferRecord, ContractRecord, SealRecord, ProofRecord)
- `ChainId` (string) is canonical — all match statements use `.as_str()`
- `SealStatus`, `ProofStatus` added to csv-store
- `SealRecord`/`ProofRecord` updated with all needed fields
- `TransactionType`/`TransactionStatus` implement `Display`
- `derive_all_chain_keys` returns `HashMap<ChainId, SecretKey>`
- `derive_address_from_chain_id` function added
- `ContractRecord` and `TransactionRecord` now derive `PartialEq, Eq`
- `ProofData` now derives `Serialize, Deserialize`
- DeployContract route removed from csv-wallet
- All borrow-of-moved-value errors fixed with `.clone()`
- All Dioxus signal move errors fixed with `.clone()` or `.as_ref()`
- All Option<String> Display errors fixed with `.as_deref().unwrap_or("N/A")`
- All ChainId FromStr error type mismatches fixed (returns `Result<Self, ()>`)
- All ProofStatus::Failed match arms added
- All string literal quote issues in rsx! macros fixed
- csv-wallet compiles successfully with only warnings

---

## Next Phases

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 2: Registry Unification | IN PROGRESS | ChainRegistry removal, DriverRegistry integration |
| Phase 3: WASM Unification | PARTIALLY STARTED | csv-core no_std ready, wallet stubs remain |
| Phase 4: Explorer Decomposition | PARTIALLY DONE | Split into 4 sub-crates |
| Phase 5: ZK & Celestia | NOT STARTED | Per gap analysis |
| Phase 6: Repository Split | NOT STARTED | Per timing guidelines |

---

## Recent Changes (Session Log)

### Type System Unification
- Fixed 30+ type mismatch errors in csv-cli where `Chain` (deprecated alias for `ChainId`) was being matched against string literals
- Updated all match statements to use `.as_str()` pattern: `match chain.as_str() { "bitcoin" => ... }`
- Fixed `ChainId::new()` vs `Chain::new()` usage across 10+ files
- Added `.clone()` where `ChainId` was moved into async closures
- Fixed `ChainMetadata` struct to use `String` fields instead of `&'static str` for dynamic chain support

### csv-core Cleanup
- Removed unused import `Deserialize` from mcp.rs
- Removed unused import `alloc::string::String` from nullifier.rs  
- Removed unused import `std::collections::HashMap` from driver.rs
- Fixed `SanadOwnershipProof` → `OwnershipProof` in client.rs tests
- Fixed `builtin` module import in mcp.rs tests
- Added `start_block` field to ChainConfig in custom_chain_adapter example
- Created missing basic_sanad.rs example

### csv-keys Test Fixes
- Replaced deprecated `generate_mnemonic()` with `Mnemonic::generate(MnemonicType::Words12)`
- Replaced `Keystore`/`KeystoreConfig` with `KeystoreFile`/`Passphrase`
- Fixed `KeystoreError` variant usage to match actual enum definition

---

## Canonical Architecture

```
Level 1: Protocol (csv-core)
  ├── Seal, Commitment, Hash, DAGSegment, ProofBundle
  ├── SealProtocol (core trait)
  ├── ChainBackend (full implementation contract)
  └── ConsignmentValidator

Level 2: Chain Implementations (csv-{chain})
  ├── Implements: SealProtocol, ChainBackend
  ├── node.rs (chain node connection)
  └── Registers via DriverMetadata

Level 3: Surfaces (csv-sdk, csv-cli, csv-wallet, csv-explorer)
  ├── csv-sdk: unified facade + WASM bindings
  ├── csv-cli: thin CLI
  ├── csv-wallet: Dioxus UI (WASM)
  └── csv-explorer: 4 sub-crates
```

### Naming Rules

1. No pattern names as prefixes (`ChainDriver`, not `ChainAdapter`)
2. No `Ref` suffix on non-reference types (`SealPoint`, not `SealRef`)
3. Error types use `Error` suffix, singular
4. File name = primary type name
5. `Backend` = full implementation, `Driver` = descriptor

---

## Key Files

- `csv-core/src/protocol_version.rs` — ChainId definition
- `csv-store/src/state/domain.rs` — Canonical domain types
- `csv-wallet/src/context/types.rs` — Re-exports csv-store types
- `csv-keys/src/bip44.rs` — derive_all_chain_keys, derive_address_from_chain_id
- `csv-wallet/src/context/wallet.rs` — WalletContext using canonical types
