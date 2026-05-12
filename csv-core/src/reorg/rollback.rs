//! Rollback Handler
//!
//! Handles rollback of operations affected by a reorg.

use crate::protocol_version::ChainId;
use super::detector::ReorgEvent;

/// Rollback handler
pub struct RollbackHandler {
    /// Callback for rollback operations
    on_rollback: alloc::collections::BTreeMap<String, Box<dyn Fn(&ReorgEvent) + Send + Sync>>,
}

impl RollbackHandler {
    /// Create a new rollback handler
    pub fn new() -> Self {
        Self {
            on_rollback: alloc::collections::BTreeMap::new(),
        }
    }

    /// Register a rollback callback for a chain
    pub fn register_callback<F>(&mut self, chain: ChainId, callback: F)
    where
        F: Fn(&ReorgEvent) + Send + Sync + 'static,
    {
        self.on_rollback.insert(chain.as_str().to_string(), Box::new(callback));
    }

    /// Handle a reorg event
    pub fn handle_reorg(&self, event: &ReorgEvent) {
        if let Some(callback) = self.on_rollback.get(event.chain.as_str()) {
            callback(event);
        }
    }

    /// Roll back transfers affected by a reorg
    pub fn rollback_transfers(&self, _chain: ChainId, _from_height: u64, _to_height: u64) {
        // Placeholder - would query storage for affected transfers
        // and mark them as rolled back
    }
}

impl Default for RollbackHandler {
    fn default() -> Self {
        Self::new()
    }
}
