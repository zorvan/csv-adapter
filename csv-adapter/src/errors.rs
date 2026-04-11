//! Unified error types for the CSV Adapter meta-crate.
//!
//! This module provides a single error enum that wraps all error sources
//! from the underlying crates, with integration to [`ErrorSuggestion`]
//! from agent_types for machine-actionable fix suggestions.

use thiserror::Error;

use csv_adapter_core::agent_types::{ErrorSuggestion, FixAction};
use csv_adapter_core::Chain;

/// Unified error type for all CSV operations.
///
/// Every variant integrates with [`ErrorSuggestion`] to provide
/// machine-actionable fix hints for autonomous agents.
#[derive(Error, Debug)]
pub enum CsvError {
    /// The requested chain is not supported or not enabled.
    #[error("Chain not supported: {0}")]
    ChainNotSupported(Chain),

    /// The wallet has insufficient funds for the operation.
    #[error("Insufficient funds: available {available}, needed {needed}")]
    InsufficientFunds {
        /// Available balance (chain-specific units).
        available: String,
        /// Required balance.
        needed: String,
        /// Which chain the funds are needed on.
        chain: Chain,
    },

    /// The specified Right ID is invalid or malformed.
    #[error("Invalid Right ID: {0}")]
    InvalidRightId(String),

    /// The specified Right was not found.
    #[error("Right not found: {0}")]
    RightNotFound(String),

    /// The specified transfer was not found.
    #[error("Transfer not found: {0}")]
    TransferNotFound(String),

    /// A Right has already been consumed (single-use violation).
    #[error("Right already consumed: {0}")]
    RightAlreadyConsumed(String),

    /// The commitment hash is invalid.
    #[error("Invalid commitment: {0}")]
    InvalidCommitment(String),

    /// Proof verification failed.
    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),

    /// The wallet operation failed.
    #[error("Wallet error: {0}")]
    WalletError(String),

    /// Network or RPC communication failed.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Serialization or deserialization failed.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// The store backend operation failed.
    #[error("Store error: {0}")]
    StoreError(String),

    /// A builder validation error.
    #[error("Builder validation error: {0}")]
    BuilderError(String),

    /// An event stream error.
    #[error("Event stream error: {0}")]
    EventStreamError(String),

    /// A chain-specific adapter error wrapped from the underlying crate.
    #[error("Adapter error on {chain}: {message}")]
    AdapterError {
        /// Which chain the error occurred on.
        chain: Chain,
        /// Human-readable error message.
        message: String,
    },

    /// A generic error with a message.
    #[error("CSV error: {0}")]
    Generic(String),
}

impl CsvError {
    /// Convert this error into an [`ErrorSuggestion`] for agent consumption.
    ///
    /// Each variant maps to a machine-readable error code and a suggested
    /// fix action where applicable.
    pub fn to_suggestion(&self) -> ErrorSuggestion {
        match self {
            Self::ChainNotSupported(chain) => ErrorSuggestion::new(
                "CSV_001",
                format!("Chain {} is not supported or not enabled", chain),
                "https://docs.csv.dev/errors/CSV_001",
            ),
            Self::InsufficientFunds { chain, needed, .. } => {
                let mut suggestion = ErrorSuggestion::new(
                    "CSV_002",
                    format!("Insufficient funds on {} chain", chain),
                    "https://docs.csv.dev/errors/CSV_002",
                );
                // Add faucet hint for testnets
                suggestion = suggestion.with_fix(FixAction::FundFromFaucet {
                    url: format!("https://faucet.{}.csv.dev", chain),
                    amount: needed.clone(),
                });
                suggestion
            }
            Self::InvalidRightId(id) => ErrorSuggestion::new(
                "CSV_003",
                format!("Invalid Right ID: {}", id),
                "https://docs.csv.dev/errors/CSV_003",
            ),
            Self::RightNotFound(id) => ErrorSuggestion::new(
                "CSV_004",
                format!("Right not found: {}. Rights exist in client state, not on-chain.", id),
                "https://docs.csv.dev/errors/CSV_004",
            ),
            Self::TransferNotFound(id) => ErrorSuggestion::new(
                "CSV_005",
                format!("Transfer not found: {}", id),
                "https://docs.csv.dev/errors/CSV_005",
            ),
            Self::RightAlreadyConsumed(id) => ErrorSuggestion::new(
                "CSV_006",
                format!("Right {} has already been consumed (single-use seal violated)", id),
                "https://docs.csv.dev/errors/CSV_006",
            ),
            Self::InvalidCommitment(msg) => ErrorSuggestion::new(
                "CSV_007",
                format!("Invalid commitment: {}", msg),
                "https://docs.csv.dev/errors/CSV_007",
            ),
            Self::ProofVerificationFailed(msg) => {
                let mut suggestion = ErrorSuggestion::new(
                    "CSV_008",
                    format!("Proof verification failed: {}", msg),
                    "https://docs.csv.dev/errors/CSV_008",
                );
                suggestion = suggestion.with_fix(FixAction::CheckState {
                    url: "https://docs.csv.dev/proof-verification".to_string(),
                    what: "Check source chain confirmations and proof format".to_string(),
                });
                suggestion
            }
            Self::WalletError(msg) => ErrorSuggestion::new(
                "CSV_009",
                format!("Wallet error: {}", msg),
                "https://docs.csv.dev/errors/CSV_009",
            ),
            Self::NetworkError(msg) => {
                let mut suggestion = ErrorSuggestion::new(
                    "CSV_010",
                    format!("Network error: {}", msg),
                    "https://docs.csv.dev/errors/CSV_010",
                );
                suggestion = suggestion.with_fix(FixAction::Retry {
                    parameter_changes: std::collections::HashMap::new(),
                });
                suggestion
            }
            Self::SerializationError(msg) => ErrorSuggestion::new(
                "CSV_011",
                format!("Serialization error: {}", msg),
                "https://docs.csv.dev/errors/CSV_011",
            ),
            Self::ConfigError(msg) => ErrorSuggestion::new(
                "CSV_012",
                format!("Configuration error: {}", msg),
                "https://docs.csv.dev/errors/CSV_012",
            ),
            Self::StoreError(msg) => ErrorSuggestion::new(
                "CSV_013",
                format!("Store error: {}", msg),
                "https://docs.csv.dev/errors/CSV_013",
            ),
            Self::BuilderError(msg) => ErrorSuggestion::new(
                "CSV_014",
                format!("Builder validation error: {}", msg),
                "https://docs.csv.dev/errors/CSV_014",
            ),
            Self::EventStreamError(msg) => ErrorSuggestion::new(
                "CSV_015",
                format!("Event stream error: {}", msg),
                "https://docs.csv.dev/errors/CSV_015",
            ),
            Self::AdapterError { chain, message } => ErrorSuggestion::new(
                "CSV_016",
                format!("Adapter error on {}: {}", chain, message),
                "https://docs.csv.dev/errors/CSV_016",
            ),
            Self::Generic(msg) => ErrorSuggestion::new(
                "CSV_099",
                format!("CSV error: {}", msg),
                "https://docs.csv.dev/errors/CSV_099",
            ),
        }
    }

    /// Check if this error is retryable (transient).
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_) | Self::StoreError(_) | Self::AdapterError { .. }
        )
    }

    /// Check if this error indicates an insufficient funds condition.
    pub fn is_insufficient_funds(&self) -> bool {
        matches!(self, Self::InsufficientFunds { .. })
    }
}

// Conversion from csv-adapter-core errors
impl From<csv_adapter_core::AdapterError> for CsvError {
    fn from(err: csv_adapter_core::AdapterError) -> Self {
        CsvError::Generic(err.to_string())
    }
}

impl From<csv_adapter_core::StoreError> for CsvError {
    fn from(err: csv_adapter_core::StoreError) -> Self {
        CsvError::StoreError(err.to_string())
    }
}

// Conversion from common Rust errors
impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for CsvError {
    fn from(err: serde_json::Error) -> Self {
        CsvError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for CsvError {
    fn from(err: toml::de::Error) -> Self {
        CsvError::ConfigError(err.to_string())
    }
}

impl From<hex::FromHexError> for CsvError {
    fn from(err: hex::FromHexError) -> Self {
        CsvError::Generic(format!("Hex decoding error: {}", err))
    }
}
