//! RPC Metrics
//!
//! This module provides metrics collection for RPC operations,
//! tracking latency, success rates, and provider health.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// RPC operation metrics
#[derive(Clone, Debug)]
pub struct RpcMetrics {
    /// Total number of requests
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Total latency in milliseconds
    pub total_latency_ms: u64,
    /// Provider-specific metrics
    pub provider_metrics: BTreeMap<String, ProviderMetrics>,
}

/// Provider-specific metrics
#[derive(Clone, Debug)]
pub struct ProviderMetrics {
    /// Provider URL
    pub url: String,
    /// Number of requests to this provider
    pub requests: u64,
    /// Number of successful requests
    pub successful: u64,
    /// Number of failed requests
    pub failed: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Last successful response timestamp
    pub last_success: Option<u64>,
    /// Last failure timestamp
    pub last_failure: Option<u64>,
}

impl RpcMetrics {
    /// Create new RPC metrics
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_latency_ms: 0,
            provider_metrics: BTreeMap::new(),
        }
    }

    /// Record a successful RPC request
    pub fn record_success(&mut self, provider: &str, latency_ms: u64) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.total_latency_ms += latency_ms;

        let metrics = self.provider_metrics.entry(provider.to_string()).or_insert_with(|| {
            ProviderMetrics {
                url: provider.to_string(),
                requests: 0,
                successful: 0,
                failed: 0,
                avg_latency_ms: 0.0,
                last_success: None,
                last_failure: None,
            }
        });

        metrics.requests += 1;
        metrics.successful += 1;
        metrics.last_success = Some(self.current_timestamp());
        
        // Update average latency
        let total_latency = metrics.avg_latency_ms * (metrics.requests - 1) as f64;
        metrics.avg_latency_ms = (total_latency + latency_ms as f64) / metrics.requests as f64;
    }

    /// Record a failed RPC request
    pub fn record_failure(&mut self, provider: &str) {
        self.total_requests += 1;
        self.failed_requests += 1;

        let metrics = self.provider_metrics.entry(provider.to_string()).or_insert_with(|| {
            ProviderMetrics {
                url: provider.to_string(),
                requests: 0,
                successful: 0,
                failed: 0,
                avg_latency_ms: 0.0,
                last_success: None,
                last_failure: None,
            }
        });

        metrics.requests += 1;
        metrics.failed += 1;
        metrics.last_failure = Some(self.current_timestamp());
    }

    /// Get the success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Get the average latency
    pub fn avg_latency_ms(&self) -> f64 {
        if self.successful_requests == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.successful_requests as f64
        }
    }

    /// Get provider metrics
    pub fn get_provider_metrics(&self, provider: &str) -> Option<&ProviderMetrics> {
        self.provider_metrics.get(provider)
    }

    /// Get all provider metrics
    pub fn all_provider_metrics(&self) -> Vec<&ProviderMetrics> {
        self.provider_metrics.values().collect()
    }

    fn current_timestamp(&self) -> u64 {
        // Placeholder - would use actual system time
        0
    }
}

impl Default for RpcMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_metrics_creation() {
        let metrics = RpcMetrics::new();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.successful_requests, 0);
    }

    #[test]
    fn test_record_success() {
        let mut metrics = RpcMetrics::new();
        metrics.record_success("provider1", 100);
        
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.success_rate(), 1.0);
    }

    #[test]
    fn test_record_failure() {
        let mut metrics = RpcMetrics::new();
        metrics.record_failure("provider1");
        
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_avg_latency() {
        let mut metrics = RpcMetrics::new();
        metrics.record_success("provider1", 100);
        metrics.record_success("provider1", 200);
        
        assert_eq!(metrics.avg_latency_ms(), 150.0);
    }
}
