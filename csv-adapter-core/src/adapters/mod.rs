//! Chain adapter implementations.

pub use super::chain_adapter::{
    ChainAdapter, ChainAdapterExt, ChainError, ChainResult, RpcClient, Wallet,
};
pub use super::chain_config::{AccountModel, ChainCapabilities, ChainConfig};

// Mock adapters for testing
#[cfg(test)]
pub mod mock;

// New scalable adapters
pub mod aptos_adapter;
pub mod bitcoin_adapter;
pub mod ethereum_adapter;
pub mod solana_adapter;
pub mod sui_adapter;

#[cfg(test)]
pub use mock::MockAdapter;

// Re-export new scalable adapters
pub use aptos_adapter::AptosAdapter as ScalableAptosAdapter;
pub use bitcoin_adapter::BitcoinAdapter as ScalableBitcoinAdapter;
pub use ethereum_adapter::EthereumAdapter as ScalableEthereumAdapter;
pub use solana_adapter::SolanaAdapter as ScalableSolanaAdapter;
pub use sui_adapter::SuiAdapter as ScalableSuiAdapter;
