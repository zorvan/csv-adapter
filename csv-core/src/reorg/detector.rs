//! Reorg Detector
//!
//! Detects blockchain reorganizations by comparing current chain state
//! with previously known state.

use crate::hash::Hash;
use crate::protocol_version::ChainId;

/// Reorg event
#[derive(Clone, Debug)]
pub struct ReorgEvent {
    /// Chain where reorg occurred
    pub chain: ChainId,
    /// Old block height
    pub old_height: u64,
    /// New block height
    pub new_height: u64,
    /// Old block hash
    pub old_hash: Hash,
    /// New block hash
    pub new_hash: Hash,
    /// Depth of reorg (number of blocks rolled back)
    pub depth: u64,
}

/// Reorg detector
pub struct ReorgDetector {
    /// Last known block hashes per chain
    last_known: alloc::collections::BTreeMap<String, (u64, Hash)>,
}

impl ReorgDetector {
    /// Create a new reorg detector
    pub fn new() -> Self {
        Self {
            last_known: alloc::collections::BTreeMap::new(),
        }
    }

    /// Update the detector with current chain state
    ///
    /// Returns Some(ReorgEvent) if a reorg is detected, None otherwise.
    pub fn update(&mut self, chain: ChainId, height: u64, hash: Hash) -> Option<ReorgEvent> {
        let chain_str = chain.as_str().to_string();
        
        match self.last_known.get(&chain_str) {
            Some((last_height, last_hash)) => {
                if height < *last_height {
                    // Reorg detected - chain rolled back
                    let depth = last_height - height;
                    Some(ReorgEvent {
                        chain,
                        old_height: *last_height,
                        new_height: height,
                        old_hash: *last_hash,
                        new_hash: hash,
                        depth,
                    })
                } else if height == *last_height && hash != *last_hash {
                    // Same height but different hash - reorg at same height
                    Some(ReorgEvent {
                        chain,
                        old_height: *last_height,
                        new_height: height,
                        old_hash: *last_hash,
                        new_hash: hash,
                        depth: 0,
                    })
                } else {
                    // Normal progression
                    self.last_known.insert(chain_str, (height, hash));
                    None
                }
            }
            None => {
                // First time seeing this chain
                self.last_known.insert(chain_str, (height, hash));
                None
            }
        }
    }

    /// Get the last known state for a chain
    pub fn get_last_known(&self, chain: &ChainId) -> Option<(u64, Hash)> {
        self.last_known.get(chain.as_str()).copied()
    }
}

impl Default for ReorgDetector {
    fn default() -> Self {
        Self::new()
    }
}
