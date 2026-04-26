//! Page components - modular structure.
//! 
//! Organized into feature modules:
//! - `accounts` - Dashboard and account management
//! - `rights` - Rights management (list, create, show, transfer, consume)
//! - `proofs` - Proof generation and verification
//! - `cross_chain` - Cross-chain transfers
//! - `contracts` - Contract deployment and management
//! - `seals` - Seal creation and verification
//! - `tests` - Test scenarios
//! - `validate` - Validation utilities
//! - `transactions` - Transaction history
//! - `settings` - Application settings
//! - `common` - Shared UI helpers
//!
//! Note: During the migration from old_pages.rs, some modules re-export
//! components from old_pages. These will be fully migrated incrementally.

// Common UI helpers (fully migrated)
pub mod common;

// NFT and Wallet pages (already separate files)
pub mod nft_page;
pub mod wallet_page;

// Feature modules (re-exporting from old_pages during migration)
pub mod accounts;
pub mod rights;
pub mod proofs;
pub mod cross_chain;
pub mod contracts;
pub mod seals;
pub mod tests;
pub mod validate;
pub mod transactions;
pub mod settings;

// Re-exports from nft_page and wallet_page (standalone files)
pub use nft_page::{NftCollections, NftDetail, NftGallery};
pub use wallet_page::WalletPage;

// Re-exports from accounts module
pub use accounts::{Dashboard, AccountTransactions};

// Re-exports from rights module (already migrated)
pub use rights::{
    Rights, CreateRight, ShowRight, 
    TransferRight, ConsumeRight, RightJourney,
};

// Re-exports from proofs module
pub use proofs::{
    Proofs, GenerateProof, VerifyProof, VerifyCrossChainProof,
};

// Re-exports from cross_chain module
pub use cross_chain::{
    CrossChain, CrossChainTransfer, CrossChainStatus, CrossChainRetry,
};

// Re-exports from contracts module
pub use contracts::{
    Contracts, DeployContract, AddContract, ContractStatus,
};

// Re-exports from seals module
pub use seals::{
    Seals, CreateSeal, ConsumeSeal, VerifySeal,
};

// Re-exports from tests module
pub use tests::{
    Test, RunTests, RunScenario,
};

// Re-exports from validate module
pub use validate::{
    Validate, ValidateConsignment, ValidateProof, ValidateSeal, ValidateCommitmentChain,
};

// Re-exports from transactions module
pub use transactions::{
    Transactions, TransactionDetail,
};

// Re-exports from settings module
pub use settings::Settings;

// Common UI helpers - re-export everything from common module for convenience

// Migration complete: old_pages.rs has been removed
