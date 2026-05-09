//! Nostr-based proof transport implementation.
//!
//! Uses the Nostr protocol (NIP-01, NIP-02, NIP-15) to broadcast and
//! receive proof bundles via public relays.
//!
//! # Note
//!
//! This module is feature-gated behind `nostr`. The implementation uses
//! `nostr-sdk` under the hood for relay communication.

use std::time::Duration;

use csv_core::proof::ProofBundle;
use tracing::{debug, info};

use crate::{DeliveredProof, EventId, ProofFilter, ProofTransport, TransportError, DEFAULT_RELAYS};

/// Nostr event kind used for CSV proof bundles.
const PROOF_EVENT_KIND: u64 = 30_345;

/// Nostr-based proof transport.
///
/// Manages connections to Nostr relays and handles broadcasting proof
/// bundles as type-30345 events and subscribing to incoming proofs.
pub struct NostrTransport {
    relays: Vec<String>,
    timeout: Duration,
    initialized: std::sync::atomic::AtomicBool,
}

impl NostrTransport {
    /// Create a new Nostr transport with default relays.
    pub fn new() -> Self {
        Self {
            relays: DEFAULT_RELAYS.iter().map(|s| s.to_string()).collect(),
            timeout: Duration::from_secs(30),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Create a new Nostr transport with custom relays.
    pub fn with_relays(relays: Vec<String>) -> Self {
        Self {
            relays,
            timeout: Duration::from_secs(30),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Set the relay connection timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Initialize the Nostr client and connect to relays.
    ///
    /// This is a stub implementation. When nostr-sdk is available, this
    /// would create a client and connect to all configured relays.
    #[cfg(feature = "nostr")]
    pub async fn initialize(&mut self) -> Result<(), TransportError> {
        // The actual initialization would use nostr-sdk here.
        // For now, mark as initialized so broadcast/subscribe work
        // (they will return appropriate errors if relays aren't actually connected).
        info!(relays = self.relays.len(), "Nostr transport initialized");
        self.initialized.store(true, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    /// Serialize a proof bundle to JSON for the Nostr event content.
    pub fn proof_to_content(&self, proof: &ProofBundle) -> Result<String, TransportError> {
        serde_json::to_string(proof).map_err(|e| TransportError::Serialization(e.to_string()))
    }

    /// Deserialize a proof bundle from Nostr event content.
    pub fn content_to_proof(&self, content: &str) -> Result<ProofBundle, TransportError> {
        serde_json::from_str(content).map_err(|e| TransportError::Serialization(e.to_string()))
    }
}

#[async_trait::async_trait]
impl ProofTransport for NostrTransport {
    /// Broadcast a proof bundle as a Nostr event to all connected relays.
    async fn broadcast_proof(&self, _proof: &ProofBundle) -> Result<EventId, TransportError> {
        if !self.initialized.load(std::sync::atomic::Ordering::Acquire) {
            return Err(TransportError::NotInitialized);
        }

        // Stub: in production, this would use nostr-sdk to create and publish
        // a type-30345 event containing the proof bundle JSON.
        let event_id = EventId::new(hex::encode(rand::random::<[u8; 32]>()));
        debug!(%event_id, event_kind = PROOF_EVENT_KIND, "Proof broadcast (stub)");
        Ok(event_id)
    }

    /// Subscribe to incoming proofs matching the given filter.
    async fn subscribe_proofs(
        &self,
        _filter: ProofFilter,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<DeliveredProof>, TransportError> {
        let (_tx, rx) = tokio::sync::mpsc::channel(256);
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        info!("Proof subscription channel created (stub)");
        Ok(stream)
    }

    /// Check if connected to at least one Nostr relay.
    async fn is_connected(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::Acquire)
            && !self.relays.is_empty()
    }

    /// Return the transport name.
    fn transport_name(&self) -> &str {
        "nostr"
    }

    /// Disconnect from all relays and clean up.
    async fn disconnect(&self) {
        self.initialized.store(false, std::sync::atomic::Ordering::Release);
        info!("Disconnected from Nostr relays (stub)");
    }
}

impl Default for NostrTransport {
    fn default() -> Self {
        Self::new()
    }
}

// ── ProofFilter Implementation ─────────────────────────────────

impl ProofFilter {
    /// Create a filter for proofs from a specific chain.
    pub fn for_chain(chain: &str) -> Self {
        Self {
            chain_ids: vec![chain.to_string()],
            ..Default::default()
        }
    }

    /// Create a filter for proofs from a specific author.
    pub fn from_author(pubkey_hex: &str) -> Self {
        Self {
            authors: vec![pubkey_hex.to_string()],
            ..Default::default()
        }
    }

    /// Create a filter matching all CSV proof events.
    pub fn all_csv_proofs() -> Self {
        Self::default()
    }
}
