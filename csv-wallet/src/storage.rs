//! Re-export browser storage from csv-adapter-store.
//!
//! This module provides browser-specific localStorage persistence for the unified
//! storage format, enabling csv-wallet to share data with csv-cli.
//!
//! The actual implementation is in `csv-adapter-store` with the `browser-storage` feature.

pub use csv_adapter_store::browser_storage::*;

/// Backward-compatible type alias for storage error.
///
/// Previously `StorageError`, now re-exported as `BrowserStorageError` from csv-adapter-store.
pub type StorageError = BrowserStorageError;
