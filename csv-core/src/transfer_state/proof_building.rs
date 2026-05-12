//! Proof Building State
//!
//! Building zero-knowledge proof for the transfer.

use super::TransferData;

/// Transfer is in proof building phase
#[derive(Clone, Debug)]
pub struct ProofBuilding {
    /// Shared transfer data
    pub data: TransferData,
    /// Proof generation started at
    pub started_at: u64,
    /// Proof progress (0-100)
    pub progress: u8,
}

impl ProofBuilding {
    /// Create a new proof building state
    pub fn new(data: TransferData) -> Self {
        Self {
            data,
            started_at: 0, // Will be set when proof building starts
            progress: 0,
        }
    }

    /// Update proof progress
    pub fn update_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }

    /// Get the transfer data
    pub fn data(&self) -> &TransferData {
        &self.data
    }
}
