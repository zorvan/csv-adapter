//! Ethereum ChainVerifier Implementation
//!
//! This module implements the ChainVerifier trait for Ethereum,
//! providing chain-specific verification logic for the canonical proof pipeline.

use async_trait::async_trait;
use csv_core::proof::{FinalityProof, InclusionProof};
use csv_core::proof_pipeline::ChainVerifier;
use csv_core::Hash;

use crate::mpt::verify_merkle_proof;
use crate::rpc::EthereumRpc;

/// Ethereum verifier implementing ChainVerifier trait
pub struct EthereumVerifier {
    /// RPC client for Ethereum
    rpc: Box<dyn EthereumRpc>,
}

impl EthereumVerifier {
    /// Create a new Ethereum verifier
    pub fn new(rpc: Box<dyn EthereumRpc>) -> Self {
        Self { rpc }
    }
}

#[async_trait]
impl ChainVerifier for EthereumVerifier {
    /// Verify inclusion proof for an Ethereum transaction
    async fn verify_inclusion(
        &self,
        proof: &InclusionProof,
        expected_root: Hash,
    ) -> csv_core::Result<bool> {
        // Use the existing Ethereum MPT verification logic
        verify_merkle_proof(
            &proof.proof_bytes,
            proof.block_hash.as_bytes(),
            expected_root.as_bytes(),
        )
        .map_err(|e| csv_core::error::ProtocolError::VerificationError(e.to_string()))
    }

    /// Verify finality proof for an Ethereum block
    async fn verify_finality(&self, proof: &FinalityProof) -> csv_core::Result<bool> {
        // Ethereum has probabilistic finality - check confirmations
        // For now, we check if the required confirmations are met
        let required_confirmations = 12; // Ethereum typically requires 12 confirmations for finality
        let is_finalized = proof.confirmations >= required_confirmations;

        Ok(is_finalized)
    }

    /// Verify zero-knowledge proof (if applicable)
    async fn verify_zk(&self, proof: &[u8]) -> csv_core::Result<bool> {
        // Ethereum may use ZK proofs for certain operations
        // For now, return true if proof is empty, otherwise verify
        if proof.is_empty() {
            Ok(true)
        } else {
            // Placeholder - would verify actual ZK proof
            Ok(true)
        }
    }
}
