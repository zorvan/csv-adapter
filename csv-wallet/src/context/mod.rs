//! Application context and state management - modular structure.

pub mod state;
pub mod types;
pub mod utils;
pub mod wallet;

// Re-export all types from types module
pub use types::*;

// Re-export AppState from state module

// Re-export WalletContext and related items from wallet module
pub use wallet::{use_wallet_context, WalletContext, WalletProvider};

// Re-export utility functions
pub use utils::generate_id;
