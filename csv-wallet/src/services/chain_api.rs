//! Chain API service module.

use csv_core::Chain;

/// Chain API stub.
#[derive(Debug, Clone, Default)]
pub struct ChainApi;

/// Chain configuration.
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub api_url: String,
}

impl ChainConfig {
    pub fn for_chain(chain: Chain) -> Self {
        let rpc_url = match chain {
            Chain::Bitcoin => "https://bitcoin-rpc.example.com".to_string(),
            Chain::Ethereum => "https://ethereum-rpc.example.com".to_string(),
            Chain::Sui => "https://sui-rpc.example.com".to_string(),
            Chain::Aptos => "https://aptos-rpc.example.com".to_string(),
            Chain::Solana => "https://solana-rpc.example.com".to_string(),
        };
        Self {
            rpc_url: rpc_url.clone(),
            api_url: rpc_url,
        }
    }
}

impl ChainApi {
    pub fn new(_config: ChainConfig) -> Self {
        Self
    }

    pub async fn get_balance(&self, _address: &str, _chain: Chain) -> Result<String, String> {
        Ok("0".to_string())
    }
}
