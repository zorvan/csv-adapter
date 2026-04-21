//! Chain discovery and automatic configuration loading.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::adapter_factory::AdapterFactory;
use super::chain_config::{ChainConfig, ChainConfigLoader};
use super::chain_plugin::{ChainPlugin, ChainPluginRegistry};
use super::chain_system::SimpleChainRegistry;

/// Chain discovery system for automatic chain loading
pub struct ChainDiscovery {
    config_loader: ChainConfigLoader,
    registry: SimpleChainRegistry,
    plugins: ChainPluginRegistry,
}

impl ChainDiscovery {
    /// Create new chain discovery system
    pub fn new() -> Self {
        Self {
            config_loader: ChainConfigLoader::new(),
            registry: SimpleChainRegistry::new(),
            plugins: ChainPluginRegistry::new(),
        }
    }

    /// Discover and load all chains from the chains directory
    pub fn discover_chains(&mut self, chains_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("Discovering chains from: {}", chains_dir.display());

        // Load all chain configurations
        self.config_loader.load_from_directory(chains_dir)?;

        // Register all discovered chains
        for (chain_id, config) in self.config_loader.all_configs() {
            println!("Registering chain: {} ({})", chain_id, config.chain_name);
            self.registry
                .register_chain(chain_id.clone(), config.chain_name.clone());
        }

        let discovered_count = self.registry.supported_chains().len();
        println!("Successfully discovered {} chains", discovered_count);

        Ok(())
    }

    /// Register a plugin so discovered configs can create adapters dynamically.
    pub fn register_plugin(&mut self, plugin: Arc<dyn ChainPlugin>) {
        self.plugins.register(plugin);
    }

    /// Access the plugin registry.
    pub fn plugins(&self) -> &ChainPluginRegistry {
        &self.plugins
    }

    /// Create an adapter for a discovered chain using its loaded configuration.
    pub fn create_adapter(
        &self,
        chain_id: &str,
    ) -> Option<Box<dyn crate::chain_adapter::ChainAdapter>> {
        let config = self.get_chain_config(chain_id).cloned();
        self.plugins.create_adapter(chain_id, config)
    }

    /// Build an adapter factory from the registered plugins.
    pub fn build_factory(&self) -> AdapterFactory {
        let mut factory = AdapterFactory::empty();
        factory.register_plugins_from_registry(&self.plugins);
        factory
    }

    /// Get the chain registry
    pub fn registry(&self) -> &SimpleChainRegistry {
        &self.registry
    }

    /// Get the chain registry (mutable)
    pub fn registry_mut(&mut self) -> &mut SimpleChainRegistry {
        &mut self.registry
    }

    /// Get configuration for a specific chain
    pub fn get_chain_config(&self, chain_id: &str) -> Option<&ChainConfig> {
        self.config_loader.get_config(chain_id)
    }

    /// Get all chain configurations
    pub fn all_chain_configs(&self) -> &HashMap<String, ChainConfig> {
        self.config_loader.all_configs()
    }

    /// Get supported chain IDs
    pub fn supported_chain_ids(&self) -> Vec<String> {
        self.registry
            .supported_chains()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if a chain supports NFTs
    pub fn supports_nfts(&self, chain_id: &str) -> bool {
        self.registry.supports_nfts(chain_id)
    }

    /// Get chains that support NFTs
    pub fn nft_supported_chains(&self) -> Vec<String> {
        self.registry
            .supported_chains()
            .into_iter()
            .filter(|chain_id| self.registry.supports_nfts(chain_id))
            .map(|s| s.to_string())
            .collect()
    }

    /// Load chains from default directory
    pub fn load_default_chains(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(chains_dir) = self.config_loader.load_from_default_locations()? {
            self.refresh_registry_from_configs();
            println!(
                "Loaded chains from default directory: {}",
                chains_dir.display()
            );
            Ok(())
        } else {
            println!("Default chains directory not found, no chains loaded");
            Ok(())
        }
    }

    fn refresh_registry_from_configs(&mut self) {
        self.registry = SimpleChainRegistry::new();
        for (chain_id, config) in self.config_loader.all_configs() {
            self.registry
                .register_chain(chain_id.clone(), config.chain_name.clone());
        }
    }
}

impl Default for ChainDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_chain_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let chains_dir = temp_dir.path();

        // Create a test chain config with all required fields
        let test_config = r#"
chain_id = "test-chain"
chain_name = "Test Chain"
default_network = "testnet"
rpc_endpoints = ["https://test-rpc.example.com"]
block_explorer_urls = ["https://test-explorer.example.com"]

[capabilities]
supports_nfts = true
supports_smart_contracts = true
account_model = "Account"
confirmation_blocks = 12
max_batch_size = 100
supported_networks = ["mainnet", "testnet"]
supports_cross_chain = false

[custom_settings]
test_setting = "value"
"#;

        let config_path = chains_dir.join("test-chain.toml");
        fs::write(&config_path, test_config).unwrap();

        let mut discovery = ChainDiscovery::new();
        discovery.discover_chains(chains_dir).unwrap();

        let supported_chains = discovery.supported_chain_ids();
        assert_eq!(supported_chains.len(), 1);
        assert_eq!(supported_chains[0], "test-chain");

        let config = discovery.get_chain_config("test-chain");
        assert!(config.is_some());
        assert_eq!(config.unwrap().chain_name, "Test Chain");
    }
}
