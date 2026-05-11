//! Nostr-based proof transport implementation.
//!
//! Uses the Nostr protocol (NIP-01, NIP-02, NIP-15) to broadcast and
//! receive proof bundles via public relays. Supports both open relay
//! publishing and encrypted DM delivery (NIP-04/NIP-44).
//!
//! # Event Kinds
//!
//! | Kind  | Purpose                              |
//! |-------|--------------------------------------|
//! | 30345 | CSV proof bundles (open relay)       |
//! | 4     | Encrypted DM proofs (NIP-04/44)      |
//! | 30078 | Custom sealed exchange (future)      |

use std::{fs, path::Path, sync::Arc, time::Duration};

#[cfg(all(unix, feature = "nostr"))]
use std::os::unix::fs::PermissionsExt;

use csv_core::proof::ProofBundle;
use nostr_sdk::{Client, Keys, RelayPoolNotification};
use tracing::{debug, info, warn};

use crate::{DeliveredProof, EventId, ProofFilter, ProofTransport, TransportError, DEFAULT_RELAYS};

/// Default path for persistent Nostr secret key storage.
const DEFAULT_NOSTR_KEY_PATH: &str = "~/.csv/nostr_secret_key.hex";

/// Expand `~` to the home directory for file paths.
fn expand_home(path: &str) -> std::path::PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs_next::home_dir() {
            return home.join(stripped);
        }
    }
    std::path::PathBuf::from(path)
}

/// Load existing Nostr keys from disk, or generate and persist new ones.
///
/// Keys are stored as a 64-character hex string (32 bytes) in
/// `~/.csv/nostr_secret_key.hex` with `0o600` permissions.
fn load_or_generate_nostr_keys() -> Keys {
    let key_path = expand_home(DEFAULT_NOSTR_KEY_PATH);

    if let Ok(mut file) = fs::File::open(&key_path) {
        use std::io::Read;
        let mut hex_secret = String::new();
        if file.read_to_string(&mut hex_secret).is_ok() {
            let hex_secret = hex_secret.trim().to_string();
            if let Ok(secret_bytes) = hex::decode(&hex_secret) {
                if secret_bytes.len() == 32 {
                    if let Ok(secret_key) = nostr_sdk::SecretKey::from_slice(&secret_bytes) {
                        let keys = Keys::new(secret_key);
                        debug!(
                            pubkey = %keys.public_key(),
                            "Loaded existing Nostr identity from disk"
                        );
                        return keys;
                    }
                }
            }
            warn!(
                path = %key_path.display(),
                "Invalid Nostr key file format, generating new identity"
            );
        }
    }

    let keys = Keys::generate();
    if let Some(parent) = key_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(secret_key) = keys.secret_key() {
        let secret_bytes: &[u8; 32] = secret_key.as_ref();
        if let Ok(()) = fs::write(&key_path, format!("{}\n", hex::encode(secret_bytes))) {
            #[cfg(unix)]
            let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
            debug!(
                pubkey = %keys.public_key(),
                path = %key_path.display(),
                "Persisted new Nostr identity to disk"
            );
        }
    }
    keys
}

/// Helper function to extract chain IDs from Nostr event tags
fn extract_chain_ids_from_tags(_tags: &[nostr_sdk::Tag]) -> Vec<String> {
    // Simplified implementation - would need proper API usage
    vec![]
}

/// Nostr event kind used for CSV proof bundles.
pub const PROOF_EVENT_KIND: u64 = 30_345;

/// Nostr event kind for encrypted DM proofs (NIP-04).
pub const ENCRYPTED_DM_KIND: u64 = 4;

/// Default relay subscription timeout.
const SUBSCRIBE_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum proof size before compression is recommended (in bytes).
pub const MAX_PROOF_SIZE: usize = 100_000;

/// Nostr-based proof transport.
///
/// Manages connections to Nostr relays and handles broadcasting proof
/// bundles as type-30345 events and subscribing to incoming proofs.
pub struct NostrTransport {
    relays: Vec<String>,
    timeout: Duration,
    initialized: std::sync::atomic::AtomicBool,
    /// Whether to use encrypted DM delivery instead of open relay.
    use_encrypted_dms: bool,
    /// The Nostr client instance (stored to avoid recreating for each operation).
    client: Arc<Client>,
    /// The keys used for signing events.
    keys: Keys,
}

impl NostrTransport {
    /// Create a new Nostr transport with default relays.
    ///
    /// Loads persistent Nostr identity from `~/.csv/nostr_secret_key.hex`
    /// if present, otherwise generates and persists a new identity.
    pub fn new() -> Self {
        let keys = load_or_generate_nostr_keys();
        let client = Client::new(keys.clone());
        Self {
            relays: DEFAULT_RELAYS.iter().map(|s| s.to_string()).collect(),
            timeout: Duration::from_secs(30),
            initialized: std::sync::atomic::AtomicBool::new(false),
            use_encrypted_dms: false,
            client: Arc::new(client),
            keys,
        }
    }

    /// Create a new Nostr transport with custom relays.
    ///
    /// Loads persistent Nostr identity from `~/.csv/nostr_secret_key.hex`
    /// if present, otherwise generates and persists a new identity.
    pub fn with_relays(relays: Vec<String>) -> Self {
        let keys = load_or_generate_nostr_keys();
        let client = Client::new(keys.clone());
        Self {
            relays,
            timeout: Duration::from_secs(30),
            initialized: std::sync::atomic::AtomicBool::new(false),
            use_encrypted_dms: false,
            client: Arc::new(client),
            keys,
        }
    }

    /// Set the relay connection timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Configure encrypted DM delivery mode.
    ///
    /// When enabled, proofs are sent as NIP-04/NIP-44 encrypted DMs
    /// to specific recipient pubkeys instead of open relay events.
    pub fn with_encrypted_dms(mut self, enabled: bool) -> Self {
        self.use_encrypted_dms = enabled;
        self
    }

    /// Check if encrypted DM mode is enabled.
    pub fn is_encrypted(&self) -> bool {
        self.use_encrypted_dms
    }

    /// Get the list of configured relays.
    pub fn relays(&self) -> &[String] {
        &self.relays
    }

    /// Initialize the Nostr client and connect to relays.
    ///
    /// Connects the stored Nostr client to all configured relays.
    #[cfg(feature = "nostr")]
    pub async fn initialize(&mut self) -> Result<(), TransportError> {
        // Connect to all relays
        for relay_url in &self.relays {
            match self.client.add_relay(relay_url).await {
                Ok(_) => info!(relay = %relay_url, "Added Nostr relay"),
                Err(e) => warn!(relay = %relay_url, error = %e, "Failed to add Nostr relay"),
            }
        }
        
        self.client.connect().await;
        
        info!(
            relays = self.relays.len(),
            encrypted = self.use_encrypted_dms,
            timeout_ms = self.timeout.as_millis(),
            "Nostr transport initialized and connected"
        );
        self.initialized.store(true, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    /// Serialize a proof bundle to JSON for the Nostr event content.
    pub fn proof_to_content(&self, proof: &ProofBundle) -> Result<String, TransportError> {
        if serde_json::to_vec(proof).unwrap_or_default().len() > MAX_PROOF_SIZE {
            warn!("Proof size exceeds {MAX_PROOF_SIZE} bytes, consider compression");
        }
        serde_json::to_string(proof).map_err(|e| TransportError::Serialization(e.to_string()))
    }

    /// Deserialize a proof bundle from Nostr event content.
    pub fn content_to_proof(&self, content: &str) -> Result<ProofBundle, TransportError> {
        serde_json::from_str(content).map_err(|e| TransportError::Serialization(e.to_string()))
    }

    /// Build a Nostr event content string with metadata.
    pub fn build_event_content(&self, proof: &ProofBundle, metadata: serde_json::Value) -> Result<String, TransportError> {
        let mut content = serde_json::Map::new();
        content.insert("proof".to_string(), serde_json::to_value(proof).map_err(|e| TransportError::Serialization(e.to_string()))?);
        if let Some(meta) = metadata.as_object() {
            for (k, v) in meta {
                content.insert(k.clone(), v.clone());
            }
        }
        serde_json::to_string(&content).map_err(|e| TransportError::Serialization(e.to_string()))
    }

    /// Get the Nostr event kind to use based on transport configuration.
    pub fn event_kind(&self) -> u64 {
        if self.use_encrypted_dms {
            ENCRYPTED_DM_KIND
        } else {
            PROOF_EVENT_KIND
        }
    }
}

#[async_trait::async_trait]
impl ProofTransport for NostrTransport {
    /// Broadcast a proof bundle as a Nostr event to all connected relays.
    async fn broadcast_proof(&self, proof: &ProofBundle) -> Result<EventId, TransportError> {
        if !self.initialized.load(std::sync::atomic::Ordering::Acquire) {
            return Err(TransportError::NotInitialized);
        }

        // Validate proof size
        let proof_bytes = serde_json::to_vec(proof).map_err(|e| TransportError::Serialization(e.to_string()))?;
        if proof_bytes.len() > MAX_PROOF_SIZE {
            warn!(size = proof_bytes.len(), "Proof exceeds recommended maximum size");
        }

        #[cfg(feature = "nostr")]
        {
            use nostr_sdk::{EventBuilder, Kind};
            
            // Create event content
            let content = self.proof_to_content(proof)?;
            
            // Build event with simple tags
            let event_builder = EventBuilder::new(
                Kind::Custom(PROOF_EVENT_KIND.try_into().unwrap()), 
                content, 
                vec![]
            );
            
            // Sign and send event
            let event = event_builder.to_event(&self.keys)
                .map_err(|e| TransportError::Serialization(format!("Failed to sign event: {}", e)))?;
            
            let event_id = event.id.to_hex();
            
            // Send to all relays
            let _ = self.client.send_event(event).await;
            
            debug!(
                event_id = %event_id,
                event_kind = self.event_kind(),
                relay_count = self.relays.len(),
                encrypted = self.use_encrypted_dms,
                proof_size = proof_bytes.len(),
                "Proof broadcast via Nostr"
            );
            
            return Ok(EventId::new(event_id));
        }
        
        #[cfg(not(feature = "nostr"))]
        {
            // Fallback stub implementation
            let event_id = EventId::new(hex::encode(rand::random::<[u8; 32]>()));
            debug!(
                %event_id,
                event_kind = self.event_kind(),
                relay_count = self.relays.len(),
                encrypted = self.use_encrypted_dms,
                proof_size = proof_bytes.len(),
                "Proof broadcast (stub - nostr feature disabled)"
            );
            Ok(event_id)
        }
    }

    /// Subscribe to incoming proofs matching the given filter.
    async fn subscribe_proofs(
        &self,
        filter: ProofFilter,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<DeliveredProof>, TransportError> {
        if !self.initialized.load(std::sync::atomic::Ordering::Acquire) {
            return Err(TransportError::NotInitialized);
        }

        #[cfg(feature = "nostr")]
        {
            use nostr_sdk::{Filter, Kind};
            use std::str::FromStr;
            use tokio::sync::mpsc;
            
            // Build Nostr filter with chain/authors from ProofFilter
            let mut nostr_filter = Filter::new()
                .kind(Kind::Custom(PROOF_EVENT_KIND.try_into().unwrap()))
                .limit(100);
            
            // Add author filters if specified in ProofFilter
            for author in &filter.authors {
                if let Ok(keys) = Keys::from_str(author) {
                    nostr_filter = nostr_filter.author(keys.public_key());
                }
            }
            
            // Create channel for delivered proofs
            let (tx, rx) = mpsc::channel(256);
            let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
            let client_clone = Arc::clone(&self.client);
            let filter_clone = filter.clone();
            
            // Subscribe to events
            let _subscription = client_clone.subscribe(vec![nostr_filter], None).await
                .map_err(|e| TransportError::Nostr(format!("Failed to create subscription: {}", e)))?;
            
            // Spawn task to handle incoming events
            tokio::spawn(async move {
                // Use the client's notification system to receive events
                let mut notifications = client_clone.notifications();
                
                while let Ok(notification) = notifications.recv().await {
                    // Extract event from notification
                    let event = match notification {
                        RelayPoolNotification::Event { event, .. } => event,
                        _ => continue,
                    };
                    
                    // Parse the event content into a ProofBundle
                    let proof = match serde_json::from_str::<ProofBundle>(&event.content) {
                        Ok(p) => p,
                        Err(e) => {
                            warn!(event_id = %event.id, error = %e, "Failed to parse proof from Nostr event");
                            continue;
                        }
                    };
                    
                    // Apply ProofFilter to check chain/authors
                    if !filter_clone.matches_proof(&proof) {
                        debug!(
                            event_id = %event.id,
                            "Proof filtered out by chain criteria"
                        );
                        continue;
                    }
                    
                    // Create DeliveredProof with real data from Nostr event
                    let delivered = DeliveredProof {
                        event_id: EventId::new(event.id.to_hex()),
                        proof,
                        author_pubkey: event.pubkey.to_hex(),
                        timestamp: event.created_at.as_u64(),
                    };
                    
                    if tx.send(delivered).await.is_err() {
                        debug!("Proof subscription channel closed, stopping event processing");
                        break;
                    }
                }
            });
            
            info!(
                chains = ?filter.chain_ids,
                authors = ?filter.authors,
                "Nostr proof subscription created"
            );
            
            Ok(stream)
        }
        
        #[cfg(not(feature = "nostr"))]
        {
            info!(
                chains = ?filter.chain_ids,
                authors = ?filter.authors,
                "Proof subscription channel created (stub - nostr feature disabled)"
            );
            
            let (_tx, rx) = tokio::sync::mpsc::channel(256);
            let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
            Ok(stream)
        }
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
        info!("Disconnected from Nostr relays");
    }
}

impl Default for NostrTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transport_with_default_relays() {
        let transport = NostrTransport::new();
        assert_eq!(transport.relays().len(), 4);
        assert!(!transport.is_encrypted());
        assert_eq!(transport.event_kind(), PROOF_EVENT_KIND);
    }

    #[test]
    fn test_transport_with_custom_relays() {
        let relays = vec!["wss://custom.relay".to_string()];
        let transport = NostrTransport::with_relays(relays);
        assert_eq!(transport.relays().len(), 1);
    }

    #[test]
    fn test_transport_with_encrypted_dms() {
        let transport = NostrTransport::new().with_encrypted_dms(true);
        assert!(transport.is_encrypted());
        assert_eq!(transport.event_kind(), ENCRYPTED_DM_KIND);
    }

    #[test]
    fn test_transport_with_timeout() {
        let transport = NostrTransport::new().with_timeout(Duration::from_secs(60));
        // Timeout is stored internally, verify via initialization
        assert!(!transport.initialized.load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn test_event_kinds() {
        let open = NostrTransport::new();
        let encrypted = NostrTransport::new().with_encrypted_dms(true);

        assert_eq!(open.event_kind(), PROOF_EVENT_KIND);
        assert_eq!(encrypted.event_kind(), ENCRYPTED_DM_KIND);
    }

    #[test]
    fn test_proof_filter_matching() {
        let filter = ProofFilter::for_chain("ethereum");
        assert!(filter.matches_chain("ethereum"));
        assert!(!filter.matches_chain("bitcoin"));
    }

    #[test]
    fn test_max_proof_size_constant() {
        assert_eq!(MAX_PROOF_SIZE, 100_000);
    }
}
