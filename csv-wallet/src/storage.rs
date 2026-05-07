//! Re-export browser storage from csv-store.
//!
//! This module provides browser-specific localStorage persistence for the unified
//! storage format, enabling csv-wallet to share data with csv-cli.
//!
//! The actual implementation is in `csv-store` with the `browser-storage` feature.

pub use csv_store::browser_storage::*;

/// Backward-compatible type alias for storage error.
pub type StorageError = BrowserStorageError;
