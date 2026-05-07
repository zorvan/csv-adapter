//! Blockchain service stubs for wallet compatibility.
//!
//! These stubs provide the types needed by wallet code.
//! The actual chain operations delegate to csv-sdk when available.

use csv_store::state::ChainId;

/// Blockchain error type.
#[derive(Debug, Clone)]
pub struct BlockchainError {
    pub message: String,
    pub chain: Option<ChainId>,
    pub code: Option<u32>,
}

impl std::fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockchainError: {}", self.message)
    }
}

impl std::error::Error for BlockchainError {}

/// Wallet type enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletType {
    MetaMask,
    Phantom,
    Petra,
    Leather,
    Native,
    Custom,
    SuiWallet,
    AptosWallet,
    SolanaWallet,
}

/// Native wallet stub.
#[derive(Debug, Clone)]
pub struct NativeWallet {
    pub address: String,
}

impl NativeWallet {
    /// Create a new native wallet.
    pub fn new(address: String) -> Self {
        Self { address }
    }

    /// Get the wallet address.
    pub fn address(&self) -> &str {
        &self.address
    }
}

/// Browser wallet stub.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserWallet {
    pub address: String,
    pub chain: Option<ChainId>,
    pub wallet_type: WalletType,
}

/// Contract type enum.
#[derive(Debug, Clone, Copy)]
pub enum ContractType {
    Registry,
    Bridge,
    Lock,
}

/// Contract deployment stub.
#[derive(Debug, Clone)]
pub struct ContractDeployment {
    pub address: String,
    pub tx_hash: String,
    pub chain: Option<ChainId>,
    pub contract_address: String,
    pub contract_type: ContractType,
    pub deployed_at: u64,
}

/// Blockchain service stub.
#[derive(Debug, Clone, Default)]
pub struct BlockchainService {
    _private: (),
}

impl BlockchainService {
    /// Create a new blockchain service.
    pub fn new(_config: BlockchainConfig) -> Self {
        Self { _private: () }
    }

    /// Transfer sanad locally (stub).
    pub async fn transfer_sanad_local(
        &self,
        _chain: ChainId,
        _sanad_id: &str,
        _to: &str,
    ) -> Result<TransferResult, BlockchainError> {
        Err(BlockchainError {
            message: "transfer_sanad_local not implemented".to_string(),
            chain: Some(_chain),
            code: Some(501),
        })
    }

   /// Execute cross-chain transfer (stub).
    pub async fn execute_cross_chain_transfer(
        &self,
        _from_chain: ChainId,
        _to_chain: ChainId,
        _sanad_id: &str,
        _to_address: &str,
        _contracts: &std::collections::HashMap<ChainId, ContractDeployment>,
        _signer: &NativeWallet,
    ) -> Result<TransferResult, BlockchainError> {
        Err(BlockchainError {
            message: "execute_cross_chain_transfer not implemented".to_string(),
            chain: Some(_from_chain),
            code: Some(501),
        })
    }
}

/// Blockchain configuration stub.
#[derive(Debug, Clone, Default)]
pub struct BlockchainConfig {
    _private: (),
}

/// Transfer result stub.
#[derive(Debug, Clone)]
pub struct TransferResult {
    pub transfer_id: String,
    pub source_fee: String,
    pub dest_fee: String,
    pub lock_tx_hash: String,
    pub mint_tx_hash: String,
}

/// Wallet connection utilities stub.
pub mod wallet_connection {
    use super::{WalletType, ChainId, NativeWallet};

    /// Get recommended wallet type for a chain.
    pub fn recommended_wallet(_chain: ChainId) -> WalletType {
        WalletType::MetaMask
    }

    /// Check if MetaMask is installed.
    pub fn is_metamask_installed() -> bool {
        false
    }

    /// Check if Phantom is installed.
    pub fn is_phantom_installed() -> bool {
        false
    }

    /// Connect to MetaMask (stub).
    pub async fn connect_metamask() -> Result<NativeWallet, String> {
        Err("MetaMask not available".to_string())
    }

    /// Create a native wallet from address.
    pub fn native_wallet(address: &str) -> NativeWallet {
        NativeWallet::new(address.to_string())
    }

    /// Check if wallet is installed.
    pub fn is_wallet_installed(_wallet_type: &WalletType) -> bool {
        false
    }
}
