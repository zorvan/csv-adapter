//! Bitcoin ChainVerifier Implementation
//!
//! This module implements the ChainVerifier trait for Bitcoin,
//! providing chain-specific verification logic for the canonical proof pipeline.

use async_trait::async_trait;
use csv_core::proof::{FinalityProof, InclusionProof};
use csv_core::proof_pipeline::ChainVerifier;
use csv_core::Hash;

use crate::proofs::verify_inclusion_proof;
use crate::rpc::BitcoinRpc;

/// Bitcoin verifier implementing ChainVerifier trait
pub struct BitcoinVerifier {
    /// RPC client for Bitcoin
    rpc: Box<dyn BitcoinRpc + Send + Sync>,
}

impl BitcoinVerifier {
    /// Create a new Bitcoin verifier
    pub fn new(rpc: Box<dyn BitcoinRpc + Send + Sync>) -> Self {
        Self { rpc }
    }
}

#[async_trait]
impl ChainVerifier for BitcoinVerifier {
    /// Verify inclusion proof for a Bitcoin transaction
    async fn verify_inclusion(
        &self,
        proof: &InclusionProof,
        expected_root: Hash,
    ) -> csv_core::Result<bool> {
        // Use the existing Bitcoin SPV verification logic
        verify_inclusion_proof(
            &proof.proof_bytes,
            proof.block_hash.as_bytes(),
            expected_root.as_bytes(),
        )
        .map_err(|e| csv_core::error::ProtocolError::VerificationError(e.to_string()))
    }

    /// Verify finality proof for a Bitcoin block
    async fn verify_finality(&self, proof: &FinalityProof) -> csv_core::Result<bool> {
        // Bitcoin finality is probabilistic - check confirmations
        // FinalityProof has confirmations field directly
        let confirmations = proof.confirmations;

        // Require at least 6 confirmations for Bitcoin finality
        Ok(confirmations >= 6)
    }

    /// Verify zero-knowledge proof (if applicable)
    async fn verify_zk(&self, _proof: &[u8]) -> csv_core::Result<bool> {
        // Bitcoin SPV doesn't use ZK proofs
        Ok(true)
    }
}
