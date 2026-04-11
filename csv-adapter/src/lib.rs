//! CSV Adapter — Unified Meta-Crate
//!
//! This crate provides a single entry point for all CSV (Client-Side Validation)
//! operations, unifying the individual chain adapter crates behind a coherent,
//! ergonomic API.
//!
//! # Architecture
//!
//! ```text
//! csv-adapter (this crate)
//! ├── csv-adapter-core       (always included)
//! ├── csv-adapter-bitcoin    (optional, feature: "bitcoin")
//! ├── csv-adapter-ethereum   (optional, feature: "ethereum")
//! ├── csv-adapter-sui        (optional, feature: "sui")
//! ├── csv-adapter-aptos      (optional, feature: "aptos")
//! └── csv-adapter-store      (optional, feature: "sqlite")
//! ```
//!
//! # Quick Start
//!
//! ```no_run
//! use csv_adapter::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Build a client with Bitcoin support
//!     let client = CsvClient::builder()
//!         .with_chain(Chain::Bitcoin)
//!         .with_store_backend(StoreBackend::InMemory)
//!         .build()?;
//!
//!     // Access managers
//!     let rights = client.rights();
//!     let transfers = client.transfers();
//!     let proofs = client.proofs();
//!
//!     Ok(())
//! }
//! ```
//!
//! # Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `bitcoin` | Enable Bitcoin adapter |
//! | `ethereum` | Enable Ethereum adapter |
//! | `sui` | Enable Sui adapter |
//! | `aptos` | Enable Aptos adapter |
//! | `all-chains` | Enable all chain adapters |
//! | `tokio` | Enable tokio async runtime (default) |
//! | `async-std` | Enable async-std runtime |
//! | `sqlite` | Enable SQLite persistence |
//! | `in-memory` | Enable in-memory store backend |
//! | `wallet` | Enable unified wallet management |
//!
//! # Key Concepts
//!
//! - **Right**: A verifiable, single-use digital right that can be transferred
//!   cross-chain. Exists in client state, not on any chain.
//! - **Seal**: The on-chain mechanism that enforces a Right's single-use.
//!   Chain-specific and exists on one chain only.
//! - **Client-Side Validation (CSV)**: The client does the verification, not
//!   the blockchain. The chain only records commitments and enforces single-use.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

// Internal modules
pub mod builder;
pub mod client;
pub mod config;
pub mod errors;
pub mod events;
pub mod prelude;
pub mod proofs;
pub mod rights;
pub mod transfers;
pub mod wallet;

// Re-export core types from csv-adapter-core
pub use csv_adapter_core::{
    AnchorLayer, AnchorRecord, AnchorRef, Commitment, Consignment, DAGNode, DAGSegment,
    FinalityProof, Genesis, Hash, InMemorySealStore, InclusionProof, MpcLeaf, MpcProof, MpcTree,
    OwnedState, OwnershipProof, ProofBundle, Right, RightId, Schema, SealRecord, SealRef,
    SealStore, StateRef, Transition, CONSIGNMENT_VERSION, SCHEMA_VERSION,
};

// Re-export agent-friendly types
pub use csv_adapter_core::agent_types::{Chain, ErrorSuggestion, FixAction, TransferStatus};

// Re-export error types
pub use csv_adapter_core::{AdapterError, Result as CoreResult, StoreError};

// Re-export our unified error type
pub use errors::CsvError;

/// Unified result type alias.
///
/// Equivalent to `Result<T, CsvError>`.
pub type Result<T> = core::result::Result<T, CsvError>;
