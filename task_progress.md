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

1. WebSocket / Adaptive Polling
1a. Explorer's WebSocket Subscription Manager
File: /home/zorvan/Work/projects/csv-adapter/csv-explorer/api/src/websocket.rs (339 lines)
This is the full WebSocket subscription manager. Key components:

- SubscriptionManager (line 86): Manages active subscriptions per address using Arc<RwLock<HashMap<String, Vec<mpsc::UnboundedSender<SubscriptionEvent>>>>>. It supports subscribe, unsubscribe, and broadcast operations.
- SubscriptionEvent (line 19-57): Enum with variants NewSanad, NewSeal, NewTransfer, IndexingComplete, and IndexingError. Each event carries address, chain, and chain-specific IDs.
- handle_websocket (line 235-321): The async handler that runs a tokio::select! loop processing incoming SubscriptionRequest messages (subscribe/unsubscribe actions) and forwarding events from subscription receivers to the WebSocket.
- subscription_ws (line 324-332): Warp filter that creates the /ws/subscriptions endpoint.
Notable gap: The SubscriptionManager is address-based (not chain-aware), and there is no per-chain adaptive interval or jitter logic. The task_progress.md confirms: "Wire wallet's WebSocket subscription manager for per-chain adaptive intervals +/-20% jitter" is STILL TODO.
1b. Wallet's Polling Mechanism
The wallet does not have its own polling mechanism. Instead, it uses:
File: /home/zorvan/Work/projects/csv-adapter/csv-wallet/src/services/explorer.rs (444 lines)
The ExplorerService is a simple HTTP client that makes synchronous REST calls to the explorer API. Methods include get_sanad, get_seals_by_owner, get_transfers, get_address_data, and priority indexing APIs. There is no polling loop -- the wallet fetches data on-demand when triggered by UI actions.
File: /home/zorvan/Work/projects/csv-adapter/csv-wallet/src/seals/monitor.rs (99 lines)
The SealMonitor has a start_monitoring method (line 81-92) that accepts seal_ids and interval_secs, but the implementation is a stub:
pub async fn start_monitoring(
    &self,
    seal_ids: Vec<String>,
    interval_secs: u64,
) -> tokio::sync::oneshot::Sender<()> {
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    // Note: tokio not available in wasm, this would use gloo-timers in production
    let_ = (seal_ids, interval_secs, rx);
    tx
}
It is a no-op placeholder with a comment noting tokio is unavailable in wasm.
1c. Per-Chain Config in Explorer
File: /home/zorvan/Work/projects/csv-adapter/csv-explorer/shared/src/config.rs (288 lines)
The ExplorerConfig struct contains:
- indexer: IndexerConfig with a single global poll_interval_ms (default 5000ms)
- chains: HashMap<String, ChainConfig> where each ChainConfig has enabled, network, rpc_url, and start_block -- but no per-chain poll interval
The ChainConfig struct (line 92-102) does NOT include a poll_interval_ms field. The global poll_interval_ms in IndexerConfig is used uniformly for all chains in SyncCoordinator (see sync.rs line 173: sleep(Duration::from_millis(self.poll_interval_ms)).await).
File: /home/zorvan/Work/projects/csv-adapter/csv-explorer/config.example.toml (42 lines)
Shows per-chain sections ([chains.bitcoin], [chains.ethereum], etc.) with enabled, network, rpc_url, and start_block fields. The poll_interval_ms = 5000 is set at the [indexer] level (global), not per-chain.
1d. Search Results Summary
The grep for polling|adaptive|ws_subscription|subscription_manager returned 12 matches total:
- Cargo.lock references to the polling crate (dependency)
- task_progress.md references to per-chain adaptive polling and the TODO item
- docs/masterplan.md references describing the current state and what's still needed
- csv-explorer/api/src/websocket.rs -- the SubscriptionManager implementation

---
1. ML-DSA-65 / Post-Quantum
2a. Full csv-core/src/signature.rs (545 lines)
File: /home/zorvan/Work/projects/csv-adapter/csv-core/src/signature.rs
Key findings:

- SignatureScheme enum (line 25-34): Three variants -- Secp256k1 (ECDSA), Ed25519, and MlDsa65 (ML-DSA-65, FIPS 204). The MlDsa65 variant is documented as "Post-quantum secure. Required default for all long-lived proof bundles. 65-byte security level, public key ~1312 bytes, signature ~2420 bytes."
- Default impl (line 36-42): Returns SignatureScheme::MlDsa65 -- the post-quantum default from genesis (Decision D-1).
- Signature::verify (line 66-82): The MlDsa65 arm returns an error: "ML-DSA-65 verification not yet implemented - Phase 5 engineering task". Secp256k1 and Ed25519 are fully implemented with actual cryptographic verification using secp256k1 and ed25519-dalek crates respectively.
- Tests: 17 tests covering Secp256k1 (valid, invalid, edge cases) and Ed25519 (valid, invalid, edge cases). No ML-DSA-65 tests exist because the implementation is a placeholder.
2b. csv-core/src/sanad.rs -- Scheme Byte Handling
File: /home/zorvan/Work/projects/csv-adapter/csv-core/src/sanad.rs (719 lines)
The canonical serialization/deserialization handles the signature scheme as a 1-byte field:
to_canonical_bytes (line 287-293):
out.push(match self.owner.scheme {
    None => 0,
    Some(crate::signature::SignatureScheme::Secp256k1) => 1,
    Some(crate::signature::SignatureScheme::Ed25519) => 2,
    Some(crate::signature::SignatureScheme::MlDsa65) => 3,
});
from_canonical_bytes (line 375-381):
let scheme = match bytes[pos] {
    0 => None,
    1 => Some(crate::signature::SignatureScheme::Secp256k1),
    2 => Some(crate::signature::SignatureScheme::Ed25519),
    3 => Some(crate::signature::SignatureScheme::MlDsa65),
    _ => return Err(SanadError::InvalidEncoding),
};
The OwnershipProof struct (line 68-77) carries an Option<SignatureScheme> for the scheme field, which is used during Sanad::verify() (line 245-253) to call signature.verify(scheme).
2c. csv-core/Cargo.toml -- PQ Dependencies
File: /home/zorvan/Work/projects/csv-adapter/csv-core/Cargo.toml (61 lines)
There are no ML-DSA-65 / post-quantum dependencies in csv-core's Cargo.toml. The cryptographic dependencies are:
- secp256k1 = { version = "0.28", features = ["rand-std"] } -- for Secp256k1
- ed25519-dalek = { version = "2.0", features = ["rand_core"] } -- for Ed25519
- sha2 = { version = "0.10", default-features = false } -- for hashing
No ml-dsa, pqcrypto, or any lattice-based cryptography crate is present. The task_progress.md confirms: "Actual ML-DSA-65 cryptographic implementation" is STILL TODO (Phase 5).
2d. Search Results Summary
The grep for ml_dsa|MlDsa|pq|post_quantum returned 14 matches:
- task_progress.md: References to MlDsa65 variant and default impl
- docs/masterplan.md: Documentation of the MlDsa65 addition
- csv-core/src/sanad.rs: Lines 292 and 379 (scheme byte 3 mapping)
- csv-core/src/signature.rs: Lines 33, 40, 74 (MlDsa65 enum variant, default, and verify placeholder)
- The other matches are false positives (base64 alphabet strings containing "pq")

---
1. WASM Storage Gate
3a. csv-store with encrypted-storage Feature for wasm32-unknown-unknown
File: /home/zorvan/Work/projects/csv-adapter/csv-store/Cargo.toml (52 lines)
The encrypted-storage feature (line 52):
encrypted-storage = ["browser-storage", "dep:wasm-bindgen-futures", "dep:serde-wasm-bindgen", "dep:js-sys", "dep:aes-gcm", "dep:hmac", "dep:sha2", "dep:pbkdf2", "dep:zeroize", "dep:getrandom"]
It transitively enables browser-storage which depends on web-sys and wasm-bindgen. The getrandom dependency has features = ["js"] for WASM support.
File: /home/zorvan/Work/projects/csv-adapter/csv-store/src/lib.rs (742 lines)
The module gating:

- Line 14-15: #[cfg(feature = "browser-storage")] pub mod browser_storage;
- Line 17-18: #[cfg(feature = "encrypted-storage")] pub mod encrypted_storage;
- Line 29-30: #[cfg(all(not(target_arch = "wasm32"), feature = "file-storage"))] -- FileStorage is explicitly excluded from wasm32
File: /home/zorvan/Work/projects/csv-adapter/csv-store/src/encrypted_storage.rs (593 lines)
This file has extensive #[cfg(target_arch = "wasm32")] gating:
- Line 214-221: save is gated with #[cfg(target_arch = "wasm32")]
- Line 242-259: load is gated with #[cfg(target_arch = "wasm32")]
- Line 262-266: delete is gated with #[cfg(target_arch = "wasm32")]
- Lines 269-544: All IndexedDB operations (open_database, await_idb_request, set_indexeddb, get_indexeddb, delete_indexeddb, add_index_key, remove_index_key, migrate_local_storage_prefix, list_keys, load_all) are gated with #[cfg(target_arch = "wasm32")]
The EncryptedEnvelope struct and its encrypt/decrypt methods have #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] annotations, meaning the crypto operations compile on non-wasm32 targets but produce dead-code warnings.
Potential compile issue: The EncryptedStorageManager struct has a key: [u8; 32] field that is not gated. The Drop impl (line 547-550) calls self.key.zeroize() which requires zeroize to be available. The zeroize crate does support wasm32, so this should be fine.
However, there is a concern: The EncryptedEnvelope::encrypt method uses Aes256Gcm::generate_nonce(&mut OsRng) (line 128). The OsRng from aes-gcm uses getrandom under the hood, which has features = ["js"] when the encrypted-storage feature is enabled. This should work for wasm32-unknown-unknown since getrandom with the js feature uses the browser's crypto.getRandomValues().
3b. Verification Gate Status
The task_progress.md (line 33) lists this as an unchecked item:
- [ ] WASM Storage Gate: csv-store --no-default-features --features encrypted-storage must compile for wasm32
The code structure appears designed for wasm32 compilation, but the gate has not been verified with an actual cross-compilation test. The encrypted_storage.rs file has all its public methods gated behind #[cfg(target_arch = "wasm32")], and the lib.rs properly gates the file-storage backend out of wasm32. The dependency chain (getrandom with js feature, aes-gcm, pbkdf2, hmac, sha2, zeroize) should all compile for wasm32-unknown-unknown.

---
Summary Table
Area Component Status
WebSocket SubscriptionManager (csv-explorer) Implemented but address-based only, no per-chain awareness
WebSocket Wallet polling Stub only (seals/monitor.rs), no real polling
Adaptive Polling Per-chain config ChainConfig lacks poll_interval_ms; only global IndexerConfig.poll_interval_ms exists
Adaptive Polling +/- 20% jitter Not implemented
ML-DSA-65 SignatureScheme::MlDsa65 Enum variant exists, Default returns it
ML-DSA-65 Verification Placeholder returning error ("Phase 5 engineering task")
ML-DSA-65 Cargo deps No PQ crypto crate in csv-core
ML-DSA-65 Sanad scheme byte Byte 3 = MlDsa65 in serialization/deserialization
WASM Gate encrypted-storage feature Structurally prepared for wasm32, not yet verified by cross-compilation test
