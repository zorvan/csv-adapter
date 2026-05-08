# Masterplan Remaining Critical Tasks

Based on analysis of docs/masterplan.md and codebase exploration:

## §4.1 - Ethereum Transaction Encoding (Non-blocking fix)

- [x] Chain ID configurable applied
- [x] RLP encoding applied
- [ ] **STILL REQUIRED**: Add a regression test vector derived from a real mainnet or Sepolia transaction

## §4.3 - Seal Nullifiers in Unencrypted LocalStorage

- [x] Encrypted IndexedDB module (csv-store/src/encrypted_storage.rs) - DONE
- [x] EncryptedSealManager surface (csv-wallet/src/services/seal_service.rs) - DONE
- [ ] **STILL REQUIRED**: Move WalletContext persistence off UnifiedStorage.seals in localStorage
- [ ] **STILL REQUIRED**: Derive encrypted seal key from wallet unlock/keystore flow
- [ ] **STILL REQUIRED**: Switch seal pages to async encrypted reads after migration

## §4.6 - Two-Chain Polling (Phase 2)

- [x] Explorer per-chain adaptive polling in config
- [ ] **STILL TODO**: Wire wallet's WebSocket subscription manager for per-chain adaptive intervals ±20% jitter

## §4.7 - Post-Quantum Signing (Phase 5)

- [x] SignatureScheme enum with MlDsa65 variant
- [x] Default impl returns SignatureScheme::MlDsa65
- [x] Sanad canonical serialization supports scheme byte 3
- [ ] **STILL TODO**: Actual ML-DSA-65 cryptographic implementation

## Phase 0.5 - Verification Gates

- [ ] WASM Storage Gate: csv-store --no-default-features --features encrypted-storage must compile for wasm32
- [ ] Migration Gate: Browser seal storage migration tests
- [ ] Warning Gate: Zero Rust warnings on key crates
- [ ] Doctest Gate: Documentation examples
- [ ] False-Fix Gate: Blockers closed only when call sites use them
