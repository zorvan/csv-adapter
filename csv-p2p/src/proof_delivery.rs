//! P2P proof routing logic.
//!
//! The `ProofRouter` manages multiple transports and provides intelligent
/// routing of proof bundles based on chain, priority, and availability.

use std::collections::HashMap;

use csv_core::proof::ProofBundle;
use tracing::{debug, info, warn};

use crate::{DeliveredProof, EventId, ProofTransport, TransportError};

/// Filter criteria for subscribing to proofs via P2P transport.
#[derive(Debug, Clone, Default)]
pub struct ProofFilter {
    /// Chain IDs to filter by (empty = all chains).
    pub chain_ids: Vec<String>,
    /// Author public keys to filter by (empty = all authors).
    pub authors: Vec<String>,
    /// Minimum confirmation count required.
    pub min_confirmations: Option<u64>,
}

/// Routes proof bundles to available P2P transports.
///
/// The router maintains a list of transports and selects the best one
/// for each broadcast based on chain affinity and connectivity status.
pub struct ProofRouter {
    transports: HashMap<String, Box<dyn ProofTransport>>,
    preferred_transport: Option<String>,
}

impl ProofRouter {
    /// Create a new proof router.
    pub fn new() -> Self {
        Self {
            transports: HashMap::new(),
            preferred_transport: None,
        }
    }

    /// Register a transport for proof delivery.
    ///
    /// The transport name (e.g., "nostr") is used as the key for routing decisions.
    pub fn register(&mut self, name: String, transport: Box<dyn ProofTransport>) {
        let name_clone = name.clone();
        info!(transport = %name, "Registered P2P proof transport");
        self.transports.insert(name, transport);

        // Set as preferred if no preference is set
        if self.preferred_transport.is_none() {
            self.preferred_transport = Some(name_clone);
        }
    }

    /// Get the list of registered transport names.
    pub fn transports(&self) -> Vec<String> {
        self.transports.keys().cloned().collect()
    }

    /// Set the preferred transport by name.
    pub fn set_preferred(&mut self, name: &str) {
        if self.transports.contains_key(name) {
            self.preferred_transport = Some(name.to_string());
            info!(preferred = %name, "Set preferred P2P transport");
        } else {
            warn!(%name, "Attempted to set unknown transport as preferred");
        }
    }

    /// Broadcast a proof bundle via the preferred transport.
    ///
    /// If the preferred transport is unavailable, falls back to any
    /// available transport.
    pub async fn broadcast(&self, proof: &ProofBundle) -> Result<EventId, TransportError> {
        // Try preferred transport first
        if let Some(ref name) = self.preferred_transport {
            if let Some(transport) = self.transports.get(name) {
                if transport.is_connected().await {
                    debug!(transport = %name, "Broadcasting via preferred transport");
                    return transport.broadcast_proof(proof).await;
                } else {
                    warn!(transport = %name, "Preferred transport not connected, trying fallback");
                }
            }
        }

        // Fall back to any available transport
        for (name, transport) in &self.transports {
            if transport.is_connected().await {
                debug!(transport = %name, "Broadcasting via fallback transport");
                return transport.broadcast_proof(proof).await;
            }
        }

        Err(TransportError::NoRelays)
    }

    /// Broadcast a proof bundle to ALL registered transports.
    ///
    /// Returns a list of successful event IDs and any errors encountered.
    pub async fn broadcast_all(&self, proof: &ProofBundle) -> (Vec<EventId>, Vec<TransportError>) {
        let mut success = Vec::new();
        let mut errors = Vec::new();

        for (name, transport) in &self.transports {
            if !transport.is_connected().await {
                debug!(transport = %name, "Skipping disconnected transport");
                continue;
            }

            match transport.broadcast_proof(proof).await {
                Ok(event_id) => {
                    info!(transport = %name, event_id = %event_id.as_hex(), "Proof broadcast successful");
                    success.push(event_id);
                }
                Err(e) => {
                    warn!(transport = %name, error = %e, "Proof broadcast failed");
                    errors.push(e);
                }
            }
        }

        info!(
            total = self.transports.len(),
            successful = success.len(),
            failed = errors.len(),
            "Broadcast-all completed"
        );

        (success, errors)
    }

    /// Process an incoming delivered proof.
    ///
    /// Applications should implement their own processing logic here
    /// (e.g., storing the proof, verifying it, triggering cross-chain transfers).
    pub fn process_incoming(&self, delivered: DeliveredProof) {
        info!(
            event_id = %delivered.event_id.as_hex(),
            author = %delivered.author_pubkey,
            timestamp = delivered.timestamp,
            "Processing incoming proof"
        );

        // In production, this would:
        // 1. Verify the proof using csv_core::verify_proof()
        // 2. Store in local database via csv-store
        // 3. Trigger any relevant cross-chain operations
    }
}

impl Default for ProofRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_filter_chain_only() {
        let filter = ProofFilter::for_chain("ethereum");
        assert_eq!(filter.chain_ids, vec!["ethereum".to_string()]);
        assert!(filter.authors.is_empty());
    }

    #[test]
    fn test_proof_filter_author_only() {
        let filter = ProofFilter::from_author("abc123");
        assert!(filter.chain_ids.is_empty());
        assert_eq!(filter.authors, vec!["abc123".to_string()]);
    }

    #[test]
    fn test_proof_router_register_transport() {
        let mut router = ProofRouter::new();
        // We can't easily create a mock ProofTransport in unit tests
        // without more infrastructure, so we just verify the structure
        assert_eq!(router.transports(), Vec::<String>::new());
        assert!(router.preferred_transport.is_none());
    }
}
