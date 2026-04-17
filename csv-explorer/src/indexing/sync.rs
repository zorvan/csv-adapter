//! Chain synchronizer for real-time indexing
//!
//! Handles synchronization with blockchain nodes and emits indexing events.

use std::sync::Arc;
use std::time::{Duration, Instant};
use csv_adapter_core::{Hash, Chain};
use crate::indexing::events::{IndexingEvent, EventFilter};
use crate::indexing::storage::IndexStorage;

/// Chain synchronizer for monitoring blockchain events
pub struct ChainSynchronizer {
    chain: String,
    storage: Arc<IndexStorage>,
    current_block_height: u64,
    last_sync_time: Instant,
    is_running: bool,
    sync_interval: Duration,
}

impl ChainSynchronizer {
    /// Create a new chain synchronizer
    pub fn new(chain: &str, storage: Arc<IndexStorage>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            chain: chain.to_string(),
            storage,
            current_block_height: 0,
            last_sync_time: Instant::now(),
            is_running: false,
            sync_interval: Duration::from_secs(5), // Sync every 5 seconds
        })
    }
    
    /// Start the synchronizer
    pub async fn start(&mut self, event_buffer: Arc<tokio::sync::RwLock<Vec<IndexingEvent>>>) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_running {
            return Err("Synchronizer already running".into());
        }
        
        self.is_running = true;
        println!("Starting chain synchronizer for: {}", self.chain);
        
        // Initialize sync status
        self.initialize_sync_status().await?;
        
        // Start sync loop
        let chain = self.chain.clone();
        let storage = self.storage.clone();
        let interval = self.sync_interval;
        
        tokio::spawn(async move {
            let mut sync = ChainSynchronizer {
                chain,
                storage,
                current_block_height: 0,
                last_sync_time: Instant::now(),
                is_running: true,
                sync_interval: interval,
            };
            
            while sync.is_running {
                if let Err(e) = sync.sync_chain(&event_buffer).await {
                    eprintln!("Error syncing {}: {}", sync.chain, e);
                }
                tokio::time::sleep(interval).await;
            }
        });
        
        Ok(())
    }
    
    /// Stop the synchronizer
    pub fn stop(&mut self) {
        self.is_running = false;
        println!("Stopping chain synchronizer for: {}", self.chain);
    }
    
    /// Initialize sync status
    async fn initialize_sync_status(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would query the blockchain for the latest block
        // For demo purposes, we'll start from block 0
        self.current_block_height = 0;
        
        let sync_info = crate::indexing::storage::ChainSyncInfo {
            chain: self.chain.clone(),
            block_height: self.current_block_height,
            last_block_hash: Hash::zero(),
            last_sync_time: self.last_sync_time,
            is_syncing: true,
        };
        
        // Store sync status
        let chain_enum = self.parse_chain()?;
        self.storage.update_chain_sync_status(
            &chain_enum,
            sync_info.block_height,
            sync_info.last_block_hash,
            sync_info.last_sync_time,
        ).await?;
        
        Ok(())
    }
    
    /// Sync chain and emit events
    async fn sync_chain(&mut self, event_buffer: &Arc<tokio::sync::RwLock<Vec<IndexingEvent>>>) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would:
        // 1. Query the blockchain for new blocks
        // 2. Parse transactions for CSV-related events
        // 3. Emit appropriate indexing events
        
        // For demo purposes, we'll simulate some events
        let new_block_height = self.current_block_height + 1;
        
        // Simulate finding some events in the new block
        let events = self.simulate_block_events(new_block_height).await?;
        
        // Add events to buffer
        if !events.is_empty() {
            let mut buffer = event_buffer.write().await;
            buffer.extend(events);
        }
        
        // Update sync status
        self.current_block_height = new_block_height;
        self.last_sync_time = Instant::now();
        
        let chain_enum = self.parse_chain()?;
        self.storage.update_chain_sync_status(
            &chain_enum,
            self.current_block_height,
            Hash::new([new_block_height as u8; 32]),
            self.last_sync_time,
        ).await?;
        
        if self.current_block_height % 10 == 0 {
            println!("Synced {} to block {}", self.chain, self.current_block_height);
        }
        
        Ok(())
    }
    
    /// Simulate events found in a block
    async fn simulate_block_events(&self, block_height: u64) -> Result<Vec<IndexingEvent>, Box<dyn std::error::Error>> {
        let mut events = Vec::new();
        let chain_enum = self.parse_chain()?;
        
        // Simulate finding a right creation every 5 blocks
        if block_height % 5 == 0 {
            let right_id = Hash::new([block_height as u8; 32]);
            let event = IndexingEvent::RightCreated {
                right_id,
                chain: chain_enum,
                owner: format!("owner_{}", block_height),
                created_at: Instant::now(),
                metadata: serde_json::json!({
                    "block_height": block_height,
                    "simulated": true,
                }),
            };
            events.push(event);
        }
        
        // Simulate finding a transfer every 7 blocks
        if block_height % 7 == 0 && block_height > 7 {
            let transfer_id = Hash::new([block_height as u8; 32]);
            let right_id = Hash::new([(block_height - 7) as u8; 32]);
            
            let event = IndexingEvent::RightTransferred {
                right_id,
                from_chain: chain_enum,
                to_chain: self.get_random_chain(),
                transfer_id,
                created_at: Instant::now(),
                proof_bundle: None, // In real implementation, this would contain the actual proof
            };
            events.push(event);
        }
        
        // Simulate transfer status updates
        if block_height % 11 == 0 && block_height > 11 {
            let transfer_id = Hash::new([(block_height - 11) as u8; 32]);
            
            let event = IndexingEvent::TransferUpdated {
                transfer_id,
                old_status: csv_adapter_core::TransferStatus::Pending,
                new_status: csv_adapter_core::TransferStatus::Completed,
                updated_at: Instant::now(),
            };
            events.push(event);
        }
        
        // Always emit chain sync event
        let sync_event = IndexingEvent::ChainSynced {
            chain: chain_enum,
            block_height,
            last_block_hash: Hash::new([block_height as u8; 32]),
            synced_at: Instant::now(),
        };
        events.push(sync_event);
        
        Ok(events)
    }
    
    /// Parse chain string to Chain enum
    fn parse_chain(&self) -> Result<Chain, Box<dyn std::error::Error>> {
        match self.chain.as_str() {
            "bitcoin" => Ok(Chain::Bitcoin),
            "ethereum" => Ok(Chain::Ethereum),
            "sui" => Ok(Chain::Sui),
            "aptos" => Ok(Chain::Aptos),
            "solana" => Ok(Chain::Solana),
            _ => Err(format!("Unknown chain: {}", self.chain).into()),
        }
    }
    
    /// Get a random chain for simulation
    fn get_random_chain(&self) -> Chain {
        use std::collections::HashMap;
        
        let chains = vec![
            Chain::Bitcoin,
            Chain::Ethereum,
            Chain::Sui,
            Chain::Aptos,
            Chain::Solana,
        ];
        
        // Simple pseudo-random based on current block height
        let index = (self.current_block_height % (chains.len() as u64)) as usize;
        chains[index].clone()
    }
    
    /// Get current sync status
    pub fn get_sync_status(&self) -> SyncStatus {
        SyncStatus {
            chain: self.chain.clone(),
            current_block_height: self.current_block_height,
            last_sync_time: self.last_sync_time,
            is_running: self.is_running,
            sync_interval: self.sync_interval,
        }
    }
}

/// Synchronization status
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub chain: String,
    pub current_block_height: u64,
    pub last_sync_time: Instant,
    pub is_running: bool,
    pub sync_interval: Duration,
}

/// Multi-chain synchronizer manager
pub struct MultiChainSynchronizer {
    synchronizers: HashMap<String, Arc<tokio::sync::Mutex<ChainSynchronizer>>>,
    event_buffer: Arc<tokio::sync::RwLock<Vec<IndexingEvent>>>,
}

impl MultiChainSynchronizer {
    /// Create a new multi-chain synchronizer
    pub fn new(storage: Arc<IndexStorage>) -> Result<Self, Box<dyn std::error::Error>> {
        let event_buffer = Arc::new(tokio::sync::RwLock::new(Vec::new()));
        let mut synchronizers = HashMap::new();
        
        let chains = vec!["bitcoin", "ethereum", "sui", "aptos", "solana"];
        
        for chain in chains {
            let sync = ChainSynchronizer::new(chain, storage.clone())?;
            synchronizers.insert(chain.to_string(), Arc::new(tokio::sync::Mutex::new(sync)));
        }
        
        Ok(Self {
            synchronizers,
            event_buffer,
        })
    }
    
    /// Start all synchronizers
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (chain_name, sync) in &self.synchronizers {
            let mut sync_guard = sync.lock().await;
            sync_guard.start(self.event_buffer.clone()).await?;
            println!("Started synchronizer for: {}", chain_name);
        }
        
        Ok(())
    }
    
    /// Stop all synchronizers
    pub async fn stop_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (chain_name, sync) in &self.synchronizers {
            let mut sync_guard = sync.lock().await;
            sync_guard.stop();
            println!("Stopped synchronizer for: {}", chain_name);
        }
        
        Ok(())
    }
    
    /// Get status of all synchronizers
    pub async fn get_all_status(&self) -> Vec<SyncStatus> {
        let mut status = Vec::new();
        
        for sync in self.synchronizers.values() {
            let sync_guard = sync.lock().await;
            status.push(sync_guard.get_sync_status());
        }
        
        status
    }
    
    /// Get event buffer reference
    pub fn get_event_buffer(&self) -> &Arc<tokio::sync::RwLock<Vec<IndexingEvent>>> {
        &self.event_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chain_synchronizer_creation() {
        let storage = Arc::new(IndexStorage::new().unwrap());
        let sync = ChainSynchronizer::new("ethereum", storage);
        assert!(sync.is_ok());
    }
    
    #[tokio::test]
    async fn test_multi_chain_synchronizer() {
        let storage = Arc::new(IndexStorage::new().unwrap());
        let multi_sync = MultiChainSynchronizer::new(storage);
        assert!(multi_sync.is_ok());
        
        let multi_sync = multi_sync.unwrap();
        let status = multi_sync.get_all_status().await;
        assert_eq!(status.len(), 5); // 5 chains
    }
}
