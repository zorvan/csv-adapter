//! Adapter factory for dynamic chain adapter instantiation.
//!
//! This module provides a factory pattern for creating chain adapters
//! based on chain IDs, enabling dynamic chain support without hardcoding.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::adapters::{
    ScalableAptosAdapter, ScalableBitcoinAdapter, ScalableEthereumAdapter, ScalableSolanaAdapter,
    ScalableSuiAdapter,
};
use crate::chain_adapter::ChainAdapter;
use crate::chain_config::ChainConfig;
use crate::chain_plugin::ChainPluginRegistry;

/// Factory function type for creating chain adapters
type AdapterFactoryFn = Arc<dyn Fn(Option<ChainConfig>) -> Box<dyn ChainAdapter> + Send + Sync>;

/// Registry of adapter factories for dynamic chain instantiation
pub struct AdapterFactory {
    factories: HashMap<String, AdapterFactoryFn>,
}

impl AdapterFactory {
    /// Create a new empty adapter factory.
    pub fn empty() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Create a new adapter factory with all built-in chains registered
    pub fn new() -> Self {
        let mut factory = Self::empty();

        // Register built-in chain adapters
        factory.register_built_in_adapters();

        factory
    }

    /// Register all built-in chain adapters
    fn register_built_in_adapters(&mut self) {
        // Bitcoin
        self.register("bitcoin", || Box::new(ScalableBitcoinAdapter::new()));

        // Ethereum
        self.register("ethereum", || Box::new(ScalableEthereumAdapter::new()));

        // Solana
        self.register("solana", || Box::new(ScalableSolanaAdapter::new()));

        // Sui
        self.register("sui", || Box::new(ScalableSuiAdapter::new()));

        // Aptos
        self.register("aptos", || Box::new(ScalableAptosAdapter::new()));
    }

    /// Register a custom adapter factory
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The unique identifier for the chain
    /// * `factory` - A factory function that creates the adapter
    ///
    /// # Example
    ///
    /// ```rust
    /// use csv_adapter_core::adapter_factory::AdapterFactory;
    /// use csv_adapter_core::adapters::ScalableSolanaAdapter;
    ///
    /// let mut factory = AdapterFactory::new();
    /// factory.register("solana", || {
    ///     Box::new(ScalableSolanaAdapter::new())
    /// });
    /// ```
    pub fn register<F>(&mut self, chain_id: &str, factory: F)
    where
        F: Fn() -> Box<dyn ChainAdapter> + Send + Sync + 'static,
    {
        self.factories
            .insert(chain_id.to_string(), Arc::new(move |_| factory()));
    }

    /// Register a factory that receives the discovered chain configuration.
    pub fn register_with_config(&mut self, chain_id: &str, factory: AdapterFactoryFn) {
        self.factories.insert(chain_id.to_string(), factory);
    }

    /// Register all plugins from a plugin registry.
    pub fn register_plugins_from_registry(&mut self, registry: &ChainPluginRegistry) {
        for (chain_id, plugin) in registry.all_plugins() {
            let plugin = Arc::clone(plugin);
            self.register_with_config(
                chain_id,
                Arc::new(move |config| plugin.create_adapter(config)),
            );
        }
    }

    /// Create an adapter for the specified chain
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The chain identifier (e.g., "bitcoin", "solana")
    ///
    /// # Returns
    ///
    /// Returns `Some(Box<dyn ChainAdapter>)` if the chain is registered,
    /// or `None` if no factory exists for the chain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use csv_adapter_core::adapter_factory::AdapterFactory;
    ///
    /// let factory = AdapterFactory::new();
    /// let adapter = factory.create_adapter("bitcoin");
    /// assert!(adapter.is_some());
    /// ```
    pub fn create_adapter(&self, chain_id: &str) -> Option<Box<dyn ChainAdapter>> {
        self.create_adapter_with_config(chain_id, None)
    }

    /// Create an adapter with a discovered configuration.
    pub fn create_adapter_with_config(
        &self,
        chain_id: &str,
        config: Option<ChainConfig>,
    ) -> Option<Box<dyn ChainAdapter>> {
        self.factories.get(chain_id).map(|factory| factory(config))
    }

    /// Check if a chain is supported
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The chain identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the chain has a registered factory, `false` otherwise.
    pub fn is_supported(&self, chain_id: &str) -> bool {
        self.factories.contains_key(chain_id)
    }

    /// Get all supported chain IDs
    ///
    /// # Returns
    ///
    /// A vector of all registered chain IDs.
    pub fn supported_chains(&self) -> Vec<&str> {
        self.factories.keys().map(|k| k.as_str()).collect()
    }

    /// Get the number of registered adapters
    pub fn adapter_count(&self) -> usize {
        self.factories.len()
    }

    /// Create adapters for all supported chains
    ///
    /// # Returns
    ///
    /// A vector of tuples containing (chain_id, adapter) for all registered chains.
    pub fn create_all_adapters(&self) -> Vec<(String, Box<dyn ChainAdapter>)> {
        self.factories
            .iter()
            .map(|(chain_id, factory)| (chain_id.clone(), factory(None)))
            .collect()
    }
}

impl Default for AdapterFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global adapter factory singleton for convenience
///
/// Note: In production code, prefer dependency injection over singletons.
static GLOBAL_FACTORY: OnceLock<AdapterFactory> = OnceLock::new();

/// Initialize the global adapter factory
///
/// This function initializes the global factory if it hasn't been initialized yet.
/// It is safe to call multiple times - only the first call will initialize the factory.
pub fn init_global_factory() {
    GLOBAL_FACTORY.get_or_init(AdapterFactory::new);
}

/// Get the global adapter factory
///
/// # Panics
///
/// Panics if the global factory hasn't been initialized.
pub fn global_factory() -> &'static AdapterFactory {
    GLOBAL_FACTORY
        .get()
        .expect("Global adapter factory not initialized. Call init_global_factory() first.")
}

/// Create an adapter using the global factory
///
/// # Arguments
///
/// * `chain_id` - The chain identifier
///
/// # Returns
///
/// Returns `Some(Box<dyn ChainAdapter>)` if the chain is registered,
/// or `None` if no factory exists for the chain.
///
/// # Panics
///
/// Panics if the global factory hasn't been initialized.
pub fn create_adapter(chain_id: &str) -> Option<Box<dyn ChainAdapter>> {
    global_factory().create_adapter(chain_id)
}

/// Check if a chain is supported by the global factory
///
/// # Arguments
///
/// * `chain_id` - The chain identifier to check
///
/// # Returns
///
/// `true` if the chain has a registered factory, `false` otherwise.
///
/// # Panics
///
/// Panics if the global factory hasn't been initialized.
pub fn is_chain_supported(chain_id: &str) -> bool {
    global_factory().is_supported(chain_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_factory_creation() {
        let factory = AdapterFactory::new();
        assert!(factory.is_supported("bitcoin"));
        assert!(factory.is_supported("solana"));
        assert!(!factory.is_supported("unknown_chain"));
    }

    #[test]
    fn test_create_adapter() {
        let factory = AdapterFactory::new();

        let bitcoin = factory.create_adapter("bitcoin");
        assert!(bitcoin.is_some());
        assert_eq!(bitcoin.unwrap().chain_id(), "bitcoin");

        let unknown = factory.create_adapter("unknown");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_supported_chains() {
        let factory = AdapterFactory::new();
        let chains = factory.supported_chains();

        assert!(chains.contains(&"bitcoin"));
        assert!(chains.contains(&"solana"));
    }

    #[test]
    fn test_create_all_adapters() {
        let factory = AdapterFactory::new();
        let adapters = factory.create_all_adapters();

        assert_eq!(adapters.len(), 5); // bitcoin, ethereum, solana, sui, aptos

        for (chain_id, adapter) in adapters {
            assert_eq!(chain_id, adapter.chain_id());
        }
    }

    #[test]
    fn test_custom_registration() {
        let mut factory = AdapterFactory::new();

        // Register a custom chain
        factory.register("custom_chain", || {
            Box::new(ScalableSolanaAdapter::new()) // Using Solana as placeholder
        });

        assert!(factory.is_supported("custom_chain"));
        let custom = factory.create_adapter("custom_chain");
        assert!(custom.is_some());
    }
}
