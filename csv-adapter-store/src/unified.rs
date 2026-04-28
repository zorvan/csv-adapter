//! Unified storage types for CSV Wallet and CLI
//!
//! This module provides a common storage format that can be used by both
//! csv-wallet (browser) and csv-cli (desktop) applications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Dev,
    #[default]
    Test,
    Main,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Dev => write!(f, "dev"),
            Network::Test => write!(f, "test"),
            Network::Main => write!(f, "main"),
        }
    }
}

/// Supported chains
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Sui,
    Aptos,
    Solana,
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Bitcoin => write!(f, "bitcoin"),
            Chain::Ethereum => write!(f, "ethereum"),
            Chain::Sui => write!(f, "sui"),
            Chain::Aptos => write!(f, "aptos"),
            Chain::Solana => write!(f, "solana"),
        }
    }
}

impl std::str::FromStr for Chain {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bitcoin" => Ok(Chain::Bitcoin),
            "ethereum" => Ok(Chain::Ethereum),
            "sui" => Ok(Chain::Sui),
            "aptos" => Ok(Chain::Aptos),
            "solana" => Ok(Chain::Solana),
            _ => Err(format!("Unknown chain: {}", s)),
        }
    }
}

impl std::str::FromStr for Network {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dev" => Ok(Network::Dev),
            "test" => Ok(Network::Test),
            "main" => Ok(Network::Main),
            _ => Err(format!("Unknown network: {}", s)),
        }
    }
}

/// Chain-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Network environment
    pub network: Network,
    /// Contract/package address (if deployed)
    pub contract_address: Option<String>,
    /// Chain ID (for EVM chains) or magic bytes (Bitcoin)
    pub chain_id: Option<u64>,
    /// Finality depth (confirmations required)
    pub finality_depth: u64,
    /// Default gas price / fee rate
    pub default_fee: Option<u64>,
}

/// Wallet account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    /// Account ID (UUID or derived from public key)
    pub id: String,
    /// Chain this account belongs to
    pub chain: Chain,
    /// Human-readable name
    pub name: String,
    /// Public address
    pub address: String,
    /// Private key (hex encoded, encrypted at rest) - DEPRECATED: Use keystore_ref instead
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    /// Keystore reference (UUID pointing to encrypted key in keystore)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keystore_ref: Option<String>,
    /// Extended public key for HD wallets
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xpub: Option<String>,
    /// Derivation path (BIP-44/86)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivation_path: Option<String>,
}

impl WalletAccount {
    /// Check if this account uses the new keystore-based storage.
    pub fn uses_keystore(&self) -> bool {
        self.keystore_ref.is_some()
    }

    /// Check if this account still has plaintext private key (needs migration).
    pub fn needs_migration(&self) -> bool {
        self.private_key.is_some() && self.keystore_ref.is_none()
    }

    /// Create a new keystore-based account.
    pub fn with_keystore(id: String, chain: Chain, name: String, address: String, keystore_ref: String) -> Self {
        Self {
            id,
            chain,
            name,
            address,
            private_key: None,
            keystore_ref: Some(keystore_ref),
            xpub: None,
            derivation_path: None,
        }
    }
}

/// Wallet configuration - can use mnemonic or individual private keys
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletConfig {
    /// Master mnemonic phrase (encrypted at rest, optional if using individual keys)
    pub mnemonic: Option<String>,
    /// Mnemonic passphrase (optional, encrypted at rest)
    pub mnemonic_passphrase: Option<String>,
    /// Individual accounts (one per chain or multiple)
    pub accounts: Vec<WalletAccount>,
}

/// Faucet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaucetConfig {
    /// Faucet endpoint URL
    pub url: String,
    /// Amount to request (chain-specific units)
    pub amount: Option<u64>,
}

/// Right status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RightStatus {
    Active,
    Transferred,
    Consumed,
}

impl std::fmt::Display for RightStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RightStatus::Active => write!(f, "active"),
            RightStatus::Transferred => write!(f, "transferred"),
            RightStatus::Consumed => write!(f, "consumed"),
        }
    }
}

/// A tracked Right
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightRecord {
    /// Right ID (hash)
    pub id: String,
    /// Chain where this Right is anchored
    pub chain: Chain,
    /// Seal reference (chain-specific bytes, base64 encoded for JSON)
    pub seal_ref: String,
    /// Current owner address
    pub owner: String,
    /// Value/amount
    pub value: u64,
    /// Commitment hash (base64)
    pub commitment: String,
    /// Nullifier (if consumed, base64)
    pub nullifier: Option<String>,
    /// Current status
    pub status: RightStatus,
    /// Creation timestamp
    pub created_at: u64,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    Initiated,
    Locked,
    Verifying,
    Minting,
    Completed,
    Failed,
}

impl std::fmt::Display for TransferStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferStatus::Initiated => write!(f, "initiated"),
            TransferStatus::Locked => write!(f, "locked"),
            TransferStatus::Verifying => write!(f, "verifying"),
            TransferStatus::Minting => write!(f, "minting"),
            TransferStatus::Completed => write!(f, "completed"),
            TransferStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A cross-chain transfer record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    /// Transfer ID (hash of source seal + dest chain)
    pub id: String,
    /// Source chain
    pub source_chain: Chain,
    /// Destination chain
    pub dest_chain: Chain,
    /// Right ID being transferred
    pub right_id: String,
    /// Sender address on source chain
    pub sender_address: Option<String>,
    /// Destination owner address
    pub destination_address: Option<String>,
    /// Source transaction hash
    pub source_tx_hash: Option<String>,
    /// Source transaction fee
    pub source_fee: Option<u64>,
    /// Destination transaction hash
    pub dest_tx_hash: Option<String>,
    /// Destination transaction fee
    pub dest_fee: Option<u64>,
    /// Destination contract address
    pub destination_contract: Option<String>,
    /// Inclusion proof (base64 encoded)
    pub proof: Option<String>,
    /// Transfer status
    pub status: TransferStatus,
    /// Created timestamp
    pub created_at: u64,
    /// Completed timestamp
    pub completed_at: Option<u64>,
}

/// Deployed contract info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRecord {
    pub chain: Chain,
    pub address: String,
    pub tx_hash: String,
    pub deployed_at: u64,
}

/// Seal record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealRecord {
    /// Seal reference (base64 encoded)
    pub seal_ref: String,
    /// Chain
    pub chain: Chain,
    /// Value
    pub value: u64,
    /// Whether consumed
    pub consumed: bool,
    /// Creation timestamp
    pub created_at: u64,
}

/// Proof record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRecord {
    /// Chain
    pub chain: Chain,
    /// Right ID
    pub right_id: String,
    /// Proof type
    pub proof_type: String,
    /// Whether verified
    pub verified: bool,
    /// Proof data (base64 encoded)
    pub proof_data: Option<String>,
}

/// Transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Transfer,
    ContractDeployment,
    ContractCall,
    RightCreation,
    RightTransfer,
    SealCreation,
    SealConsumption,
    CrossChainLock,
    CrossChainMint,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

/// A transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub id: String,
    pub chain: Chain,
    pub tx_hash: String,
    pub tx_type: TransactionType,
    pub status: TransactionStatus,
    pub from_address: String,
    pub to_address: Option<String>,
    pub amount: Option<u64>,
    pub fee: Option<u64>,
    pub block_number: Option<u64>,
    pub confirmations: Option<u64>,
    pub created_at: u64,
    pub explorer_url: Option<String>,
}

/// Gas payment account per chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasAccount {
    pub chain: Chain,
    pub address: String,
}

/// The unified storage format - used by both CLI and Wallet
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnifiedStorage {
    /// Version for migration compatibility
    pub version: u32,
    
    /// Chain configurations (CLI + Wallet)
    #[serde(default)]
    pub chains: HashMap<Chain, ChainConfig>,
    
    /// Wallet configuration with accounts
    #[serde(default)]
    pub wallet: WalletConfig,
    
    /// Faucet configurations (mainly CLI)
    #[serde(default)]
    pub faucets: HashMap<Chain, FaucetConfig>,
    
    /// Tracked rights (both)
    #[serde(default)]
    pub rights: Vec<RightRecord>,
    
    /// Tracked transfers (both)
    #[serde(default)]
    pub transfers: Vec<TransferRecord>,
    
    /// Deployed contracts (both)
    #[serde(default)]
    pub contracts: Vec<ContractRecord>,
    
    /// Seal records (both)
    #[serde(default)]
    pub seals: Vec<SealRecord>,
    
    /// Proof records (both)
    #[serde(default)]
    pub proofs: Vec<ProofRecord>,
    
    /// Transaction history (mainly Wallet)
    #[serde(default)]
    pub transactions: Vec<TransactionRecord>,
    
    /// Gas accounts per chain (CLI)
    #[serde(default)]
    pub gas_accounts: Vec<GasAccount>,
    
    /// Selected chain (UI state - mainly Wallet)
    #[serde(default)]
    pub selected_chain: Option<Chain>,
    
    /// Selected network (UI state - mainly Wallet)
    #[serde(default)]
    pub selected_network: Option<Network>,
    
    /// Whether wallet has been initialized
    #[serde(default)]
    pub initialized: bool,
    
    /// Data directory path (CLI only, for file-based storage)
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

fn default_data_dir() -> String {
    "~/.csv/data".to_string()
}

impl UnifiedStorage {
    /// Create new empty storage with current version
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Default::default()
        }
    }
    
    /// Get default chain configurations
    pub fn default_chains() -> HashMap<Chain, ChainConfig> {
        let mut chains = HashMap::new();
        
        chains.insert(
            Chain::Bitcoin,
            ChainConfig {
                rpc_url: "https://mempool.space/signet/api/".to_string(),
                network: Network::Test,
                contract_address: None,
                chain_id: None,
                finality_depth: 6,
                default_fee: Some(10),
            },
        );
        
        chains.insert(
            Chain::Ethereum,
            ChainConfig {
                rpc_url: "https://ethereum-sepolia-rpc.publicnode.com".to_string(),
                network: Network::Test,
                contract_address: None,
                chain_id: Some(11155111),
                finality_depth: 15,
                default_fee: Some(20_000_000_000),
            },
        );
        
        chains.insert(
            Chain::Sui,
            ChainConfig {
                rpc_url: "https://fullnode.testnet.sui.io:443".to_string(),
                network: Network::Test,
                contract_address: None,
                chain_id: None,
                finality_depth: 1,
                default_fee: Some(1000),
            },
        );
        
        chains.insert(
            Chain::Aptos,
            ChainConfig {
                rpc_url: "https://fullnode.testnet.aptoslabs.com/v1".to_string(),
                network: Network::Test,
                contract_address: None,
                chain_id: None,
                finality_depth: 1,
                default_fee: Some(100),
            },
        );
        
        chains.insert(
            Chain::Solana,
            ChainConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                network: Network::Test,
                contract_address: None,
                chain_id: None,
                finality_depth: 32,
                default_fee: Some(5000),
            },
        );
        
        chains
    }
    
    /// Get default faucet configurations
    pub fn default_faucets() -> HashMap<Chain, FaucetConfig> {
        let mut faucets = HashMap::new();
        
        faucets.insert(
            Chain::Bitcoin,
            FaucetConfig {
                url: "https://signet.bc-2.jp".to_string(),
                amount: Some(100_000),
            },
        );
        
        faucets.insert(
            Chain::Sui,
            FaucetConfig {
                url: "https://faucet.testnet.sui.io/v1/gas".to_string(),
                amount: Some(10_000_000_000),
            },
        );
        
        faucets.insert(
            Chain::Aptos,
            FaucetConfig {
                url: "https://faucet.testnet.aptoslabs.com".to_string(),
                amount: Some(100_000_000),
            },
        );
        
        faucets.insert(
            Chain::Ethereum,
            FaucetConfig {
                url: "https://sepoliafaucet.com".to_string(),
                amount: Some(100_000_000_000_000_000),
            },
        );
        
        faucets.insert(
            Chain::Solana,
            FaucetConfig {
                url: "https://faucet.devnet.solana.com".to_string(),
                amount: Some(1_000_000_000),
            },
        );
        
        faucets
    }
    
    /// Initialize with defaults
    pub fn with_defaults(mut self) -> Self {
        self.chains = Self::default_chains();
        self.faucets = Self::default_faucets();
        self.selected_chain = Some(Chain::Bitcoin);
        self.selected_network = Some(Network::Test);
        self
    }
    
    /// Find a right by ID
    pub fn get_right(&self, id: &str) -> Option<&RightRecord> {
        self.rights.iter().find(|r| r.id == id)
    }
    
    /// Find a transfer by ID
    pub fn get_transfer(&self, id: &str) -> Option<&TransferRecord> {
        self.transfers.iter().find(|t| t.id == id)
    }
    
    /// Get contracts for a chain
    pub fn get_contracts(&self, chain: &Chain) -> Vec<&ContractRecord> {
        self.contracts.iter().filter(|c| &c.chain == chain).collect()
    }
    
    /// Get account for a chain
    pub fn get_account(&self, chain: &Chain) -> Option<&WalletAccount> {
        self.wallet.accounts.iter().find(|a| &a.chain == chain)
    }
    
    /// Add or update account
    pub fn set_account(&mut self, account: WalletAccount) {
        if let Some(existing) = self.wallet.accounts.iter_mut().find(|a| a.id == account.id) {
            *existing = account;
        } else {
            self.wallet.accounts.push(account);
        }
    }
    
    /// Get gas account for a chain
    pub fn get_gas_account(&self, chain: &Chain) -> Option<&str> {
        self.gas_accounts
            .iter()
            .find(|g| &g.chain == chain)
            .map(|g| g.address.as_str())
            .or_else(|| self.get_account(chain).map(|a| a.address.as_str()))
    }
}

/// Storage error types
#[derive(Debug, thiserror::Error)]
pub enum UnifiedStorageError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializeError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
}

/// Trait for storage backends (file-based, localStorage, SQLite, etc.)
pub trait StorageBackend {
    /// Load unified storage
    fn load(&self) -> Result<UnifiedStorage, UnifiedStorageError>;
    
    /// Save unified storage
    fn save(&self, storage: &UnifiedStorage) -> Result<(), UnifiedStorageError>;
    
    /// Check if storage exists
    fn exists(&self) -> bool;
}

/// File-based storage backend (for CLI)
#[cfg(all(not(target_arch = "wasm32"), feature = "file-storage"))]
pub struct FileStorage {
    pub path: String,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file-storage"))]
impl FileStorage {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }
    
    pub fn default_path() -> String {
        if let Some(home) = dirs::home_dir() {
            home.join(".csv/unified_storage.json").to_string_lossy().to_string()
        } else {
            std::env::temp_dir().join("csv-unified-storage.json").to_string_lossy().to_string()
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "file-storage"))]
impl StorageBackend for FileStorage {
    fn load(&self) -> Result<UnifiedStorage, UnifiedStorageError> {
        let path = std::path::Path::new(&self.path);
        if !path.exists() {
            return Ok(UnifiedStorage::new().with_defaults());
        }
        
        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| UnifiedStorageError::IoError(e.to_string()))?;
        
        let storage: UnifiedStorage = serde_json::from_str(&content)
            .map_err(|e| UnifiedStorageError::SerializeError(e.to_string()))?;
        
        Ok(storage)
    }
    
    fn save(&self, storage: &UnifiedStorage) -> Result<(), UnifiedStorageError> {
        let path = std::path::Path::new(&self.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| UnifiedStorageError::IoError(e.to_string()))?;
        }
        
        let content = serde_json::to_string_pretty(storage)
            .map_err(|e| UnifiedStorageError::SerializeError(e.to_string()))?;
        
        std::fs::write(&self.path, content)
            .map_err(|e| UnifiedStorageError::IoError(e.to_string()))?;
        
        Ok(())
    }
    
    fn exists(&self) -> bool {
        std::path::Path::new(&self.path).exists()
    }
}
