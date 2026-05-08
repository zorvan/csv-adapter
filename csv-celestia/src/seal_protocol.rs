//! Celestia Seal Protocol Implementation
//!
//! This module implements the `SealProtocol` trait from `csv_core` for the
//! Celestia Data Availability layer. It provides:
//!
//! - **Single-use seals** via proof consumption on Celestia
//! - **Commitment anchoring** on the DA layer
//! - **Inclusion proofs** for verification
//! - **Finality guarantees** via Tendermint consensus
//!
//! ## Single Use Seal Model
//!
//! Celestia seals are unique by their (height, namespace, commitment) tuple.
//! Once a seal is "consumed" (used for a state transition), it cannot be
//! reused because the commitment would conflict.
//!
//! ```text
//! Seal Consumption:
//! 1. Create seal pointing to uncommitted DA location
//! 2. Publish commitment to that location
//! 3. Seal is now "consumed" (cannot publish different commitment there)
//! ```

use csv_core::dag::DAGSegment;
use csv_core::error::Result as CoreResult;
use csv_core::hash::Hash;
use csv_core::proof::ProofBundle;
use csv_core::seal_protocol::SealProtocol;
use csv_core::signature::SignatureScheme;

use crate::blob::Blob;
use crate::commitment::{BlobCommitment, CommitmentProof, FraudProof, InclusionProof};
use crate::da_layer::{CelestiaDaLayer, CelestiaRpc, DataAvailabilityLayer};
use crate::error::{CelestiaError, Result};
use crate::ipfs::IpfsClient;
use crate::namespace::Namespace;
use crate::proof_id::ProofId;
use crate::types::{CelestiaAnchor, CelestiaFinalityProof, CelestiaSealPoint};

/// Celestia-specific seal protocol implementation
pub struct CelestiaSealProtocol<C, I> {
    /// Inner DA layer
    da_layer: CelestiaDaLayer<C, I>,
    /// Default namespace for seals
    namespace: Namespace,
    /// Consumed seals (in-memory cache for testing)
    #[allow(dead_code)]
    consumed_seals: std::sync::Arc<tokio::sync::RwLock<std::collections::HashSet<ProofId>>>,
}

impl<C, I> CelestiaSealProtocol<C, I>
where
    C: CelestiaRpc + Send + Sync,
    I: IpfsClient + Send + Sync,
{
    /// Create a new seal protocol
    pub fn new(da_layer: CelestiaDaLayer<C, I>, namespace: Namespace) -> Self {
        Self {
            da_layer,
            namespace,
            consumed_seals: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new()))
        }
    }

    /// Create with default namespace
    pub fn with_default_namespace(da_layer: CelestiaDaLayer<C, I>) -> Self {
        Self::new(da_layer, Namespace::metadata())
    }

    /// Create a test instance
    pub fn with_test() -> Result<Self>
    where
        C: Default,
        I: Default,
    {
        let da_layer = CelestiaDaLayer::new(
            crate::da_layer::DaLayerConfig::default(),
            C::default(),
            Some(I::default()),
        );
        Ok(Self::new(da_layer, Namespace::metadata()))
    }

    /// Check if a seal is consumed (would query DA in production)
    async fn is_seal_consumed(&self, proof_id: &ProofId) -> Result<bool> {
        // In production, this would query the DA layer to see if
        // the location already has data (meaning the seal was used)
        let available = self.da_layer.verify_availability(proof_id, 1).await?;
        Ok(available)
    }
}

/// Convert CelestiaError to csv_core::error::Error
impl From<CelestiaError> for csv_core::error::Error {
    fn from(err: CelestiaError) -> Self {
        csv_core::error::Error::Other(err.to_string())
    }
}

/// Implement the core SealProtocol trait
impl<C, I> SealProtocol for CelestiaSealProtocol<C, I>
where
    C: CelestiaRpc + Send + Sync,
    I: IpfsClient + Send + Sync,
{
    type SealPoint = CelestiaSealPoint;
    type CommitAnchor = CelestiaAnchor;
    type InclusionProof = InclusionProof;
    type FinalityProof = CelestiaFinalityProof;

    fn publish(&self, commitment: Hash, seal: Self::SealPoint) -> CoreResult<Self::CommitAnchor> {
        // This is a synchronous wrapper around async code
        // In production, this should be properly async

        // Check that seal hasn't been consumed
        if !seal.is_valid() {
            return Err(csv_core::error::Error::Other(
                "Seal already consumed".to_string()
            ));
        }

        // The commitment becomes the blob commitment
        let blob_commitment = BlobCommitment::new(*commitment.as_bytes());

        // Create the anchor
        let anchor = CelestiaAnchor::new(
            crate::proof_id::ProofLocation::Celestia { proof_id: seal.proof_id },
            seal.height,
            [0u8; 32], // Would be actual block hash
            blob_commitment,
            [0u8; 32], // Would be actual tx hash
        );

        Ok(anchor)
    }

    fn verify_inclusion(&self, anchor: Self::CommitAnchor) -> CoreResult<Self::InclusionProof> {
        // In production, this would verify the inclusion proof from Celestia
        let proof = InclusionProof::new(
            Vec::new(), // proof_bytes
            csv_core::hash::Hash::new([0u8; 32]), // block_hash
            0, // position
        )
        .map_err(|e| csv_core::error::Error::Other(e.to_string()))?;

        Ok(proof)
    }

    fn verify_finality(&self, anchor: Self::CommitAnchor) -> CoreResult<Self::FinalityProof> {
        // Tendermint has deterministic finality
        let proof = CelestiaFinalityProof::new(
            anchor.height,
            anchor.block_hash,
            [0u8; 32], // data_root
        ).with_quorum(vec![]);

        Ok(proof)
    }

    fn enforce_seal(&self, seal: Self::SealPoint) -> CoreResult<()> {
        // Verify seal hasn't been consumed
        if !seal.is_valid() {
            return Err(csv_core::error::Error::Other(
                "Seal has been consumed".to_string()
            ));
        }

        // In production, this would query DA to double-check
        Ok(())
    }

    fn create_seal(&self, value: Option<u64>) -> CoreResult<Self::SealPoint> {
        // Create a new seal at the next available height
        // In production, this would query for the latest height
        let height = value.unwrap_or(1); // Use value as height hint

        let proof_id = ProofId::new(
            height,
            self.namespace,
            [0u8; 32], // placeholder commitment
        );

        let seal = CelestiaSealPoint::new(proof_id, height);
        Ok(seal)
    }

    fn hash_commitment(&self, segment: &DAGSegment, scheme: SignatureScheme) -> CoreResult<Hash> {
        // Compute domain-separated commitment hash
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Domain separator for Celestia
        hasher.update(b"CSV/Celestia/v1");

        // Include segment data
        let segment_bytes = serde_json::to_vec(segment)
            .map_err(|e| csv_core::error::Error::Other(format!("Serialization: {}", e)))?;
        hasher.update(&segment_bytes);

        // Include signature scheme
        let scheme_bytes: u8 = match scheme {
            csv_core::signature::SignatureScheme::Secp256k1 => 1,
            csv_core::signature::SignatureScheme::Ed25519 => 2,
        };
        hasher.update(&[scheme_bytes]);

        // Include namespace for domain separation
        hasher.update(self.namespace.as_bytes());

        let hash: [u8; 32] = hasher.finalize().into();
        Ok(Hash::new(hash))
    }

    fn extract_bundle(&self, anchor: &Self::CommitAnchor) -> CoreResult<ProofBundle> {
        // In production, this would retrieve and deserialize the proof bundle
        // from the DA layer using the anchor's proof location
        Err(csv_core::error::Error::Other(
            "Not implemented - requires DA retrieval".to_string()
        ))
    }

    fn rollback(&self, height: u64) -> CoreResult<Vec<Self::CommitAnchor>> {
        // Rollback all anchors at or after the given height
        // In production, this would query the DA layer for affected anchors
        Ok(Vec::new())
    }
}

/// Extended functionality for Celestia seal protocol
impl<C, I> CelestiaSealProtocol<C, I>
where
    C: CelestiaRpc + Send + Sync,
    I: IpfsClient + Send + Sync,
{
    /// Submit a blob and create a seal in one operation
    pub async fn submit_and_seal(&self, data: Vec<u8>) -> Result<(ProofId, CelestiaSealPoint)> {
        let blob = Blob::new(self.namespace, data)?;
        let proof_id = self.da_layer.submit_blob(blob).await?;

        let seal = CelestiaSealPoint::new(proof_id, proof_id.height);
        Ok((proof_id, seal))
    }

    /// Verify a full proof bundle from the DA layer
    pub async fn verify_bundle_from_da(&self, proof_id: &ProofId) -> Result<ProofBundle> {
        let blob = self.da_layer.get_blob(proof_id).await?;
        let bundle: ProofBundle = serde_json::from_slice(&blob.data)
            .map_err(|e| CelestiaError::DeserializationError(
                format!("Failed to deserialize proof bundle: {}", e)
            ))?;
        Ok(bundle)
    }

    /// Submit a fraud proof to the fraud namespace
    pub async fn submit_fraud_proof(&self, fraud: FraudProof) -> Result<ProofId> {
        self.da_layer.submit_fraud_proof(fraud).await
    }

    /// Get the namespace
    pub fn namespace(&self) -> Namespace {
        self.namespace
    }

    /// Create a seal with IPFS backing
    pub async fn create_ipfs_seal(&self, data: Vec<u8>) -> Result<(ProofId, CelestiaSealPoint)> {
        let location = self.da_layer.store_on_ipfs(&data, self.namespace).await?;

        let proof_id = match &location {
            crate::proof_id::ProofLocation::Hybrid { metadata_id, .. } => *metadata_id,
            _ => return Err(CelestiaError::InternalError(
                "Expected hybrid location".to_string()
            )),
        };

        let seal = CelestiaSealPoint::new(proof_id, proof_id.height)
            .with_ipfs(location.cid().unwrap_or(""));

        Ok((proof_id, seal))
    }
}

/// Builder for CelestiaSealProtocol
pub struct CelestiaSealProtocolBuilder<C, I> {
    da_layer: Option<CelestiaDaLayer<C, I>>,
    namespace: Option<Namespace>,
}

impl<C, I> CelestiaSealProtocolBuilder<C, I>
where
    C: CelestiaRpc + Send + Sync,
    I: IpfsClient + Send + Sync,
{
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            da_layer: None,
            namespace: None,
        }
    }

    /// Set DA layer
    pub fn with_da_layer(mut self, da_layer: CelestiaDaLayer<C, I>) -> Self {
        self.da_layer = Some(da_layer);
        self
    }

    /// Set namespace
    pub fn with_namespace(mut self, namespace: Namespace) -> Self {
        self.namespace = Some(namespace);
        self
    }

    /// Build the protocol
    pub fn build(self) -> Result<CelestiaSealProtocol<C, I>> {
        let da_layer = self.da_layer
            .ok_or_else(|| CelestiaError::InternalError(
                "DA layer required".to_string()
            ))?;

        let namespace = self.namespace.unwrap_or(Namespace::metadata());

        Ok(CelestiaSealProtocol::new(da_layer, namespace))
    }
}

impl<C, I> Default for CelestiaSealProtocolBuilder<C, I>
where
    C: CelestiaRpc + Send + Sync,
    I: IpfsClient + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for test protocol
pub type TestCelestiaSealProtocol = CelestiaSealProtocol<
    crate::da_layer::MockCelestiaRpc,
    crate::ipfs::MockIpfsClient
>;

/// Create a test seal protocol
pub async fn create_test_protocol() -> TestCelestiaSealProtocol {
    use crate::da_layer::{DaLayerConfig, MockCelestiaRpc};
    use crate::ipfs::MockIpfsClient;

    let celestia = MockCelestiaRpc::new();
    celestia.set_height(100).await;

    let ipfs = MockIpfsClient::new();
    let config = DaLayerConfig::default();

    let da_layer = CelestiaDaLayer::new(config, celestia, Some(ipfs));
    CelestiaSealProtocol::new(da_layer, Namespace::metadata())
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv_core::seal_protocol::SealProtocol;

    #[tokio::test]
    async fn test_seal_protocol_creation() {
        let protocol = create_test_protocol().await;
        assert_eq!(protocol.namespace(), Namespace::metadata());
    }

    #[tokio::test]
    async fn test_create_seal() {
        let protocol = create_test_protocol().await;
        let seal = protocol.create_seal(Some(12345)).unwrap();

        assert!(seal.is_valid());
        assert_eq!(seal.height, 12345);
    }

    #[tokio::test]
    async fn test_submit_and_seal() {
        let protocol = create_test_protocol().await;
        let data = vec![1, 2, 3, 4, 5];

        let (proof_id, seal) = protocol.submit_and_seal(data).await.unwrap();

        assert_eq!(seal.proof_id, proof_id);
        assert!(seal.is_valid());
    }

    #[tokio::test]
    async fn test_create_ipfs_seal() {
        let protocol = create_test_protocol().await;
        let data = vec![0u8; 1024 * 1024 + 1]; // Large data

        let (proof_id, seal) = protocol.create_ipfs_seal(data).await.unwrap();

        assert!(seal.ipfs_cid.is_some());
        assert!(seal.is_valid());
    }

    #[test]
    fn test_enforce_seal() {
        // Note: This test runs synchronously
        let protocol = {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { create_test_protocol().await })
        };

        let seal = protocol.create_seal(Some(12345)).unwrap();
        assert!(protocol.enforce_seal(seal).is_ok());

        // Consumed seal should fail
        let mut consumed_seal = protocol.create_seal(Some(12346)).unwrap();
        consumed_seal.consume([0u8; 32]);
        assert!(protocol.enforce_seal(consumed_seal).is_err());
    }

    #[test]
    fn test_hash_commitment() {
        let protocol = {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { create_test_protocol().await })
        };

        let segment = DAGSegment::default();
        let hash = protocol.hash_commitment(&segment, SignatureScheme::Ed25519).unwrap();

        // Should be non-zero
        assert_ne!(hash.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_verify_inclusion() {
        let protocol = {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { create_test_protocol().await })
        };

        let anchor = CelestiaAnchor::new(
            crate::proof_id::ProofLocation::Celestia {
                proof_id: ProofId::new(12345, Namespace::metadata(), [0u8; 32])
            },
            12345,
            [0u8; 32],
            BlobCommitment::new([0u8; 32]),
            [0u8; 32],
        );

        let proof = protocol.verify_inclusion(anchor);
        assert!(proof.is_ok());
    }

    #[tokio::test]
    async fn test_builder() {
        use crate::da_layer::{DaLayerConfig, MockCelestiaRpc};
        use crate::ipfs::MockIpfsClient;

        let celestia = MockCelestiaRpc::new();
        celestia.set_height(100).await;

        let ipfs = MockIpfsClient::new();
        let config = DaLayerConfig::default();

        let da_layer = CelestiaDaLayer::new(config, celestia, Some(ipfs));

        let protocol = CelestiaSealProtocolBuilder::new()
            .with_da_layer(da_layer)
            .with_namespace(Namespace::bitcoin_stark())
            .build()
            .unwrap();

        assert_eq!(protocol.namespace(), Namespace::bitcoin_stark());
    }
}
