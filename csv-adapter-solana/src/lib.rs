//! Solana Adapter for CSV (Client-Side Validation)
//!
//! This adapter implements the AnchorLayer trait for Solana,
//! using program accounts as single-use seals and program instructions for commitment publication.

#![warn(missing_docs)]
#![allow(missing_docs)]
#![allow(dead_code)]

pub mod adapter;
pub mod config;
pub mod error;
pub mod mint;
pub mod program;
pub mod rpc;
pub mod seal;
pub mod types;
pub mod wallet;

pub use adapter::SolanaAnchorLayer;
pub use config::{Network, SolanaConfig};
pub use mint::mint_right_from_hex_key;
pub use rpc::SolanaRpc;
pub use types::{SolanaAnchorRef, SolanaFinalityProof, SolanaInclusionProof, SolanaSealRef};
pub use wallet::{ProgramWallet, WalletError};

#[cfg(feature = "rpc")]
pub use rpc::RealSolanaRpc;
