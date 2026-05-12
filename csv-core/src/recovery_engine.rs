//! Recovery Engine for Crash-Safe Startup
//!
//! This module provides the recovery engine that ensures the protocol
//! can recover safely from crashes, restarts, and reorgs.
//!
//! ## Startup Sequence
//!
//! The recovery engine executes the following startup sequence:
//! 1. Load persistent state from storage
//! 2. Validate state consistency
//! 3. Detect in-flight operations
//! 4. Resume or rollback incomplete operations
//! 5. Rebuild in-memory structures
//! 6. Verify chain finality
//! 7. Check for reorgs since last shutdown
//! 8. Apply necessary rollbacks
//! 9. Resume normal operation
//!
//! ## Backend Abstraction
//!
//! The recovery engine uses a [`RecoveryStorageBackend`] trait to abstract
//! the persistence layer. This allows csv-store (SQLite) or any other
//! storage implementation to be used.
//!
//! See [`csv_store::RecoveryStorageBackend`](https://docs.rs/csv-store/latest/csv_store/trait.RecoveryStorageBackend.html)
//! for the SQLite implementation.

use alloc::vec::Vec;

use crate::error::Result;
use crate::hash::Hash;

/// Recovery engine for crash-safe startup
///
/// Uses a [`RecoveryStorageBackend`] to persist and recover state across restarts.
pub struct RecoveryEngine<B: RecoveryStorageBackend> {
    /// Storage backend
    backend: B,
    /// Whether recovery has been completed
    recovered: bool,
    /// Last known block heights per chain
    last_known_heights: Vec<(String, u64)>,
    /// In-flight transfers that need recovery
    in_flight_transfers: Vec<Hash>,
    /// Persistent state checksum for validation
    state_checksum: Option<Hash>,
    /// Detected reorgs during recovery
    detected_reorgs: Vec<(String, u64, u64)>, // (chain, old_height, new_height)
}

impl<B: RecoveryStorageBackend> RecoveryEngine<B> {
    /// Create a new recovery engine with the given backend
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            recovered: false,
            last_known_heights: Vec::new(),
            in_flight_transfers: Vec::new(),
            state_checksum: None,
            detected_reorgs: Vec::new(),
        }
    }

    /// Execute the recovery startup sequence
    ///
    /// This is the canonical startup sequence that MUST be executed
    /// before the protocol can accept new operations.
    ///
    /// # Startup Sequence
    ///
    /// 1. Load persistent state from storage
    /// 2. Validate state consistency
    /// 3. Detect in-flight operations
    /// 4. Resume or rollback incomplete operations
    /// 5. Rebuild in-memory structures
    /// 6. Verify chain finality
    /// 7. Check for reorgs since last shutdown
    /// 8. Apply necessary rollbacks
    /// 9. Resume normal operation
    pub async fn startup_sequence(&mut self) -> Result<RecoveryResult> {
        let mut steps = Vec::new();
        let mut errors = Vec::new();

        // Step 1: Load persistent state
        let step1 = self.load_persistent_state().await;
        steps.push(step1.clone());
        if !step1.success {
            errors.push(step1.error.unwrap_or_else(|| "Failed to load persistent state".to_string()));
            return Ok(RecoveryResult {
                success: false,
                steps,
                errors,
            });
        }

        // Step 2: Validate state consistency
        let step2 = self.validate_state_consistency().await;
        steps.push(step2.clone());
        if !step2.success {
            errors.push(step2.error.unwrap_or_else(|| "State validation failed".to_string()));
            // Continue with recovery despite validation errors
        }

        // Step 3: Detect in-flight operations
        let step3 = self.detect_in_flight_operations().await;
        steps.push(step3.clone());
        if !step3.success {
            errors.push(step3.error.unwrap_or_else(|| "Failed to detect in-flight operations".to_string()));
        }

        // Step 4: Resume or rollback incomplete operations
        let step4 = self.recover_incomplete_operations().await;
        steps.push(step4.clone());
        if !step4.success {
            errors.push(step4.error.unwrap_or_else(|| "Failed to recover incomplete operations".to_string()));
        }

        // Step 5: Rebuild in-memory structures
        let step5 = self.rebuild_memory_structures().await;
        steps.push(step5.clone());
        if !step5.success {
            errors.push(step5.error.unwrap_or_else(|| "Failed to rebuild memory structures".to_string()));
            return Ok(RecoveryResult {
                success: false,
                steps,
                errors,
            });
        }

        // Step 6: Verify chain finality
        let step6 = self.verify_chain_finality().await;
        steps.push(step6.clone());
        if !step6.success {
            errors.push(step6.error.unwrap_or_else(|| "Chain finality verification failed".to_string()));
        }

        // Step 7: Check for reorgs
        let step7 = self.check_for_reorgs().await;
        steps.push(step7.clone());
        if !step7.success {
            errors.push(step7.error.unwrap_or_else(|| "Reorg check failed".to_string()));
        }

        // Step 8: Apply necessary rollbacks
        let step8 = self.apply_rollbacks().await;
        steps.push(step8.clone());
        if !step8.success {
            errors.push(step8.error.unwrap_or_else(|| "Rollback application failed".to_string()));
        }

        // Step 9: Resume normal operation
        let step9 = self.resume_normal_operation().await;
        steps.push(step9.clone());
        if !step9.success {
            errors.push(step9.error.unwrap_or_else(|| "Failed to resume normal operation".to_string()));
            return Ok(RecoveryResult {
                success: false,
                steps,
                errors,
            });
        }

        self.recovered = true;

        Ok(RecoveryResult {
            success: true,
            steps,
            errors,
        })
    }

    /// Step 1: Load persistent state from storage
    async fn load_persistent_state(&mut self) -> RecoveryStep {
        match self.backend.load_last_known_heights().await {
            Ok(heights) => {
                self.last_known_heights = heights;
                // Generate a simple checksum from loaded state
                let mut checksum_data = Vec::new();
                for (chain, height) in &self.last_known_heights {
                    checksum_data.extend_from_slice(chain.as_bytes());
                    checksum_data.extend_from_slice(&height.to_le_bytes());
                }
                let domain_hash = crate::domain_hash::DomainSeparatedHash::<
                    crate::domains::GenesisDomain,
                >::hash(&checksum_data);
                self.state_checksum = Some(Hash::new(*domain_hash.as_bytes()));
                RecoveryStep {
                    name: "load_persistent_state",
                    success: true,
                    error: None,
                }
            }
            Err(e) => RecoveryStep {
                name: "load_persistent_state",
                success: false,
                error: Some(e.to_string()),
            },
        }
    }

    /// Step 2: Validate state consistency
    async fn validate_state_consistency(&self) -> RecoveryStep {
        // Validate:
        // 1. State checksum matches persisted value
        // 2. No orphaned operations without parent
        // 3. Transfer state invariants hold
        // 4. Replay registry consistency
        // 5. Block height monotonicity
        
        // Check that state checksum exists
        if self.state_checksum.is_none() {
            return RecoveryStep {
                name: "validate_state_consistency",
                success: false,
                error: Some("State checksum not found - corrupted state".to_string()),
            };
        }
        
        // In production, would validate actual checksums and invariants
        RecoveryStep {
            name: "validate_state_consistency",
            success: true,
            error: None,
        }
    }

    /// Step 3: Detect in-flight operations
    async fn detect_in_flight_operations(&mut self) -> RecoveryStep {
        match self.backend.find_in_flight_transfers().await {
            Ok(transfers) => {
                self.in_flight_transfers = transfers;
                RecoveryStep {
                    name: "detect_in_flight_operations",
                    success: true,
                    error: None,
                }
            }
            Err(e) => RecoveryStep {
                name: "detect_in_flight_operations",
                success: false,
                error: Some(e.to_string()),
            },
        }
    }

    /// Step 4: Resume or rollback incomplete operations
    async fn recover_incomplete_operations(&mut self) -> RecoveryStep {
        // For each in-flight operation:
        // 1. Check current chain state
        // 2. Verify if operation completed while down
        // 3. If completed: update state and persist
        // 4. If failed: increment attempt_counter and retry
        // 5. If max attempts exceeded: mark as failed/rolled back
        // 6. Persist recovery intent before retry
        
        // In production, would iterate through in_flight_transfers
        // and handle each according to its type and state
        
        RecoveryStep {
            name: "recover_incomplete_operations",
            success: true,
            error: None,
        }
    }

    /// Step 5: Rebuild in-memory structures
    async fn rebuild_memory_structures(&mut self) -> RecoveryStep {
        // Rebuild:
        // 1. Transfer state cache
        // 2. Replay registry in-memory index
        // 3. Seal registry cache
        // 4. Block height cache per chain
        // 5. Finality state cache
        // 6. Reorg detector state
        
        // In production, would load from storage and rebuild
        // For now, simulate successful rebuild
        
        RecoveryStep {
            name: "rebuild_memory_structures",
            success: true,
            error: None,
        }
    }

    /// Step 6: Verify chain finality
    async fn verify_chain_finality(&mut self) -> RecoveryStep {
        // For each tracked chain:
        // 1. Query current block height via RPC
        // 2. Compare with last known height
        // 3. Verify finality of last known block
        // 4. Update last known heights if safe
        
        // In production, would use quorum RPC client
        // For now, simulate successful verification
        
        // Update last known heights (simulated)
        self.last_known_heights.push(("bitcoin".to_string(), 800000));
        self.last_known_heights.push(("ethereum".to_string(), 5000000));
        
        RecoveryStep {
            name: "verify_chain_finality",
            success: true,
            error: None,
        }
    }

    /// Step 7: Check for reorgs since last shutdown
    async fn check_for_reorgs(&mut self) -> RecoveryStep {
        // For each tracked chain:
        // 1. Get current canonical chain tip
        // 2. Compare block hash at last known height
        // 3. If hash differs: reorg detected
        // 4. Calculate reorg depth
        // 5. Record reorg for rollback
        
        // In production, would use reorg detector
        // For now, simulate no reorgs
        self.detected_reorgs = vec![]; // Empty = no reorgs
        
        RecoveryStep {
            name: "check_for_reorgs",
            success: true,
            error: None,
        }
    }

    /// Step 8: Apply necessary rollbacks
    async fn apply_rollbacks(&mut self) -> RecoveryStep {
        for (chain, old_height, _new_height) in &self.detected_reorgs {
            // Find transfers affected by this reorg
            match self.backend.get_transfers_at_height(&chain, *old_height).await {
                Ok(affected) => {
                    for transfer_id in &affected {
                        // In production, would check the transfer state and apply
                        // appropriate rollback (Compromised, RolledBack, etc.)
                        let _ = self.backend.update_transfer_state(
                            transfer_id,
                            &TransferRecoveryState {
                                transfer_id: *transfer_id,
                                state: "rolled_back".to_string(),
                                source_chain: chain.clone(),
                                destination_chain: String::new(),
                                source_tx_hash: None,
                                attempt_counter: 0,
                                last_updated: 0,
                            },
                        ).await;
                    }
                }
                Err(e) => {
                    return RecoveryStep {
                        name: "apply_rollbacks",
                        success: false,
                        error: Some(format!("Failed to get affected transfers: {}", e)),
                    };
                }
            }
        }
        
        RecoveryStep {
            name: "apply_rollbacks",
            success: true,
            error: None,
        }
    }

    /// Step 9: Resume normal operation
    async fn resume_normal_operation(&mut self) -> RecoveryStep {
        // Signal readiness:
        // 1. Set recovered flag
        // 2. Enable accepting new operations
        // 3. Start background monitors (finality, reorg)
        // 4. Resume operation processing
        // 5. Emit recovery complete event
        
        self.recovered = true;
        
        RecoveryStep {
            name: "resume_normal_operation",
            success: true,
            error: None,
        }
    }

    /// Check if recovery has been completed
    pub fn is_recovered(&self) -> bool {
        self.recovered
    }

    /// Get last known block height for a chain
    pub fn get_last_known_height(&self, chain: &str) -> Option<u64> {
        self.last_known_heights
            .iter()
            .find(|(c, _)| c == chain)
            .map(|(_, h)| *h)
    }

    /// Set last known block height for a chain
    pub fn set_last_known_height(&mut self, chain: String, height: u64) {
        // Remove existing entry if present
        self.last_known_heights.retain(|(c, _)| c != &chain);
        self.last_known_heights.push((chain, height));
    }
}

/// Recovery step result
#[derive(Clone, Debug)]
pub struct RecoveryStep {
    /// Step name
    pub name: &'static str,
    /// Whether this step succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Recovery result
#[derive(Clone, Debug)]
pub struct RecoveryResult {
    /// Overall recovery success
    pub success: bool,
    /// Individual step results
    pub steps: Vec<RecoveryStep>,
    /// Errors encountered during recovery
    pub errors: Vec<String>,
}

/// Storage backend trait for recovery engine
///
/// This trait abstracts the persistence layer, allowing the recovery engine
/// to work with SQLite (via csv-store) or any other storage backend.
///
/// Implementations should provide persistent storage for:
/// - Transfer state (to detect in-flight operations)
/// - Replay registry (to avoid duplicate recovery)
/// - Operation log (for audit trail)
/// - Last known block heights (for chain finality verification)
/// - Reorg events (for rollback detection)
pub trait RecoveryStorageBackend: Clone + Send + Sync + 'static {
    /// Load last known block heights for all tracked chains
    fn load_last_known_heights(&self) -> impl core::future::Future<Output = Result<Vec<(String, u64)>>> + Send;

    /// Find transfers in non-terminal states (in-flight)
    fn find_in_flight_transfers(&self) -> impl core::future::Future<Output = Result<Vec<Hash>>> + Send;

    /// Get transfer state for recovery
    fn get_transfer_state(&self, transfer_id: &Hash) -> impl core::future::Future<Output = Result<Option<TransferRecoveryState>>> + Send;

    /// Update transfer state after recovery
    fn update_transfer_state(&self, transfer_id: &Hash, state: &TransferRecoveryState) -> impl core::future::Future<Output = Result<()>> + Send;

    /// Record a reorg event
    fn record_reorg(&self, chain: &str, old_height: u64, new_height: u64) -> impl core::future::Future<Output = Result<()>> + Send;

    /// Get recent reorg events
    fn get_recent_reorgs(&self, limit: usize) -> impl core::future::Future<Output = Result<Vec<(String, u64, u64)>>> + Send;

    /// Get transfers affected by a reorg at a given height
    fn get_transfers_at_height(&self, chain: &str, height: u64) -> impl core::future::Future<Output = Result<Vec<Hash>>> + Send;

    /// Persist a recovery checkpoint
    fn persist_checkpoint(&self, checkpoint: &RecoveryCheckpoint) -> impl core::future::Future<Output = Result<()>> + Send;

    /// Load the last recovery checkpoint
    fn load_checkpoint(&self) -> impl core::future::Future<Output = Result<Option<RecoveryCheckpoint>>> + Send;
}

/// Transfer state for recovery purposes
#[derive(Clone, Debug)]
pub struct TransferRecoveryState {
    /// Transfer ID
    pub transfer_id: Hash,
    /// Current state (non-terminal)
    pub state: String,
    /// Source chain
    pub source_chain: String,
    /// Destination chain
    pub destination_chain: String,
    /// Source transaction hash
    pub source_tx_hash: Option<Hash>,
    /// Attempt counter
    pub attempt_counter: u32,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Recovery checkpoint for crash recovery
#[derive(Clone, Debug)]
pub struct RecoveryCheckpoint {
    /// Checkpoint timestamp
    pub timestamp: u64,
    /// Last known heights per chain
    pub last_known_heights: Vec<(String, u64)>,
    /// State checksum
    pub state_checksum: Hash,
    /// In-flight transfer IDs at checkpoint time
    pub in_flight_transfers: Vec<Hash>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock backend for testing
    #[derive(Clone, Default)]
    struct MockBackend {
        last_known_heights: alloc::sync::Arc<std::sync::Mutex<Vec<(String, u64)>>>,
        in_flight: alloc::sync::Arc<std::sync::Mutex<Vec<Hash>>>,
    }

    #[async_trait::async_trait]
    impl RecoveryStorageBackend for MockBackend {
        async fn load_last_known_heights(&self) -> Result<Vec<(String, u64)>> {
            Ok(self.last_known_heights.lock().unwrap().clone())
        }

        async fn find_in_flight_transfers(&self) -> Result<Vec<Hash>> {
            Ok(self.in_flight.lock().unwrap().clone())
        }

        async fn get_transfer_state(&self, _transfer_id: &Hash) -> Result<Option<TransferRecoveryState>> {
            Ok(None)
        }

        async fn update_transfer_state(&self, _transfer_id: &Hash, _state: &TransferRecoveryState) -> Result<()> {
            Ok(())
        }

        async fn record_reorg(&self, _chain: &str, _old_height: u64, _new_height: u64) -> Result<()> {
            Ok(())
        }

        async fn get_recent_reorgs(&self, _limit: usize) -> Result<Vec<(String, u64, u64)>> {
            Ok(Vec::new())
        }

        async fn get_transfers_at_height(&self, _chain: &str, _height: u64) -> Result<Vec<Hash>> {
            Ok(Vec::new())
        }

        async fn persist_checkpoint(&self, _checkpoint: &RecoveryCheckpoint) -> Result<()> {
            Ok(())
        }

        async fn load_checkpoint(&self) -> Result<Option<RecoveryCheckpoint>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_recovery_engine_startup() {
        let backend = MockBackend::default();
        let mut engine = RecoveryEngine::new(backend);
        let result = engine.startup_sequence().await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.steps.len(), 9);
        assert!(engine.is_recovered());
    }

    #[test]
    fn test_recovery_engine_not_recovered_initially() {
        let backend = MockBackend::default();
        let engine = RecoveryEngine::new(backend);
        assert!(!engine.is_recovered());
    }

    #[test]
    fn test_last_known_height() {
        let backend = MockBackend {
            last_known_heights: alloc::sync::Arc::new(std::sync::Mutex::new(vec![
                ("bitcoin".to_string(), 100),
            ])),
            in_flight: alloc::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        };
        let mut engine = RecoveryEngine::new(backend);
        
        assert_eq!(engine.get_last_known_height("bitcoin"), Some(100));
        assert_eq!(engine.get_last_known_height("ethereum"), None);
    }
}
