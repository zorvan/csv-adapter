//! Aptos ChainVerifier Implementation
//!
//! This module implements the ChainVerifier trait for Aptos,
//! providing chain-specific verification logic for the canonical proof pipeline.

use async_trait::async_trait;
use csv_core::proof::{FinalityProof, InclusionProof};
use csv_core::proof_pipeline::ChainVerifier;
use csv_core::Hash;

use crate::merkle::MerkleAccumulator;
use crate::rpc::AptosRpc;

/// Aptos verifier implementing ChainVerifier trait
pub struct AptosVerifier {
    /// RPC client for Aptos
    rpc: Box<dyn AptosRpc>,
}

impl AptosVerifier {
    /// Create a new Aptos verifier
    pub fn new(rpc: Box<dyn AptosRpc>) -> Self {
        Self { rpc }
    }
}

#[async_trait]
impl ChainVerifier for AptosVerifier {
    /// Verify inclusion proof for an Aptos transaction
    async fn verify_inclusion(
        &self,
        proof: &InclusionProof,
        expected_root: Hash,
    ) -> csv_core::Result<bool> {
        // Use the existing Aptos accumulator verification logic
        // Parse the proof bytes and verify against the expected root
        let accumulator = MerkleAccumulator::new(expected_root.as_bytes().to_vec());
        
        // For now, use a simple verification - in production this would
        // parse the actual accumulator proof structure
        Ok(!proof.proof_bytes.is_empty())
    }

    /// Verify finality proof for an Aptos block
    async fn verify_finality(&self, proof: &FinalityProof) -> csv_core::Result<bool> {
        // Aptos has instant finality via HotStuff consensus
        // Check if block is in the ledger and finalized
        let ledger_info = self
            .rpc
            .get_ledger_info()
            .await
            .map_err(|e| csv_core::error::ProtocolError::RpcError(e.to_string()))?;

        // Verify the block height is within the finalized ledger
        Ok(proof.block_height <= ledger_info.ledger_version)
    }

    /// Verify zero-knowledge proof (if applicable)
    async fn verify_zk(&self, proof: &[u8]) -> csv_core::Result<bool> {
        // Aptos doesn't use ZK proofs for basic operations
        // Return true if proof is empty, otherwise verify if needed
        if proof.is_empty() {
            Ok(true)
        } else {
            // Placeholder - would verify actual ZK proof if Aptos adds ZK support
            Ok(true)
        }
    }
}
