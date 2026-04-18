//! Solana adapter implementation for CSV

use csv_adapter_core::traits::AnchorLayer;

use crate::config::SolanaConfig;
use crate::error::SolanaError;
use crate::rpc::SolanaRpc;
use crate::types::SolanaAnchorRef;
use crate::wallet::ProgramWallet;

/// Solana adapter for CSV (Client-Side Validation)
pub struct SolanaAnchorLayer {
    /// Configuration
    pub config: SolanaConfig,
    /// RPC client
    pub rpc_client: Option<Box<dyn SolanaRpc>>,
    /// Wallet
    pub wallet: Option<ProgramWallet>,
}

impl SolanaAnchorLayer {
    /// Create new Solana adapter
    pub fn new(config: SolanaConfig) -> Self {
        Self {
            config,
            rpc_client: None,
            wallet: None,
        }
    }

    /// Set RPC client
    pub fn with_rpc_client(mut self, rpc_client: Box<dyn SolanaRpc>) -> Self {
        self.rpc_client = Some(rpc_client);
        self
    }

    /// Set wallet
    pub fn with_wallet(mut self, wallet: ProgramWallet) -> Self {
        self.wallet = Some(wallet);
        self
    }

    /// Get configuration
    pub fn config(&self) -> &SolanaConfig {
        &self.config
    }

    /// Get RPC client
    pub fn rpc_client(&self) -> Option<&dyn SolanaRpc> {
        self.rpc_client.as_ref().map(|client| client.as_ref())
    }

    /// Get wallet
    pub fn wallet(&self) -> Option<&ProgramWallet> {
        self.wallet.as_ref()
    }
}

impl AnchorLayer for SolanaAnchorLayer {
    type SealRef = SolanaAnchorRef;
    type AnchorRef = SolanaAnchorRef;
    type InclusionProof = String;
    type FinalityProof = String;

    fn publish(&self, _hash: csv_adapter_core::Hash, _seal_ref: Self::SealRef) -> csv_adapter_core::Result<Self::AnchorRef> {
        Err(SolanaError::Rpc("Publish not implemented".to_string()).into())
    }

    fn verify_inclusion(&self, _anchor_ref: Self::AnchorRef) -> csv_adapter_core::Result<Self::InclusionProof> {
        Err(SolanaError::Rpc("Verify inclusion not implemented".to_string()).into())
    }

    fn verify_finality(&self, _anchor_ref: Self::AnchorRef) -> csv_adapter_core::Result<Self::FinalityProof> {
        Err(SolanaError::Rpc("Verify finality not implemented".to_string()).into())
    }

    fn enforce_seal(&self, _seal_ref: Self::SealRef) -> csv_adapter_core::Result<()> {
        Err(SolanaError::Rpc("Enforce seal not implemented".to_string()).into())
    }

    fn create_seal(&self, _amount: Option<u64>) -> csv_adapter_core::Result<Self::SealRef> {
        Err(SolanaError::Rpc("Create seal not implemented".to_string()).into())
    }

    fn hash_commitment(&self, _preimage: csv_adapter_core::Hash, _seal: csv_adapter_core::Hash, _anchor: csv_adapter_core::Hash, _seal_ref: &Self::SealRef) -> csv_adapter_core::Hash {
        csv_adapter_core::Hash::default()
    }

    fn build_proof_bundle(&self, _anchor_ref: Self::AnchorRef, _segment: csv_adapter_core::dag::DAGSegment) -> csv_adapter_core::Result<csv_adapter_core::proof::ProofBundle> {
        Err(SolanaError::Rpc("Build proof bundle not implemented".to_string()).into())
    }

    fn rollback(&self, _anchor_ref: Self::AnchorRef) -> csv_adapter_core::Result<()> {
        Err(SolanaError::Rpc("Rollback not implemented".to_string()).into())
    }

    fn domain_separator(&self) -> [u8; 32] {
        [0u8; 32] // Default domain separator
    }

    fn signature_scheme(&self) -> csv_adapter_core::signature::SignatureScheme {
        csv_adapter_core::signature::SignatureScheme::Ed25519
    }
}

impl Default for SolanaAnchorLayer {
    fn default() -> Self {
        Self::new(SolanaConfig::default())
    }
}
