//! Aptos chain adapter implementation for the scalable system.

use async_trait::async_trait;
use crate::chain_adapter::{ChainAdapter, ChainResult, ChainError, RpcClient, Wallet, ChainCapabilities, AccountModel};
use crate::chain_config::ChainConfig;
use crate::Chain;

/// Aptos chain adapter for the scalable system
#[derive(Debug, Clone)]
pub struct AptosAdapter;

impl AptosAdapter {
    /// Create new Aptos adapter
    pub fn new() -> Self {
        Self
    }
    
    /// Get Aptos capabilities
    pub fn capabilities() -> ChainCapabilities {
        ChainCapabilities {
            supports_nfts: true,
            supports_smart_contracts: true,
            account_model: AccountModel::Object,
            confirmation_blocks: 1,
            max_batch_size: 100,
            supported_networks: vec!["mainnet".to_string(), "testnet".to_string(), "devnet".to_string()],
            supports_cross_chain: true,
            custom_features: Default::default(),
        }
    }
}

impl Default for AptosAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChainAdapter for AptosAdapter {
    fn chain_id(&self) -> &'static str {
        "aptos"
    }
    
    fn chain_name(&self) -> &'static str {
        "Aptos"
    }
    
    fn capabilities(&self) -> ChainCapabilities {
        Self::capabilities()
    }
    
    async fn create_client(&self, _config: &ChainConfig) -> ChainResult<Box<dyn RpcClient>> {
        Err(ChainError::NotImplemented("Aptos RPC client creation".to_string()))
    }
    
    async fn create_wallet(&self, _config: &ChainConfig) -> ChainResult<Box<dyn Wallet>> {
        Err(ChainError::NotImplemented("Aptos wallet creation".to_string()))
    }
    
    fn csv_program_id(&self) -> Option<&'static str> {
        Some("0x1::csv_seal::SealStore")
    }
    
    fn to_core_chain(&self) -> Chain {
        Chain::Aptos
    }
    
    fn default_network(&self) -> &'static str {
        "testnet"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aptos_adapter() {
        let adapter = AptosAdapter::new();
        assert_eq!(adapter.chain_id(), "aptos");
        assert_eq!(adapter.chain_name(), "Aptos");
        assert_eq!(adapter.to_core_chain(), Chain::Aptos);
    }
    
    #[test]
    fn test_aptos_capabilities() {
        let caps = AptosAdapter::capabilities();
        assert!(caps.supports_nfts);
        assert!(caps.supports_smart_contracts);
        assert!(caps.supports_cross_chain);
        assert_eq!(caps.confirmation_blocks, 1);
    }
}
