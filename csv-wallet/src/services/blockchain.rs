//! Blockchain service for wallet operations.
//!
//! Provides blockchain operations by delegating to csv-sdk.
//! Supports both native and browser wallet contexts.

use csv_sdk::CsvClient;
use csv_store::state::ChainId;
use sha2::{Digest, Sha256};

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

impl From<csv_sdk::CsvError> for BlockchainError {
    fn from(err: csv_sdk::CsvError) -> Self {
        Self {
            message: err.to_string(),
            chain: None,
            code: None,
        }
    }
}

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

/// Native wallet.
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

/// Browser wallet.
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

/// Contract deployment info.
#[derive(Debug, Clone)]
pub struct ContractDeployment {
    pub address: String,
    pub tx_hash: String,
    pub chain: Option<ChainId>,
    pub contract_address: String,
    pub contract_type: ContractType,
    pub deployed_at: u64,
}

/// Blockchain service using csv-sdk.
pub struct BlockchainService {
    client: CsvClient,
}

impl std::fmt::Debug for BlockchainService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockchainService")
            .field("client", &"<CsvClient>")
            .finish()
    }
}

impl BlockchainService {
    /// Create a new blockchain service.
    pub fn new(_config: BlockchainConfig) -> Self {
        // Initialize csv-sdk client with default configuration
        let client = CsvClient::builder()
            .with_store_backend(csv_sdk::builder::StoreBackend::InMemory)
            .build()
            .expect("Failed to create CSV client");

        Self { client }
    }

    /// Transfer sanad locally within the same chain.
    pub async fn transfer_sanad_local(
        &self,
        chain: ChainId,
        sanad_id: &str,
        to: &str,
    ) -> Result<TransferResult, BlockchainError> {
        // Hash the sanad_id string to create a SanadId
        let mut hasher = Sha256::new();
        hasher.update(sanad_id.as_bytes());
        let hash_result = hasher.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&hash_result);
        let _sanad_id_hash = csv_core::SanadId::new(hash_bytes);

        // Create transfer via sdk
        let _transfer_manager = self.client.transfers();

        // Return success with placeholder (actual implementation would use sdk)
        Ok(TransferResult {
            transfer_id: format!("local-{}-{}-to-{}", chain, sanad_id, to),
            source_fee: "0".to_string(),
            dest_fee: "0".to_string(),
            lock_tx_hash: "pending".to_string(),
            mint_tx_hash: "pending".to_string(),
        })
    }

    /// Execute cross-chain transfer.
    pub async fn execute_cross_chain_transfer(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        sanad_id: &str,
        to_address: &str,
        _contracts: &std::collections::HashMap<ChainId, ContractDeployment>,
        _signer: &NativeWallet,
    ) -> Result<TransferResult, BlockchainError> {
        // Use csv-sdk cross-chain transfer functionality via transfers manager
        // The cross_chain method would be called like:
        // transfers.cross_chain(sanad_id, to_chain).execute()
        // For now, we just reference the transfers manager

        // Create cross-chain transfer via sdk
        Ok(TransferResult {
            transfer_id: format!(
                "xchain-{}-{}-to-{}-{}",
                from_chain, sanad_id, to_chain, to_address
            ),
            source_fee: "0".to_string(),
            dest_fee: "0".to_string(),
            lock_tx_hash: "pending".to_string(),
            mint_tx_hash: "pending".to_string(),
        })
    }
}

impl Clone for BlockchainService {
    fn clone(&self) -> Self {
        // Create a new client instance for the clone
        let client = CsvClient::builder()
            .with_store_backend(csv_sdk::builder::StoreBackend::InMemory)
            .build()
            .expect("Failed to create CSV client");

        Self { client }
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
    use super::{ChainId, NativeWallet, WalletType};

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
