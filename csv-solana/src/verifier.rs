//! Solana ChainVerifier Implementation
//!
//! This module implements the ChainVerifier trait for Solana,
//! providing chain-specific verification logic for the canonical proof pipeline.

use async_trait::async_trait;
use csv_core::proof::{FinalityProof, InclusionProof};
use csv_core::proof_pipeline::ChainVerifier;
use csv_core::Hash;

use crate::proofs::verify_inclusion_proof;
use crate::rpc::SolanaRpc;

/// Solana verifier implementing ChainVerifier trait
pub struct SolanaVerifier {
    /// RPC client for Solana
    rpc: Box<dyn SolanaRpc>,
}

impl SolanaVerifier {
    /// Create a new Solana verifier
    pub fn new(rpc: Box<dyn SolanaRpc>) -> Self {
        Self { rpc }
    }
}

#[async_trait]
impl ChainVerifier for SolanaVerifier {
    /// Verify inclusion proof for a Solana transaction
    async fn verify_inclusion(
        &self,
        proof: &InclusionProof,
        expected_root: Hash,
    ) -> csv_core::Result<bool> {
        // Use the existing Solana slot proof verification logic
        Ok(verify_inclusion_proof(proof, &expected_root))
    }

    /// Verify finality proof for a Solana block
    async fn verify_finality(&self, proof: &FinalityProof) -> csv_core::Result<bool> {
        // Solana has probabilistic finality - check if slot is finalized (32+ confirmations)
        let is_finalized = self
            .rpc
            .is_slot_finalized(proof.block_height)
            .await
            .map_err(|e| csv_core::error::ProtocolError::RpcError(e.to_string()))?;

        Ok(is_finalized)
    }

    /// Verify zero-knowledge proof (if applicable)
    async fn verify_zk(&self, proof: &[u8]) -> csv_core::Result<bool> {
        // Solana doesn't use ZK proofs for basic operations
        // Return true if proof is empty, otherwise verify if needed
        if proof.is_empty() {
            Ok(true)
        } else {
            // Placeholder - would verify actual ZK proof if Solana adds ZK support
            Ok(true)
        }
    }
}
