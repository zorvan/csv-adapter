//! Proof Validated State
//!
//! Proof has been validated, ready for minting on destination chain.

use super::TransferData;

/// Transfer proof has been validated
#[derive(Clone, Debug)]
pub struct ProofValidated {
    /// Shared transfer data
    pub data: TransferData,
    /// Validated proof bytes
    pub proof: Vec<u8>,
    /// Proof validation timestamp
    pub validated_at: u64,
}

impl ProofValidated {
    /// Create a new proof validated state
    pub fn new(data: TransferData, proof: Vec<u8>) -> Self {
        Self {
            data,
            proof,
            validated_at: 0, // Will be set when validation completes
        }
    }

    /// Get the transfer data
    pub fn data(&self) -> &TransferData {
        &self.data
    }

    /// Get the proof
    pub fn proof(&self) -> &[u8] {
        &self.proof
    }
}
