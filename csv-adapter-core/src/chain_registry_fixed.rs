//! Chain registry for dynamic chain management.

use std::collections::HashMap;
use super::chain_system::ChainInfo;

/// Registry for managing chain adapters
pub struct ChainRegistry {
    chains: HashMap<String, ChainInfo>,
}

impl ChainRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
        }
    }
    
    /// Register a new chain
    pub fn register_chain(&mut self, chain_id: String, chain_name: String) {
        let info = ChainInfo {
            chain_id: chain_id.clone(),
            chain_name,
            supports_nfts: true,
            supports_smart_contracts: true,
        };
        
        self.chains.insert(chain_id, info);
    }
    
    /// Get chain info by ID
    pub fn get_chain_info(&self, chain_id: &str) -> Option<&ChainInfo> {
        self.chains.get(chain_id)
    }
    
    /// Get all supported chain IDs
    pub fn supported_chains(&self) -> Vec<String> {
        self.chains.keys().cloned().collect()
    }
    
    /// Check if chain supports NFTs
    pub fn supports_nfts(&self, chain_id: &str) -> bool {
        self.chains
            .get(chain_id)
            .map(|info| info.supports_nfts)
            .unwrap_or(false)
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}
