//! Cross-chain transfer commands

use anyhow::Result;
use clap::Subcommand;
use std::time::{SystemTime, UNIX_EPOCH};

use csv_adapter_core::cross_chain::{
    ChainId, CrossChainFinalityProof, CrossChainSealRegistry, CrossChainTransferProof,
    LockProvider, MintProvider, TransferVerifier,
};
use csv_adapter_core::hash::Hash;
use csv_adapter_core::right::OwnershipProof;

use crate::config::{Chain, Config};
use crate::output;
use crate::state::{State, TrackedTransfer, TransferStatus};

use super::cross_chain_impl::*;

#[derive(Subcommand)]
pub enum CrossChainAction {
    /// Execute a cross-chain Right transfer
    Transfer {
        /// Source chain
        #[arg(long)]
        from: Chain,
        /// Destination chain
        #[arg(long)]
        to: Chain,
        /// Right ID to transfer (hex)
        #[arg(long)]
        right_id: String,
        /// Destination owner address (hex)
        #[arg(long)]
        dest_owner: Option<String>,
    },
    /// Check transfer status
    Status {
        /// Transfer ID (hex)
        transfer_id: String,
    },
    /// List all transfers
    List {
        /// Filter by source chain
        #[arg(long, value_enum)]
        from: Option<Chain>,
        /// Filter by destination chain
        #[arg(long, value_enum)]
        to: Option<Chain>,
    },
    /// Retry a failed transfer
    Retry {
        /// Transfer ID (hex)
        transfer_id: String,
    },
}

pub fn execute(action: CrossChainAction, config: &Config, state: &mut State) -> Result<()> {
    match action {
        CrossChainAction::Transfer {
            from,
            to,
            right_id,
            dest_owner,
        } => cmd_transfer(from, to, right_id, dest_owner, config, state),
        CrossChainAction::Status { transfer_id } => cmd_status(transfer_id, state),
        CrossChainAction::List { from, to } => cmd_list(from, to, state),
        CrossChainAction::Retry { transfer_id } => cmd_retry(transfer_id, config, state),
    }
}

fn cmd_transfer(
    from: Chain,
    to: Chain,
    right_id: String,
    dest_owner: Option<String>,
    config: &Config,
    state: &mut State,
) -> Result<()> {
    if from == to {
        return Err(anyhow::anyhow!(
            "Source and destination chains must be different"
        ));
    }

    let from_str: String = from.to_string();
    let to_str: String = to.to_string();

    output::header(&format!("Cross-Chain Transfer: {} → {}", from_str, to_str));

    // Parse right ID
    let bytes = hex::decode(right_id.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Invalid Right ID: {}", e))?;
    if bytes.len() < 32 {
        return Err(anyhow::anyhow!(
            "Invalid Right ID: expected at least 32 bytes, got {} bytes",
            bytes.len()
        ));
    }
    let mut right_bytes = [0u8; 32];
    right_bytes.copy_from_slice(&bytes[..32]);
    let right_id_hash = Hash::new(right_bytes);

    // Create ownership proof for destination
    let dest_owner_bytes = match dest_owner {
        Some(addr) => hex::decode(addr.trim_start_matches("0x")).unwrap_or_else(|_| vec![0xFF; 32]),
        None => vec![0xFF; 32],
    };

    let dest_owner_proof = OwnershipProof {
        proof: vec![0x01],
        owner: dest_owner_bytes.clone(),
        scheme: None,
    };

    // Create source ownership proof
    let source_owner_proof = OwnershipProof {
        proof: vec![0x01],
        owner: state
            .get_address(&from)
            .map(|a| a.as_bytes().to_vec())
            .unwrap_or_else(|| vec![0xEE; 32]),
        scheme: None,
    };

    // Generate transfer ID
    let transfer_id_bytes = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(right_bytes);
        hasher.update(from.to_string().as_bytes());
        hasher.update(to.to_string().as_bytes());
        hasher.finalize().into()
    };
    let transfer_id = Hash::new(transfer_id_bytes);

    output::kv_hash("Transfer ID", &transfer_id_bytes);
    output::kv_hash("Right ID", &right_bytes);
    output::kv("From", &from_str);
    output::kv("To", &to_str);

    // Create chain-specific providers
    let source_chain_id = chain_to_chain_id(&from);
    let dest_chain_id = chain_to_chain_id(&to);

    // Step 1: Lock on source chain
    output::progress(1, 6, &format!("Step 1: Locking Right on {}...", from_str));
    let lock_provider = create_lock_provider(&from, source_chain_id.clone());
    let (lock_event, inclusion_proof) = lock_provider
        .lock_right(
            right_id_hash,
            right_id_hash,
            source_owner_proof,
            dest_chain_id.clone(),
            dest_owner_proof.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Lock failed: {:?}", e))?;

    // Step 2: Build transfer proof
    output::progress(2, 6, "Step 2: Building transfer proof...");

    // Get current block heights from RPC for finality verification
    // TODO: Replace with real RPC calls once contracts are deployed
    // For now, use realistic testnet heights from config
    let source_height = get_chain_height(&from, config);
    let confirmations = get_chain_confirmations(&from);
    let current_height = source_height + confirmations;

    // Only mark as finalized for chains with deterministic finality.
    // Bitcoin and Ethereum use probabilistic finality (confirmation depth).
    let is_finalized = matches!(source_chain_id, ChainId::Sui | ChainId::Aptos);

    let transfer_proof = CrossChainTransferProof {
        lock_event,
        inclusion_proof,
        finality_proof: CrossChainFinalityProof {
            source_chain: source_chain_id.clone(),
            height: source_height,
            current_height,
            is_finalized,
            depth: confirmations,
        },
        source_state_root: Hash::new([0u8; 32]),
    };

    // Step 3: Verify on destination chain
    output::progress(3, 6, "Step 3: Verifying proof on destination...");
    // Build registry from state's consumed seals for double-spend detection
    let mut registry = CrossChainSealRegistry::new();
    // Inject state's known seals into the registry
    for seal_bytes in &state.consumed_seals {
        use csv_adapter_core::right::RightId;
        use csv_adapter_core::seal::SealRef;
        use csv_adapter_core::seal_registry::{ChainId as CoreChainId, SealConsumption};
        if let Ok(seal_ref) = SealRef::new(seal_bytes.clone(), None) {
            let consumption = SealConsumption {
                chain: CoreChainId::Ethereum,
                seal_ref,
                right_id: RightId(Hash::new([0u8; 32])),
                block_height: 0,
                tx_hash: Hash::new([0u8; 32]),
                recorded_at: 0,
            };
            let _ = registry.record_consumption(consumption);
        }
    }
    let verifier = UniversalTransferVerifier { registry };
    verifier
        .verify_transfer_proof(&transfer_proof)
        .map_err(|e| anyhow::anyhow!("Verification failed: {:?}", e))?;

    // Step 4: Check CrossChainSealRegistry
    output::progress(4, 6, "Step 4: Checking seal registry...");
    if state.is_seal_consumed(&right_bytes) {
        output::error("Right has already been transferred (seal consumed)");
        return Err(anyhow::anyhow!("Double-spend detected"));
    }

    // Step 5: Mint on destination chain
    output::progress(5, 6, &format!("Step 5: Minting Right on {}...", to_str));
    let mint_provider =
        create_mint_provider(&to, dest_chain_id.clone()).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mint_result = mint_provider
        .mint_right(&transfer_proof)
        .map_err(|e| anyhow::anyhow!("Mint failed: {:?}", e))?;

    // Step 6: Record in registry
    output::progress(6, 6, "Step 6: Recording transfer...");
    state.record_seal_consumption(right_bytes.to_vec());

    // Create tracked transfer
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let transfer = TrackedTransfer {
        id: transfer_id,
        source_chain: from,
        dest_chain: to,
        right_id: right_id_hash,
        source_tx_hash: Some(transfer_proof.lock_event.source_tx_hash),
        dest_tx_hash: Some(mint_result.registry_entry.mint_tx_hash),
        proof: Some(serde_json::to_vec(&transfer_proof).unwrap_or_default()),
        status: TransferStatus::Completed,
        created_at: timestamp,
    };
    state.add_transfer(transfer);

    println!();
    output::success(&format!(
        "Cross-chain transfer complete: {} → {}",
        from_str, to_str
    ));
    output::kv_hash("Transfer ID", &transfer_id_bytes);
    output::kv(
        "Destination Right ID",
        &hex::encode(mint_result.destination_right.id.0.as_bytes()),
    );
    output::kv(
        "Destination Seal",
        &hex::encode(mint_result.destination_seal.to_vec()),
    );

    Ok(())
}

fn create_lock_provider(chain: &Chain, chain_id: ChainId) -> Box<dyn LockProvider> {
    match chain {
        Chain::Bitcoin => Box::new(BitcoinLockProvider {
            _chain_id: chain_id,
        }),
        Chain::Sui => Box::new(SuiLockProvider {
            _chain_id: chain_id,
        }),
        Chain::Aptos => Box::new(AptosLockProvider {
            _chain_id: chain_id,
        }),
        Chain::Ethereum => Box::new(EthereumLockProvider {
            _chain_id: chain_id,
        }),
    }
}

fn create_mint_provider(chain: &Chain, chain_id: ChainId) -> Result<Box<dyn MintProvider>, String> {
    match chain {
        Chain::Bitcoin => Err("Bitcoin is UTXO-native and does not support minting Rights. Bitcoin can only be used as a source chain for cross-chain transfers.".to_string()),
        Chain::Sui => Ok(Box::new(SuiMintProvider { chain_id })),
        Chain::Aptos => Ok(Box::new(AptosMintProvider { chain_id })),
        Chain::Ethereum => Ok(Box::new(EthereumMintProvider { chain_id })),
    }
}

fn chain_to_chain_id(chain: &Chain) -> ChainId {
    match chain {
        Chain::Bitcoin => ChainId::Bitcoin,
        Chain::Ethereum => ChainId::Ethereum,
        Chain::Sui => ChainId::Sui,
        Chain::Aptos => ChainId::Aptos,
    }
}

fn cmd_status(transfer_id: String, state: &State) -> Result<()> {
    let bytes = hex::decode(transfer_id.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Invalid Transfer ID: {}", e))?;
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&bytes[..32]);
    let transfer_id_hash = Hash::new(hash_bytes);

    output::header(&format!("Transfer: {}", transfer_id));

    if let Some(transfer) = state.get_transfer(&transfer_id_hash) {
        output::kv("Source Chain", &transfer.source_chain.to_string());
        output::kv("Destination Chain", &transfer.dest_chain.to_string());
        output::kv_hash("Right ID", transfer.right_id.as_bytes());
        output::kv("Status", &format!("{:?}", transfer.status));

        if let Some(source_tx) = &transfer.source_tx_hash {
            output::kv_hash("Source TX", source_tx.as_bytes());
        }
        if let Some(dest_tx) = &transfer.dest_tx_hash {
            output::kv_hash("Destination TX", dest_tx.as_bytes());
        }
    } else {
        output::warning("Transfer not found");
    }

    Ok(())
}

fn cmd_list(from: Option<Chain>, to: Option<Chain>, state: &State) -> Result<()> {
    output::header("Cross-Chain Transfers");

    let headers = vec!["Transfer ID", "From", "To", "Right ID", "Status"];
    let mut rows = Vec::new();

    for transfer in &state.transfers {
        if let Some(ref filter_from) = from {
            if transfer.source_chain != *filter_from {
                continue;
            }
        }
        if let Some(ref filter_to) = to {
            if transfer.dest_chain != *filter_to {
                continue;
            }
        }

        let status_str = match &transfer.status {
            TransferStatus::Completed => "Completed".to_string(),
            TransferStatus::Failed { reason } => format!("Failed: {}", reason),
            other => format!("{:?}", other),
        };

        rows.push(vec![
            hex::encode(transfer.id.as_bytes())[..10].to_string(),
            transfer.source_chain.to_string(),
            transfer.dest_chain.to_string(),
            hex::encode(transfer.right_id.as_bytes())[..10].to_string(),
            status_str,
        ]);
    }

    if rows.is_empty() {
        output::info("No transfers recorded. Use 'csv cross-chain transfer' to start one.");
    } else {
        output::table(&headers, &rows);
    }

    Ok(())
}

fn cmd_retry(transfer_id: String, _config: &Config, state: &mut State) -> Result<()> {
    output::header("Retrying Transfer");
    output::kv("Transfer ID", &transfer_id);

    // Parse transfer ID
    let bytes = hex::decode(transfer_id.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Invalid Transfer ID: {}", e))?;
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&bytes[..32]);
    let transfer_id_hash = Hash::new(hash_bytes);

    // Look up transfer
    let transfer = state.get_transfer(&transfer_id_hash);
    match transfer {
        Some(t) => {
            output::kv("Source", &t.source_chain.to_string());
            output::kv("Destination", &t.dest_chain.to_string());
            output::kv("Status", &format!("{:?}", t.status));

            match &t.status {
                TransferStatus::Failed { reason } => {
                    output::warning(&format!("Failure reason: {}", reason));
                    output::info("If lock was successful but mint failed, wait for timeout (24h) and the source chain seal will be recoverable via refund.");
                    output::info("For timed-out locks: the refund function is available on the source chain contract.");
                }
                TransferStatus::Locked | TransferStatus::Initiated => {
                    output::info(
                        "Transfer is in progress. If stuck, wait for lock timeout and refund.",
                    );
                }
                TransferStatus::Completed => {
                    output::success("Transfer already completed successfully.");
                }
                _ => {
                    output::info("Transfer status does not support retry.");
                }
            }
        }
        None => {
            output::warning("Transfer not found in state.");
        }
    }

    Ok(())
}

/// Get the current block/checkpoint height for a chain.
/// TODO: Replace with real RPC call.
fn get_chain_height(chain: &Chain, _config: &Config) -> u64 {
    // Approximate testnet heights (would be fetched from RPC in production)
    match chain {
        Chain::Bitcoin => 300_000,    // Signet
        Chain::Ethereum => 7_000_000, // Sepolia
        Chain::Sui => 350_000_000,    // Testnet checkpoints
        Chain::Aptos => 15_000_000,   // Testnet version
    }
}

/// Get the required confirmation depth for a chain.
fn get_chain_confirmations(chain: &Chain) -> u64 {
    match chain {
        Chain::Bitcoin => 6,   // ~1 hour on signet
        Chain::Ethereum => 15, // ~3 minutes
        Chain::Sui => 1,       // Finality is ~1 checkpoint
        Chain::Aptos => 1,     // Finality is ~1 block (HotStuff)
    }
}
