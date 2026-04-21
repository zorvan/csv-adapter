use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tempfile::TempDir;

use csv_adapter_core::{
    AccountModel, AdapterFactory, Chain, ChainAdapter, ChainCapabilities, ChainConfig,
    ChainDiscovery, ChainError, ChainPluginBuilder, ChainPluginRegistry, ChainResult, RpcClient,
    Wallet,
};

#[derive(Debug, Clone)]
struct CustomAdapter;

#[async_trait]
impl ChainAdapter for CustomAdapter {
    fn chain_id(&self) -> &'static str {
        "custom-chain"
    }

    fn chain_name(&self) -> &'static str {
        "Custom Chain"
    }

    fn capabilities(&self) -> ChainCapabilities {
        ChainCapabilities {
            supports_nfts: true,
            supports_smart_contracts: false,
            account_model: AccountModel::Account,
            confirmation_blocks: 5,
            max_batch_size: 25,
            supported_networks: vec!["devnet".to_string()],
            supports_cross_chain: false,
            custom_features: HashMap::new(),
        }
    }

    async fn create_client(&self, _config: &ChainConfig) -> ChainResult<Box<dyn RpcClient>> {
        Err(ChainError::NotImplemented("custom client".to_string()))
    }

    async fn create_wallet(&self, _config: &ChainConfig) -> ChainResult<Box<dyn Wallet>> {
        Err(ChainError::NotImplemented("custom wallet".to_string()))
    }

    fn csv_program_id(&self) -> Option<&'static str> {
        Some("custom-program")
    }

    fn to_core_chain(&self) -> Chain {
        Chain::Solana
    }

    fn default_network(&self) -> &'static str {
        "devnet"
    }
}

fn custom_chain_config() -> ChainConfig {
    ChainConfig {
        chain_id: "custom-chain".to_string(),
        chain_name: "Custom Chain".to_string(),
        default_network: "devnet".to_string(),
        rpc_endpoints: vec!["https://custom.devnet.example".to_string()],
        program_id: Some("custom-program".to_string()),
        block_explorer_urls: vec!["https://custom.explorer.example".to_string()],
        capabilities: ChainCapabilities {
            supports_nfts: true,
            supports_smart_contracts: false,
            account_model: AccountModel::Account,
            confirmation_blocks: 5,
            max_batch_size: 25,
            supported_networks: vec!["devnet".to_string()],
            supports_cross_chain: false,
            custom_features: HashMap::new(),
        },
        custom_settings: HashMap::new(),
    }
}

#[test]
fn test_factory_registers_plugins_from_registry() {
    let registry_plugin = ChainPluginBuilder::new("custom-chain", "Custom Chain")
        .capabilities(custom_chain_config().capabilities.clone())
        .adapter_factory(|_| Box::new(CustomAdapter))
        .config_factory(custom_chain_config)
        .build()
        .expect("plugin should build");

    let mut registry = ChainPluginRegistry::new();
    registry.register(registry_plugin);

    let mut factory = AdapterFactory::empty();
    factory.register_plugins_from_registry(&registry);

    let adapter = factory
        .create_adapter_with_config("custom-chain", Some(custom_chain_config()))
        .expect("adapter should be created from plugin registry");

    assert_eq!(adapter.chain_id(), "custom-chain");
    assert!(factory.is_supported("custom-chain"));
}

#[test]
fn test_chain_discovery_creates_adapter_from_loaded_config() {
    let temp_dir = TempDir::new().expect("temp dir");
    let config_path = temp_dir.path().join("custom-chain.toml");
    fs::write(
        &config_path,
        r#"
chain_id = "custom-chain"
chain_name = "Custom Chain"
default_network = "devnet"
rpc_endpoints = ["https://custom.devnet.example"]
program_id = "custom-program"
block_explorer_urls = ["https://custom.explorer.example"]

[capabilities]
supports_nfts = true
supports_smart_contracts = false
account_model = "Account"
confirmation_blocks = 5
max_batch_size = 25
supported_networks = ["devnet"]
supports_cross_chain = false

[custom_settings]
"#,
    )
    .expect("config write");

    let seen_network = Arc::new(Mutex::new(String::new()));
    let seen_network_clone = Arc::clone(&seen_network);
    let plugin = ChainPluginBuilder::new("custom-chain", "Custom Chain")
        .capabilities(custom_chain_config().capabilities.clone())
        .adapter_factory(move |config| {
            *seen_network_clone.lock().expect("poisoned") = config
                .as_ref()
                .map(|cfg| cfg.default_network.clone())
                .unwrap_or_default();
            Box::new(CustomAdapter)
        })
        .config_factory(custom_chain_config)
        .build()
        .expect("plugin should build");

    let mut discovery = ChainDiscovery::new();
    discovery.register_plugin(plugin);
    discovery
        .discover_chains(temp_dir.path())
        .expect("discovery should succeed");

    let adapter = discovery
        .create_adapter("custom-chain")
        .expect("discovery should create adapter");

    assert_eq!(adapter.chain_name(), "Custom Chain");
    assert_eq!(seen_network.lock().expect("poisoned").as_str(), "devnet");
}
