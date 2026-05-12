//! Rollback Handler
//!
//! Handles rollback of operations affected by a reorg.
//! When a chain reorg occurs, this module identifies and rolls back
//! transfers that were dependent on blocks in the reorged chain segment.

use crate::protocol_version::ChainId;
use crate::reorg::detector::ReorgEvent;

/// Type of rollback action to take
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackAction {
    /// Transfer should be marked as rolled back (source lock still valid)
    RolledBack,
    /// Transfer should be marked as compromised (source lock invalidated)
    Compromised,
    /// Transfer should be resumed from a prior state
    ///
    /// The `from_state` field indicates which state the transfer should resume from.
    Resume {
        /// State to resume from (e.g., "proof_validated")
        from_state: String,
    },
}

/// Rollback handler for reorg events
///
/// When a reorg is detected, this handler:
/// 1. Identifies transfers affected by the reorg
/// 2. Determines the appropriate rollback action based on transfer state
/// 3. Executes the rollback (marking transfers as RolledBack or Compromised)
/// 4. Emits events for observability
pub struct RollbackHandler {
    /// Callbacks registered per chain
    on_rollback: alloc::collections::BTreeMap<String, Box<dyn Fn(&ReorgEvent) + Send + Sync>>,
    /// Whether to auto-execute rollbacks or require manual approval
    auto_execute: bool,
}

impl RollbackHandler {
    /// Create a new rollback handler
    pub fn new() -> Self {
        Self {
            on_rollback: alloc::collections::BTreeMap::new(),
            auto_execute: true,
        }
    }

    /// Set whether rollbacks are auto-executed
    pub fn set_auto_execute(&mut self, auto: bool) {
        self.auto_execute = auto;
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

    /// Determine the rollback action for a transfer based on its state and the reorg
    ///
    /// The decision logic:
    /// - If the transfer's source lock was at or below the reorg depth: Compromised
    /// - If the transfer's proof was built on a reorged block: RolledBack
    /// - If the transfer's mint was on a reorged block: Resume from ProofValidated
    pub fn determine_rollback_action(
        &self,
        transfer_state: &str,
        source_block_height: u64,
        reorg_from_height: u64,
        reorg_to_height: u64,
    ) -> RollbackAction {
        // If the source lock is below the reorg range, the lock is invalidated
        if source_block_height < reorg_from_height {
            return RollbackAction::Compromised;
        }

        // If the transfer was in a state that depends on the reorged blocks
        match transfer_state {
            // Locking state - source lock may be invalidated
            "locking" | "awaiting_finality" if source_block_height <= reorg_to_height => {
                RollbackAction::Compromised
            }
            // Proof building state - proof may be invalid
            "proof_building" | "proof_validated" if source_block_height <= reorg_to_height => {
                RollbackAction::RolledBack
            }
            // Minting state - mint may have been on reorged block
            "minting" if source_block_height <= reorg_to_height => {
                RollbackAction::Resume {
                    from_state: "proof_validated".to_string(),
                }
            }
            // Completed transfers are not affected (finality confirmed)
            "completed" => RollbackAction::RolledBack, // No action needed
            // Other states - conservative rollback
            _ => RollbackAction::RolledBack,
        }
    }

    /// Roll back transfers affected by a reorg
    ///
    /// This is a placeholder that would query storage for affected transfers
    /// and mark them as rolled back. In production, this integrates with
    /// the recovery engine and transfer state machine.
    pub fn rollback_transfers(
        &self,
        chain: ChainId,
        from_height: u64,
        to_height: u64,
        affected_transfers: &[(String, String, u64)],
        // (transfer_id, state, source_block_height)
    ) {
        for (transfer_id, state, block_height) in affected_transfers {
            let action = self.determine_rollback_action(state, *block_height, from_height, to_height);

            match action {
                RollbackAction::Compromised => {
                    // In production: mark transfer as CompromisedTransfer
                    // Emit Compromised event
                    log::warn!(
                        "Transfer {} on chain {} marked as COMPROMISED (source lock invalidated at height {})",
                        transfer_id, chain, block_height
                    );
                }
                RollbackAction::RolledBack => {
                    // In production: mark transfer as RolledBackTransfer
                    // Emit RollbackExecuted event
                    log::warn!(
                        "Transfer {} on chain {} marked as ROLLED BACK (reorg at height {})",
                        transfer_id, chain, block_height
                    );
                }
                RollbackAction::Resume { from_state } => {
                    // In production: resume transfer from prior state
                    log::info!(
                        "Transfer {} on chain {} resuming from {} (reorg at height {})",
                        transfer_id, chain, from_state, block_height
                    );
                }
            }
        }
    }
}

impl Default for RollbackHandler {
    fn default() -> Self {
        Self::new()
    }
}
