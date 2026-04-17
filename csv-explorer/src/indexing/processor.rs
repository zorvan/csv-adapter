//! Event processor for indexing pipeline
//!
//! Handles the processing of indexing events and updates the storage layer.

use std::sync::Arc;
use std::time::Instant;
use csv_adapter_core::{Hash, Chain, TransferStatus};
use crate::indexing::events::IndexingEvent;
use crate::indexing::storage::IndexStorage;

/// Event processor for indexing events
pub struct EventProcessor {
    storage: Arc<IndexStorage>,
    stats: ProcessorStats,
}

/// Processor statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    pub events_processed: u64,
    pub errors: u64,
    pub average_processing_time: std::time::Duration,
    pub last_processed_time: Instant,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new(storage: Arc<IndexStorage>) -> Self {
        Self {
            storage,
            stats: ProcessorStats::default(),
        }
    }
    
    /// Process a single event
    pub async fn process_event(&mut self, event: &IndexingEvent) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        let result = match event {
            IndexingEvent::RightCreated { right_id, chain, owner, created_at, metadata } => {
                self.handle_right_created(*right_id, *chain, owner, *created_at, metadata).await
            }
            IndexingEvent::RightTransferred { right_id, from_chain, to_chain, transfer_id, created_at, proof_bundle } => {
                self.handle_right_transferred(*right_id, *from_chain, *to_chain, *transfer_id, *created_at, proof_bundle.as_ref()).await
            }
            IndexingEvent::TransferUpdated { transfer_id, old_status, new_status, updated_at } => {
                self.handle_transfer_updated(*transfer_id, *old_status, *new_status, *updated_at).await
            }
            IndexingEvent::RightUpdated { right_id, chain, old_metadata, new_metadata, updated_at } => {
                self.handle_right_updated(*right_id, *chain, old_metadata, new_metadata, *updated_at).await
            }
            IndexingEvent::ChainSynced { chain, block_height, last_block_hash, synced_at } => {
                self.handle_chain_synced(*chain, *block_height, *last_block_hash, *synced_at).await
            }
            IndexingEvent::Error { error, chain, timestamp, context } => {
                self.handle_error(error, chain.as_ref(), *timestamp, context).await
            }
        };
        
        let processing_time = start.elapsed();
        self.update_stats(result.is_ok(), processing_time);
        
        result
    }
    
    /// Handle right created event
    async fn handle_right_created(
        &mut self,
        right_id: Hash,
        chain: Chain,
        owner: &str,
        created_at: Instant,
        metadata: &serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let indexed_right = crate::indexing::IndexedRight {
            id: right_id,
            owner: owner.to_string(),
            chain: chain.to_string(),
            created_at,
            updated_at: created_at,
            status: TransferStatus::Created,
            metadata: metadata.clone(),
        };
        
        self.storage.store_right(&indexed_right).await?;
        Ok(())
    }
    
    /// Handle right transferred event
    async fn handle_right_transferred(
        &mut self,
        right_id: Hash,
        from_chain: Chain,
        to_chain: Chain,
        transfer_id: Hash,
        created_at: Instant,
        proof_bundle: Option<&csv_adapter_core::proof::ProofBundle>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let indexed_transfer = crate::indexing::IndexedTransfer {
            id: transfer_id,
            right_id,
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            status: TransferStatus::Pending,
            created_at,
            updated_at: created_at,
            proof_bundle: proof_bundle.cloned(),
            metadata: serde_json::json!({
                "from_chain": from_chain.to_string(),
                "to_chain": to_chain.to_string(),
            }),
        };
        
        self.storage.store_transfer(&indexed_transfer).await?;
        
        // Update right status and chain
        if let Some(mut right) = self.storage.get_right_by_id(&right_id).await? {
            right.status = TransferStatus::Pending;
            right.chain = to_chain.to_string();
            right.updated_at = created_at;
            self.storage.store_right(&right).await?;
        }
        
        Ok(())
    }
    
    /// Handle transfer updated event
    async fn handle_transfer_updated(
        &mut self,
        transfer_id: Hash,
        _old_status: TransferStatus,
        new_status: TransferStatus,
        updated_at: Instant,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut transfer) = self.storage.get_transfer_by_hash(&transfer_id).await? {
            transfer.status = new_status;
            transfer.updated_at = updated_at;
            self.storage.store_transfer(&transfer).await?;
            
            // Update right status if transfer is completed
            if new_status == TransferStatus::Completed {
                if let Some(mut right) = self.storage.get_right_by_id(&transfer.right_id).await? {
                    right.status = TransferStatus::Completed;
                    right.updated_at = updated_at;
                    self.storage.store_right(&right).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle right updated event
    async fn handle_right_updated(
        &mut self,
        right_id: Hash,
        _chain: Chain,
        _old_metadata: &serde_json::Value,
        new_metadata: &serde_json::Value,
        updated_at: Instant,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut right) = self.storage.get_right_by_id(&right_id).await? {
            right.metadata = new_metadata.clone();
            right.updated_at = updated_at;
            self.storage.store_right(&right).await?;
        }
        
        Ok(())
    }
    
    /// Handle chain synced event
    async fn handle_chain_synced(
        &mut self,
        chain: Chain,
        block_height: u64,
        last_block_hash: Hash,
        synced_at: Instant,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.storage.update_chain_sync_status(&chain, block_height, last_block_hash, synced_at).await?;
        Ok(())
    }
    
    /// Handle error event
    async fn handle_error(
        &mut self,
        error: &str,
        chain: Option<&Chain>,
        timestamp: Instant,
        context: &serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Log error to storage for monitoring
        self.storage.log_error(error, chain, timestamp, context).await?;
        Ok(())
    }
    
    /// Update processor statistics
    fn update_stats(&mut self, success: bool, processing_time: std::time::Duration) {
        self.stats.events_processed += 1;
        if !success {
            self.stats.errors += 1;
        }
        
        // Update average processing time
        let total_time = self.stats.average_processing_time * (self.stats.events_processed - 1) + processing_time;
        self.stats.average_processing_time = total_time / self.stats.events_processed;
        
        self.stats.last_processed_time = Instant::now();
    }
    
    /// Get processor statistics
    pub fn get_stats(&self) -> &ProcessorStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ProcessorStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexing::storage::IndexStorage;
    
    #[tokio::test]
    async fn test_event_processor_creation() {
        let storage = Arc::new(IndexStorage::new().unwrap());
        let processor = EventProcessor::new(storage);
        
        assert_eq!(processor.get_stats().events_processed, 0);
        assert_eq!(processor.get_stats().errors, 0);
    }
    
    #[tokio::test]
    async fn test_right_created_event() {
        let storage = Arc::new(IndexStorage::new().unwrap());
        let mut processor = EventProcessor::new(storage);
        
        let event = IndexingEvent::RightCreated {
            right_id: Hash::zero(),
            chain: Chain::Ethereum,
            owner: "test_owner".to_string(),
            created_at: Instant::now(),
            metadata: serde_json::json!({"test": "data"}),
        };
        
        let result = processor.process_event(&event).await;
        assert!(result.is_ok());
        assert_eq!(processor.get_stats().events_processed, 1);
        assert_eq!(processor.get_stats().errors, 0);
    }
}
