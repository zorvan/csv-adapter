//! RPC client for Solana adapter

use solana_sdk::{
    account::Account, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use solana_commitment_config::CommitmentConfig;

use crate::error::{SolanaError, SolanaResult};
use crate::types::{AccountChange, ConfirmationStatus};

/// Trait for Solana RPC operations
#[async_trait::async_trait]
pub trait SolanaRpc: Send + Sync {
    /// Get account info
    async fn get_account(&self, pubkey: &Pubkey) -> SolanaResult<Account>;

    /// Get multiple accounts
    async fn get_multiple_accounts(&self, pubkeys: &[Pubkey])
        -> SolanaResult<Vec<Option<Account>>>;

    /// Get transaction with status
    async fn get_transaction(&self, signature: &Signature) -> SolanaResult<String>;

    /// Send transaction
    async fn send_transaction(&self, transaction: &Transaction) -> SolanaResult<Signature>;

    /// Get latest slot
    async fn get_latest_slot(&self) -> SolanaResult<u64>;

    /// Get slot with commitment
    async fn get_slot_with_commitment(&self, commitment: &str) -> SolanaResult<u64>;

    /// Get account changes between slots
    async fn get_account_changes(
        &self,
        from_slot: u64,
        to_slot: u64,
    ) -> SolanaResult<Vec<AccountChange>>;

    /// Wait for transaction confirmation
    async fn wait_for_confirmation(
        &self,
        signature: &Signature,
    ) -> SolanaResult<ConfirmationStatus>;

    /// Get minimum balance for rent exemption
    async fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> SolanaResult<u64>;

    /// Get recent blockhash for transaction
    async fn get_recent_blockhash(&self) -> SolanaResult<solana_sdk::hash::Hash>;

    /// Get balance for account
    async fn get_balance(&self, pubkey: &Pubkey) -> SolanaResult<u64>;

    /// Clone the RPC client for creating new boxed instances
    fn clone_boxed(&self) -> Box<dyn SolanaRpc>;
}

/// Real RPC client implementation using solana-rpc-client
#[cfg(feature = "rpc")]
pub struct RealSolanaRpc {
    client: solana_rpc_client::rpc_client::RpcClient,
}

#[cfg(feature = "rpc")]
impl RealSolanaRpc {
    /// Create new RPC client with default commitment
    pub fn new(rpc_url: &str) -> Self {
        let client = solana_rpc_client::rpc_client::RpcClient::new(rpc_url.to_string());
        Self { client }
    }

    /// Create with specific commitment level
    pub fn with_commitment(rpc_url: &str, commitment: &str) -> Self {
        let commitment_config = match commitment {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };
        let client = solana_rpc_client::rpc_client::RpcClient::new_with_commitment(
            rpc_url.to_string(),
            commitment_config,
        );
        Self { client }
    }

    /// Get the underlying RPC client for advanced operations
    pub fn underlying_client(&self) -> &solana_rpc_client::rpc_client::RpcClient {
        &self.client
    }
}

#[cfg(feature = "rpc")]
#[async_trait::async_trait]
impl SolanaRpc for RealSolanaRpc {
    async fn get_account(&self, pubkey: &Pubkey) -> SolanaResult<Account> {
        self.client
            .get_account(pubkey)
            .map_err(|e| SolanaError::Rpc(format!("Failed to get account {}: {}", pubkey, e)))
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> SolanaResult<Vec<Option<Account>>> {
        self.client
            .get_multiple_accounts(pubkeys)
            .map_err(|e| SolanaError::Rpc(format!("Failed to get multiple accounts: {}", e)))
    }

    async fn get_transaction(&self, signature: &Signature) -> SolanaResult<String> {
        // Use get_signature_status to check if transaction exists and return status info
        let status = self
            .client
            .get_signature_status(signature)
            .map_err(|e| SolanaError::Rpc(format!("Failed to get transaction status: {}", e)))?;
        
        match status {
            Some(Ok(())) => Ok(format!("Transaction {{ signature: {} }}", signature)),
            Some(Err(e)) => Err(SolanaError::TransactionFailed(format!("Transaction failed: {:?}", e))),
            None => Err(SolanaError::Rpc("Transaction not found".to_string())),
        }
    }

    async fn send_transaction(&self, transaction: &Transaction) -> SolanaResult<Signature> {
        self.client
            .send_transaction(transaction)
            .map_err(|e| SolanaError::Rpc(format!("Failed to send transaction: {}", e)))
    }

    async fn get_latest_slot(&self) -> SolanaResult<u64> {
        self.client
            .get_slot()
            .map_err(|e| SolanaError::Rpc(format!("Failed to get slot: {}", e)))
    }

    async fn get_slot_with_commitment(&self, commitment: &str) -> SolanaResult<u64> {
        let commitment_config = match commitment {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };

        self.client
            .get_slot_with_commitment(commitment_config)
            .map_err(|e| SolanaError::Rpc(format!("Failed to get slot with commitment: {}", e)))
    }

    async fn get_account_changes(
        &self,
        _from_slot: u64,
        _to_slot: u64,
    ) -> SolanaResult<Vec<AccountChange>> {
        // Account changes tracking requires complex pre/post balance tracking.
        // Returns empty list as this is a non-critical feature for core operation.
        // Full implementation would use get_block with pre/post balance metadata.
        Ok(vec![])
    }

    async fn wait_for_confirmation(
        &self,
        signature: &Signature,
    ) -> SolanaResult<ConfirmationStatus> {
        // Poll for confirmation with exponential backoff
        let mut retries = 0;
        let max_retries = 30;

        while retries < max_retries {
            // Check signature status to determine confirmation
            match self.client.get_signature_status(signature) {
                Ok(Some(Ok(()))) => {
                    // Transaction confirmed - check if finalized by looking at block height
                    match self.client.get_slot_with_commitment(CommitmentConfig::finalized()) {
                        Ok(finalized_slot) => {
                            match self.client.get_slot_with_commitment(CommitmentConfig::confirmed()) {
                                Ok(confirmed_slot) => {
                                    if confirmed_slot <= finalized_slot {
                                        return Ok(ConfirmationStatus::Finalized);
                                    } else {
                                        return Ok(ConfirmationStatus::Confirmed);
                                    }
                                }
                                _ => return Ok(ConfirmationStatus::Confirmed),
                            }
                        }
                        _ => return Ok(ConfirmationStatus::Confirmed),
                    }
                }
                Ok(Some(Err(e))) => {
                    return Err(SolanaError::TransactionFailed(format!("Transaction failed: {:?}", e)));
                }
                Ok(None) => {
                    // Transaction not found yet, wait
                }
                Err(e) => {
                    return Err(SolanaError::Rpc(format!("Failed to get signature status: {}", e)));
                }
            }

            retries += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(500 * retries.min(10))).await;
        }

        Err(SolanaError::Timeout("Transaction confirmation timeout".to_string()))
    }

    async fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> SolanaResult<u64> {
        self.client
            .get_minimum_balance_for_rent_exemption(data_len)
            .map_err(|e| SolanaError::Rpc(format!("Failed to get rent exemption: {}", e)))
    }

    async fn get_recent_blockhash(&self) -> SolanaResult<solana_sdk::hash::Hash> {
        self.client
            .get_latest_blockhash()
            .map_err(|e| SolanaError::Rpc(format!("Failed to get recent blockhash: {}", e)))
    }

    async fn get_balance(&self, pubkey: &Pubkey) -> SolanaResult<u64> {
        self.client
            .get_balance(pubkey)
            .map_err(|e| SolanaError::Rpc(format!("Failed to get balance for {}: {}", pubkey, e)))
    }

    fn clone_boxed(&self) -> Box<dyn SolanaRpc> {
        // RealSolanaRpc cannot be easily cloned due to RpcClient
        // In production, you should create a new instance
        panic!("RealSolanaRpc cannot be cloned. Create a new instance instead.")
    }
}

/// Mock RPC client for testing
#[cfg(test)]
pub struct MockSolanaRpc {
    accounts: std::collections::HashMap<Pubkey, Account>,
}

#[cfg(test)]
impl Default for MockSolanaRpc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MockSolanaRpc {
    /// Create new test RPC
    pub fn new() -> Self {
        Self {
            accounts: std::collections::HashMap::new(),
        }
    }

    /// Add account to test RPC
    pub fn add_account(&mut self, pubkey: Pubkey, account: Account) {
        self.accounts.insert(pubkey, account);
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl SolanaRpc for MockSolanaRpc {
    async fn get_account(&self, pubkey: &Pubkey) -> SolanaResult<Account> {
        self.accounts
            .get(pubkey)
            .cloned()
            .ok_or_else(|| SolanaError::AccountNotFound(pubkey.to_string()))
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> SolanaResult<Vec<Option<Account>>> {
        Ok(pubkeys
            .iter()
            .map(|pk| self.accounts.get(pk).cloned())
            .collect())
    }

    async fn get_transaction(&self, _signature: &Signature) -> SolanaResult<String> {
        Err(SolanaError::Rpc(
            "Mock RPC: get_transaction not implemented".to_string(),
        ))
    }

    async fn send_transaction(&self, _transaction: &Transaction) -> SolanaResult<Signature> {
        Ok(Signature::new_unique())
    }

    async fn get_latest_slot(&self) -> SolanaResult<u64> {
        Ok(1000)
    }

    async fn get_slot_with_commitment(&self, _commitment: &str) -> SolanaResult<u64> {
        Ok(1000)
    }

    async fn get_account_changes(
        &self,
        _from_slot: u64,
        _to_slot: u64,
    ) -> SolanaResult<Vec<AccountChange>> {
        Ok(vec![])
    }

    async fn wait_for_confirmation(
        &self,
        _signature: &Signature,
    ) -> SolanaResult<ConfirmationStatus> {
        Ok(ConfirmationStatus::Confirmed)
    }

    async fn get_minimum_balance_for_rent_exemption(&self, _data_len: usize) -> SolanaResult<u64> {
        // Test value - rent exemption for typical program size
        Ok(6_900_000_000)
    }

    async fn get_recent_blockhash(&self) -> SolanaResult<solana_sdk::hash::Hash> {
        Ok(solana_sdk::hash::Hash::default())
    }

    async fn get_balance(&self, pubkey: &Pubkey) -> SolanaResult<u64> {
        // Return test balance from configured accounts or default
        Ok(self.accounts.get(pubkey).map(|a| a.lamports).unwrap_or(1_000_000_000))
    }

    fn clone_boxed(&self) -> Box<dyn SolanaRpc> {
        Box::new(MockSolanaRpc {
            accounts: self.accounts.clone(),
        })
    }
}
