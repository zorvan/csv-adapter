//! CSV Mint Contract Bindings
//!
//! Type-safe bindings for the CSV Mint contract using Alloy.

// Placeholder for Alloy-generated bindings
// This would be generated from the CSVMint.sol contract using alloy-sol-types

/// CSV Mint contract interface
pub struct CsvMint {
    /// Contract address
    pub address: [u8; 20],
}

impl CsvMint {
    /// Create a new CSV Mint contract reference
    pub fn new(address: [u8; 20]) -> Self {
        Self { address }
    }

    /// Get the contract address
    pub fn address(&self) -> [u8; 20] {
        self.address
    }
}
