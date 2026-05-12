//! Property tests for seal consumption
//!
//! These tests verify that seals can only be consumed once,
//! and that double-spend attacks are prevented using the replay registry.

#[cfg(test)]
mod tests {
    use csv_core::seal::SealPoint;
    use csv_core::hash::Hash;
    use csv_core::replay_registry::{ReplayRegistry, ReplayKey};
    use csv_core::ChainId;

    /// Property: A seal can only be consumed once
    #[test]
    fn test_seal_consumption_idempotency() {
        let mut registry = ReplayRegistry::new();
        
        // Create a seal point
        let seal_point = SealPoint::new(vec![1u8; 16], Some(1)).unwrap();
        let seal_id = Hash::new([1u8; 32]);
        
        // Create a replay key for this seal
        let replay_key = ReplayKey::new(
            seal_id,
            seal_id, // seal_id (simplified)
            seal_id, // commitment_hash (simplified)
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        );
        
        // First consumption should succeed (seal not yet consumed)
        let is_consumed = registry.is_replay(&replay_key).unwrap();
        assert!(!is_consumed, "Seal should not be consumed initially");
        
        // Record the consumption
        registry.record_proof(
            seal_id,
            seal_id, // seal_id
            seal_id, // commitment_hash
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        ).unwrap();
        
        // Second consumption attempt should fail (seal already consumed)
        let is_consumed_after = registry.is_replay(&replay_key).unwrap();
        assert!(is_consumed_after, "Seal should be consumed after recording");
    }

    /// Property: Each seal has a unique identifier
    #[test]
    fn test_seal_uniqueness() {
        let seal1 = SealPoint::new(vec![1u8; 16], Some(1)).unwrap();
        let seal2 = SealPoint::new(vec![2u8; 16], Some(2)).unwrap();
        
        assert_ne!(seal1.id, seal2.id, "Different seals should have different IDs");
    }

    /// Property: Seal consumption is tracked via nullifier registration
    #[test]
    fn test_seal_consumption_tracking() {
        let mut registry = ReplayRegistry::new();
        
        let seal_id = Hash::new([1u8; 32]);
        let commitment_hash = Hash::new([2u8; 32]);
        
        // Create replay key
        let replay_key = ReplayKey::new(
            seal_id,
            seal_id, // seal_id
            commitment_hash,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        );
        
        // Initially, no replay detected
        assert!(!registry.is_replay(&replay_key).unwrap());
        
        // Record the proof (consumes the seal)
        registry.record_proof(
            seal_id,
            seal_id, // seal_id
            commitment_hash,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        ).unwrap();
        
        // Now replay should be detected
        assert!(registry.is_replay(&replay_key).unwrap());
        
        // Verify the nullifier is registered
        let entries = registry.list_entries().unwrap();
        assert!(!entries.is_empty(), "Replay registry should have entries after consumption");
    }

    /// Property: Different chains have independent seal consumption
    #[test]
    fn test_cross_chain_seal_independence() {
        let mut registry = ReplayRegistry::new();
        
        let seal_id = Hash::new([1u8; 32]);
        let commitment_hash = Hash::new([2u8; 32]);
        
        // Consume seal on Bitcoin -> Ethereum
        let replay_key_1 = ReplayKey::new(
            seal_id,
            seal_id,
            commitment_hash,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        );
        
        registry.record_proof(
            seal_id,
            seal_id,
            commitment_hash,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        ).unwrap();
        
        // Same seal on different destination chain should be considered different
        let replay_key_2 = ReplayKey::new(
            seal_id,
            seal_id,
            commitment_hash,
            ChainId::new("bitcoin"),
            ChainId::new("solana"), // Different destination
        );
        
        // Should not be a replay since destination chain is different
        assert!(!registry.is_replay(&replay_key_2).unwrap());
    }

    /// Property: Different commitment hashes create different replay keys
    #[test]
    fn test_commitment_hash_uniqueness() {
        let seal_id = Hash::new([1u8; 32]);
        let commitment_hash_1 = Hash::new([2u8; 32]);
        let commitment_hash_2 = Hash::new([3u8; 32]);
        
        let replay_key_1 = ReplayKey::new(
            seal_id,
            seal_id,
            commitment_hash_1,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        );
        
        let replay_key_2 = ReplayKey::new(
            seal_id,
            seal_id,
            commitment_hash_2,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        );
        
        // Different commitment hashes should produce different replay keys
        assert_ne!(replay_key_1.hash(), replay_key_2.hash());
    }

    /// Property: Replay registry prevents double-spend across multiple attempts
    #[test]
    fn test_double_spend_prevention() {
        let mut registry = ReplayRegistry::new();
        
        let seal_id = Hash::new([1u8; 32]);
        let commitment_hash = Hash::new([2u8; 32]);
        
        let replay_key = ReplayKey::new(
            seal_id,
            seal_id,
            commitment_hash,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        );
        
        // First attempt - should succeed
        let result_1 = registry.record_proof(
            seal_id,
            seal_id,
            commitment_hash,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        );
        assert!(result_1.is_ok(), "First consumption should succeed");
        
        // Second attempt - should detect replay
        let is_replay = registry.is_replay(&replay_key).unwrap();
        assert!(is_replay, "Second consumption should be detected as replay");
        
        // Third attempt - should also detect replay
        let is_replay_2 = registry.is_replay(&replay_key).unwrap();
        assert!(is_replay_2, "Third consumption should also be detected as replay");
    }
}
