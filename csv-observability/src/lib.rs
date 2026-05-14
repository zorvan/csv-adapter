//! CSV Observability
//!
//! This crate provides observability features for the CSV Protocol,
//! including metrics, logging, and monitoring.

pub mod logging;
pub mod metrics;

// Re-exports
pub use logging::{LogLevel, LogEntry, StructuredLogger, TraceSpan, Tracer};
pub use metrics::{RpcMetrics, ProviderMetrics};
