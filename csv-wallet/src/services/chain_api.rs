//! ChainId API service stub - re-exported for wallet compatibility.

use csv_store::state::ChainId;

/// ChainId API stub.
#[derive(Debug, Clone, Default)]
pub struct ChainApi;

/// ChainId configuration.
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub api_url: String,
}

impl ChainConfig {
    /// Get chain config.
    pub fn for_chain(chain: &ChainId) -> Self {
        let rpc_url = match chain.as_str() {
            "bitcoin" => "https://bitcoin-rpc.example.com".to_string(),
            "ethereum" => "https://ethereum-rpc.example.com".to_string(),
            "sui" => "https://sui-rpc.example.com".to_string(),
            "aptos" => "https://aptos-rpc.example.com".to_string(),
            "solana" => "https://solana-rpc.example.com".to_string(),
            _ => "https://unknown-rpc.example.com".to_string(),
        };
        Self {
            rpc_url: rpc_url.clone(),
            api_url: rpc_url,
        }
    }
}

impl ChainApi {
    /// Create new chain API.
    pub fn new(_config: ChainConfig) -> Self {
        Self
    }

    /// Get balance stub.
    pub async fn get_balance(&self, _address: &str, _chain: ChainId) -> Result<String, String> {
        Ok("0".to_string())
    }
}
