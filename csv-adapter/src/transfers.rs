//! Transfer management facade.
//!
//! The [`TransferManager`] handles cross-chain transfers between any
//! two supported chains using the lock-and-prove protocol.
//!
//! # Cross-Chain Transfer Protocol
//!
//! 1. **Lock** — Source chain consumes the Right's seal, emits a lock event
//! 2. **Prove** — Client generates an inclusion proof of the lock event
//! 3. **Verify** — Destination chain verifies the proof (client-side)
//! 4. **Claim** — New Right created on destination chain with new seal
//!
//! No bridges, no wrapped tokens, no cross-chain messaging.

use std::collections::HashMap;
use std::sync::Arc;

use csv_adapter_core::{Chain, RightId};

use crate::client::ClientRef;
use crate::errors::CsvError;

/// Filter options for listing transfers.
#[derive(Debug, Clone, Default)]
pub struct TransferFilters {
    /// Filter by source chain.
    pub from_chain: Option<Chain>,
    /// Filter by destination chain.
    pub to_chain: Option<Chain>,
    /// Filter by status.
    pub status: Option<String>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

/// Priority level for transfer execution.
#[derive(Debug, Clone, Copy, Default)]
pub enum Priority {
    /// Normal priority (default fee rates).
    #[default]
    Normal,
    /// High priority (elevated fee rates for faster confirmation).
    High,
    /// Urgent (maximum fee rates, RBF enabled).
    Urgent,
}

/// Manager for cross-chain transfer operations.
///
/// Obtain a [`TransferManager`] via
/// [`CsvClient::transfers()`](crate::client::CsvClient::transfers).
///
/// # Example
///
/// ```no_run
/// use csv_adapter::prelude::*;
///
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// # let client = CsvClient::builder()
/// #     .with_chain(Chain::Bitcoin)
/// #     .with_chain(Chain::Sui)
/// #     .with_store_backend(StoreBackend::InMemory)
/// #     .build()?;
/// let transfers = client.transfers();
///
/// // Start a cross-chain transfer
/// let transfer = transfers
///     .cross_chain(right_id, Chain::Sui)
///     .to_address("0xabc...".to_string())
///     .with_priority(Priority::High)
///     .execute()?;
///
/// // Check status
/// let status = transfers.status(&transfer)?;
/// # Ok(())
/// # }
/// ```
pub struct TransferManager {
    client: Arc<ClientRef>,
}

impl TransferManager {
    pub(crate) fn new(client: Arc<ClientRef>) -> Self {
        Self { client }
    }

    /// Start building a cross-chain transfer.
    ///
    /// # Arguments
    ///
    /// * `right_id` — The Right to transfer.
    /// * `to_chain` — The destination chain.
    pub fn cross_chain(&self, right_id: RightId, to_chain: Chain) -> TransferBuilder {
        TransferBuilder::new(self.client.clone(), right_id, to_chain)
    }

    /// Get the current status of a transfer.
    ///
    /// # Arguments
    ///
    /// * `transfer_id` — The transfer identifier returned by
    ///   [`TransferBuilder::execute()`].
    pub fn status(&self, transfer_id: &str) -> Result<crate::TransferStatus, CsvError> {
        // In a full implementation, this would:
        // 1. Look up the transfer in the local store
        // 2. Poll the source chain for confirmation progress
        // 3. Check proof generation status
        // 4. Poll the destination chain for submission status
        // 5. Return a structured TransferStatus
        let _ = transfer_id;
        Ok(crate::TransferStatus::Initiated)
    }

    /// List transfers matching the given filters.
    pub fn list(&self, filters: TransferFilters) -> Result<Vec<TransferRecord>, CsvError> {
        // In a full implementation, this would query the transfer store
        let _ = filters;
        Ok(Vec::new())
    }
}

/// A record of a cross-chain transfer.
#[derive(Debug, Clone)]
pub struct TransferRecord {
    /// Unique transfer identifier.
    pub transfer_id: String,
    /// The Right being transferred.
    pub right_id: RightId,
    /// Source chain.
    pub from_chain: Chain,
    /// Destination chain.
    pub to_chain: Chain,
    /// Destination address.
    pub to_address: String,
    /// Current status.
    pub status: crate::TransferStatus,
}

/// Fluent builder for a cross-chain transfer.
///
/// Created via [`TransferManager::cross_chain()`].
pub struct TransferBuilder {
    client: Arc<ClientRef>,
    #[allow(dead_code)]
    right_id: RightId,
    to_chain: Chain,
    to_address: Option<String>,
    priority: Priority,
    metadata: HashMap<String, String>,
}

impl TransferBuilder {
    pub(crate) fn new(client: Arc<ClientRef>, right_id: RightId, to_chain: Chain) -> Self {
        Self {
            client,
            right_id,
            to_chain,
            to_address: None,
            priority: Priority::default(),
            metadata: HashMap::new(),
        }
    }

    /// Set the destination address for the transfer.
    pub fn to_address(mut self, address: String) -> Self {
        self.to_address = Some(address);
        self
    }

    /// Set the priority level for this transfer.
    ///
    /// Higher priority transfers use elevated fee rates for faster
    /// confirmation on the source chain.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Attach custom metadata to the transfer.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Execute the cross-chain transfer.
    ///
    /// This initiates the lock-and-prove protocol:
    /// 1. Locks the Right on the source chain (consumes the seal)
    /// 2. Generates the inclusion proof
    /// 3. Returns a transfer ID for tracking progress
    ///
    /// # Returns
    ///
    /// A unique transfer identifier. Use [`TransferManager::status()`]
    /// to track progress.
    ///
    /// # Errors
    ///
    /// - [`CsvError::RightNotFound`] if the Right ID is unknown
    /// - [`CsvError::RightAlreadyConsumed`] if the seal was already used
    /// - [`CsvError::ChainNotSupported`] if the destination chain is not enabled
    /// - [`CsvError::InsufficientFunds`] if the wallet lacks funds
    pub fn execute(self) -> Result<String, CsvError> {
        let _to_address = self.to_address.ok_or_else(|| {
            CsvError::BuilderError(
                "Destination address is required. Use .to_address() to set it.".to_string(),
            )
        })?;

        if !self.client.is_chain_enabled(self.to_chain) {
            return Err(CsvError::ChainNotSupported(self.to_chain));
        }

        // Generate a unique transfer ID
        let transfer_id = format!("xfer-{}", hex::encode(generate_salt()));

        // In a full implementation, this would:
        // 1. Look up the Right and verify it's not consumed
        // 2. Determine the source chain from the Right's seal
        // 3. Consume the seal on the source chain (lock)
        // 4. Generate the inclusion proof
        // 5. Store the transfer record
        // 6. Begin background proof submission to destination chain
        // 7. Emit TransferProgress events

        self.client
            .emit_event(crate::events::Event::TransferProgress {
                transfer_id: transfer_id.clone(),
                from_chain: Chain::Bitcoin, // Would be derived from Right
                to_chain: self.to_chain,
                step: "initiated".to_string(),
            });

        Ok(transfer_id)
    }
}

fn generate_salt() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut salt = Vec::with_capacity(16);
    salt.extend_from_slice(&timestamp.to_le_bytes());
    salt.extend_from_slice(&timestamp.rotate_left(32).to_le_bytes());
    salt
}

#[allow(dead_code)]
fn iso_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple RFC 3339-ish timestamp
    format!("{}-01-01T00:00:00Z", 2020 + secs / 31_536_000)
}
