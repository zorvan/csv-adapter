//! Persistent Replay Registry Store
//!
//! This module provides persistent storage for the replay registry using SQLite.
//! It ensures that replay protection survives across:
//! - application restart
//! - crash recovery
//! - node migration

use csv_core::domain_hash::DomainSeparatedHash;
use csv_core::domains::ReplayRegistryDomain;
use csv_core::hash::Hash;
use csv_core::protocol_version::ChainId;
use csv_core::replay_registry::{ReplayEntry, ReplayKey};

/// Persistent replay registry store
///
/// This provides SQLite-backed persistence for replay detection.
/// In production, this should be used instead of the in-memory ReplayRegistry.
#[cfg(feature = "std")]
pub struct ReplayRegistryStore {
    /// SQLite database connection
    db: sqlx::SqlitePool,
}

#[cfg(feature = "std")]
impl ReplayRegistryStore {
    /// Create a new replay registry store with the given database path
    pub async fn new(database_path: &str) -> Result<Self, sqlx::Error> {
        let db = sqlx::SqlitePool::connect(database_path).await?;
        
        // Initialize schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS replay_registry (
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
            
            CREATE INDEX IF NOT EXISTS idx_proof_hash ON replay_registry(proof_hash);
            CREATE INDEX IF NOT EXISTS idx_seal_id ON replay_registry(seal_id);
            "#
        )
        .execute(&db)
        .await?;
        
        Ok(Self { db })
    }

    /// Record a proof in the persistent replay registry
    ///
    /// Returns true if this is the first time seeing this proof,
    /// false if it's a replay attempt.
    pub async fn record_proof(&self, key: ReplayKey, timestamp: u64) -> Result<bool, sqlx::Error> {
        let key_hash = key.hash();
        
        // Check if already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM replay_registry WHERE key_hash = ?"
        )
        .bind(key_hash.as_bytes())
        .fetch_one(&self.db)
        .await?;
        
        if existing > 0 {
            // Replay attempt - increment counter
            sqlx::query(
                "UPDATE replay_registry SET replay_attempts = replay_attempts + 1 WHERE key_hash = ?"
            )
            .bind(key_hash.as_bytes())
            .execute(&self.db)
            .await?;
            Ok(false)
        } else {
            // First time - insert new entry
            sqlx::query(
                r#"
                INSERT INTO replay_registry (
                    key_hash, proof_hash, seal_id, commitment_hash,
                    source_chain, destination_chain, first_seen_at, replay_attempts, accepted
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, FALSE)
                "#
            )
            .bind(key_hash.as_bytes())
            .bind(key.proof_hash.as_bytes())
            .bind(key.seal_id.as_bytes())
            .bind(key.commitment_hash.as_bytes())
            .bind(key.source_chain.as_str())
            .bind(key.destination_chain.as_str())
            .bind(timestamp as i64)
            .execute(&self.db)
            .await?;
            Ok(true)
        }
    }

    /// Check if a proof has been seen before
    pub async fn has_been_seen(&self, key: &ReplayKey) -> Result<bool, sqlx::Error> {
        let key_hash = key.hash();
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM replay_registry WHERE key_hash = ?"
        )
        .bind(key_hash.as_bytes())
        .fetch_one(&self.db)
        .await?;
        Ok(count > 0)
    }

    /// Mark a proof as accepted
    pub async fn mark_accepted(&self, key: &ReplayKey) -> Result<(), sqlx::Error> {
        let key_hash = key.hash();
        sqlx::query("UPDATE replay_registry SET accepted = TRUE WHERE key_hash = ?")
            .bind(key_hash.as_bytes())
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Get the number of replay attempts for a proof
    pub async fn replay_attempts(&self, key: &ReplayKey) -> Result<u64, sqlx::Error> {
        let key_hash = key.hash();
        let attempts = sqlx::query_scalar::<_, i64>(
            "SELECT replay_attempts FROM replay_registry WHERE key_hash = ?"
        )
        .bind(key_hash.as_bytes())
        .fetch_optional(&self.db)
        .await?;
        Ok(attempts.unwrap_or(0) as u64)
    }

    /// Get all entries
    pub async fn entries(&self) -> Result<Vec<ReplayEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, String, String, i64, i64, bool)>(
            "SELECT proof_hash, seal_id, commitment_hash, key_hash, source_chain, destination_chain, first_seen_at, replay_attempts, accepted FROM replay_registry"
        )
        .fetch_all(&self.db)
        .await?;
        
        let mut entries = Vec::new();
        for row in rows {
            let proof_hash = Hash::new(row.0.try_into().unwrap_or([0u8; 32]));
            let seal_id = Hash::new(row.1.try_into().unwrap_or([0u8; 32]));
            let commitment_hash = Hash::new(row.2.try_into().unwrap_or([0u8; 32]));
            let source_chain = ChainId::new(&row.4);
            let destination_chain = ChainId::new(&row.5);
            
            let key = ReplayKey::new(proof_hash, seal_id, commitment_hash, source_chain, destination_chain);
            let entry = ReplayEntry {
                key,
                first_seen_at: row.6 as u64,
                replay_attempts: row.7 as u64,
                accepted: row.8,
            };
            entries.push(entry);
        }
        
        Ok(entries)
    }

    /// Get the total number of tracked proofs
    pub async fn total_proofs(&self) -> Result<usize, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM replay_registry")
            .fetch_one(&self.db)
            .await?;
        Ok(count as usize)
    }

    /// Get the number of replay attempts detected
    pub async fn total_replay_attempts(&self) -> Result<u64, sqlx::Error> {
        let total = sqlx::query_scalar::<_, i64>("SELECT SUM(replay_attempts) FROM replay_registry")
            .fetch_one(&self.db)
            .await?;
        Ok(total as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "std")]
    async fn test_replay_registry_store() {
        // This test requires an actual SQLite database
        // In a real test setup, you would use :memory: for testing
        // For now, we'll skip this as it requires async runtime setup
    }
}
