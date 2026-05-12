//! CSV Lock Contract Bindings
//!
//! Type-safe bindings for the CSV Lock contract using Alloy.

// Placeholder for Alloy-generated bindings
// This would be generated from the CSVLock.sol contract using alloy-sol-types

/// CSV Lock contract interface
pub struct CsvLock {
    /// Contract address
    pub address: [u8; 20],
}

impl CsvLock {
    /// Create a new CSV Lock contract reference
    pub fn new(address: [u8; 20]) -> Self {
        Self { address }
    }

    /// Get the contract address
    pub fn address(&self) -> [u8; 20] {
        self.address
    }
}
