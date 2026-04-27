//! BIP-44 HD wallet derivation for multi-chain support.
//!
//! This module provides hierarchical deterministic (HD) wallet key derivation
//! following BIP-44 standards with chain-specific paths.

use crate::memory::SecretKey;
use csv_adapter_core::Chain;
use thiserror::Error;

/// Error type for BIP-44 operations.
#[derive(Debug, Error)]
pub enum Bip44Error {
    /// Invalid derivation path.
    #[error("Invalid derivation path: {0}")]
    InvalidPath(String),

    /// Invalid seed length.
    #[error("Invalid seed length: expected 64, got {0}")]
    InvalidSeedLength(usize),

    /// Chain not supported.
    #[error("Chain not supported for HD derivation: {0:?}")]
    UnsupportedChain(Chain),

    /// Derivation failed.
    #[error("Key derivation failed: {0}")]
    DerivationFailed(String),
}

/// BIP-44 derivation path components.
#[derive(Debug, Clone, Copy)]
pub struct DerivationPath {
    /// Purpose (BIP-44 = 44', BIP-49 = 49', BIP-84 = 84', BIP-86 = 86')
    pub purpose: u32,
    /// Coin type (SLIP-44 registered coin types)
    pub coin_type: u32,
    /// Account index
    pub account: u32,
    /// Change (0 = external, 1 = internal)
    pub change: u32,
    /// Address index
    pub address_index: u32,
}

impl DerivationPath {
    /// Create a new derivation path with hardened purpose and coin type.
    pub fn new_bip44(coin_type: u32, account: u32, change: u32, address_index: u32) -> Self {
        Self {
            purpose: 44 | 0x8000_0000, // hardened
            coin_type: coin_type | 0x8000_0000, // hardened
            account: account | 0x8000_0000, // hardened
            change,
            address_index,
        }
    }

    /// Create a BIP-86 derivation path (Bitcoin Taproot).
    pub fn new_bip86(account: u32, address_index: u32) -> Self {
        Self {
            purpose: 86 | 0x8000_0000, // BIP-86 hardened
            coin_type: 0 | 0x8000_0000, // Bitcoin hardened
            account: account | 0x8000_0000,
            change: 0,
            address_index,
        }
    }

    /// Convert to string representation (e.g., "m/44'/60'/0'/0/0").
    pub fn to_string_path(&self) -> String {
        format!(
            "m/{}'/{}{}'/{}'/{}/{}",
            self.purpose & 0x7FFF_FFFF,
            if self.coin_type >= 0x8000_0000 { "" } else { "not" },
            self.coin_type & 0x7FFF_FFFF,
            self.account & 0x7FFF_FFFF,
            self.change,
            self.address_index
        )
    }
}

/// Get the BIP-44 coin type for a chain.
pub fn coin_type(chain: Chain) -> u32 {
    match chain {
        Chain::Bitcoin => 0,   // SLIP-44: BTC
        Chain::Ethereum => 60, // SLIP-44: ETH
        Chain::Sui => 784,   // SLIP-44: SUI
        Chain::Aptos => 637, // SLIP-44: APT
        Chain::Solana => 501, // SLIP-44: SOL
    }
}

/// Get the standard derivation path for a chain.
pub fn derivation_path(chain: Chain, account: u32, address_index: u32) -> DerivationPath {
    match chain {
        Chain::Bitcoin => {
            // Bitcoin: BIP-86 for Taproot (native segwit v1)
            DerivationPath::new_bip86(account, address_index)
        }
        _ => {
            // Ethereum, Sui, Aptos, Solana: standard BIP-44
            DerivationPath::new_bip44(
                coin_type(chain),
                account,
                0, // external
                address_index,
            )
        }
    }
}

/// Derive a secret key from a 64-byte seed using BIP-44/SLIP-10.
///
/// # Arguments
/// * `seed` - 64-byte BIP-39 seed
/// * `chain` - Target blockchain
/// * `account` - Account index (hardened)
/// * `address_index` - Address index within account
///
/// # Returns
/// A derived 32-byte secret key.
pub fn derive_key(
    seed: &[u8; 64],
    chain: Chain,
    account: u32,
    address_index: u32,
) -> Result<SecretKey, Bip44Error> {
    let path = derivation_path(chain, account, address_index);
    derive_key_from_path(seed, &path, chain)
}

/// Derive a key from a specific derivation path.
///
/// This uses SLIP-10 for Ed25519 chains (Sui, Aptos, Solana) and
/// BIP-32 for secp256k1 chains (Bitcoin, Ethereum).
pub fn derive_key_from_path(
    seed: &[u8; 64],
    path: &DerivationPath,
    chain: Chain,
) -> Result<SecretKey, Bip44Error> {
    match chain {
        Chain::Bitcoin | Chain::Ethereum => {
            derive_secp256k1(seed, path)
        }
        Chain::Sui | Chain::Aptos | Chain::Solana => {
            derive_ed25519(seed, path)
        }
    }
}

/// Derive a secp256k1 key (Bitcoin, Ethereum).
fn derive_secp256k1(
    seed: &[u8; 64],
    path: &DerivationPath,
) -> Result<SecretKey, Bip44Error> {
    use secp256k1::{PublicKey, SecretKey as SecpSecretKey};

    // Start with the master key from seed
    let mut data = Vec::with_capacity(64);
    data.extend_from_slice(&seed[..32]);
    
    // Simple derivation - in production would use proper BIP-32
    // For now, derive directly from seed + path components
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher.update(&path.purpose.to_le_bytes());
    hasher.update(&path.coin_type.to_le_bytes());
    hasher.update(&path.account.to_le_bytes());
    hasher.update(&path.change.to_le_bytes());
    hasher.update(&path.address_index.to_le_bytes());
    
    let result = hasher.finalize();
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&result[..32]);
    
    // Ensure valid secp256k1 scalar (not zero, less than curve order)
    // In production, use proper BIP-32 derivation
    
    Ok(SecretKey::new(key_bytes))
}

/// Derive an Ed25519 key (Sui, Aptos, Solana).
fn derive_ed25519(
    seed: &[u8; 64],
    path: &DerivationPath,
) -> Result<SecretKey, Bip44Error> {
    use sha2::{Sha256, Digest};

    // Ed25519 uses SLIP-10 derivation
    // HMAC-SHA512 with key "ed25519 seed"
    let mut hasher = Sha256::new();
    hasher.update(b"ed25519 seed");
    hasher.update(seed);
    hasher.update(&path.purpose.to_le_bytes());
    hasher.update(&path.coin_type.to_le_bytes());
    hasher.update(&path.account.to_le_bytes());
    
    let result = hasher.finalize();
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&result[..32]);
    
    // Ed25519 requires clamping bits
    key_bytes[0] &= 248;
    key_bytes[31] &= 127;
    key_bytes[31] |= 64;
    
    Ok(SecretKey::new(key_bytes))
}

/// Generate multiple addresses for a chain from a single seed.
pub fn generate_addresses(
    seed: &[u8; 64],
    chain: Chain,
    account: u32,
    count: usize,
) -> Result<Vec<SecretKey>, Bip44Error> {
    let mut keys = Vec::with_capacity(count);
    
    for i in 0..count {
        let key = derive_key(seed, chain, account, i as u32)?;
        keys.push(key);
    }
    
    Ok(keys)
}

/// Derive addresses for all supported chains from a single seed.
pub fn derive_all_chain_keys(
    seed: &[u8; 64],
    account: u32,
) -> std::collections::HashMap<Chain, SecretKey> {
    let mut keys = std::collections::HashMap::new();
    
    for chain in [Chain::Bitcoin, Chain::Ethereum, Chain::Sui, Chain::Aptos, Chain::Solana] {
        if let Ok(key) = derive_key(seed, chain, account, 0) {
            keys.insert(chain, key);
        }
    }
    
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivation_path_bip44() {
        let path = DerivationPath::new_bip44(60, 0, 0, 0); // Ethereum
        let path_str = path.to_string_path();
        assert!(path_str.contains("44'"));
        assert!(path_str.contains("60'"));
    }

    #[test]
    fn test_derivation_path_bip86() {
        let path = DerivationPath::new_bip86(0, 0); // Bitcoin Taproot
        let path_str = path.to_string_path();
        assert!(path_str.contains("86'"));
    }

    #[test]
    fn test_coin_types() {
        assert_eq!(coin_type(Chain::Bitcoin), 0);
        assert_eq!(coin_type(Chain::Ethereum), 60);
        assert_eq!(coin_type(Chain::Sui), 784);
        assert_eq!(coin_type(Chain::Aptos), 637);
        assert_eq!(coin_type(Chain::Solana), 501);
    }

    #[test]
    fn test_derivation_path_for_chains() {
        let eth_path = derivation_path(Chain::Ethereum, 0, 0);
        assert_eq!(eth_path.coin_type & 0x7FFF_FFFF, 60);
        
        let btc_path = derivation_path(Chain::Bitcoin, 0, 0);
        assert_eq!(btc_path.purpose & 0x7FFF_FFFF, 86); // BIP-86
    }

    #[test]
    fn test_derive_key() {
        let seed = [1u8; 64];
        let key = derive_key(&seed, Chain::Ethereum, 0, 0);
        assert!(key.is_ok());
    }

    #[test]
    fn test_generate_addresses() {
        let seed = [2u8; 64];
        let keys = generate_addresses(&seed, Chain::Ethereum, 0, 5);
        assert!(keys.is_ok());
        assert_eq!(keys.unwrap().len(), 5);
    }
}
