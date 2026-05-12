//! Locked State
//!
//! Transfer is locked on source chain, awaiting proof submission.

use super::TransferData;

/// Transfer is locked on source chain
#[derive(Clone, Debug)]
pub struct Locked {
    /// Shared transfer data
    pub data: TransferData,
    /// Block height when lock occurred
    pub lock_height: u64,
    /// Lock transaction hash
    pub lock_tx_hash: Vec<u8>,
}

impl Locked {
    /// Create a new locked state
    pub fn new(data: TransferData, lock_height: u64, lock_tx_hash: Vec<u8>) -> Self {
        Self {
            data,
            lock_height,
            lock_tx_hash,
        }
    }

    /// Get the transfer data
    pub fn data(&self) -> &TransferData {
        &self.data
    }
}
