# CSV Protocol — Production Audit
**Auditor**: Independent Review | **Date**: May 2026 | **Scope**: Full repo (repomix snapshot)
**Target scenario**: CLI wallet create → CSV wallet create → fund → Sanad create → multi-hop cross-chain transfer → explore

---

## AUDIT VERDICT SUMMARY

| Area | Status | Blockers |
|---|---|---|
| Key Storage — CLI | ⚠️ Partial | State file unencrypted |
| Key Storage — Wallet | ✅ Adequate | Browser: AES-GCM + HMAC |
| Contract Deployment| 🔴 Broken | wallets and chain specific private-key formats|
| P2P Proof Delivery | 🔴 Broken | Nostr publish/subscribe stubs |
| Cross-chain Transfer | 🔴 Simulated | Not real chain state |
| Offline Verification | ⚠️ Wired (wallet only) | Not reachable from CLI |
| Explorer — Transactions | ⚠️ Schema ready | Indexer not live |
| CI / Production Gates | 🔴 Wrong paths | Scans nonexistent dirs |
| Test Coverage | 🔴 None end-to-end | No scenario test exists |
| Masterplan alignment | ⚠️ Partially stale | 3 must-ships still open |

---

## PART 1 — SECURITY

### SEC-01 ✅ RESOLVED — CLI State Store Is Encrypted on Disk

**File**: `csv-cli/src/state.rs` → `UnifiedStateManager::save()` / `load()`  
**Encryption module**: `csv-cli/src/encrypt.rs`

**Fix applied**: AES-256-GCM + Argon2id key derivation applied to `UnifiedStateManager`. The `save()` method calls `encrypt::save()` which encrypts the JSON blob before writing. The `load()` method detects encrypted files via `encrypt::is_encrypted()`, parses the `EncryptedState` wrapper, and calls `encrypt::decrypt()` with the user's passphrase. Legacy plaintext files are loaded transparently and encrypted on the next save.

**Verification**: `encrypt.rs` uses `aes_gcm::Aes256Gcm` with Argon2id (64 MiB memory, 4 iterations, 4 lanes, 32-byte output). The state file is only ever written in encrypted form.

---

### SEC-02 🔴 CRITICAL — Ethereum Finality Bypass Still in Production Code (SV-01b)

**File**: `csv-ethereum/src/ops.rs` → `verify_finality_proof`

The masterplan correctly identifies this. It has not been fixed. The `#[cfg(not(feature = "rpc"))]` block returns `Ok(true)` unconditionally, meaning any finality proof passes without validation in non-RPC builds. This is a **double-spend enablement path**.

**Fix**: 30 minutes. Change `Ok(true)` to `Err(ChainOpError::FeatureNotEnabled("rpc feature required for finality verification"))`. Add a compile-time assert or CI check that this branch is never active in release builds.

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

### SEC-05 ⚠️ HIGH — Missing File Permission Enforcement on Keystore Dir

**File**: `csv-keys/src/file_keystore.rs`

Directory `~/.csv/keystore/` is created via `std::fs::create_dir_all()` with no explicit mode set. On Linux/macOS this defaults to `0o755` (world-readable). Any process on the system can read the encrypted keystore files. While the files themselves are encrypted, their existence, naming convention, and timestamps leak information.

**Fix**: Use `std::os::unix::fs::DirBuilderExt` to set `0o700`. Apply `chmod 600` to each keystore JSON file after creation.

---

### SEC-06 ⚠️ MEDIUM — Passphrase Minimum Length is 8 Characters

**File**: `csv-cli/src/commands/wallet/generate.rs`

```rust
if passphrase.len() < 8 {
    anyhow::bail!("Passphrase must be at least 8 characters");
}
```

8 characters is the 1990s standard. AES-256-GCM with Argon2id KDF compensates somewhat, but the usability message anchors users to weak passphrases. Industry standard for crypto wallets is 12+ characters with complexity requirements, or a diceware phrase.

**Fix**: Raise to 12 minimum. Add entropy estimation (zxcvbn) with a warning (not a block) for low-entropy passes. Recommend diceware in the UX copy.

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

### ARCH-01 🔴 BLOCKING — Ethereum Contract 

**What's needed**:
**
In CSV-CONTRACTS Scripts (for all chains)
So both csv-cli and csv-wallet should be able to get contract addresses for each chain to create and send Sanads.

- Compile and deploy `CSVLock.sol` + `CSVMint.sol` to Sepolia usinf foundry
- Send deployed addresses in `chains/ethereum.toml` under `[testnet]`
- send deployed address into `lock_contract_address` field on `EthereumBackend`

---

### ARCH-02 🔴 BLOCKING — P2P Proof Delivery Is Structurally Incomplete

**File**: `csv-p2p/src/nostr.rs`

`NostrTransport` struct exists with relay connections and key management. `ProofTransport` trait is defined. But:
- `publish()` has placeholder logic — event signing via `nostr_sdk` API is not correctly wired
- `subscribe()` notification loop uses `RelayPoolNotification` type but the actual event parsing into `DeliveredProof` is empty
- `extract_chain_ids_from_tags()` returns empty vec
- No retry on relay failures; no relay health monitoring

**Impact**: Cross-chain proof delivery cannot complete. Both wallets can create Sanads, but the receiving wallet cannot get the proof bundle to finalize the mint.

**What's needed** (1 week estimate from masterplan is correct):
1. Wire `nostr_sdk::Client::publish_event()` with `EventBuilder::new(Kind::Custom(30345), proof_json)`
2. Parse incoming events in subscription loop: `RelayPoolNotification::Event(_, event)` → deserialize `ProofBundle` from `event.content`
3. Filter on `event.kind == 30345` and tag `["chain_id", ...]`
4. Persist Nostr keypair (see SEC-03)
5. Add relay failover: try next relay if primary times out

---

### ARCH-03 🔴 BLOCKING — Cross-Chain Transfer Is Client-Side Simulation Only

**Files**: `csv-cli/src/commands/cross_chain/transfer.rs`, `csv-sdk/src/transfers.rs`

CLI `cmd_transfer` creates a `CsvClient`, calls `client.transfers()`, then has a comment: `// In a full implementation...` and returns success after updating local state. No actual lock transaction is submitted to any chain. The `TransferManager` in `csv-sdk/src/transfers.rs` similarly has the protocol documented but the actual lock→prove→verify→claim pipeline is not executed.

**The demo scenario requires**:
1. Source chain `seal_consume()` → lock tx on-chain
2. Wait for finality (N confirmations)  
3. Build inclusion proof from the lock tx
4. Deliver proof bundle via P2P (Nostr)
5. Destination chain verify proof → mint new Sanad

Steps 1, 4, and 5 are partially wired. Steps 2 and 3 need connecting. Step 4 requires ARCH-02 fix.

**What's needed**: In `TransferManager::initiate()`, after building the lock tx, actually call `rpc.send_raw_transaction()` on the source chain backend, poll for finality, then call `proof_provider.build_inclusion_proof()`, then hand off to P2P delivery.

---

### ARCH-04 ⚠️ HIGH — CLI Offline Verification Not Exposed

The wallet has a full offline verification page (`csv-wallet/src/pages/validate/offline.rs`) — file upload, JSON paste, crypto verification, result display. The CLI has no equivalent command.

**Impact on demo scenario step 4**: "All transfers are traceable both on wallet and CLI" — CLI cannot verify proofs from files.

**What's needed**: Add `csv validate offline --file <proof.json>` command in `csv-cli/src/commands/validate.rs` that calls `csv_core::verifier::verify_proof()` and prints the result table.

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

### ARCH-08 ⚠️ MEDIUM — Sanad Commitment Is Synthetic Hash, Not Chain-Anchored

**File**: `csv-cli/src/commands/sanads.rs` → `cmd_create()`

```rust
let commitment_bytes: [u8; 32] = {
    let mut hasher = Sha256::new();
    hasher.update(b"commitment-");
    hasher.update(chain.to_string().as_bytes());
    // ... not anchored to any on-chain transaction
};
```

The Sanad commitment is computed locally and stored in state. It is never submitted to the source chain as an anchor transaction. A real Sanad must have an on-chain anchor (UTXO for Bitcoin, object for Sui, event for Ethereum) that proves existence at a specific block.

**Impact**: Sanad IDs generated by CLI cannot be verified by a third party or the explorer. The transfer will fail at proof generation because there is no real inclusion proof to build.

---

## PART 3 — CI / TESTING

### TEST-01 🔴 CRITICAL — Production Guarantee CI Scans Nonexistent Directories

**File**: `.github/workflows/production-guarantee.yml`

```yaml
rg ... csv-adapter/src csv-adapter-core/src csv-adapter-bitcoin/src \
    csv-adapter-ethereum/src csv-adapter-sui/src ...
```

These directories do not exist. The repo uses `csv-core/`, `csv-bitcoin/`, `csv-ethereum/` etc. The production guarantee jobs that check for TODOs, stubs, fake data, and hardcoded keys are **completely inoperative**. They silently succeed because `rg` finds nothing in paths that don't exist.

**Fix**: Update all path references in `production-guarantee.yml`:
```yaml
csv-core/src csv-bitcoin/src csv-ethereum/src csv-sui/src \
csv-aptos/src csv-solana/src csv-wallet/src csv-cli/src csv-sdk/src
```

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

### TEST-04 ⚠️ HIGH — No Seal Double-Spend Test

No test verifies that consuming a Sanad's seal prevents a second consumption. This is the most critical invariant of the entire protocol.

**Fix**: Add a unit test in `csv-core/src/verifier.rs`:
```rust
#[test]
fn test_seal_double_spend_rejected() {
    let proof = build_test_proof();
    let mut registry = MockSealRegistry::new();
    assert!(verify_proof(&proof, &mut registry).is_ok());
    assert!(verify_proof(&proof, &mut registry).is_err()); // second consume must fail
}
```

---

### TEST-05 ⚠️ MEDIUM — WASM Chain ID Bug (SV-04) Has No Regression Test

The TypeScript SDK WASM build has a `chain_id` bug marked "in progress." There is no test that would catch a regression after the fix.

**Fix**: Add Jest test in `typescript-sdk/`:
```typescript
test('chain_id survives WASM round-trip', async () => {
    const sdk = await loadWasm();
    expect(sdk.chain_id_roundtrip('bitcoin')).toBe('bitcoin');
    expect(sdk.chain_id_roundtrip('ethereum')).toBe('ethereum');
});
```

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
- [ ] Sanad commitment must be on-chain anchored (ARCH-08)
- [ ] `csv cross-chain transfer` must execute real lock tx (ARCH-03)
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
9. Anchor Sanad commitment to actual chain tx (2 days) — ARCH-08
10. Wire real lock tx in `cmd_transfer` and `TransferManager` (3 days)
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
