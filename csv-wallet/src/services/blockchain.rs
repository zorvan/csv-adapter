//! Blockchain service module.

use csv_core::Chain;

/// Blockchain configuration.
#[derive(Debug, Clone, Default)]
pub struct BlockchainConfig {}

/// Blockchain service stub.
pub struct BlockchainService {
    config: BlockchainConfig,
}

impl BlockchainService {
    pub fn new(config: BlockchainConfig) -> Self {
        Self { config }
    }

    pub async fn transfer_sanad_local(&self, _chain: Chain, _sanad_id: &str, _to: &str) -> Result<TransferResult, BlockchainError> {
        Ok(TransferResult {
            transfer_id: String::new(),
            source_fee: String::new(),
            dest_fee: String::new(),
            lock_tx_hash: String::new(),
            mint_tx_hash: String::new(),
        })
    }

    pub async fn execute_cross_chain_transfer(&self, _from: Chain, _to: Chain, _sanad_id: &str, _dest_addr: &str, _contracts: &std::collections::HashMap<Chain, ContractDeployment>, _signer: &NativeWallet) -> Result<TransferResult, BlockchainError> {
        Ok(TransferResult {
            transfer_id: String::new(),
            source_fee: String::new(),
            dest_fee: String::new(),
            lock_tx_hash: String::new(),
            mint_tx_hash: String::new(),
        })
    }
}

/// Native wallet stub.
#[derive(Debug, Clone)]
pub struct NativeWallet {
    pub address: String,
}

impl NativeWallet {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}

/// Browser wallet stub.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserWallet {
    pub address: String,
    pub chain: Option<Chain>,
    pub wallet_type: WalletType,
}

/// Wallet type enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletType {
    MetaMask,
    Phantom,
    SuiWallet,
    Petra,
    Leather,
    Native,
    Custom(String),
}

/// Wallet connection module.
pub mod wallet_connection {
    use super::*;

    pub fn is_metamask_installed() -> bool { false }
    pub fn is_phantom_installed() -> bool { false }
    pub async fn connect_metamask() -> Result<BrowserWallet, BlockchainError> {
        Err(BlockchainError { message: "MetaMask not available".to_string(), chain: None, code: None })
    }
    pub fn native_wallet(_account: &str) -> NativeWallet {
        NativeWallet { address: String::new() }
    }
    pub fn recommended_wallet(_chain: Chain) -> WalletType {
        WalletType::Native
    }
}

/// Blockchain error type.
#[derive(Debug)]
pub struct BlockchainError {
    pub message: String,
    pub chain: Option<Chain>,
    pub code: Option<i32>,
}

impl std::fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BlockchainError {}

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
    pub chain: Option<Chain>,
    pub contract_address: String,
    pub contract_type: ContractType,
    pub deployed_at: u64,
}

/// Contract deployment result.
#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub address: String,
    pub tx_hash: String,
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

pub mod types {
    pub use super::{ContractDeployment, ContractType};
}
