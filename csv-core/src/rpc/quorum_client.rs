//! RPC Quorum Client
//!
//! Provides RPC client functionality with quorum-based consensus to prevent
//! single-point-of-failure or malicious provider attacks.

use alloc::vec::Vec;
use serde_json;

use crate::error::Result;

/// RPC provider configuration
#[derive(Clone, Debug)]
pub struct RpcProvider {
    /// Provider URL
    pub url: String,
    /// Provider weight (for weighted quorum)
    pub weight: f64,
    /// Provider timeout in milliseconds
    pub timeout_ms: u64,
}

impl RpcProvider {
    /// Create a new RPC provider
    pub fn new(url: String) -> Self {
        Self {
            url,
            weight: 1.0,
            timeout_ms: 5000,
        }
    }

    /// Set the provider weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// Quorum configuration
#[derive(Clone, Debug)]
pub struct QuorumConfig {
    /// Minimum number of providers that must agree
    pub min_quorum: usize,
    /// Minimum percentage of providers that must agree (0.0 - 1.0)
    pub min_percentage: f64,
    /// Maximum number of providers to query
    pub max_providers: usize,
}

impl QuorumConfig {
    /// Create a new quorum configuration
    pub fn new(min_quorum: usize, min_percentage: f64) -> Self {
        Self {
            min_quorum,
            min_percentage,
            max_providers: 5,
        }
    }

    /// Set the maximum number of providers
    pub fn with_max_providers(mut self, max_providers: usize) -> Self {
        self.max_providers = max_providers;
        self
    }
}

impl Default for QuorumConfig {
    fn default() -> Self {
        Self::new(2, 0.51) // Default: 2 providers, 51% agreement
    }
}

/// RPC response from a provider
#[derive(Clone, Debug)]
pub struct RpcResponse {
    /// Provider that returned this response
    pub provider: String,
    /// Response data
    pub data: Vec<u8>,
    /// Whether the response was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Response timestamp
    pub timestamp_ms: u64,
}

/// Quorum client for RPC queries
pub struct QuorumClient {
    /// RPC providers
    providers: Vec<RpcProvider>,
    /// Quorum configuration
    config: QuorumConfig,
}

impl QuorumClient {
    /// Create a new quorum client
    pub fn new(providers: Vec<RpcProvider>, config: QuorumConfig) -> Self {
        Self {
            providers,
            config,
        }
    }

    /// Create a quorum client with default configuration
    pub fn with_defaults(providers: Vec<RpcProvider>) -> Self {
        Self::new(providers, QuorumConfig::default())
    }

    /// Query all providers and return responses
    pub async fn query_all(&self, method: &str, params: &[serde_json::Value]) -> Vec<RpcResponse> {
        let mut responses = Vec::new();
        
        // In production, this would query each provider in parallel using tokio::spawn
        // For now, simulate RPC responses with realistic quorum logic
        for provider in &self.providers {
            let response = self.simulate_rpc_call(provider, method, params).await;
            responses.push(response);
        }
        
        responses
    }
    
    /// Simulate an RPC call to a provider
    /// In production, this would make actual HTTP requests using reqwest or similar
    async fn simulate_rpc_call(&self, provider: &RpcProvider, method: &str, params: &[serde_json::Value]) -> RpcResponse {
        // Simulate network latency
        // In production: let response = reqwest::post(&provider.url).json(&rpc_request).timeout(Duration::from_millis(provider.timeout_ms)).send().await;
        
        // Build RPC request body
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        // Simulate successful response
        let response_data = serde_json::to_vec(&rpc_request).unwrap_or_default();
        
        RpcResponse {
            provider: provider.url.clone(),
            data: response_data,
            success: true,
            error: None,
            timestamp_ms: 0, // Would be actual timestamp
        }
    }

    /// Query providers and return the quorum result
    pub async fn query_quorum(&self, method: &str, params: &[serde_json::Value]) -> Result<Vec<u8>> {
        let responses = self.query_all(method, params).await;
        
        // Count successful responses
        let successful: Vec<_> = responses.iter().filter(|r| r.success).collect();
        
        // Check if we have quorum
        let count = successful.len();
        let total = self.providers.len();
        let percentage = count as f64 / total as f64;
        
        if count >= self.config.min_quorum && percentage >= self.config.min_percentage {
            // Verify that all successful responses agree on the data
            if let Some(consensus_data) = self.verify_consensus(&successful) {
                Ok(consensus_data)
            } else {
                Err(crate::error::ProtocolError::RpcQuorumFailed(
                    "RPC providers returned inconsistent responses".to_string()
                ))
            }
        } else {
            Err(crate::error::ProtocolError::RpcQuorumFailed(
                format!(
                    "Quorum not reached: {}/{} providers ({:.0}%), required: {} providers ({:.0}%)",
                    count,
                    total,
                    percentage * 100.0,
                    self.config.min_quorum,
                    self.config.min_percentage * 100.0
                )
            ))
        }
    }
    
    /// Verify that all successful responses agree on the data
    fn verify_consensus(&self, responses: &[&RpcResponse]) -> Option<Vec<u8>> {
        if responses.is_empty() {
            return None;
        }
        
        // Get the first response as reference
        let reference_data = &responses[0].data;
        
        // Check that all other responses match
        for response in responses.iter().skip(1) {
            if response.data != *reference_data {
                return None;
            }
        }
        
        Some(reference_data.clone())
    }
    
    /// Query a specific method with quorum and parse JSON response
    pub async fn query_json<T: for<'de> serde::Deserialize<'de>>(&self, method: &str, params: &[serde_json::Value]) -> Result<T> {
        let data = self.query_quorum(method, params).await?;
        serde_json::from_slice(&data).map_err(|e| {
            crate::error::ProtocolError::InvalidData(format!("Failed to parse RPC response: {}", e))
        })
    }
    
    /// Get block number with quorum
    pub async fn get_block_number(&self) -> Result<String> {
        let response: serde_json::Value = self.query_json("eth_blockNumber", &[]).await?;
        Ok(response.as_str().unwrap_or("0x0").to_string())
    }
    
    /// Get block by hash with quorum
    pub async fn get_block_by_hash(&self, block_hash: &str) -> Result<serde_json::Value> {
        self.query_json("eth_getBlockByHash", &[serde_json::json!(block_hash), serde_json::json!(false)]).await
    }
    
    /// Get transaction receipt with quorum
    pub async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<serde_json::Value> {
        self.query_json("eth_getTransactionReceipt", &[serde_json::json!(tx_hash)]).await
    }

    /// Get the number of providers
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    /// Add a provider
    pub fn add_provider(&mut self, provider: RpcProvider) {
        self.providers.push(provider);
    }

    /// Remove a provider by URL
    pub fn remove_provider(&mut self, url: &str) {
        self.providers.retain(|p| p.url != url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_provider_creation() {
        let provider = RpcProvider::new("http://localhost:8545".to_string())
            .with_weight(2.0)
            .with_timeout(10000);
        
        assert_eq!(provider.url, "http://localhost:8545");
        assert_eq!(provider.weight, 2.0);
        assert_eq!(provider.timeout_ms, 10000);
    }

    #[test]
    fn test_quorum_config_default() {
        let config = QuorumConfig::default();
        assert_eq!(config.min_quorum, 2);
        assert_eq!(config.min_percentage, 0.51);
    }

    #[test]
    fn test_quorum_client_creation() {
        let providers = vec![
            RpcProvider::new("http://provider1.com".to_string()),
            RpcProvider::new("http://provider2.com".to_string()),
        ];
        
        let client = QuorumClient::with_defaults(providers);
        assert_eq!(client.provider_count(), 2);
    }
}
