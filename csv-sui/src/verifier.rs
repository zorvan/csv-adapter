//! Sui ChainVerifier Implementation
//!
//! This module implements the ChainVerifier trait for Sui,
//! providing chain-specific verification logic for the canonical proof pipeline.

use async_trait::async_trait;
use csv_core::proof::{FinalityProof, InclusionProof};
use csv_core::proof_pipeline::ChainVerifier;
use csv_core::Hash;

use crate::rpc::SuiRpc;

/// Sui verifier implementing ChainVerifier trait
pub struct SuiVerifier {
    /// RPC client for Sui
    rpc: Box<dyn SuiRpc>,
}

impl SuiVerifier {
    /// Create a new Sui verifier
    pub fn new(rpc: Box<dyn SuiRpc>) -> Self {
        Self { rpc }
    }
}

#[async_trait]
impl ChainVerifier for SuiVerifier {
    /// Verify inclusion proof for a Sui transaction
    async fn verify_inclusion(
        &self,
        proof: &InclusionProof,
        _expected_root: Hash,
    ) -> csv_core::Result<bool> {
        // Use the existing Sui transaction proof verification logic
        // For now, check if proof bytes are non-empty
        Ok(!proof.proof_bytes.is_empty())
    }

    /// Verify finality proof for a Sui block
    async fn verify_finality(&self, proof: &FinalityProof) -> csv_core::Result<bool> {
        // Sui has instant finality via Narwhal/Bullshark consensus
        // Check if block has confirmations
        Ok(proof.confirmations > 0)
    }

    /// Verify zero-knowledge proof (if applicable)
    async fn verify_zk(&self, proof: &[u8]) -> csv_core::Result<bool> {
        // Sui doesn't use ZK proofs for basic operations
        // Return true if proof is empty, otherwise verify if needed
        if proof.is_empty() {
            Ok(true)
        } else {
            // Placeholder - would verify actual ZK proof if Sui adds ZK support
            Ok(true)
        }
    }

    /// Verify seal registry (check if seal has been consumed)
    async fn verify_seal_registry(&self, _seal_id: Hash) -> csv_core::Result<bool> {
        // Placeholder - would query Sui blockchain to check if object is consumed
        Ok(true)
    }

    /// Verify signature on proof bundle
    async fn verify_signature(&self, _bundle: &csv_core::proof::ProofBundle) -> csv_core::Result<bool> {
        // Placeholder - would verify signature on proof bundle
        Ok(true)
    }
}
