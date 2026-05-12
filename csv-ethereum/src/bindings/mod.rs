//! Ethereum Contract Bindings
//!
//! This module contains type-safe bindings for Ethereum smart contracts
//! generated using Alloy for ABI encoding/decoding.

// Placeholder for Alloy-generated bindings
// In production, this would contain generated types from:
// alloy-sol-types and alloy-contract

pub mod csv_lock;
pub mod csv_mint;

// Re-exports
pub use csv_lock::CsvLock;
pub use csv_mint::CsvMint;
