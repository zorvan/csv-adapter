# CSV Protocol — Codebase Map

> **Purpose**: One-line purpose for every `src/` file across all crates.
> **Rule**: This file MUST be updated on every PR that adds, removes, or renames a file.

---

## csv-adapter-core/src/ — Protocol Core Types & Traits

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point; re-exports all core CSV types and modules |
| `hash.rs` | 32-byte cryptographic hash (SHA-256 based) with safe conversion between bytes, hex, and internal forms |
| `right.rs` | Universal Seal Primitive — canonical `Right` type with single-use enforcement across all chains |
| `seal.rs` | `SealRef` and `AnchorRef` reference types with size limits and serialization support |
| `commitment.rs` | `Commitment` type with canonical encoding binding off-chain state transitions to the anchoring layer |
| `tagged_hash.rs` | BIP-340 style tagged hashing with domain separation to prevent cross-protocol hash collisions |
| `signature.rs` | Chain-agnostic signature verification trait with secp256k1 and ed25519 implementations |
| `protocol_version.rs` | Single source of truth for protocol version, chain IDs, transfer status, error codes, and capability flags |
| `traits.rs` | Core `AnchorLayer` trait — the primary security boundary that all chain adapters implement |
| `error.rs` | Unified `AdapterError` enum with seal replay detection and all adapter-specific error variants |
| `proof.rs` | Proof bundle types: `InclusionProof`, `FinalityProof`, `ProofBundle` for off-chain verification |
| `proof_verify.rs` | Proof verification pipeline — cryptographic gatekeeper for authenticity, integrity, uniqueness, and finality |
| `consignment.rs` | Consignment wire format: complete provable history of a contract for state transfer between peers |
| `genesis.rs` | Contract genesis: initial state definition with global state and initial seal assignments |
| `schema.rs` | Schema definitions: valid state types, transition rules, and validation rules for contract classes |
| `state.rs` | Typed state enums for CSV contracts defining structured state with schema validation |
| `transition.rs` | Typed state transitions: consumes inputs, produces outputs, updates global state, attaches metadata |
| `dag.rs` | State transition DAG types representing deterministic state transitions verified off-chain |
| `commitment_chain.rs` | Commitment chain verification: walks commitments from present back to genesis verifying hash linkage |
| `cross_chain.rs` | Cross-chain right transfer: lock-and-prove protocol for transferring Rights between chains |
| `client.rs` | Client-side validation engine that receives and verifies consignments and seal proofs from peers |
| `validator.rs` | Consignment validation pipeline: verifies seal consumption, commitment linkage, and DAG integrity |
| `seal_registry.rs` | Cross-chain seal registry that prevents double-consumption and cross-chain double-spends |
| `state_store.rs` | State history store for contracts enabling client-side validation of full commit/transition history |
| `monitor.rs` | Reorg monitoring and censorship detection tracking chain state across all adapters |
| `performance.rs` | Performance optimization: caching, bloom filters, and parallel processing for proof verification |
| `hardening.rs` | Production hardening: bounded queues, circuit breakers, timeouts, and memory limits |
| `advanced_commitments.rs` | Advanced commitment types: multiple scheme versions, proof metadata, and extensible commitment registry |
| `events.rs` | Standardized event types for cross-chain rights used by adapters, indexers, and SDKs |
| `agent_types.rs` | Agent-friendly types with self-describing errors, structured status, and machine-actionable fix suggestions |
| `store.rs` | Persistent seal and anchor storage trait-based abstraction for persistence across restarts |
| `mpc.rs` | Multi-party computation threshold signatures (experimental) |
| `rgb_compat.rs` | RGB protocol compatibility bridge — maps RGB assets to CSV rights (experimental) |
| `tapret_verify.rs` | Tapret commitment verification per BIP-341 script path spend semantics |
| `vm/` | Deterministic VM module: `aluvm.rs` (AluVM adapter), `passthrough.rs` (testing), `metered.rs` (gas tracking) |
| `zk_proof.rs` | Zero-knowledge seal proof types and traits for trustless verification without RPC |
| `chain_adapter.rs` | `ChainAdapter` trait and `ChainRegistry` for dynamic chain support with capability tracking |
| `chain_operations.rs` | Core chain operation traits: `ChainQuery`, `ChainSigner`, `ChainBroadcaster`, `ChainDeployer`, `ChainProofProvider`, `ChainRightOps` |
| `chain_plugin.rs` | Plugin system for plug-and-play chain registration and discovery at runtime |
| `chain_config.rs` | Chain configuration types including capabilities, account models, and chain settings |
| `chain_discovery.rs` | Single public API for discovering, registering, and instantiating chain adapters |
| `adapter_factory.rs` | Factory pattern for creating chain adapters based on chain IDs with feature-gated chain support |
| `adapters/mod.rs` | Re-exports core adapter traits and configuration types |

---

## csv-adapter-bitcoin/src/ — Bitcoin Adapter

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point for Bitcoin adapter; re-exports all Bitcoin-specific modules |
| `proofs.rs` | Production-grade Bitcoin SPV inclusion proofs with pure-Rust and rust-bitcoin dual Merkle tree implementations |
| `adapter.rs` | Bitcoin `AnchorLayer` implementation with HD wallet support using UTXOs as seals and Taproot commitments |
| `chain_adapter_impl.rs` | `ChainAdapter` trait implementation for `BitcoinAnchorLayer` enabling unified chain adapter interface |
| `chain_operations.rs` | Bitcoin implementation of core chain operation traits for real Bitcoin chain interactions |
| `tx_builder.rs` | Commitment transaction builder with UTXO selection, Taproot tree building, fee estimation, and dust protection |
| `tapret.rs` | Bitcoin Tapret/Opret commitment script construction with nonce mining and Opret fallback |
| `bip341.rs` | BIP-341 Taproot key derivation and output key tweaking for P2TR addresses |
| `spv.rs` | SPV (Simplified Payment Verification) for Bitcoin with block header tracking and confirmation management |
| `signatures.rs` | Bitcoin ECDSA/secp256k1 signature verification for message authentication |
| `seal.rs` | Bitcoin seal management with seal registry for tracking used seals and preventing replay |
| `types.rs` | Bitcoin-specific type definitions including `BitcoinSealRef` (UTXO OutPoint) |
| `config.rs` | Bitcoin adapter configuration: network, finality depth, RPC URL, and HD wallet xpub |
| `rpc.rs` | Bitcoin RPC trait definition and test helpers for mock RPC interactions |
| `real_rpc.rs` | Real Bitcoin RPC client wrapping `bitcoincore-rpc` behind the `BitcoinRpc` trait for production use |
| `mempool_rpc.rs` | Production RPC via mempool.space Signet REST API — no local Bitcoin Core node needed |
| `wallet.rs` | Seal wallet for Bitcoin UTXO management with BIP-32/86 HD key derivation |
| `error.rs` | Bitcoin adapter specific error types including RPC, transaction, and UTXO errors |
| `deploy.rs` | Bitcoin Taproot script contract deployment via RPC with BIP-86 key derivation |
| `testnet_deploy.rs` | Testnet deployment helpers for Signet and Testnet3 with pre-configured RPC endpoints |

---

## csv-adapter-ethereum/src/ — Ethereum Adapter

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point for Ethereum adapter; re-exports all Ethereum-specific modules |
| `adapter.rs` | Ethereum `AnchorLayer` implementation using storage slots as seals and LOG events for commitments |
| `chain_adapter_impl.rs` | `ChainAdapter` trait implementation for `EthereumAnchorLayer` enabling unified chain adapter interface |
| `chain_operations.rs` | Ethereum implementation of core chain operation traits for real Ethereum chain interactions |
| `mpt.rs` | Merkle-Patricia Trie verification using `alloy-trie` for state root computation and proof verification |
| `proofs.rs` | Ethereum proof generation and verification using MPT storage proofs |
| `finality.rs` | Ethereum finality checker supporting post-merge finalized checkpoints and confirmation depth fallback |
| `seal_contract.rs` | Ethereum seal contract ABI and interface for managing single-use storage slot seals |
| `signatures.rs` | Ethereum signature verification using ECDSA/secp256k1 with ecrecover |
| `seal.rs` | Ethereum seal management with seal registry for tracking used storage slots |
| `types.rs` | Ethereum-specific type definitions including `EthereumSealRef` (contract address + slot) |
| `config.rs` | Ethereum adapter configuration: network, finality depth, RPC URL, and contract addresses |
| `rpc.rs` | Ethereum RPC trait definition with `alloy`-based provider interface |
| `real_rpc.rs` | Real Ethereum RPC client implementation using `alloy` for production use |
| `error.rs` | Ethereum adapter specific error types including RPC, contract, and transaction errors |
| `deploy.rs` | Ethereum CSV seal contract deployment with Solidity contract bytecode and ABI |

---

## csv-adapter-sui/src/ — Sui Adapter

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point for Sui adapter; re-exports all Sui-specific modules |
| `adapter.rs` | Sui `AnchorLayer` implementation using owned objects as seals and dynamic fields as anchors |
| `chain_adapter_impl.rs` | `ChainAdapter` trait implementation for `SuiAnchorLayer` enabling unified chain adapter interface |
| `chain_operations.rs` | Sui implementation of core chain operation traits for real Sui chain interactions |
| `proofs.rs` | Sui proof generation and verification using object state proofs |
| `checkpoint.rs` | Sui checkpoint finality verifier using Narwhal consensus certified checkpoints |
| `signatures.rs` | Sui Ed25519 signature verification for message authentication |
| `seal.rs` | Sui seal management with seal registry for tracking used objects |
| `types.rs` | Sui-specific type definitions including `SuiSealRef` (object ID) |
| `config.rs` | Sui adapter configuration: network, finality depth, RPC URL, and package ID |
| `rpc.rs` | Sui RPC trait definition with JSON-RPC interface for Sui node communication |
| `real_rpc.rs` | Real Sui RPC client implementation for production use |
| `error.rs` | Sui adapter specific error types including RPC, object, and transaction errors |
| `deploy.rs` | Sui CSV program deployment using Move language and Sui SDK |
| `mint.rs` | Mint operations for CSV rights on Sui using JSON-RPC transaction submission |

---

## csv-adapter-aptos/src/ — Aptos Adapter

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point for Aptos adapter; re-exports all Aptos-specific modules |
| `adapter.rs` | Aptos `AnchorLayer` implementation using Move resources as seals and events as anchors |
| `chain_adapter_impl.rs` | `ChainAdapter` trait implementation for `AptosAnchorLayer` enabling unified chain adapter interface |
| `chain_operations.rs` | Aptos implementation of core chain operation traits for real Aptos chain interactions |
| `proofs.rs` | Aptos proof generation and verification using Merkle accumulator proofs |
| `merkle.rs` | Aptos Merkle accumulator implementation for native state verification |
| `checkpoint.rs` | Aptos checkpoint finality verifier using HotStuff consensus certified blocks |
| `signatures.rs` | Aptos Ed25519 signature verification for message authentication |
| `seal.rs` | Aptos seal management with seal registry for tracking used resources |
| `types.rs` | Aptos-specific type definitions including `AptosSealRef` (resource type + address) |
| `config.rs` | Aptos adapter configuration: network, finality depth, RPC URL, and package ID |
| `rpc.rs` | Aptos RPC trait definition with JSON-RPC interface for Aptos node communication |
| `real_rpc.rs` | Real Aptos RPC client implementation for production use |
| `error.rs` | Aptos adapter specific error types including RPC, resource, and transaction errors |
| `deploy.rs` | Aptos CSV Move module deployment using Aptos SDK |

---

## csv-adapter-solana/src/ — Solana Adapter

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point for Solana adapter; re-exports all Solana-specific modules |
| `adapter.rs` | Solana `AnchorLayer` implementation using program accounts as seals and instructions for commitments |
| `chain_adapter_impl.rs` | `ChainAdapter` trait implementation for `SolanaAnchorLayer` enabling unified chain adapter interface |
| `chain_operations.rs` | Solana implementation of core chain operation traits for real Solana chain interactions |
| `program.rs` | Solana program interface for CSV operations with instruction building and account management |
| `seal.rs` | Solana seal management with seal registry for tracking used program accounts |
| `types.rs` | Solana-specific type definitions including `SolanaSealRef` (account pubkey) |
| `config.rs` | Solana adapter configuration: cluster URL, finality depth, RPC settings, and program ID |
| `rpc.rs` | Solana RPC trait definition with JSON-RPC interface for Solana cluster communication |
| `wallet.rs` | Solana wallet management for keypair operations and address derivation |
| `error.rs` | Solana adapter specific error types including RPC, program, and transaction errors |
| `deploy.rs` | Solana CSV program deployment using solana-sdk and BPF bytecode |
| `mint.rs` | Mint operations for CSV rights on Solana using JSON-RPC and solana-sdk |

---

## csv-adapter-store/src/ — Persistence Layer

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point for CSV persistence with native and WASM backends |
| `state/core.rs` | Core types: `Chain`, `Network`, and `ChainConfig` for supported blockchain networks |
| `state/domain.rs` | Core CSV domain model: `Right`, transfers, contracts, seals, proofs, and transactions |
| `state/wallet.rs` | Wallet account structures referencing encrypted keys in csv-adapter-keystore |
| `state/storage.rs` | `StateStorage` — central data structure for CSV application state including chains, wallets, and domain records |
| `state/backend.rs` | `StorageBackend` trait with `NativeBackend` (sled) and `WasmBackend` (IndexedDB via rexie) |
| `state/mod.rs` | State module re-exports for core, wallet, domain, storage, and backend sub-modules |
| `browser_storage.rs` | Browser IndexedDB/localStorage implementation for WASM wallet targets |

---

## csv-adapter-keystore/src/ — Cryptographic Key Management

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point for keystore with BIP-39/BIP-44 support |
| `keystore.rs` | Encrypted keystore file format (ETH-compatible) using AES-256-GCM with scrypt KDF |
| `memory.rs` | Memory-safe key types with automatic zeroization on drop using the `zeroize` crate |
| `bip39.rs` | BIP-39 mnemonic phrase generation and recovery supporting 12-24 word mnemonics |
| `bip44.rs` | BIP-44 HD wallet derivation for multi-chain support with chain-specific paths |
| `browser_keystore.rs` | Browser LocalStorage-based keystore for WASM targets with session-based key caching |

---

## csv-adapter/src/ — Unified Meta-Crate

| File | Purpose |
|------|---------|
| `lib.rs` | Unified meta-crate providing single entry point for all CSV operations |
| `client.rs` | `CsvClient` — main entry point for all CSV operations with builder pattern |
| `builder.rs` | Fluent builder for `CsvClient` allowing any combination of chain support, wallet, and storage |
| `scalable_builder.rs` | Scalable builder using dynamic chain registry for flexible client construction |
| `facade.rs` | Unified facade functions delegating to appropriate chain adapters with consistent cross-chain API |
| `prelude.rs` | Ergonomic prelude module for importing all CSV types with a single statement |
| `wallet.rs` | Multi-chain HD wallet supporting BIP-44 derivation paths for all supported chains |
| `rights.rs` | `RightsManager` providing high-level API for creating, querying, and managing Rights across chains |
| `proofs.rs` | `ProofManager` handling generation and verification of cryptographic proofs for Rights and transfers |
| `transfers.rs` | `TransferManager` handling cross-chain transfers using the lock-and-prove protocol |
| `cross_chain.rs` | Cross-chain operations for minting rights on destination chains during cross-chain transfers |
| `events.rs` | `EventStream` providing real-time event streaming via tokio broadcast channels |
| `deploy.rs` | Contract deployment manager providing unified interface for deploying CSV contracts across all chains |
| `errors.rs` | Unified `CsvError` enum wrapping all underlying error sources with machine-actionable fix suggestions |
| `config.rs` | Configuration management with TOML loading and environment variable overrides |

---

## csv-cli/src/ — Command-Line Interface

| File | Purpose |
|------|---------|
| `main.rs` | CLI application entry point with argument parsing and command dispatch |
| `config.rs` | CLI configuration management loading settings from file and environment |
| `state.rs` | CLI application state management for persistent CLI settings and history |
| `output.rs` | Formatted output rendering for CLI display including tables and structured output |
| `keystore_migration.rs` | Keystore migration utilities for upgrading from old key formats |
| `chain_registry.rs` | Chain registry management for tracking supported chains and their configurations |
| `chain_management.rs` | Chain management commands for discovering, listing, and configuring blockchain chains |
| `commands/mod.rs` | Commands module re-exports organizing all CLI subcommands |
| `commands/chain.rs` | Chain status and information commands |
| `commands/proofs.rs` | Proof generation and verification commands |
| `commands/seals.rs` | Seal management commands for creating, listing, and consuming seals |
| `commands/rights.rs` | Rights management commands for creating, listing, and transferring rights |
| `commands/validate.rs` | Consignment and proof validation commands |
| `commands/tests.rs` | Test execution commands for CSV protocol testing |
| `commands/wallet_ext.rs` | Extended wallet operations beyond basic commands |
| `commands/wallet/mod.rs` | Wallet commands module re-exports |
| `commands/wallet/balance.rs` | Wallet balance query commands |
| `commands/wallet/generate.rs` | Wallet generation and creation commands |
| `commands/wallet/import_export.rs` | Wallet import and export commands for key migration |
| `commands/wallet/types.rs` | Wallet type definitions for CLI command structures |
| `commands/wallet/fund.rs` | Wallet funding commands for acquiring testnet tokens |
| `commands/contracts/mod.rs` | Contract commands module re-exports |
| `commands/contracts/deploy.rs` | Contract deployment commands across supported chains |
| `commands/contracts/status.rs` | Contract status query commands |
| `commands/contracts/types.rs` | Contract type definitions for CLI command structures |
| `commands/cross_chain/mod.rs` | Cross-chain commands module re-exports |
| `commands/cross_chain/transfer.rs` | Cross-chain transfer commands implementing lock-and-prove protocol |
| `commands/cross_chain/status.rs` | Cross-chain transfer status query commands |
| `state/keystore_migration.rs` | State-level keystore migration utilities |

---

## csv-wallet/src/ — Dioxus Web Wallet

| File | Purpose |
|------|---------|
| `main.rs` | Wallet application entry point with Dioxus WASM initialization |
| `routes.rs` | Application routing configuration for all wallet pages |
| `layout.rs` | Main layout component with sidebar, header, and content area |
| `wallet_core.rs` | Core wallet logic coordinating keystore, storage, and blockchain services |
| `storage.rs` | Persistent storage abstraction for wallet data |
| `components/design_tokens.rs` | Design token definitions: color palette, spacing, typography, radii, shadows |
| `components/seal_status.rs` | Seal status indicator using `SealState` enum colors |
| `components/seal_visualizer.rs` | Seal visualization component for graphical representation |
| `components/hash_display.rs` | Hash display component with truncation, copy button, and expand-on-hover |
| `components/proof_inspector.rs` | Proof inspection component for viewing proof DAG details |
| `components/card.rs` | Reusable card UI component with design token styling |
| `components/sidebar.rs` | Sidebar navigation component with design token styling |
| `components/header.rs` | Application header component with navigation |
| `components/dropdown.rs` | Dropdown menu component |
| `components/onboarding.rs` | Onboarding flow component for new users |
| `components/chain_display.rs` | Chain information display component |
| `context/state.rs` | Global wallet state context provider |
| `context/types.rs` | Context type definitions |
| `context/utils.rs` | Context utility functions |
| `context/wallet.rs` | Wallet-specific context provider |
| `core/wallet.rs` | Core wallet implementation |
| `core/key_manager.rs` | Cryptographic key management |
| `core/storage.rs` | Core storage abstraction (thin wrapper over csv-adapter-store) |
| `core/encryption.rs` | AES-256-GCM encryption utilities for key protection |
| `hooks/use_wallet.rs` | Wallet state hook |
| `hooks/use_wallet_connection.rs` | Wallet connection management hook |
| `hooks/use_balance.rs` | Balance query hook |
| `hooks/use_assets.rs` | Asset tracking hook |
| `hooks/use_seals.rs` | Seal management hook |
| `hooks/use_network.rs` | Network state hook |
| `pages/seals/list.rs` | Seals listing page |
| `pages/seals/create.rs` | Seal creation page — primary user action |
| `pages/seals/consume.rs` | Seal consumption page |
| `pages/seals/verify.rs` | Seal verification page |
| `pages/rights/list.rs` | Rights listing page |
| `pages/rights/create.rs` | Rights creation page |
| `pages/rights/transfer.rs` | Rights transfer page |
| `pages/rights/consume.rs` | Rights consumption page with step-by-step progress |
| `pages/rights/show.rs` | Rights detail page |
| `pages/rights/journey.rs` | Rights lifecycle journey visualization |
| `pages/cross_chain/list.rs` | Cross-chain transfer listing |
| `pages/cross_chain/transfer.rs` | Cross-chain transfer creation page with progress indicator |
| `pages/cross_chain/status.rs` | Cross-chain transfer status with timeline visualization |
| `pages/cross_chain/detail.rs` | Cross-chain transfer detail |
| `pages/cross_chain/retry.rs` | Cross-chain transfer retry page |
| `pages/proofs/list.rs` | Proof listing page |
| `pages/proofs/generate.rs` | Proof generation page |
| `pages/proofs/verify.rs` | Proof verification page — pass/fail with proof DAG visualization |
| `pages/proofs/verify_cross_chain.rs` | Cross-chain proof verification page |
| `pages/validate/list.rs` | Validation listing page |
| `pages/validate/seal.rs` | Seal validation page |
| `pages/validate/proof.rs` | Proof validation page |
| `pages/validate/consignment.rs` | Consignment validation page |
| `pages/validate/commitment_chain.rs` | Commitment chain validation page |
| `pages/transactions/list.rs` | Transaction listing page |
| `pages/transactions/detail.rs` | Transaction detail page |
| `pages/transactions/card.rs` | Transaction card component |
| `pages/accounts/transactions.rs` | Account transactions listing |
| `pages/contracts/list.rs` | Contract listing page |
| `pages/contracts/deploy.rs` | Contract deployment page |
| `pages/contracts/status.rs` | Contract status page |
| `pages/settings/page.rs` | Settings configuration page |
| `services/seal_service.rs` | Seal CRUD operations service |
| `services/chain_api.rs` | Chain API abstraction layer |
| `services/explorer.rs` | Explorer integration service |
| `services/network.rs` | Network connectivity service |
| `services/asset_service.rs` | Asset management service |
| `services/transaction_builder.rs` | Transaction building service |
| `services/blockchain/service.rs` | Blockchain service orchestration |
| `services/blockchain/signer.rs` | Transaction signing service |
| `services/blockchain/submitter.rs` | Transaction submission service |
| `services/blockchain/estimator.rs` | Gas fee estimation service |
| `seals/manager.rs` | Seal lifecycle management |
| `seals/monitor.rs` | Seal monitoring and status tracking |
| `seals/store.rs` | Seal persistence storage |

---

## csv-explorer/ — Block Explorer

| File | Purpose |
|------|---------|
| `api/src/main.rs` | API server main entry point |
| `api/src/server.rs` | HTTP server setup and configuration |
| `api/src/websocket.rs` | WebSocket server for real-time explorer data streaming |
| `api/src/graphql/schema.rs` | GraphQL schema definitions |
| `api/src/graphql/types.rs` | GraphQL type definitions |
| `api/src/rest/handlers.rs` | REST endpoint handlers for explorer data |
| `indexer/src/main.rs` | Indexer daemon main entry point |
| `indexer/src/chain_indexer.rs` | Generic chain indexer abstraction |
| `indexer/src/indexer_plugin.rs` | Plugin system for chain-specific indexers |
| `indexer/src/bitcoin.rs` | Bitcoin chain indexer plugin |
| `indexer/src/ethereum.rs` | Ethereum chain indexer plugin |
| `indexer/src/sui.rs` | Sui chain indexer plugin |
| `indexer/src/aptos.rs` | Aptos chain indexer plugin |
| `indexer/src/solana.rs` | Solana chain indexer plugin |
| `indexer/src/rpc_manager.rs` | RPC connection management for indexers |
| `indexer/src/metrics.rs` | Indexer metrics collection |
| `shared/src/types.rs` | Shared type definitions across API, indexer, and UI |
| `shared/src/config.rs` | Shared configuration types |
| `shared/src/error.rs` | Shared error types |

---

## Deleted Files (Phase 0)

| File | Reason |
|------|--------|
| `csv-adapter/src/scalable_builder.rs` (old) | Replaced by `scalable_builder_v2.rs` → `scalable_builder.rs` |
| `csv-adapter-bitcoin/src/proofs.rs` (old) | Replaced by `proofs_new.rs` → `proofs.rs` (enhanced with merged implementation) |
| `csv-adapter-core/src/chain_registry.rs` | Absorbed by `ChainDiscovery` + `ChainRegistry` in `chain_adapter.rs` |
| `csv-adapter-core/src/chain_system.rs` | Absorbed by `ChainDiscovery` as `ChainCatalog` internal struct |
