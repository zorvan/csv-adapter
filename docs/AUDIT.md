# CSV Protocol — Production Audit

**Auditor**: Independent Review | **Date**: May 2026 | **Scope**: Full repo (repomix snapshot)
**Target scenario**: CLI wallet create → CSV wallet create → fund → Sanad create → multi-hop cross-chain transfer → explore

---

## AUDIT VERDICT SUMMARY

| Area | Status | Blockers |
|---|---|---|
| Key Storage — CLI | ✅ Fixed | AES-256-GCM + Argon2id encryption |
| Key Storage — Wallet | ✅ Adequate | Browser: AES-GCM + HMAC |
| Contract Deployment| 🔴 Broken | Contracts not deployed to testnet |
| P2P Proof Delivery | ✅ Fixed | Full Nostr implementation with retry |
| Cross-chain Transfer | ⚠️ Partial | Lock works, mint needs contracts |
| Offline Verification | ✅ Fixed | CLI command with explorer links |
| Explorer — Transactions | ⚠️ Schema ready | Indexer not live |
| CI / Production Gates | ✅ Fixed | Paths corrected, scans working |
| Test Coverage | ⚠️ Partial | Double-spend + WASM tests exist, no E2E |
| Masterplan alignment | ⚠️ Partially stale | 1 must-ship still open |

---

## PART 1 — SECURITY

### SEC-01 ✅ RESOLVED — CLI State Store Is Encrypted on Disk

**File**: `csv-cli/src/state.rs` → `UnifiedStateManager::save()` / `load()`  
**Encryption module**: `csv-cli/src/encrypt.rs`

**Fix applied**: AES-256-GCM + Argon2id key derivation applied to `UnifiedStateManager`. The `save()` method calls `encrypt::save()` which encrypts the JSON blob before writing. The `load()` method detects encrypted files via `encrypt::is_encrypted()`, parses the `EncryptedState` wrapper, and calls `encrypt::decrypt()` with the user's passphrase. Legacy plaintext files are loaded transparently and encrypted on the next save.

**Verification**: `encrypt.rs` uses `aes_gcm::Aes256Gcm` with Argon2id (64 MiB memory, 4 iterations, 4 lanes, 32-byte output). The state file is only ever written in encrypted form.

---

### SEC-02 ✅ RESOLVED — Ethereum Finality Bypass Fixed (SV-01b)

**File**: `csv-ethereum/src/ops.rs` → `verify_finality_proof` (lines 971-1016)

**Fix applied**: The `#[cfg(not(feature = "rpc"))]` block now returns `Err(ChainOpError::FeatureNotEnabled("rpc feature required for finality proof verification"))` instead of `Ok(true)`. This prevents the double-spend enablement path in non-RPC builds.

**Verification**: Lines 1009-1015 confirm the fix is in place. The non-RPC build now properly rejects finality proof verification attempts.

---

### SEC-03 ✅ RESOLVED — Nostr Identity Keys Persisted to Disk

**File**: `csv-p2p/src/nostr.rs` → `load_or_generate_nostr_keys()`

**Fix applied**: `NostrTransport::new()` and `NostrTransport::with_relays()` now call `load_or_generate_nostr_keys()` which:

1. Checks `~/.csv/nostr_secret_key.hex` — if it exists and is a valid 64-char hex (32 bytes), reconstructs the `Keys` object
2. If no valid key file exists, generates a new `Keys::generate()`, writes hex-encoded secret key to disk with `0o600` permissions
3. The `keys` field on `NostrTransport` is used for signing events (line 272: `event_builder.to_event(&self.keys)`)

Keys now survive across restarts, enabling pre-subscription by recipients and relay-side filtering.

---

### SEC-04 ✅ RESOLVED — Demo API Keys Removed from Chain Configs

**Files**: `chains/ethereum.toml`, `chains/solana.toml`, `chains/aptos.toml`

**Fix applied**: All hardcoded demo/public API keys have been removed from chain config files. RPC endpoints now reference public/free nodes (e.g., `ethereum.publicnode.com`, `api.mainnet-beta.solana.com`). No API keys, tokens, or credentials remain in any checked-in config. The `program_id` fields contain contract/program addresses (not API keys) — two use placeholder patterns (`0x1234...`) which are stubs for contract addresses.

---

### SEC-05 ✅ RESOLVED — File Permission Enforcement on Keystore Dir

**File**: `csv-keys/src/file_keystore.rs` (lines 232-237, 258-262)

**Fix applied**: The keystore directory is now created with `0o700` permissions using `std::fs::set_permissions()` on Unix systems. Both `new()` and `with_dir()` constructors apply these restrictive permissions.

**Verification**: Lines 234-237 and 259-262 confirm the directory is created with `std::fs::Permissions::from_mode(0o700)`.

---

### SEC-06 ✅ RESOLVED — Passphrase Minimum Length Raised to 12 Characters

**File**: `csv-cli/src/commands/wallet/generate.rs` (lines 31-34)

**Fix applied**: The passphrase minimum has been raised from 8 to 12 characters. The prompt now explicitly states "min 12 chars" and the validation enforces this requirement.

**Verification**: Lines 31-34 confirm the check: `if passphrase.len() < 12 { anyhow::bail!("Passphrase must be at least 12 characters"); }`

---

### SEC-07 ✅ RESOLVED — MCP Server Input Validation Added

**File**: `csv-mcp-server/src/index.ts` → validation module

**Fix applied**: Added a validation module with strict allowlists for all user-provided parameters passed to `executeCsvCommand()`:

- `validateSanadId(id)` — must be a 64-character hex string (32 bytes)
- `validateTransferId(id)` — must be a 64-character hex string
- `validateChain(chain)` — must be one of the allowed chain enums
- `validateAddress(address)` — must be a valid hex address (0x-prefixed, 20-66 chars)
- `validateHexId(id)` — generic hex ID validator (alphanumeric + 0x prefix)
- `validateJsonString(json)` — must parse as valid JSON
- `validatePositiveNumber(n)` — must be a positive number
- `validateConsignment(json)` — must parse as valid JSON object

All tool handlers now call the appropriate validator before constructing CLI commands. Invalid input returns a structured error response without invoking the CLI.

---

## PART 2 — ARCHITECTURE GAPS

### ARCH-01 🔴 BLOCKING — Ethereum Contract Deployment

**What's needed**:
In CSV-CONTRACTS Scripts (for all chains)
So both csv-cli and csv-wallet should be able to get contract addresses for each chain to create and send Sanads.

- Compile and deploy `CSVLock.sol` + `CSVMint.sol` to Sepolia using foundry
- Send deployed addresses in `chains/ethereum.toml` under `[testnet]`
- Send deployed address into `lock_contract_address` field on `EthereumBackend`

---

### ARCH-02 ✅ RESOLVED — P2P Proof Delivery Now Fully Implemented

**File**: `csv-p2p/src/nostr.rs`

**Fix applied**: The `NostrTransport` implementation is now complete:

- `broadcast_proof()` (lines 470-565) properly wires `nostr_sdk::Client::send_event()` with `EventBuilder::new(Kind::Custom(30345), ...)` and event signing
- `subscribe_proofs()` (lines 573-699) parses incoming events from `RelayPoolNotification::Event` into `DeliveredProof` with full `ProofBundle` deserialization
- `extract_chain_ids_from_tags()` (lines 92-103) correctly extracts chain IDs from event tags
- Retry logic with exponential backoff is implemented (lines 516-542)
- Relay health monitoring is available via `check_relay_health()` and `start_health_monitor()` (lines 300-339)
- Nostr keypair persistence is implemented (see SEC-03 ✅)

**Verification**: All required functionality from the audit's "What's needed" list is now implemented and functional.

---

### ARCH-03 ✅ PARTIALLY RESOLVED — Transfer Now Calls Real `lock_sanad()` on Source Chain

**Files**: `csv-sdk/src/transfers.rs`, `csv-cli/src/commands/cross_chain/transfer.rs`

**What was done**:

- `TransferManager` now holds `Arc<ChainRuntime>` (passed from `CsvClient::transfers()`)
- `TransferBuilder::execute()` is now `async` and calls `runtime.lock_sanad()` on the source chain
- Lock result (`SanadOperationResult`) is captured and stored in `TransferRecord.lock_tx_hash`
- Transfer status transitions to `Locking { current_confirmations, required_confirmations }`
- CLI `cmd_transfer` is now async and calls `execute().await`
- `TransferRecord` struct has new `lock_tx_hash: Option<String>` field

**Remaining work**:

1. Steps 2-3 (poll finality, build inclusion proof) — not yet wired into execute()
2. Step 4 (P2P proof delivery via Nostr) — requires ARCH-02 fix
3. Step 5 (destination chain mint) — not yet wired into execute()
4. Solana/Sui/Aptos backends still have stub `lock_sanad()` returning `CapabilityUnavailable`

**Current flow**:

```
CLI cmd_transfer() → client.transfers().cross_chain().execute().await
  → runtime.lock_sanad(from_chain, sanad_id, to_chain, owner_key_id)
  → backend-specific lock_sanad() (Bitcoin: real, Ethereum: real with rpc feature, others: stub)
  → TransferRecord updated with lock_tx_hash and Locking status
```

---

### ARCH-04 ✅ RESOLVED — CLI Offline Verification Now Exposed

**File**: `csv-cli/src/commands/validate.rs` → `cmd_offline()` (lines 170-295)

**Fix applied**: Enhanced the existing `csv validate offline --file <proof.json>` command to:

- Call `csv_core::verifier::verify_proof()` for full cryptographic verification
- Output verification result (valid/invalid)
- Display chain of commits and proofs via DAG structure
- Generate and output external explorer links for: seal ID, anchor block, inclusion proof, and destination chain
- Use CLI state to check if seal has been consumed (replay protection)

**Verification**: Lines 207-234 perform full cryptographic verification using the verifier pipeline. Lines 236-243 generate and display explorer links for all relevant components.

---

### ARCH-05 ⚠️ HIGH — Explorer Has Schema but No Running Indexer for Demo

**Directory**: `csv-explorer/`

The Explorer has complete SQL schema, REST API, GraphQL, and UI. But for demo step 5 ("csv-explorer list all transactions with links to source chains"), the indexer must be running against actual testnet nodes. Currently:

- `config.testnet.toml` has placeholder RPC endpoints
- Block explorer links (`blockstream.info`, `suiexplorer.com`, etc.) require real tx hashes from real chains
- The `wallet_bridge.rs` priority indexing works but needs the wallet to register addresses via the bridge API

**What's needed**:

1. Deploy explorer with testnet config to a public URL (as masterplan requires before Stage 1)
2. Wire testnet RPC endpoints in `config.testnet.toml`
3. Add WebSocket push for transfer status updates (wired in schema but the ws handler at `csv-explorer/api/src/websocket.rs` needs the subscription feed)

---

### ARCH-06 ⚠️ HIGH — Transfer Record Chain Explorer Links Missing

**File**: `csv-explorer/shared/src/types.rs` → `TransferRecord`

`lock_tx` and `mint_tx` fields exist but are not populated with block explorer URLs in the REST response. The UI `transfers.rs` page shows tx hashes but doesn't build clickable links.

**What's needed**: In `csv-explorer/api/src/rest/handlers.rs` → `get_transfer()`, append block explorer URL based on chain: `format!("https://blockstream.info/testnet/tx/{}", lock_tx)` for Bitcoin, etc. These are the "links to source chains" required by demo step 5.

---

### ARCH-07 ✅ RESOLVED — Native Keystore Wired into Key Manager

**File**: `csv-wallet/src/core/key_manager.rs` / `csv-wallet/src/core/native_keystore.rs`

**Fix applied**: `NativeKeystore` is fully wired into `KeyManager` under `#[cfg(not(target_arch = "wasm32"))]`:

- `NativeKeystore` imported and used as `Option<NativeKeystore>` field in `KeyManager` struct
- `new_with_keystore()` constructor initializes `Some(NativeKeystore::new())`
- `store_key_in_keystore()`, `retrieve_key_from_keystore()`, `has_keystore()`, `list_keystore_keys()`, `delete_key_from_keystore()` all delegate to the `NativeKeystore` with proper error mapping
- `native_keystore.rs` wraps `FileKeystore` with security policy, failed-attempts tracking, and session management

**Note**: `KeyManager::new()` still initializes `keystore: None` by default — consumers must explicitly call `new_with_keystore()` to enable persistent storage. This is intentional.

---

### ARCH-08 ✅ RESOLVED — Sanad Commitment Now Chain-Anchored via publish_seal()

**Files**: `csv-core/src/backend.rs`, `csv-sdk/src/runtime.rs`, `csv-cli/src/commands/sanads.rs`

**Implementation**:

- Added `publish_seal()` method to `ChainBackend` trait (default: `CapabilityUnavailable`)
- All 5 backends (`BitcoinBackend`, `EthereumBackend`, `SolanaBackend`, `SuiBackend`, `AptosBackend`) now store `Arc<SealProtocol>` and implement `ChainBackend::publish_seal()`
- `publish_seal()` delegates to the backend's internal `SealProtocol::publish()` which submits the commitment to the chain
- CLI `cmd_create()` now calls `runtime.create_seal()` → `runtime.publish_seal()` → `client.sanads().create()` in sequence
- `SanadRecord.anchor_tx_hash` is populated with hex-encoded `CommitAnchor::anchor_id`
- `SanadRecord.seal_ref` is populated with base64-encoded `SealPoint::to_vec()`

**SealPoint.id encoding by chain**:

- Bitcoin: `[txid: 32 bytes][vout: 4 bytes]` (36 bytes total)
- Ethereum: `[contract_address: 20 bytes][slot_index: 8 bytes]` (28 bytes, padded to 32)
- Solana: `[account: 32 bytes Pubkey]` (32 bytes, nonce=None)
- Sui: `[object_id: 32 bytes]` (32 bytes)
- Aptos: `[account_address: 32 bytes]` (32 bytes)

---

## PART 3 — CI / TESTING

### TEST-01 ✅ RESOLVED — Production Guarantee CI Paths Fixed

**File**: `.github/workflows/production-guarantee.yml`

**Fix applied**: All path references have been corrected to use the actual directory structure:

- Lines 30-33: `csv-core/src csv-bitcoin/src csv-ethereum/src csv-sui/src csv-aptos/src csv-solana/src csv-wallet/src csv-cli/src`
- Lines 47-50: Same corrected paths
- Lines 66-69: Same corrected paths
- Lines 82-84: Corrected paths for key handling checks

**Verification**: The workflow now scans the correct directories and will properly detect TODOs, stubs, fake data, and other production surface issues.

---

### TEST-02 🔴 CRITICAL — No End-to-End Test for Demo Scenario

**File**: `csv-cli/src/commands/tests.rs`

`cmd_run()` exists but performs no actual chain operations. It prints status messages and updates local state. There is no automated test that:

1. Creates a CLI wallet
2. Creates a CSV wallet
3. Deploy Contracts with Deployment scripts and get deployment address and feed it to the wallets (or chose it from a admin acount list?).
4. Creates a Sanad
5. Transfers it across chains
6. Verifies the transfer is visible in the explorer

**Fix required**: Add an integration test suite (gated by `--features integration-tests`) using testnet:

```
tests/integration/
  scenario_full_transfer.rs   # Steps 1-5 above
  scenario_offline_verify.rs  # File → verify → result
  scenario_wallet_roundtrip.rs # CLI wallet ↔ CSV wallet
```

---

### TEST-03 ⚠️ HIGH — Nextest Timeout Too Aggressive for Cryptographic Tests

**File**: `.config/nextest.toml`

```toml
slow-timeout = { period = "6s", terminate-after = "1" }
```

Tests running longer than 6 seconds are killed. Argon2id key derivation, Merkle proof generation, and RPC-backed tests regularly exceed this. The result is false test failures that hide real bugs.

**Fix**: Raise to `period = "30s", terminate-after = "3"` for the default profile. Add a `[profile.crypto]` profile with 120s for key derivation and ZK-related tests.

---

### TEST-04 ✅ RESOLVED — Seal Double-Spend Test Exists

**File**: `csv-core/src/verifier.rs` → `test_seal_double_spend_regression` (lines 577-617)

**Status**: Test already implemented and covers:

- Creating a proof bundle with a seal
- Simulating a seal registry that tracks consumed seals
- First verification succeeds (seal not consumed)
- Marking seal as consumed
- Second verification with same seal fails (double-spend rejected)
- Error message indicates seal replay/consumption

**Verification**: The test verifies the most critical invariant of the protocol - that the same seal cannot be used in multiple proof bundles.

---

### TEST-05 ✅ RESOLVED — WASM Chain ID Regression Tests Exist

**File**: `typescript-sdk/src/chains.test.ts` (lines 1-109)

**Status**: Comprehensive test suite already implemented covering:

- All chain IDs parse correctly (bitcoin, ethereum, sui, aptos, solana)
- Chain ID parsing is case-insensitive
- Invalid chain IDs are rejected
- Chain IDs convert to display strings correctly
- Chain ID serialization round-trip is consistent
- Chain IDs are within reasonable size limits (< 32 bytes)
- Chain IDs are ASCII-compatible

**Verification**: The test suite prevents regression of the WASM chain_id bug by ensuring chain identifiers remain valid when crossing the Rust-WASM boundary.

---

## PART 4 — MASTERPLAN VALIDATION

Cross-checking each "Must-Ship Before Demo" item against actual code:

| Item | Masterplan Says | Code Reality | Gap |
|---|---|---|---|
| SV-01b — ETH finality bypass | 30 min fix | **Not fixed** | Fix `ops.rs` |
| ETH contract deployment | 3–5 days | **Not started** | `CapabilityUnavailable` in `backend.rs` |
| P2P Nostr delivery | 1 week | **Skeleton only** | Event signing/parsing incomplete |
| Offline verification UX | 1 week | **Wallet: done. CLI: missing** | Add CLI command |
| Desktop filesystem keystore | 2–3 days | **✅ Done** | `key_manager.rs` delegates to `NativeKeystore` |
| MCP server — 7 tools + validation | 1–2 weeks | **✅ Done** | All tools implemented with input validation |
| TypeScript SDK npm publish | Needs SV-04 fix | **Not published** | Fix WASM bug first |

**Masterplan items that have been completed since writing**:

- AES-256-GCM encrypted browser storage (was a gap, now done in `encrypted_storage.rs`)  
- Native keystore wired into key_manager.rs (ARCH-07, now complete)
- Explorer schema and API (complete, needs deployment)
- MCP server with 7 tools + input validation (SEC-07, now complete)
- CLI state file encryption (SEC-01, now complete)
- Nostr ephemeral keys persistence (SEC-03, now complete)

**Masterplan items that are more outdated**:

- "Estimate 2–3 weeks to demo-ready" — with Ethereum blocked and P2P incomplete, more realistic: 4–6 weeks
- The "film the offline verification demo" assumes WiFi-off QR scan path works — it does for the wallet but the full cross-chain proof bundle needed for that demo requires the complete transfer pipeline

---

## PART 5 — WIRING CHECKLIST FOR DEMO SCENARIO

The exact 5-step scenario: CLI create wallet → CSV wallet create → fund → Sanad create → multi-hop transfer → explorer view.

### Step 1: `csv wallet generate` (CLI)

- [x] Mnemonic generation (BIP-39, 12/24 words)
- [x] HD key derivation per chain (BIP-44)
- [x] AES-256-GCM encrypted keystore files
- [x] Addresses displayed and saved to config
- [x] State file encrypted with AES-256-GCM + Argon2id (SEC-01 ✅)
- [ ] Keystore dir permissions `0700` (SEC-05)

### Step 2: CSV Wallet — Create New Wallet

- [x] Onboarding flow in wallet UI
- [x] File keystore on native, browser keystore on WASM
- [x] BIP-39 mnemonic display and confirmation
- [x] Native keystore wired into key_manager.rs (ARCH-07 ✅)
- [ ] Passphrase minimum entropy (SEC-06)

### Step 3: Fund Both Wallets

- [x] Balance display in wallet UI
- [x] `csv wallet balance` CLI command  
- [x] Hardcoded demo API keys removed, env vars required (SEC-04 ✅)
- [ ] Balance must reject silent-zero on RPC failure (chain_api.rs has the error type, verify it propagates)

### Step 4: Create Sanad and Transfer Multi-Hop

- [x] Sanad create UI (wallet) and CLI  
- [x] Sanad commitment must be on-chain anchored (ARCH-08 ✅)
- [x] `csv cross-chain transfer` now calls runtime.lock_sanad() on source chain (ARCH-03 ✅ partial)
- [ ] P2P proof delivery must be functional (ARCH-02)
- [ ] Ethereum must be deployable if ETH chain involved (ARCH-01)
- [ ] Transfer state must persist correctly across both wallets
- [ ] Recipient wallet must receive proof via Nostr subscription
- [x] Transfer status page in wallet UI (wired to state)
- [x] CLI `csv cross-chain status` command exists

### Step 5: Explorer — List Transactions with Chain Links

- [x] SQL schema for transfers, sanads, seals
- [x] REST API `/api/v1/transfers` with filters
- [x] Explorer UI transfers page with pagination  
- [ ] Indexer must be running against testnet (ARCH-05)
- [ ] Block explorer links must be populated (ARCH-06)
- [ ] WebSocket push for live status updates needs wiring
- [ ] Explorer must be deployed at public URL

---

## PART 6 — GUARDRAILS AND ENGINEERING RULES

These must be enforced before any code reaches `main`:

### Cryptographic Guardrails

- **No `Ok(true)` in verification paths.** Every verification path must return `Err` if data is unavailable, not a passing result.
- **No mock signatures in production code.** The production guarantee CI (once paths are fixed) must catch `fake_sig`, `mock_proof`, `[0u8; 64]` patterns in non-test code.
- **Seal registry check is mandatory.** `verify_proof()` must always call the seal registry callback. Never skip it with `//todo`.
- **Empty proof bundles are rejected.** Zero-length `inclusion_proof` bytes must fail, not pass.

### Key Management Guardrails

- **No raw private key bytes in logs.** Add a lint rule: `grep -r "private_key\|secret_key\|signing_key" --include="*.rs" | grep "println\|info!\|debug!"` must return zero results.
- **Zeroize on drop.** Any type holding a private key must implement `Zeroize` + `ZeroizeOnDrop`. Verify with `#[derive(ZeroizeOnDrop)]`.
- **No key material in state files.** `unified_storage.json` must never contain fields with names matching `key|secret|mnemonic|seed|private`.

### Chain Integration Guardrails

- **Testnet by default.** Any new chain integration must default to testnet in config. Mainnet requires explicit `--network mainnet` flag.
- **RPC endpoints via env vars only.** No hardcoded endpoints in any config file checked into the repo.
- **Finality depth is never zero.** `min_confirmations = 0` must fail at config parse time.

### State Machine Guardrails

- **Transfer status is append-only.** A transfer record cannot go from `Completed` back to `Pending`. Add validation in `UnifiedStateManager::update_transfer()`.
- **Sanad status is monotonic.** `Active → Transferred → Consumed`. Reverse transitions must panic/error.
- **Double-consume fails loudly.** Attempting to consume an already-consumed Sanad must return a specific error, not silently succeed or update the record.

### CI Guardrails (after fixing TEST-01)

- **Production guarantee gates must pass on every PR.** No merges to `main` with failing gates.
- **`cargo audit` on every push.** Already in CI — keep it.
- **`cargo clippy -- -D warnings` blocks merge.** Already in CI — keep it.
- **No `unwrap()` in production paths.** Add clippy lint `#![deny(clippy::unwrap_used)]` to `csv-core/src/lib.rs`, `csv-keys/src/lib.rs`.

---

## PART 7 — PRIORITY WORK ORDER

To reach the 5-step demo scenario as fast as possible:

**Week 1 (unblock the chain)**

1. Fix SV-01b (30 min) — `csv-ethereum/src/ops.rs`
2. ~~Wire `native_keystore.rs` into `key_manager.rs`~~ (✅ Done — ARCH-07)
3. ~~Encrypt CLI state file~~ (✅ Done — SEC-01)
4. Fix production guarantee CI paths (2 hours) — `production-guarantee.yml`
5. Fix keystore dir permissions (2 hours)

**Week 2 (make proof delivery real)**
6. ~~Persist Nostr identity keypair~~ (✅ Done — SEC-03)
7. Wire `nostr_sdk` event publish/subscribe in `nostr.rs` (3 days)
8. Connect P2P delivery into transfer manager (1 day)

**Week 3 (make transfers real)**
9. ~~Anchor Sanad commitment to actual chain tx~~ (✅ Done — ARCH-08)
10. ~~Wire real lock tx in `cmd_transfer` and `TransferManager`~~ (✅ Done — ARCH-03 partial)
11. Add CLI `csv validate offline` command (0.5 day)

**Week 4 (Ethereum + Explorer)**
12. Deploy CSVLock.sol to Sepolia, implement `deploy_lock_contract()` (4 days)
13. Deploy explorer to public testnet URL (1 day)
14. Populate block explorer links in REST response (0.5 day)
15. ~~Remove demo keys~~ (✅ Done — SEC-04)

**Week 5 (test coverage)**
16. Write integration test for 5-step demo scenario (3 days)
17. Add seal double-spend regression test (0.5 day)
18. Add WASM chain_id regression test (0.5 day)
19. Tune nextest timeouts (1 hour)

**Total honest estimate to demo-ready: 4–5 weeks** (vs. 2–3 in masterplan, which did not account for Ethereum deployment being fully absent and P2P being a skeleton).

---

*Audit scope: repomix-output.xml snapshot, MASTERPLAN_v2.md, RGB Documentation reference (client-side validation lineage). Audit does not cover deployed smart contract security (separate Solidity audit required before mainnet), ZK circuit security (not yet implemented), or supply chain dependency risk (addressed by existing `cargo audit` in CI).*
