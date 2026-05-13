
# CSV Protocol — Production Audit

**Auditor**: Independent Review | **Date**: May 2026 | **Scope**: Full repo (repomix snapshot)
**Target scenario**: CLI wallet create → CSV wallet create → fund → Sanad create → multi-hop cross-chain transfer → explore

---

## AUDIT VERDICT SUMMARY

| Area | Status | Blockers |
|---|---|---|
| Contract Deployment| 🔴 Broken | Contracts not deployed to testnet |
| Cross-chain Transfer | ⚠️ Partial | Lock works, mint needs contracts |
| Explorer — Transactions | ⚠️ Schema ready | Indexer not live |
| Test Coverage | ⚠️ Partial | Double-spend + WASM tests exist, no E2E |
| Masterplan alignment | ⚠️ Partially stale | 1 must-ship still open |

---

## PART 1 — SECURITY

*All security items from the May 2026 audit have been resolved.*

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

```text
CLI cmd_transfer() → client.transfers().cross_chain().execute().await
  → runtime.lock_sanad(from_chain, sanad_id, to_chain, owner_key_id)
  → backend-specific lock_sanad() (Bitcoin: real, Ethereum: real with rpc feature, others: stub)
  → TransferRecord updated with lock_tx_hash and Locking status
```

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

## PART 3 — CI / TESTING

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

## PART 4 — MASTERPLAN VALIDATION

Cross-checking each "Must-Ship Before Demo" item against actual code:

| Item | Masterplan Says | Code Reality | Gap |
|---|---|---|---|
| ETH contract deployment | 3–5 days | **Not started** | `CapabilityUnavailable` in `backend.rs` |
| Transfer pipeline completion | 1 week | **Complete** | Steps 1-5 wired; P2P proof delivery via Nostr |
| Explorer deployment | 1 day | **Not started** | Indexer not live |
| Block explorer links | 0.5 day | **Not started** | Links not populated |
| End-to-end integration test | 3 days | **Not started** | No automated test scenario |

**Items completed since original audit**:

- SV-01b Ethereum finality bypass fix
- CLI state file encryption (AES-256-GCM + Argon2id)
- Nostr identity key persistence
- Demo API keys removed from configs
- Keystore directory permissions (0o700)
- Passphrase minimum length (12 chars)
- MCP server with 7 tools + input validation
- P2P proof delivery (full Nostr implementation)
- CLI offline verification with explorer links
- Native keystore wired into key_manager
- Sanad commitment chain anchoring (publish_seal)
- CI production guarantee paths fixed
- Seal double-spend regression test
- WASM chain ID regression tests
- Keccak256 bug fix in sanad_contract.rs
- TransferStatus enum unification
- Recovery engine DB integration (steps 4-7)
- Reorg rollback.rs real implementation
- Reorg reconciliation.rs real implementation
- Quorum client integrated into Ethereum adapter
- ABI migration to generated Alloy bindings

---

## PART 5 — WIRING CHECKLIST FOR DEMO SCENARIO

The exact 5-step scenario: CLI create wallet → CSV wallet create → fund → Sanad create → multi-hop transfer → explorer view.

### Step 1: `csv wallet generate` (CLI)

Keystore dir permissions `0700` (SEC-05) — implemented in native_keystore.rs SecurityPolicy.

### Step 2: CSV Wallet — Create New Wallet

Passphrase minimum entropy (SEC-06) — enforced via SecurityPolicy min_passphrase_length (12 chars).

### Step 3: Fund Both Wallets

Balance must reject silent-zero on RPC failure (chain_api.rs has the error type, verify it propagates).

### Step 4: Create Sanad and Transfer Multi-Hop

`csv cross-chain transfer` now calls runtime.lock_sanad() on source chain (ARCH-03 partial).

- [ ] P2P proof delivery must be functional (ARCH-02)
- [ ] Ethereum must be deployable if ETH chain involved (ARCH-01)
- [ ] Transfer state must persist correctly across both wallets
- [ ] Recipient wallet must receive proof via Nostr subscription

### Step 5: Explorer — List Transactions with Chain Links

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
2. Fix production guarantee CI paths (2 hours) — `production-guarantee.yml`
3. Fix keystore dir permissions (2 hours)

**Week 2 (make proof delivery real)**
7. Wire `nostr_sdk` event publish/subscribe in `nostr.rs` (3 days)
8. Connect P2P delivery into transfer manager (1 day)

**Week 3 (make transfers real)**
11. Add CLI `csv validate offline` command (0.5 day)

**Week 4 (Ethereum + Explorer)**
12. Deploy CSVLock.sol to Sepolia, implement `deploy_lock_contract()` (4 days)
13. Deploy explorer to public testnet URL (1 day)
14. Populate block explorer links in REST response (0.5 day)

**Week 5 (test coverage)**
16. Write integration test for 5-step demo scenario (3 days)
17. Add seal double-spend regression test (0.5 day)
18. Add WASM chain_id regression test (0.5 day)
19. Tune nextest timeouts (1 hour)
