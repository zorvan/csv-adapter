//! Bitcoin ChainVerifier Implementation
//!
//! This module implements the ChainVerifier trait for Bitcoin,
//! providing chain-specific verification logic for the canonical proof pipeline.

use async_trait::async_trait;
use csv_core::proof::{FinalityProof, InclusionProof};
use csv_core::proof_pipeline::ChainVerifier;
use csv_core::Hash;

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
        _expected_root: Hash,
    ) -> csv_core::Result<bool> {
        // Use the existing Bitcoin SPV verification logic
        // For now, return true if proof bytes are non-empty
        // TODO: Implement proper SPV verification with all required parameters
        Ok(!proof.proof_bytes.is_empty())
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

    /// Verify seal registry (check if seal has been consumed)
    async fn verify_seal_registry(&self, _seal_id: Hash) -> csv_core::Result<bool> {
        // Placeholder - would query Bitcoin blockchain to check if UTXO is spent
        Ok(true)
    }

    /// Verify signature on proof bundle
    async fn verify_signature(&self, _bundle: &csv_core::proof::ProofBundle) -> csv_core::Result<bool> {
        // Placeholder - would verify signature on proof bundle
        Ok(true)
    }
}
