//! Replay Store for Persistent Replay Detection
//!
//! Provides SQLite-backed storage for replay detection with crash-safe persistence.

use csv_core::hash::Hash;

/// Persistent replay store
#[cfg(feature = "sqlite")]
pub struct ReplayStore {
    /// SQLite database connection
    db: sqlx::SqlitePool,
}

#[cfg(feature = "sqlite")]
impl ReplayStore {
    /// Create a new replay store with the given database path
    pub async fn new(database_path: &str) -> Result<Self, sqlx::Error> {
        let db = sqlx::SqlitePool::connect(database_path).await?;
        
        // Initialize schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS replay_entries (
                key_hash BLOB PRIMARY KEY,
                proof_hash BLOB NOT NULL,
                seal_id BLOB NOT NULL,
                commitment_hash BLOB NOT NULL,
                source_chain TEXT NOT NULL,
                destination_chain TEXT NOT NULL,
                first_seen_at INTEGER NOT NULL,
                replay_attempts INTEGER NOT NULL DEFAULT 0,
                accepted BOOLEAN NOT NULL DEFAULT FALSE
            );
            
            CREATE INDEX IF NOT EXISTS idx_replay_proof ON replay_entries(proof_hash);
            CREATE INDEX IF NOT EXISTS idx_replay_seal ON replay_entries(seal_id);
            "#
        )
        .execute(&db)
        .await?;
        
        Ok(Self { db })
    }

    /// Record a proof in the replay registry
    pub async fn record_proof(
        &self,
        key_hash: Hash,
        proof_hash: Hash,
        seal_id: Hash,
        commitment_hash: Hash,
        source_chain: &str,
        destination_chain: &str,
        timestamp: u64,
    ) -> Result<bool, sqlx::Error> {
        // Check if already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM replay_entries WHERE key_hash = ?"
        )
        .bind(key_hash.as_bytes())
        .fetch_one(&self.db)
        .await?;
        
        if existing > 0 {
            // Replay attempt - increment counter
            sqlx::query(
                "UPDATE replay_entries SET replay_attempts = replay_attempts + 1 WHERE key_hash = ?"
            )
            .bind(key_hash.as_bytes())
            .execute(&self.db)
            .await?;
            Ok(false)
        } else {
            // First time - insert new entry
            sqlx::query(
                r#"
                INSERT INTO replay_entries (
                    key_hash, proof_hash, seal_id, commitment_hash,
                    source_chain, destination_chain, first_seen_at, replay_attempts, accepted
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, FALSE)
                "#
            )
            .bind(key_hash.as_bytes())
            .bind(proof_hash.as_bytes())
            .bind(seal_id.as_bytes())
            .bind(commitment_hash.as_bytes())
            .bind(source_chain)
            .bind(destination_chain)
            .bind(timestamp as i64)
            .execute(&self.db)
            .await?;
            Ok(true)
        }
    }

    /// Check if a proof has been seen before
    pub async fn has_been_seen(&self, key_hash: Hash) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM replay_entries WHERE key_hash = ?"
        )
        .bind(key_hash.as_bytes())
        .fetch_one(&self.db)
        .await?;
        Ok(count > 0)
    }

    /// Get total replay attempts
    pub async fn total_replay_attempts(&self) -> Result<u64, sqlx::Error> {
        let total = sqlx::query_scalar::<_, i64>("SELECT SUM(replay_attempts) FROM replay_entries")
            .fetch_one(&self.db)
            .await?;
        Ok(total as u64)
    }
}
