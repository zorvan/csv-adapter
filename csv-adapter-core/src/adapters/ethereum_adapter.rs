//! Ethereum chain adapter implementation for the scalable system.

use crate::chain_adapter::{
    AccountModel, ChainAdapter, ChainCapabilities, ChainError, ChainResult, RpcClient, Wallet,
};
use crate::chain_config::ChainConfig;
use crate::Chain;
use async_trait::async_trait;

/// Ethereum chain adapter for the scalable system
#[derive(Debug, Clone)]
pub struct EthereumAdapter;

impl EthereumAdapter {
    /// Create new Ethereum adapter
    pub fn new() -> Self {
        Self
    }

    /// Get Ethereum capabilities
    pub fn capabilities() -> ChainCapabilities {
        ChainCapabilities {
            supports_nfts: true,
            supports_smart_contracts: true,
            account_model: AccountModel::Account,
            confirmation_blocks: 12,
            max_batch_size: 100,
            supported_networks: vec![
                "mainnet".to_string(),
                "sepolia".to_string(),
                "goerli".to_string(),
            ],
            supports_cross_chain: true,
            custom_features: Default::default(),
        }
    }
}

impl Default for EthereumAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChainAdapter for EthereumAdapter {
    fn chain_id(&self) -> &'static str {
        "ethereum"
    }

    fn chain_name(&self) -> &'static str {
        "Ethereum"
    }

    fn capabilities(&self) -> ChainCapabilities {
        Self::capabilities()
    }

    async fn create_client(&self, _config: &ChainConfig) -> ChainResult<Box<dyn RpcClient>> {
        Err(ChainError::NotImplemented(
            "Ethereum RPC client creation".to_string(),
        ))
    }

    async fn create_wallet(&self, _config: &ChainConfig) -> ChainResult<Box<dyn Wallet>> {
        Err(ChainError::NotImplemented(
            "Ethereum wallet creation".to_string(),
        ))
    }

    fn csv_program_id(&self) -> Option<&'static str> {
        Some("0xCsvSeal00000000000000000000000000000000")
    }

    fn to_core_chain(&self) -> Chain {
        Chain::Ethereum
    }

    fn default_network(&self) -> &'static str {
        "mainnet"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_adapter() {
        let adapter = EthereumAdapter::new();
        assert_eq!(adapter.chain_id(), "ethereum");
        assert_eq!(adapter.chain_name(), "Ethereum");
        assert_eq!(adapter.to_core_chain(), Chain::Ethereum);
    }

    #[test]
    fn test_ethereum_capabilities() {
        let caps = EthereumAdapter::capabilities();
        assert!(caps.supports_nfts);
        assert!(caps.supports_smart_contracts);
        assert!(caps.supports_cross_chain);
        assert_eq!(caps.confirmation_blocks, 12);
    }
}
