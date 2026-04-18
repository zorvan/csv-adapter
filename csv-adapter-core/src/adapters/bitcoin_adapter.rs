//! Bitcoin chain adapter implementation for the new interface.

use async_trait::async_trait;
use crate::chain_adapter::{ChainAdapter, ChainResult, ChainError, RpcClient, Wallet, ChainCapabilities, AccountModel};
use crate::chain_config::ChainConfig;
use crate::Chain;
use super::super::chain_system::ChainInfo;

/// Bitcoin chain adapter for the new scalable system
#[derive(Debug, Clone)]
pub struct BitcoinAdapter;

impl BitcoinAdapter {
    /// Create new Bitcoin adapter
    pub fn new() -> Self {
        Self
    }
    
    /// Get Bitcoin-specific chain info
    pub fn chain_info() -> ChainInfo {
        ChainInfo {
            chain_id: "bitcoin".to_string(),
            chain_name: "Bitcoin".to_string(),
            supports_nfts: true,
            supports_smart_contracts: false,
        }
    }
    
    /// Get Bitcoin network configuration
    pub fn network_config(network: &str) -> BitcoinNetworkConfig {
        match network {
            "mainnet" => BitcoinNetworkConfig::Mainnet,
            "testnet" => BitcoinNetworkConfig::Testnet,
            "regtest" => BitcoinNetworkConfig::Regtest,
            _ => BitcoinNetworkConfig::Mainnet,
        }
    }
    
    /// Get Bitcoin capabilities
    pub fn capabilities() -> ChainCapabilities {
        ChainCapabilities {
            supports_nfts: true,
            supports_smart_contracts: false,
            account_model: AccountModel::UTXO,
            confirmation_blocks: 6,
            max_batch_size: 100,
            supported_networks: vec!["mainnet".to_string(), "testnet".to_string(), "regtest".to_string()],
            supports_cross_chain: true,
            custom_features: Default::default(),
        }
    }
}

impl Default for BitcoinAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChainAdapter for BitcoinAdapter {
    fn chain_id(&self) -> &'static str {
        "bitcoin"
    }
    
    fn chain_name(&self) -> &'static str {
        "Bitcoin"
    }
    
    fn capabilities(&self) -> ChainCapabilities {
        Self::capabilities()
    }
    
    async fn create_client(&self, _config: &ChainConfig) -> ChainResult<Box<dyn RpcClient>> {
        Err(ChainError::NotImplemented("Bitcoin RPC client creation".to_string()))
    }
    
    async fn create_wallet(&self, _config: &ChainConfig) -> ChainResult<Box<dyn Wallet>> {
        Err(ChainError::NotImplemented("Bitcoin wallet creation".to_string()))
    }
    
    fn csv_program_id(&self) -> Option<&'static str> {
        None
    }
    
    fn to_core_chain(&self) -> Chain {
        Chain::Bitcoin
    }
    
    fn default_network(&self) -> &'static str {
        "mainnet"
    }
}

/// Bitcoin network configuration
#[derive(Debug, Clone)]
pub enum BitcoinNetworkConfig {
    /// Bitcoin main network
    Mainnet,
    /// Bitcoin test network
    Testnet,
    /// Bitcoin regression test network
    Regtest,
}

impl BitcoinNetworkConfig {
    /// Get the default RPC endpoint for this network
    pub fn default_rpc_endpoint(&self) -> &'static str {
        match self {
            Self::Mainnet => "https://blockstream.info/api",
            Self::Testnet => "https://blockstream.info/testnet/api",
            Self::Regtest => "http://localhost:8332",
        }
    }
    
    /// Get the default block explorer URL for this network
    pub fn default_block_explorer(&self) -> &'static str {
        match self {
            Self::Mainnet => "https://blockstream.info",
            Self::Testnet => "https://blockstream.info/testnet",
            Self::Regtest => "http://localhost:3000",
        }
    }
    
    /// Get the confirmation requirement for this network
    pub fn confirmations_required(&self) -> u32 {
        match self {
            Self::Mainnet => 6,
            Self::Testnet => 3,
            Self::Regtest => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bitcoin_adapter() {
        let adapter = BitcoinAdapter::new();
        assert_eq!(adapter.chain_id(), "bitcoin");
        assert_eq!(adapter.chain_name(), "Bitcoin");
    }
    
    #[test]
    fn test_bitcoin_chain_info() {
        let info = BitcoinAdapter::chain_info();
        assert_eq!(info.chain_id, "bitcoin");
        assert_eq!(info.chain_name, "Bitcoin");
        assert!(info.supports_nfts);
        assert!(!info.supports_smart_contracts);
    }
    
    #[test]
    fn test_bitcoin_network_config() {
        let mainnet = BitcoinNetworkConfig::Mainnet;
        assert_eq!(mainnet.default_rpc_endpoint(), "https://blockstream.info/api");
        assert_eq!(mainnet.confirmations_required(), 6);
        
        let testnet = BitcoinNetworkConfig::Testnet;
        assert_eq!(testnet.default_rpc_endpoint(), "https://blockstream.info/testnet/api");
        assert_eq!(testnet.confirmations_required(), 3);
        
        let regtest = BitcoinNetworkConfig::Regtest;
        assert_eq!(regtest.default_rpc_endpoint(), "http://localhost:8332");
        assert_eq!(regtest.confirmations_required(), 1);
    }
}
