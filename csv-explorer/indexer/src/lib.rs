/// Multi-chain indexer daemon for the CSV Explorer.
///
/// Coordinates chain-specific indexers, manages sync progress,
/// and exposes metrics for monitoring.
pub mod aptos;
pub mod bitcoin;
pub mod chain_indexer;
pub mod ethereum;
pub mod metrics;
pub mod rpc_manager;
pub mod solana;
pub mod sui;
pub mod sync;
pub mod wallet_bridge;

pub use chain_indexer::{AddressIndexingResult, ChainIndexer, ChainResult};
pub use rpc_manager::{load_rpc_config, AuthType, RpcConfig, RpcEndpoint, RpcManager, RpcType};
pub use sync::SyncCoordinator;
pub use wallet_bridge::{WalletIndexerBridge, WalletIndexerBridgeConfig};

use csv_explorer_shared::{ExplorerConfig, Result};
use sqlx::SqlitePool;

/// The main indexer that wraps all chain indexers.
pub struct Indexer {
    config: ExplorerConfig,
    coordinator: SyncCoordinator,
    wallet_bridge: Option<WalletIndexerBridge>,
    rpc_manager: RpcManager,
}

impl Indexer {
    /// Create a new indexer with the given configuration and database pool.
    pub async fn new(config: ExplorerConfig, pool: SqlitePool) -> Result<Self> {
        // Load RPC configuration
        let rpc_manager = RpcManager::new(load_rpc_config()?);

        // Build chain indexers based on configuration
        let mut indexers: Vec<Box<dyn ChainIndexer>> = Vec::new();

        for (chain_id, chain_config) in &config.chains {
            if !chain_config.enabled {
                continue;
            }
            if let Some(indexer) =
                build_boxed_indexer(chain_id, chain_config.clone(), rpc_manager.clone())
            {
                indexers.push(indexer);
            } else {
                tracing::warn!(chain = %chain_id, "No indexer implementation registered for discovered chain");
            }
        }

        let coordinator = SyncCoordinator::new(
            indexers,
            pool.clone(),
            config.chains.clone(),
            config.indexer.concurrency,
            config.indexer.batch_size,
            config.indexer.poll_interval_ms,
        );

        Ok(Self {
            config,
            coordinator,
            wallet_bridge: None,
            rpc_manager,
        })
    }

    /// Initialize the wallet-indexer bridge with priority indexing.
    pub async fn with_wallet_bridge(mut self, config: WalletIndexerBridgeConfig) -> Result<Self> {
        // Rebuild indexers as Arc<dyn ChainIndexer> using the existing rpc_manager
        let mut indexers: Vec<std::sync::Arc<dyn ChainIndexer>> = Vec::new();

        for (chain_id, chain_config) in &self.config.chains {
            if !chain_config.enabled {
                continue;
            }
            if let Some(indexer) =
                build_arc_indexer(chain_id, chain_config.clone(), self.rpc_manager.clone())
            {
                indexers.push(indexer);
            } else {
                tracing::warn!(chain = %chain_id, "No wallet-bridge indexer implementation registered for discovered chain");
            }
        }

        let bridge =
            WalletIndexerBridge::new(self.coordinator.get_pool().clone(), indexers, config);

        bridge.initialize().await?;

        self.wallet_bridge = Some(bridge);
        Ok(self)
    }

    /// Get a reference to the wallet bridge if available.
    pub fn wallet_bridge(&self) -> Option<&WalletIndexerBridge> {
        self.wallet_bridge.as_ref()
    }

    /// Initialize all chain indexers.
    pub async fn initialize(&self) -> Result<()> {
        self.coordinator.initialize(&self.config.chains).await?;
        Ok(())
    }

    /// Start the indexer daemon.
    pub async fn start(&self) -> Result<()> {
        // Start wallet bridge if available
        if let Some(bridge) = &self.wallet_bridge {
            let bridge_clone = bridge.clone();
            tokio::spawn(async move {
                if let Err(e) = bridge_clone.start().await {
                    tracing::error!(error = %e, "Wallet bridge error");
                }
            });
        }

        self.coordinator.start().await?;
        tracing::info!("Indexer daemon started");
        Ok(())
    }

    /// Stop the indexer daemon.
    pub async fn stop(&self) -> Result<()> {
        if let Some(bridge) = &self.wallet_bridge {
            bridge.stop().await?;
        }
        self.coordinator.stop().await?;
        tracing::info!("Indexer daemon stopped");
        Ok(())
    }

    /// Get the current indexer status.
    pub async fn status(&self) -> csv_explorer_shared::IndexerStatus {
        self.coordinator.status().await
    }

    /// Force sync a specific chain.
    pub async fn sync_chain(&self, chain: &str) -> Result<()> {
        self.coordinator.sync_chain(chain).await
    }

    /// Force sync a specific chain from a specific block (overrides config).
    pub async fn sync_chain_from_block(&self, chain: &str, from_block: u64) -> Result<()> {
        self.coordinator
            .sync_chain_from_block(chain, from_block)
            .await
    }

    /// Reindex a chain from a specific block.
    pub async fn reindex_from(&self, chain: &str, from_block: u64) -> Result<()> {
        self.coordinator.reindex_from(chain, from_block).await
    }

    /// Reset all sync progress.
    pub async fn reset_sync(&self) -> Result<()> {
        self.coordinator.reset_sync().await
    }
}

fn build_boxed_indexer(
    chain_id: &str,
    config: csv_explorer_shared::ChainConfig,
    rpc_manager: RpcManager,
) -> Option<Box<dyn ChainIndexer>> {
    match chain_id {
        "bitcoin" => Some(Box::new(bitcoin::BitcoinIndexer::new(config, rpc_manager))),
        "ethereum" => Some(Box::new(ethereum::EthereumIndexer::new(
            config,
            rpc_manager,
        ))),
        "sui" => Some(Box::new(sui::SuiIndexer::new(config, rpc_manager))),
        "aptos" => Some(Box::new(aptos::AptosIndexer::new(config, rpc_manager))),
        "solana" => Some(Box::new(solana::SolanaIndexer::new(config, rpc_manager))),
        _ => None,
    }
}

fn build_arc_indexer(
    chain_id: &str,
    config: csv_explorer_shared::ChainConfig,
    rpc_manager: RpcManager,
) -> Option<std::sync::Arc<dyn ChainIndexer>> {
    match chain_id {
        "bitcoin" => Some(std::sync::Arc::new(bitcoin::BitcoinIndexer::new(
            config,
            rpc_manager,
        ))),
        "ethereum" => Some(std::sync::Arc::new(ethereum::EthereumIndexer::new(
            config,
            rpc_manager,
        ))),
        "sui" => Some(std::sync::Arc::new(sui::SuiIndexer::new(
            config,
            rpc_manager,
        ))),
        "aptos" => Some(std::sync::Arc::new(aptos::AptosIndexer::new(
            config,
            rpc_manager,
        ))),
        "solana" => Some(std::sync::Arc::new(solana::SolanaIndexer::new(
            config,
            rpc_manager,
        ))),
        _ => None,
    }
}
