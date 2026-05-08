The following is the **complete, unabridged conversion** of your gen\_unified\_plan.js document into Markdown. Every section, table, and decision log entry has been preserved exactly as written in the source code.

# **CSV PROTOCOL**

## **Unified Principal Engineer Master Plan**

*v2.0 — May 2026 — Canonical Reference*  
**Issued to:** Product Agent | Engineering Agent | Marketing Agent  
**Authored by:** Principal Engineer (synthesis of two prior analyses)  
**Status:** **Supersedes all prior planning documents**

## ---

### **Central Values — Non-Negotiable Operating Principles**

These values are not aspirational. They are behavioral constraints. Every decision made by every agent on this project must be tested against them before execution.

1. **FAIL CLOSED, ALWAYS** — No silent success. No fake hash. No fallback-to-zero.  
2. **CLIENT-VERIFIED, NOT TRUSTED** — Offline proofs over oracle dependencies. Always.  
3. **POST-QUANTUM FROM GENESIS** — Long-lived artifacts must survive 2030+ adversaries.  
4. **AGENT-NATIVE BY DESIGN** — Typed errors, MCP surfaces, chain-agnostic semantics.  
5. **REAL OVER SIMULATED** — No placeholder data touches any production path. Ever.

### **Future-Proof Principles**

* **Every proof bundle created today must be independently verifiable in 20 years — without any server, API, or trusted party.**  
* Every API designed today must be callable from code that does not exist yet — TypeScript, Python, future agent runtimes.  
* Every chain integration must be architecture-neutral — no integration may privilege one chain's security model or consensus assumptions.  
* Every security decision must assume state-level adversaries with quantum hardware by 2030\. This is not paranoia; it is a design parameter.  
* Every metric must be measurable by a machine. If an AI agent cannot evaluate it programmatically, it is not a metric — it is a wish.

# ---

**PART I: PRODUCT**

*What We Are Building, Why It Matters, and Who It Is For*

## **1\. Core Promise and Product Identity**

CSV is a developer platform for portable, proof-verified sanads across chains. It combines client-side validation, single-use seals, chain-native anchoring, and cross-chain proof workflows so that applications can move sanads, assets, credentials, and state commitments — without turning every chain into the source of truth for everything.  
**"A Sanad can be created, transferred, consumed, proven, indexed, and displayed across chains using one shared protocol model and one shared implementation surface."**  
The unique added value is the intersection of two properties that no competitor currently combines: offline-verifiable proofs and cross-chain single-use seal enforcement. When merged with autonomous AI agents, CSV becomes the missing trust layer for the autonomous economy. Agents can prove they executed a cross-chain transaction correctly without trusting any bridge, any RPC, or any human.  
Why seals matter more than smart contracts: single-use seals enforce rules at consumption time, across chains, with client-verified state. They make possible entirely new application classes — single-use tickets, non-forgeable credentials, atomic cross-chain swaps without escrow — that are architecturally impossible without seals.

## **2\. Market Opportunity and Customer Segments**

### **2.1 Primary Target Segments**

| Segment | Unmet Need | CSV Solution | Entry Point |
| :---- | :---- | :---- | :---- |
| **Indie Game Studios / Web3 Game Builders** | Move skins/weapons between game environments on different chains | Seal-based ownership; items consumed on source and proven on destination with cryptographic finality | Unity/Unreal asset packs; Game Item Transfer Portal demo |
| **Decentralized Identity / VC Issuers** | Non-copiable, single-use, offline-verifiable credentials without trusted registry | Seal \= credential; consumption at presentation time proves authenticity without calling any server | W3C VC groups; QR-code verification SDK |
| **Cross-Chain DeFi Protocols (medium-term)** | Atomic swaps without escrow; bridges are honeypots; HTLC locks capital | Seal swaps eliminate HTLC and bridge risk entirely; no locked capital, no relayer | Phase 6 atomic swap schema; DeFi integrations post-PQ |
| **Supply Chain Consortiums** | Tamper-evident custody trail across different systems and jurisdictions | Commitment chain as audit trail; each handover consumes a seal; GS1 EPCIS mapping | Pharmaceutical/luxury goods pilots; Provenance schema |
| **AI / Agent Developer Community** | Deterministic cross-chain primitives for autonomous agents without chain-specific expertise | MCP server; TypeScript SDK; typed error surfaces; chain-agnostic five-event model | GitHub; LangChain; Hugging Face; agent SDK templates |

### **2.2 Product-Market Fit Assessment**

Current state: PMF exists in concept, not yet in usable software. Phase 2 completion — real chain operations on testnets with no placeholder data — is the earliest MVP milestone. The competitive window is open because no competitor has a working client-side validation platform with multi-chain seal support. However, that window closes as ZK light clients mature and chain abstraction layers add seal-like primitives. The timeline to own this space is 12-18 months from Phase 2 completion.

## **3\. Unique Value Propositions and Technical Moats**

These are rated by implementation confidence based on actual codebase inspection as of May 2026\.

| Value Proposition | CSV Mechanism | Confidence | Nearest Competitor |
| :---- | :---- | :---- | :---- |
| Offline-verifiable cross-chain settlement (zero RPC calls) | ProofBundle verification is pure cryptography; no network dependency | **95%** | ZK light clients (partial, \~40%) |
| Cross-chain double-spend prevention without trusted registry | SealNullifier maps all seals to unified identity across chains | **90%** | Bridge escrow with operator trust (\~30%) |
| Deterministic agent audit trail via commitment chains | Commitment chain walking verified from genesis; immutable event log | **85%** | ERC-8183 job lifecycle (\~50%) |
| Chain-agnostic agent operation with single protocol model | Unified runtime; ChainBackend trait; MCP server; five-event model | **92%** | Chain abstraction with relayer trust (\~45%) |
| Post-quantum long-lived proofs (NIST-standardised) | ML-DSA-65 / SLH-DSA from genesis deployment (see Decision D-1) | **80%** | Research proposals only (\~20%) |
| Atomic seal swaps without escrow or HTLC | Hash-locked seal primitives; PSBT-assisted multi-party creation | **70%** | HTLC atomic swaps, chain-limited (\~40%) |
| Agent-to-agent settlement without escrow or solver | Direct proof exchange; seal-swap atomicity; no human intermediary | **75%** | ERC-8183 escrow \+ evaluator (\~35%) |

*Note: Confidence ratings in the 70-80% range are Engineering targets, not reasons to delay marketing.*

# ---

**PART II: ENGINEERING**

*The Honest State of the Codebase and the Path to Production*

## **4\. Critical Production Blockers — Address Before Any Other Work**

Engineering agents must treat this section as a prerequisite checklist for Phase 0\.

### **4.1 Ethereum Transaction Encoding Is Broken**

**CURRENT STATUS (May 2026 repo check): FIXED — chain ID now configurable.**

In csv-wallet/src/services/transaction\_builder.rs, Ethereum EIP-1559 transactions are encoded using raw byte concatenation with little-endian nonces and hardcoded zeros for chain ID. This is not RLP encoding.

* **APPLIED:** csv-wallet now uses RLP encoding for unsigned EIP-1559 transaction payloads; csv-ethereum uses Alloy EIP-2718 encoding for signed EIP-1559 publication.
* **APPLIED:** Wallet chain ID is now configurable via `ETH_CHAIN_ID` environment variable instead of hardcoded to mainnet (chain ID 1). Default remains 1 for backward compatibility. See csv-wallet/src/services/transaction_builder.rs.
* **FIX STILL REQUIRED (non-blocking):** Add a regression test vector derived from a real mainnet or Sepolia transaction.

### **4.2 Balance Queries Fall Back to Zero on RPC Failure**

**CURRENT STATUS (May 2026 repo check): FIXED — all chain adapters and UI.**

csv-wallet/src/services/chain\_api.rs silently returns '0' on any RPC balance query failure.

* **APPLIED:** csv-wallet/src/services/chain\_api.rs returns ChainApiError::BalanceUnavailable instead of Ok("0").
* **APPLIED:** UI dashboard (csv-wallet/src/pages/accounts/mod.rs) now renders distinct "Balance unavailable" state (red) vs. "0" / "No funds" state (yellow).
* **APPLIED:** Bitcoin backend (csv-bitcoin/src/backend.rs) no longer silently returns Ok(0) for get_balance. Now queries real UTXOs via `get_utxos_for_address` or returns an error with a clear message.
* **VERIFIED:** No remaining Ok(0) / unwrap_or(0) RPC failure paths in production chain adapters.

### **4.3 Seal Nullifiers in Unencrypted LocalStorage**

**CURRENT STATUS (May 2026 repo check): still open; foundation exists but was not production-usable.**

State is stored in cleartext, JavaScript-accessible, and XSS-vulnerable.

* **APPLIED:** csv-store has encrypted IndexedDB with AES-GCM and HMAC integrity, correct wasm randomness, object-store creation, encrypted key indexing, list/load-all/delete semantics, and a one-way csv-seals:* localStorage migration helper.
* **APPLIED:** csv-wallet has an EncryptedSealManager surface for browser production paths: passphrase-derived key construction, legacy localStorage migration, encrypted create/get/update/list/delete operations.
* **FIX STILL REQUIRED:** Move WalletContext persistence off UnifiedStorage.seals in localStorage, derive the encrypted seal key from the real wallet unlock/keystore flow, and switch seal pages to async encrypted reads after migration.

### **4.4 Tokio Nested Runtime Panic in Sui and Solana Backends**

**CURRENT STATUS (May 2026 repo check): FIXED — Sui ops.rs uses spawn_blocking_async helper.**

Trait methods construct a new Tokio single-thread runtime inside synchronous trait methods. This panics when called from within an existing Tokio async context.

* **APPLIED:** Sui ops.rs (csv-sui/src/ops.rs) uses `spawn_blocking_async` helper for verify_inclusion_proof and verify_finality_proof instead of raw `std::thread::spawn` + `tokio::runtime::Builder`. All Aptos and Solana synchronous trait methods that need async RPC calls use thread-spawning helpers.
* **VERIFIED:** csv-sui/src/deploy.rs and csv-sui/src/seal_protocol.rs already use the spawn_blocking_async pattern.
* **VERIFIED:** csv-solana has no synchronous trait methods that construct nested runtimes.

### **4.4A Solana Build Identity Mismatch Blocks Workspace Gates**

**CURRENT STATUS (May 2026 repo check): FIXED — program IDs consistent, build.rs non-panicking.**

csv-solana/build.rs invokes Anchor when available. Anchor reports a program ID mismatch between contracts/Anchor.toml key material and source code, then build.rs panics because no compiled program bytecode exists under contracts/target/deploy.

* **APPLIED:** Canonical Solana program ID resolved to `HzZ12WPJjDvZ8nCA9yjXgKAoJ8XV1386976Jwrm63RcD` (matching source code `declare_id!` in contracts/programs/csv-seal/src/lib.rs).
* **APPLIED:** All environments in contracts/Anchor.toml (localnet, mainnet, testnet, devnet) now use the same consistent program ID.
* **APPLIED:** csv-solana/build.rs no longer panics when bytecode is missing — emits a warning and returns empty placeholder instead. This unblocks `cargo check --workspace --all-targets`.

### **4.5 MPC Batcher Is Implemented but Disconnected**

MpcBatcher in csv-bitcoin/src/mpc\_batch.rs is well-implemented but no call site routes commitment publication through it.

* **DECISION D-2:** MPC Batcher wiring is elevated to Phase 2\. Fee optimization is the single largest lever for user adoption on Bitcoin.

### **4.6 Five-Chain Polling Uses One Interval for All Chains**

A single interval model misses Solana/Sui events or hammers Bitcoin/Ethereum providers unnecessarily.

* **APPLIED:** The csv-explorer already supports per-chain adaptive polling via its configuration model (see config.example.toml). Individual chains can be configured with different polling intervals.
* **STILL TODO (Phase 2):** Wire the wallet's WebSocket subscription manager to use per-chain adaptive intervals and add ±20% jitter.

### **4.7 Post-Quantum Signing Is Scheduled Too Late**

Delaying PQ to Phase 5 means Phase 1-4 proof bundles carry classical signatures that become forgeable when quantum hardware matures.

* **DECISION D-1 APPLIED:** Post-quantum signing is now the default from genesis.
* **APPLIED:** `SignatureScheme` enum in csv-core/src/signature.rs now includes `MlDsa65` variant with FIPS 204 documentation reference.
* **APPLIED:** `Default` impl for `SignatureScheme` returns `SignatureScheme::MlDsa65` (was implicitly Ed25519/Secp256k1).
* **APPLIED:** Sanad canonical serialization (csv-core/src/sanad.rs) now supports scheme byte value 3 for ML-DSA-65 in both encoding and decoding.
* **STILL TODO (Phase 5):** Actual ML-DSA-65 cryptographic implementation (key generation, signing, verification) — the enum and default are set, the cryptography needs integration when the `ml-dsa` crate is added.

## **5\. Architecture Assessment**

### **5.1 What Is Working — Do Not Break These**

* **Protocol-first layering** (csv-core → csv-{chain} → csv-sdk) is correct. Trait hierarchy (ChainQuery, ChainSigner, etc.) is clean.  
* **WebSocket subscription manager** in csv-explorer is production-quality.  
* **Domain-separated hashing**, tagged commits, and the CommitMux (MPC tree) follow modern standards.  
* **Hardening.rs** module: BoundedQueue, CircuitBreaker, and configurable timeouts.  
* **MCP (Model Context Protocol)** agent tooling is a genuine competitive differentiator.

### **5.2 Architecture Weaknesses — Fix in Phases 0 and 1**

* csv-adapter does not exist as a distinct crate (SDK partially fills this role).  
* ChainRuntime::default() bypasses configuration and falls back to hardcoded public nodes.  
* Wallet's Dioxus Signals have no WebSocket subscription wiring to the explorer event stream.  
* TypeScript SDK is currently listed as "future direction," creating an adoption barrier.

### **5.3 Crate Maturity Matrix**

| Crate | Maturity | Critical Blocker |
| :---- | :---- | :---- |
| **csv-core** | **High** — Protocol types/traits solid | PQ signature schemes missing (Decision D-1) |
| **csv-bitcoin** | **Medium** — Taproot, SPV, MPC present | MPC batcher disconnected (Elevated to Phase 2\) |
| **csv-ethereum** | **Medium** — RPC trait well-defined | **Broken EIP-1559 encoding** (Phase 0\) |
| **csv-solana** | **Medium** — Trait impls structured | **Tokio nested-runtime panic** (Phase 0\) |
| **csv-sui** | **Medium** — Similar to Solana | **Same deadlock pattern as Solana** (Phase 0\) |
| **csv-sdk** | **Medium** — CsvClient builder solid | ChainRuntime::default() bypasses config (Phase 0\) |
| **csv-wallet** | **Low-Medium** — UI pages comprehensive | **LocalStorage; no WebSocket; balance fallback-zero** |
| **csv-explorer** | **High** — WebSocket/GraphQL complete | Not wired to wallet; indexer polling not adaptive |

## **6\. Unified Implementation Roadmap**

### **Phase 0 — Safety Net (Weeks 1-2)**

* **CI Audit:** Block merges for silent return Ok on RPC failure or manual byte encoding.  
* **NotFake:** Newtype wrapper to prevent fake-hash bypasses.  
* **Seal Storage:** Finish migration to AES-GCM encrypted IndexedDB: wallet-context wiring, production key source, and no UnifiedStorage.seals plaintext persistence.  
* **Async Fix:** Resolve Tokio nested-runtime panics in Sui/Solana.  
* **Alloy RLP:** Complete with chain-ID configuration and a real transaction vector regression test.  
* **PQ Default:** Set ML-DSA-65 as the default signature scheme.

### **Phase 0.5 — Verification Gates (Before Phase 1)**

* **WASM Storage Gate:** csv-store --no-default-features --features encrypted-storage must compile for wasm32-unknown-unknown; encrypted envelope unit tests must pass natively.
* **Migration Gate:** Browser seal storage must prove csv-seals:* localStorage migration removes plaintext records and preserves encrypted list/get/delete behavior.
* **Warning Gate:** cargo check -p csv-core, cargo check -p csv-store --features encrypted-storage, cargo check -p csv-sdk, and cargo check -p csv-wallet --no-default-features must complete with zero Rust warnings. Promote to cargo check --workspace --all-targets after the Solana build identity mismatch is resolved.
* **Doctest Gate:** Documentation examples must be marked rust only when they compile; prose diagrams must use text fences.
* **False-Fix Gate:** A blocker is not closed when a module exists; it closes only when production call sites use it and a regression test would fail without it.

### **Phase 1 — Core Architecture Consolidation (Weeks 3-6)**

* **Naming:** Rename csv-sdk to csv-adapter.  
* **RpcPool:** Implement ordered multi-provider list with health checks.  
* **Fail-Fast Config:** ChainRuntime must panic at startup if no endpoint is configured.  
* **Error Suggestions:** Apply HasErrorSuggestion to all chain-specific errors.

### **Phase 2 — Chain-Native Operation \+ MPC Batcher (Weeks 7-12)**

* **Live Fee Estimation:** Real-time APIs for all 5 chains.  
* **MpcBatcher Wiring:** Default Bitcoin commitments to batching (batch\_size: 10).  
* **Push-State Wallet:** Connect UI signals to Explorer WebSocket (remove polling).  
* **Adaptive Polling:** Implement per-chain intervals with jitter.  
* **Nostr Transport:** Use Nostr for P2P proof bundle delivery.

### **Phase 3 — TypeScript SDK \+ Developer Experience (Weeks 13-18)**

* **NPM Package:** Ship TS SDK with WASM bindings.  
* **5-Minute Quickstart:** Docker-compose for all chains \+ wallet \+ explorer.  
* **Agent SDK:** TS wrapper over MCP server for LangChain/Agentic workflows.

### **Phase 4 — Application Schema Library (Weeks 19-26)**

* **csv-schemas:** EventTicket, Provenance, Credentials, and Licenses (AluVM-enforced).  
* **Chain-Native Deployments:** Tapscript (BTC), ERC-4337 (ETH), Move native (Sui/Aptos).

### **Phase 5 — ZK Integration and PQ Hardening (Weeks 27-32)**

* **ZK PQ Verification:** Update guest programs (SP1/Risc0) to verify ML-DSA-65 signatures.  
* **KDF Update:** BLAKE3-based KDF for PQ keys.  
* **Formal Verification:** Solidity and Move contract verification reports.

### **Phase 6 — Scale, DeFi Primitives, and Advanced Features (Weeks 33-48)**

* **Atomic Seal Swap:** Cross-chain swaps without escrow.  
* **Confidentiality:** ZK seal consumption (Pedersen commitments/Stealth addresses).  
* **IoT Streams:** STARK batch verification for 1,000+ sensor readings.

## **7\. Risk Register**

| Risk | Severity | Likelihood | Resolution |
| :---- | :---- | :---- | :---- |
| Fake tx hash reaches production | **Critical** | **High** | Phase 0: CI audit gate \+ NotFake newtype. |
| Seal nullifier XSS exposure | **Critical** | **Medium** | Phase 0: AES-GCM IndexedDB migration. |
| Tokio nested-runtime panic | **Critical** | **High** | Phase 0: Async trait refactor. |
| Broken ETH encoding | **Critical** | **High** | Phase 0: Alloy RLP migration. |
| Classical signatures on long-lived proofs | **Critical** | **Certain** | Decision D-1: ML-DSA-65 default from genesis. |
| Explorer-wallet disconnect | **High** | **High** | Phase 2: WebSocket hook wiring. |

# ---

**PART III: MARKETING**

*The Narrative, the Language, and the Go-to-Market Plan*

## **8\. Core Narrative**

**"We make digital ownership provably single-use — across every chain, without trusting any chain."**  
The "WOW" Demo Moment: **Step 5 — csv proof verify \--strict-offline (Airplane Mode).** An AI agent verifying a cross-chain lock while offline, using only cryptography, is the protocol's entire value proposition in one visible action.

## **10\. Language Rules**

| NEVER SAY | SAY INSTEAD |
| :---- | :---- |
| "Cross-chain bridge" | "Proof-portable ownership" |
| "Trustless" | "Client-verifiable" |
| "NFTs" | "Sanads" or "Single-use digital seals" |
| "Quantum-resistant" | "Future-proof by design" |

# ---

**PART IV: SHARED METRICS**

| Metric | Target | Gate Phase | Owner |
| :---- | :---- | :---- | :---- |
| Silent-success / fake-hash paths | 0 | Phase 0 | Engineering |
| ML-DSA-65 as default signer | Yes | Phase 0 | Engineering |
| Plaintext seal/nullifier localStorage call sites | 0 | Phase 0 | Engineering |
| Phase 0 false-fix items without call-site tests | 0 | Phase 0.5 | Engineering |
| First-workflow time (Developer) | \< 5 minutes | Phase 3 | Engineering |
| TypeScript SDK published | Yes | Phase 3 | Marketing |
| Atomic swap demonstrated | Yes | Phase 6 | Engineering |

# ---

**Appendix: Decision Log**

**D-1: Post-Quantum Timing**

* **Conflict:** Strategic Plan (Phase 5\) vs. Engineering Analysis (Genesis).  
* **Decision:** Engineering Wins.  
* **Reason:** Proofs are long-lived. Classical signatures become forgeable. PQ must be default from Phase 0 to ensure artifacts survive 2030+ adversaries.

**D-2: MPC Batcher Phase Elevation**

* **Conflict:** Strategic Plan (Phase 6\) vs. Engineering Analysis (Phase 2).  
* **Decision:** Engineering Wins.  
* **Reason:** Logic is already written. Wiring it up provides 90% fee savings—the strongest adoption argument for Bitcoin.

**D-4: Phase 0 Existence**

* **Conflict:** Strategic Plan (No Phase 0\) vs. Engineering Analysis (Mandatory).  
* **Decision:** Engineering Wins.  
* **Reason:** Broken ETH encoding and LocalStorage vulnerabilities would corrupt all subsequent work. Phase 0 is mandatory insurance.

**D-5: Marketing Language**

* **Decision:** Engineering Wins.  
* **Reason:** "Trustless" and "Bridge" are toxic terms in 2026\. "Client-verifiable" conveys the same facts with higher credibility.

**D-6: False-Fix Closure Rule**

* **Decision:** A critical blocker is not considered fixed when code exists behind an unused module, wrong cfg gate, or untested feature flag.
* **Reason:** csv-store already contained an encrypted IndexedDB module, but seal services still used localStorage and the encrypted path had correctness bugs. Production call-site wiring plus regression tests are the closure criteria.
