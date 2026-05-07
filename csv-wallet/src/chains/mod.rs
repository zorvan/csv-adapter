//! Chain-specific integrations.

use csv_core::Chain;

pub mod aptos;
pub mod bitcoin;
pub mod ethereum;
pub mod solana;
pub mod sui;

/// Chains currently supported by the wallet UI.
pub fn supported_wallet_chains() -> Vec<Chain> {
    vec![
        Chain::Bitcoin,
        Chain::Ethereum,
        Chain::Sui,
        Chain::Aptos,
        Chain::Solana,
    ]
}
