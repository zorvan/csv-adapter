---

## Architecture Overview

**Stack:** Rust workspace — `shared` / `storage` / `indexer` / `api` / `ui` crates. SQLite via `sqlx`. 5-chain CSV protocol indexer (BTC, ETH, Solana, Sui, Aptos). GraphQL + REST API. Dioxus UI (WASM).

**CSV = Client-Side Validation** — protocol-layer concept, not Excel. Rights/Seals/Transfers cross chains. `csv-adapter-core` is an **external sibling crate** at `../csv-adapter-core`, not vendored.

---

## Critical Compile Errors

### 1. `RpcManager::create_http_client` / `get_http_client` — Never builds client
```rust
// rpc_manager.rs:5377 — BROKEN
let mut client_builder = reqwest::Client::new();  // This IS already a Client, not a Builder
if let Some(ref auth) = endpoint.auth {
    client_builder  // just returns the Client, auth headers NEVER added
}
```
**Fix:** Use `reqwest::Client::builder()`, add default headers, then `.build().unwrap()`:
```rust
pub fn create_http_client(&self, endpoint: &RpcEndpoint) -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    if let Some(ref auth) = endpoint.auth {
        let mut headers = reqwest::header::HeaderMap::new();
        let val = reqwest::header::HeaderValue::from_str(&auth.value).unwrap();
        headers.insert(reqwest::header::HeaderName::from_bytes(auth.header.as_bytes()).unwrap(), val);
        builder = builder.default_headers(headers);
    }
    builder.build().unwrap_or_default()
}
```

### 2. `sync.rs` — `start_block` priority logic inverted
```rust
// sync.rs:7170 — BUG: ignores DB progress, always uses config start_block
let from_block = if let Some(config) = chain_config {
    if let Some(start) = config.start_block {
        start  // ← always wins, DB block never used
    } else if let Some(block) = db_block { block }
    ...
```
**Fix:** DB block must take precedence:
```rust
let from_block = db_block.unwrap_or_else(|| {
    chain_config.and_then(|c| c.start_block).unwrap_or(0)
});
```

### 3. `sync.rs` — `process_block` double-fetches every block
```rust
// sync.rs:7217 — calls process_block THEN calls index_* again (7x RPC fetches per block)
match indexer.process_block(current).await {
    Ok(_) => {
        let rights = indexer.index_rights(current).await?;   // 2nd fetch
        let seals = indexer.index_seals(current).await?;     // 3rd fetch
        ...
    }
}
```
**Fix:** Remove `process_block` call. Use `index_*` results directly, or return data from `process_block`.

### 4. `sync.rs` — `reindex_from` ignores `from_block` argument
```rust
pub async fn reindex_from(&self, chain_id: &str, from_block: u64) -> Result<()> {
    self.sync_repo.reset(chain_id).await?;
    self.sync_chain(chain_id).await  // ← from_block silently dropped
}
```
**Fix:**
```rust
self.sync_chain_from_block(chain_id, from_block).await
```

### 5. Bitcoin `fetch_block` — Wrong Mempool.space API call
```rust
// bitcoin.rs:3747 — /block-height/{N} returns a block HASH string, not txids
let url = format!("{}/block-height/{}", rpc_url, block);
let txids: Vec<String> = client.get(&url).send().await?.json().await?;  // ← panics/errors
```
**Fix:** Two-step call (already has fallback but it's wrong too):
```rust
// Step 1: get block hash
let hash_url = format!("{}/block-height/{}", rpc_url, block);
let block_hash: String = client.get(&hash_url).send().await?.text().await?;
// Step 2: get txids
let txids_url = format!("{}/block/{}/txids", rpc_url, &block_hash.trim());
let txids: Vec<String> = client.get(&txids_url).send().await?.json().await?;
```

### 6. Solana `get_chain_tip` — Wrong JSON-RPC result parsing
```rust
// solana.rs:5654 — getSlot returns a NUMBER directly, not an object
let slot = result.get("slot").and_then(|v| v.as_u64()).unwrap_or(0); // always 0
```
**Fix:**
```rust
let slot = result.as_u64().unwrap_or(0);
```

### 7. Workspace root `Cargo.toml` — Missing `[workspace]` declaration
Root `Cargo.toml` has `[package]` but no `[workspace]` members block. Sub-crates use `.workspace = true` for dependencies but no workspace is declared. This is the root compile blocker.

**Fix — add to root `Cargo.toml`:**
```toml
[workspace]
members = ["shared", "storage", "indexer", "api", "ui"]
resolver = "2"

[workspace.package]
version = "0.2.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-native-tls", "chrono"] }
reqwest = { version = "0.11", features = ["json"] }
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
hex = "0.4"
clap = { version = "4", features = ["derive"] }
thiserror = "1.0"
anyhow = "1.0"
futures = "0.3"
prometheus = "0.13"
lazy_static = "1.4"
uuid = { version = "1.0", features = ["v4", "serde"] }
toml = "0.8"
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
async-graphql = { version = "7", features = ["chrono"] }
async-graphql-axum = "7"
csv-adapter-core = { path = "../csv-adapter-core" }
config = "0.14"
```

---

## Implementation Plan — High to Low Priority

### P0 — Unblock Compilation (Do First)

| # | File | Fix |
|---|------|-----|
| 1 | `Cargo.toml` (root) | Add `[workspace]` + `[workspace.dependencies]` block |
| 2 | `indexer/src/rpc_manager.rs` | Replace `Client::new()` builder pattern with proper `Client::builder().default_headers(...).build()` |
| 3 | `indexer/src/sync.rs` | Fix `start_block` logic priority — DB always wins |
| 4 | `indexer/src/sync.rs` | Remove redundant `process_block` call, use `index_*` results directly |

### P1 — Indexer Cannot Parse Blocks

| # | Chain | Bug | Fix |
|---|-------|-----|-----|
| 5 | Bitcoin | `fetch_block` calls wrong endpoint | Two-step: `GET /block-height/{N}` → hash → `GET /block/{hash}/txids` → loop txids |
| 6 | Bitcoin | `index_seals` — `involves_relevant_address` always `false` | Decode `scriptpubkey` and match against `csv_addresses` set |
| 7 | Ethereum | `fetch_block` deserializes `TxData` but ETH `eth_getBlockByNumber` nests logs under receipts — `tx.logs` always `None` | Add separate `eth_getLogs` call per block, or use `eth_getBlockReceipts` |
| 8 | Ethereum | Event signatures are hex-encoded ASCII placeholders (`RIGHT_MINTED_SIG` contains invalid hex chars `g`) | Replace with real keccak256 digests; add `sha3`/`tiny-keccak` dep |
| 9 | Solana | `getSlot` result is `u64` not object — `.get("slot")` always fails | `result.as_u64()` directly |
| 10 | Solana | `reindex_from` drops `from_block` | Pass to `sync_chain_from_block` |

### P2 — Data Integrity & Protocol Correctness

| # | Area | Issue | Fix |
|---|------|-------|-----|
| 11 | Bitcoin CSV parsing | `parse_right_from_op_return` checks `&protocol_id[0..4] != b"CSV-"` but CSV protocol IDs are 32-byte hashes, not human-readable strings | Replace with actual RGB/LNP protocol tag bytes from `csv-adapter-core` |
| 12 | Ethereum | `parse_right_from_log` — owner incorrectly set to emitting contract address, not actual token owner from log topics | Parse topic[3] (owner indexed param) |
| 13 | `sync.rs` | All chains call 7 RPC fetches per block (rights, seals, transfers, contracts × basic + enhanced) serially — massive latency | Use `tokio::join!` or `FuturesUnordered` to parallelize per-block indexing |
| 14 | `storage/schema.sql` | No tables for `enhanced_rights`, `enhanced_seals`, `enhanced_transfers` — `AdvancedProofRepository` will fail on insert | Add tables; or add to schema and `apply_schema` check |
| 15 | `SyncCoordinator::start` | Runs all chains serially in a single-threaded `while` loop, ignoring `concurrency` config | Spawn one `tokio::task` per chain, use `JoinSet` |

### P3 — Architecture Improvements

| # | Area | Recommendation |
|---|------|----------------|
| 16 | All chain indexers | `get_latest_synced_block` hardcoded to `Ok(0)` — never reads from DB | Inject `SyncRepository` reference or pass last block as parameter |
| 17 | Bitcoin indexer | Fetches every tx in a block individually (N HTTP calls per block) — use `GET /block/{hash}/txs` for batch retrieval |
| 18 | `RpcManager` | No retry logic / circuit-breaker — single failure drops entire chain sync cycle | Add `exponential_backoff` + fallback endpoint rotation on error |
| 19 | `EthereumIndexer` | `index_contracts` returns static list, not block-scanned data | Emit contract discovery events from `eth_getLogs` with `ContractDeployed` topic |
| 20 | `MetricsServer` | `init_metrics()` registers metrics but no HTTP server exposes `/metrics` | Add `axum` route or dedicated Prometheus scrape endpoint in `api` crate |
| 21 | Schema | No migration versioning — `apply_schema` checks single table existence | Adopt `sqlx migrate` with numbered migration files |
| 22 | `chain_indexer` trait | `detect_commitment_scheme` takes `&[u8]` but all impls ignore it, always return hardcoded value | Pass actual transaction/witness bytes and implement per-chain detection using `csv-adapter-core` commitment parsing |
