//! Sui chain adapter implementation for the scalable system.

use crate::chain_adapter::{
    AccountModel, ChainAdapter, ChainCapabilities, ChainError, ChainResult, RpcClient, Wallet,
};
use crate::chain_config::ChainConfig;
use crate::Chain;
use async_trait::async_trait;

/// Sui chain adapter for the scalable system
#[derive(Debug, Clone)]
pub struct SuiAdapter;

impl SuiAdapter {
    /// Create new Sui adapter
    pub fn new() -> Self {
        Self
    }

    /// Get Sui capabilities
    pub fn capabilities() -> ChainCapabilities {
        ChainCapabilities {
            supports_nfts: true,
            supports_smart_contracts: true,
            account_model: AccountModel::Object,
            confirmation_blocks: 3,
            max_batch_size: 100,
            supported_networks: vec![
                "mainnet".to_string(),
                "testnet".to_string(),
                "devnet".to_string(),
            ],
            supports_cross_chain: true,
            custom_features: Default::default(),
        }
    }
}

impl Default for SuiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChainAdapter for SuiAdapter {
    fn chain_id(&self) -> &'static str {
        "sui"
    }

    fn chain_name(&self) -> &'static str {
        "Sui"
    }

    fn capabilities(&self) -> ChainCapabilities {
        Self::capabilities()
    }

    async fn create_client(&self, _config: &ChainConfig) -> ChainResult<Box<dyn RpcClient>> {
        Err(ChainError::NotImplemented(
            "Sui RPC client creation".to_string(),
        ))
    }

    async fn create_wallet(&self, _config: &ChainConfig) -> ChainResult<Box<dyn Wallet>> {
        Err(ChainError::NotImplemented(
            "Sui wallet creation".to_string(),
        ))
    }

    fn csv_program_id(&self) -> Option<&'static str> {
        Some("0xcsv::seal::SealContract")
    }

    fn to_core_chain(&self) -> Chain {
        Chain::Sui
    }

    fn default_network(&self) -> &'static str {
        "testnet"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sui_adapter() {
        let adapter = SuiAdapter::new();
        assert_eq!(adapter.chain_id(), "sui");
        assert_eq!(adapter.chain_name(), "Sui");
        assert_eq!(adapter.to_core_chain(), Chain::Sui);
    }

    #[test]
    fn test_sui_capabilities() {
        let caps = SuiAdapter::capabilities();
        assert!(caps.supports_nfts);
        assert!(caps.supports_smart_contracts);
        assert!(caps.supports_cross_chain);
        assert_eq!(caps.confirmation_blocks, 3);
    }
}
