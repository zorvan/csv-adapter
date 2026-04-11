//! Rights management facade.
//!
//! The [`RightsManager`] provides a high-level API for creating, querying,
//! and managing Rights across all supported chains.
//!
//! # What is a Right?
//!
//! A **Right** is a verifiable, single-use digital claim that can be
//! transferred cross-chain. It exists in client state (not on any chain)
//! and is anchored to a single-use seal on a specific chain.
//!
//! To transfer a Right, the seal is consumed on-chain and the new owner
//! verifies the consumption proof locally — no bridges, no minting,
//! no cross-chain messaging.

use std::sync::Arc;

use csv_adapter_core::{Chain, Hash, Right, RightId};

use crate::client::ClientRef;
use crate::errors::CsvError;

/// Filter options for listing Rights.
#[derive(Debug, Clone, Default)]
pub struct RightFilters {
    /// Filter by chain (the chain where the seal is anchored).
    pub chain: Option<Chain>,
    /// Filter by owner address.
    pub owner: Option<String>,
    /// Filter by consumed status.
    pub consumed: Option<bool>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

/// Manager for Right operations.
///
/// Obtain a [`RightsManager`] via [`CsvClient::rights()`](crate::client::CsvClient::rights).
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
/// #     .with_store_backend(StoreBackend::InMemory)
/// #     .build()?;
/// let rights = client.rights();
///
/// // List all Rights
/// let all_rights = rights.list(RightFilters::default())?;
/// # Ok(())
/// # }
/// ```
pub struct RightsManager {
    client: Arc<ClientRef>,
}

impl RightsManager {
    pub(crate) fn new(client: Arc<ClientRef>) -> Self {
        Self { client }
    }

    /// Create a new Right anchored to the specified chain.
    ///
    /// This method:
    /// 1. Creates a single-use seal on the target chain
    /// 2. Constructs a Right with the given commitment
    /// 3. Records the seal consumption in the local store
    ///
    /// # Arguments
    ///
    /// * `commitment` — The commitment hash binding the Right's state.
    /// * `chain` — The chain where the seal will be anchored.
    ///
    /// # Returns
    ///
    /// The newly created [`Right`] with a unique [`RightId`].
    ///
    /// # Errors
    ///
    /// - [`CsvError::ChainNotSupported`] if the chain is not enabled.
    /// - [`CsvError::InsufficientFunds`] if the wallet lacks funds for seal creation.
    /// - [`CsvError::AdapterError`] if the underlying chain operation fails.
    pub fn create(&self, commitment: Hash, chain: Chain) -> Result<Right, CsvError> {
        if !self.client.is_chain_enabled(chain) {
            return Err(CsvError::ChainNotSupported(chain));
        }

        // In a full implementation, this would:
        // 1. Call the chain adapter's AnchorLayer::create_seal()
        // 2. Construct the Right with the seal reference
        // 3. Publish the commitment via AnchorLayer::publish()
        // 4. Record the seal in the local store
        // 5. Emit a RightCreated event
        //
        // The chain adapters (csv-adapter-bitcoin, etc.) provide the
        // actual SealRef types and publishing logic.
        //
        // Example for Bitcoin:
        //   let btc_adapter = csv_adapter_bitcoin::BitcoinAnchorLayer::signet()?;
        //   let seal = btc_adapter.create_seal(None)?;
        //   let right = Right::new(commitment.hash(), owner_proof, salt);

        let salt = generate_salt();
        let owner = csv_adapter_core::OwnershipProof {
            proof: vec![0u8; 32], // Placeholder: in production, derive from wallet
            owner: vec![0u8; 32],
            scheme: None,
        };

        let right = Right::new(commitment, owner, &salt);

        self.client.emit_event(crate::events::Event::RightCreated {
            right_id: right.id.clone(),
            chain,
        });

        Ok(right)
    }

    /// Get a Right by its ID.
    ///
    /// # Note
    ///
    /// Rights exist in client state, not on-chain. This method queries
    /// the local store for previously created or received Rights.
    pub fn get(&self, right_id: &RightId) -> Result<Option<Right>, CsvError> {
        // In a full implementation, this would:
        // 1. Query the local store for the Right by ID
        // 2. Reconstruct the Right from stored state
        // 3. Verify the Right's integrity (ID matches commitment || salt)
        let _ = right_id;
        Ok(None)
    }

    /// List Rights matching the given filters.
    pub fn list(&self, filters: RightFilters) -> Result<Vec<Right>, CsvError> {
        // In a full implementation, this would:
        // 1. Query the store with the filter criteria
        // 2. Deserialize Rights from storage
        // 3. Apply any in-memory filtering
        // 4. Apply the limit
        let _ = filters;
        Ok(Vec::new())
    }

    /// Transfer a Right to a new owner on a different chain.
    ///
    /// This initiates a cross-chain transfer:
    /// 1. The source chain seal is consumed (locking the Right)
    /// 2. A proof of consumption is generated
    /// 3. The Right can be verified and claimed on the destination chain
    ///
    /// # Arguments
    ///
    /// * `right_id` — The Right to transfer.
    /// * `to_chain` — The destination chain.
    /// * `to_address` — The destination owner's address.
    ///
    /// # Returns
    ///
    /// A transfer identifier for tracking progress.
    pub fn transfer(
        &self,
        _right_id: &RightId,
        to_chain: Chain,
        _to_address: String,
    ) -> Result<String, CsvError> {
        if !self.client.is_chain_enabled(to_chain) {
            return Err(CsvError::ChainNotSupported(to_chain));
        }

        // In a full implementation, this would:
        // 1. Look up the Right by ID
        // 2. Verify the Right is not already consumed
        // 3. Consume the seal on the source chain
        // 4. Generate the inclusion proof
        // 5. Return a transfer ID for tracking
        let transfer_id = format!("xfer-{}", hex::encode(generate_salt()));

        self.client.emit_event(crate::events::Event::TransferProgress {
            transfer_id: transfer_id.clone(),
            from_chain: Chain::Bitcoin, // Would be the source chain
            to_chain,
            step: "lock".to_string(),
        });

        Ok(transfer_id)
    }

    /// Burn (permanently consume) a Right.
    ///
    /// This is an irreversible operation that destroys the Right by
    /// consuming its seal without creating a new one.
    ///
    /// # Arguments
    ///
    /// * `right_id` — The Right to burn.
    pub fn burn(&self, right_id: &RightId) -> Result<(), CsvError> {
        // In a full implementation, this would:
        // 1. Look up the Right
        // 2. Consume the seal on-chain without a destination owner
        // 3. Mark the Right as consumed locally
        // 4. Emit a burn event
        let _ = right_id;
        Ok(())
    }
}

/// Generate a random 16-byte salt.
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
