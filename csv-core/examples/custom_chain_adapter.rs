use std::collections::HashMap;

use async_trait::async_trait;

use csv_core::{
    AccountModel, ChainDriver, ChainCapabilities, ChainConfig, ChainError,
    ChainId, ChainResult, DriverRegistry, RpcClient, Wallet,
};

#[derive(Debug, Clone)]
struct ExampleChainDriver;

#[async_trait]
impl ChainDriver for ExampleChainDriver {
    fn chain_id(&self) -> &'static str {
        "example-chain"
    }

    fn chain_name(&self) -> &'static str {
        "Example Chain"
    }

    fn capabilities(&self) -> ChainCapabilities {
        ChainCapabilities {
            supports_nfts: true,
            supports_smart_contracts: true,
            account_model: AccountModel::Account,
            confirmation_blocks: 8,
            max_batch_size: 128,
            supported_networks: vec!["devnet".to_string(), "mainnet".to_string()],
            supports_cross_chain: true,
            custom_features: HashMap::new(),
        }
    }

    async fn create_client(&self, _config: &ChainConfig) -> ChainResult<Box<dyn RpcClient>> {
        Err(ChainError::FeatureNotEnabled("Example chain RPC client not available".to_string()))
    }

    async fn create_wallet(&self, _config: &ChainConfig) -> ChainResult<Box<dyn Wallet>> {
        Err(ChainError::FeatureNotEnabled("Example chain wallet not available".to_string()))
    }

    fn csv_program_id(&self) -> Option<&'static str> {
        Some("example-program")
    }

    fn to_core_chain(&self) -> ChainId {
        ChainId::new("solana")
    }

    fn default_network(&self) -> &'static str {
        "devnet"
    }
}

fn example_config() -> ChainConfig {
    ChainConfig {
        chain_id: "example-chain".to_string(),
        chain_name: "Example Chain".to_string(),
        default_network: "devnet".to_string(),
        rpc_endpoints: vec!["https://rpc.example-chain.devnet".to_string()],
        program_id: Some("example-program".to_string()),
        block_explorer_urls: vec!["https://explorer.example-chain.devnet".to_string()],
        start_block: 0,
        capabilities: ExampleChainDriver.capabilities(),
        custom_settings: HashMap::new(),
    }
}

fn main() {
    let mut registry = DriverRegistry::new();
    registry.register("example-chain", || {
        let config = example_config();
        println!(
            "Creating adapter for {} on {}",
            config.chain_name, config.default_network
        );
        Box::new(ExampleChainDriver)
    });

    let driver = registry
        .create_driver("example-chain")
        .expect("custom driver should be created");

    println!(
        "Registered {} ({}) with default network {}",
        driver.chain_name(),
        driver.chain_id(),
        driver.default_network()
    );
}
