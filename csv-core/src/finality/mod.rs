//! Finality State Model
//!
//! This module provides a structured approach to defining and monitoring
//! different levels of transaction finality across chains.

pub mod state;
pub mod policy;
pub mod monitor;

// Re-exports
pub use state::{FinalityState, FinalityStatus};
pub use policy::{ChainFinalityPolicy, FinalityThreshold};
pub use monitor::FinalityMonitor;
