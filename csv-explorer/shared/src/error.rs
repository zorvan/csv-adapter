/// Error types for the CSV Explorer.
use thiserror::Error;

/// Top-level error type for the explorer.
#[derive(Error, Debug)]
pub enum ExplorerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML error: {0}")]
    Toml(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("RPC error on chain {chain}: {message}")]
    RpcError { chain: String, message: String },

    #[error("RPC parse error on chain {chain}: {message}")]
    RpcParseError { chain: String, message: String },

    #[error("Indexer stopped")]
    IndexerStopped,

    #[error("Block processing error on chain {chain} at block {block}: {message}")]
    BlockError {
        chain: String,
        block: u64,
        message: String,
    },

    #[error("Chain reorg detected on {chain} at block {block}, depth: {depth}")]
    ChainReorg {
        chain: String,
        block: u64,
        depth: u64,
    },

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("HTTP server error: {0}")]
    HttpServer(String),

    #[error("Hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ExplorerError>;
