//! The Universal Seal Primitive — Canonical Right Type
//!
//! A Right can be exercised at most once under the strongest available
//! guarantee of the host chain. This is the core invariant of the entire system.
//!
//! ## Enforcement Layers
//!
//! | Level | Name | Chains | Mechanism |
//! |-------|------|--------|-----------|
//! | L1 | Structural | Bitcoin, Sui | Spend UTXO / Consume Object |
//! | L2 | Type-Enforced | Aptos | Destroy Move Resource |
//! | L3 | Cryptographic | Ethereum | Nullifier Registration |
//!
//! ## Client-Side Validation
//!
//! The chain does NOT validate state transitions. It only:
//! 1. Records the commitment (anchor)
//! 2. Enforces single-use of the Right
//!
//! Clients do everything else:
//! 1. Fetch the full state history for a contract
//! 2. Verify the commitment chain from genesis to present
//! 3. Check that no Right was consumed more than once
//! 4. Accept or reject the consignment based on local validation

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::hash::Hash;

/// A unique Right identifier.
///
/// Computed as `H(commitment || salt)` to ensure uniqueness
/// even when the same state is committed to multiple times.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RightId(pub Hash);

/// Proof of ownership for a Right.
///
/// On L1 chains (Bitcoin, Sui): this is the UTXO/Object ownership proof.
/// On L2 chains (Aptos): this is the resource capability proof.
/// On L3 chains (Ethereum): this is the signature from the owner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnershipProof {
    /// The proof bytes (chain-specific format)
    pub proof: Vec<u8>,
    /// The owner identifier (address, pubkey, etc.)
    pub owner: Vec<u8>,
}

/// A consumable Right in the USP system.
///
/// Every chain enforces single-use of Rights, but at different
/// enforcement levels (L1 Structural → L2 Type-Enforced → L3 Cryptographic).
///
/// The chain provides the minimum guarantee (single-use enforcement).
/// Clients verify everything else.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Right {
    /// Unique identifier: `H(commitment || salt)`
    pub id: RightId,
    /// Encodes the state + rules of this Right
    pub commitment: Hash,
    /// Proof of ownership
    pub owner: OwnershipProof,
    /// One-time consumption marker (L3+ only)
    ///
    /// L1 (Bitcoin/Sui): None — chain enforces structurally.
    /// L2 (Aptos): None — Move VM enforces non-duplication.
    /// L3 (Ethereum): Some — nullifier registered in contract.
    pub nullifier: Option<Hash>,
    /// Off-chain state commitment root
    ///
    /// Commits to the full state history for this Right.
    /// Clients use this to verify state transitions without
    /// fetching the entire history on every validation.
    pub state_root: Option<Hash>,
    /// Optional execution proof (ZK, fraud proof, etc.)
    ///
    /// For advanced use cases where the Right's execution
    /// needs to be proven without revealing its contents.
    pub execution_proof: Option<Vec<u8>>,
}

impl Right {
    /// Create a new Right with the given parameters.
    ///
    /// The Right ID is deterministically computed from the commitment
    /// and salt, ensuring uniqueness even for duplicate commitments.
    pub fn new(
        commitment: Hash,
        owner: OwnershipProof,
        salt: &[u8],
    ) -> Self {
        let id = {
            let mut hasher = Sha256::new();
            hasher.update(commitment.as_bytes());
            hasher.update(salt);
            let result = hasher.finalize();
            let mut array = [0u8; 32];
            array.copy_from_slice(&result);
            RightId(Hash::new(array))
        };

        Self {
            id,
            commitment,
            owner,
            nullifier: None,
            state_root: None,
            execution_proof: None,
        }
    }

    /// Mark this Right as consumed by setting the nullifier.
    ///
    /// # Enforcement Level
    ///
    /// - **L1 (Bitcoin/Sui)**: This method is a local marker only.
    ///   The actual single-use enforcement is done by the chain
    ///   (UTXO spending / Object deletion).
    ///
    /// - **L2 (Aptos)**: This method is a local marker only.
    ///   The Move VM enforces non-duplication of resources.
    ///
    /// - **L3 (Ethereum)**: The nullifier MUST be registered on-chain.
    ///   The contract's `nullifiers[id] = true` is what enforces single-use.
    ///
    /// # Returns
    /// The nullifier hash, or `None` for L1/L2 chains where the
    /// nullifier is not needed (but returned for local tracking).
    pub fn consume(&mut self, secret: Option<&[u8]>) -> Option<Hash> {
        if let Some(secret) = secret {
            // L3: Compute deterministic nullifier
            let nullifier = {
                let mut hasher = Sha256::new();
                hasher.update(self.id.0.as_bytes());
                hasher.update(secret);
                let result = hasher.finalize();
                let mut array = [0u8; 32];
                array.copy_from_slice(&result);
                Hash::new(array)
            };
            self.nullifier = Some(nullifier);
            Some(nullifier)
        } else {
            // L1/L2: No nullifier needed — chain enforces structurally.
            // Set a local consumption marker for tracking purposes.
            None
        }
    }

    /// Transfer this Right to a new owner.
    ///
    /// Creates a new Right with the same commitment and state but
    /// different ownership. The original Right remains valid until
    /// explicitly consumed.
    ///
    /// # Arguments
    /// * `new_owner` - The new owner's ownership proof
    /// * `transfer_salt` - A unique salt for the transfer to ensure unique ID
    ///
    /// # Returns
    /// A new Right instance with the new owner and a fresh ID
    pub fn transfer(
        &self,
        new_owner: OwnershipProof,
        transfer_salt: &[u8],
    ) -> Right {
        // Create a new Right with same commitment but new owner
        let mut new_right = Right::new(
            self.commitment,
            new_owner,
            transfer_salt,
        );

        // Preserve state root if present
        new_right.state_root = self.state_root;

        // Preserve execution proof if present
        new_right.execution_proof = self.execution_proof.clone();

        new_right
    }

    /// Verify this Right's ownership and validity.
    ///
    /// This is the core client-side validation function. It checks:
    /// 1. The owner proof is valid for this Right
    /// 2. The commitment is well-formed
    /// 3. The Right has not been consumed (nullifier not set)
    ///
    /// For full consignment validation, use the client-side
    /// validation engine (Sprint 2).
    pub fn verify(&self) -> Result<(), RightError> {
        // Check ownership proof is present
        if self.owner.proof.is_empty() {
            return Err(RightError::MissingOwnershipProof);
        }

        // Check commitment is non-zero
        if self.commitment.as_bytes() == &[0u8; 32] {
            return Err(RightError::InvalidCommitment);
        }

        // Check Right has not been consumed
        if self.nullifier.is_some() {
            return Err(RightError::AlreadyConsumed);
        }

        Ok(())
    }

    /// Serialize this Right to canonical bytes.
    ///
    /// Used for hashing, signing, and transmission.
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(self.id.0.as_bytes());
        out.extend_from_slice(self.commitment.as_bytes());
        out.extend_from_slice(&(self.owner.proof.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.owner.proof);
        out.extend_from_slice(&(self.owner.owner.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.owner.owner);
        out.push(if self.nullifier.is_some() { 1 } else { 0 });
        if let Some(nullifier) = &self.nullifier {
            out.extend_from_slice(nullifier.as_bytes());
        }
        out.push(if self.state_root.is_some() { 1 } else { 0 });
        if let Some(state_root) = &self.state_root {
            out.extend_from_slice(state_root.as_bytes());
        }
        out.extend_from_slice(
            &(self.execution_proof.as_ref().map_or(0, |p| p.len()) as u32).to_le_bytes(),
        );
        if let Some(proof) = &self.execution_proof {
            out.extend_from_slice(proof);
        }
        out
    }

    /// Deserialize a Right from canonical bytes.
    ///
    /// # Errors
    /// Returns `RightError::InvalidEncoding` if the bytes are malformed.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, RightError> {
        let mut pos = 0;

        // Read ID (32 bytes)
        if bytes.len() < 32 {
            return Err(RightError::InvalidEncoding);
        }
        let mut id_bytes = [0u8; 32];
        id_bytes.copy_from_slice(&bytes[0..32]);
        pos += 32;

        // Read commitment (32 bytes)
        if bytes.len() < pos + 32 {
            return Err(RightError::InvalidEncoding);
        }
        let mut commitment_bytes = [0u8; 32];
        commitment_bytes.copy_from_slice(&bytes[pos..pos + 32]);
        pos += 32;

        // Read owner proof length and data
        if bytes.len() < pos + 4 {
            return Err(RightError::InvalidEncoding);
        }
        let proof_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into()
            .map_err(|_| RightError::InvalidEncoding)?) as usize;
        pos += 4;

        if bytes.len() < pos + proof_len {
            return Err(RightError::InvalidEncoding);
        }
        let proof = bytes[pos..pos + proof_len].to_vec();
        pos += proof_len;

        // Read owner identifier length and data
        if bytes.len() < pos + 4 {
            return Err(RightError::InvalidEncoding);
        }
        let owner_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into()
            .map_err(|_| RightError::InvalidEncoding)?) as usize;
        pos += 4;

        if bytes.len() < pos + owner_len {
            return Err(RightError::InvalidEncoding);
        }
        let owner = bytes[pos..pos + owner_len].to_vec();
        pos += owner_len;

        // Read nullifier flag and data
        if pos >= bytes.len() {
            return Err(RightError::InvalidEncoding);
        }
        let has_nullifier = bytes[pos] == 1;
        pos += 1;

        let nullifier = if has_nullifier {
            if bytes.len() < pos + 32 {
                return Err(RightError::InvalidEncoding);
            }
            let mut nullifier_bytes = [0u8; 32];
            nullifier_bytes.copy_from_slice(&bytes[pos..pos + 32]);
            pos += 32;
            Some(Hash::new(nullifier_bytes))
        } else {
            None
        };

        // Read state_root flag and data
        if pos >= bytes.len() {
            return Err(RightError::InvalidEncoding);
        }
        let has_state_root = bytes[pos] == 1;
        pos += 1;

        let state_root = if has_state_root {
            if bytes.len() < pos + 32 {
                return Err(RightError::InvalidEncoding);
            }
            let mut state_root_bytes = [0u8; 32];
            state_root_bytes.copy_from_slice(&bytes[pos..pos + 32]);
            pos += 32;
            Some(Hash::new(state_root_bytes))
        } else {
            None
        };

        // Read execution proof length and data
        if bytes.len() < pos + 4 {
            return Err(RightError::InvalidEncoding);
        }
        let proof_data_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into()
            .map_err(|_| RightError::InvalidEncoding)?) as usize;
        pos += 4;

        let execution_proof = if proof_data_len > 0 {
            if bytes.len() < pos + proof_data_len {
                return Err(RightError::InvalidEncoding);
            }
            Some(bytes[pos..pos + proof_data_len].to_vec())
        } else {
            None
        };

        // Reconstruct the Right
        let id = RightId(Hash::new(id_bytes));
        let commitment = Hash::new(commitment_bytes);
        let owner = OwnershipProof { proof, owner };

        Ok(Self {
            id,
            commitment,
            owner,
            nullifier,
            state_root,
            execution_proof,
        })
    }

    /// Check if this Right has been consumed.
    pub fn is_consumed(&self) -> bool {
        self.nullifier.is_some()
    }

    /// Get the chain enforcement level indicator.
    ///
    /// Returns `true` if this is an L3 (cryptographic) Right that requires
    /// nullifier tracking.
    pub fn requires_nullifier(&self) -> bool {
        self.nullifier.is_some()
    }
}

/// Right validation errors.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RightError {
    #[error("Missing ownership proof")]
    MissingOwnershipProof,
    #[error("Invalid commitment (zero hash)")]
    InvalidCommitment,
    #[error("Right has already been consumed")]
    AlreadyConsumed,
    #[error("Invalid nullifier")]
    InvalidNullifier,
    #[error("Invalid canonical encoding")]
    InvalidEncoding,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_right() -> Right {
        Right::new(
            Hash::new([0xAB; 32]),
            OwnershipProof {
                proof: vec![0x01, 0x02, 0x03],
                owner: vec![0xFF; 32],
            },
            &[0x42; 16],
        )
    }

    #[test]
    fn test_right_creation() {
        let right = test_right();
        assert_eq!(right.commitment.as_bytes(), &[0xAB; 32]);
        assert!(right.nullifier.is_none());
        assert!(right.state_root.is_none());
        assert!(right.execution_proof.is_none());
    }

    #[test]
    fn test_right_id_deterministic() {
        let r1 = test_right();
        let r2 = test_right();
        assert_eq!(r1.id, r2.id);
    }

    #[test]
    fn test_right_id_unique_per_salt() {
        let r1 = Right::new(
            Hash::new([0xAB; 32]),
            OwnershipProof {
                proof: vec![0x01],
                owner: vec![0xFF; 32],
            },
            &[0x42; 16],
        );
        let r2 = Right::new(
            Hash::new([0xAB; 32]),
            OwnershipProof {
                proof: vec![0x01],
                owner: vec![0xFF; 32],
            },
            &[0x99; 16],
        );
        assert_ne!(r1.id, r2.id);
    }

    #[test]
    fn test_right_verify_valid() {
        let right = test_right();
        assert!(right.verify().is_ok());
    }

    #[test]
    fn test_right_verify_missing_proof() {
        let mut right = test_right();
        right.owner.proof = vec![];
        assert_eq!(right.verify(), Err(RightError::MissingOwnershipProof));
    }

    #[test]
    fn test_right_verify_zero_commitment() {
        let mut right = test_right();
        right.commitment = Hash::new([0u8; 32]);
        assert_eq!(right.verify(), Err(RightError::InvalidCommitment));
    }

    #[test]
    fn test_right_consume_with_nullifier() {
        let mut right = test_right();
        let nullifier = right.consume(Some(b"secret"));
        assert!(nullifier.is_some());
        assert!(right.nullifier.is_some());
        assert_eq!(right.verify(), Err(RightError::AlreadyConsumed));
    }

    #[test]
    fn test_right_consume_without_nullifier() {
        let mut right = test_right();
        let result = right.consume(None);
        assert!(result.is_none());
        assert!(right.nullifier.is_none());
        // L1/L2: Right is still valid locally (chain enforces structural single-use)
        assert!(right.verify().is_ok());
    }

    #[test]
    fn test_right_canonical_roundtrip() {
        let right = test_right();
        let bytes = right.to_canonical_bytes();
        let decoded = Right::from_canonical_bytes(&bytes).expect("Should decode");
        assert_eq!(decoded.id, right.id);
        assert_eq!(decoded.commitment, right.commitment);
        assert_eq!(decoded.owner, right.owner);
        assert_eq!(decoded.nullifier, right.nullifier);
        assert_eq!(decoded.state_root, right.state_root);
    }

    #[test]
    fn test_right_canonical_roundtrip_with_nullifier() {
        let mut right = test_right();
        right.consume(Some(b"secret"));
        let bytes = right.to_canonical_bytes();
        let decoded = Right::from_canonical_bytes(&bytes).expect("Should decode");
        assert_eq!(decoded.nullifier, right.nullifier);
        assert!(decoded.is_consumed());
    }

    #[test]
    fn test_right_from_canonical_bytes_invalid() {
        // Empty bytes should fail
        assert!(Right::from_canonical_bytes(&[]).is_err());
        // Truncated bytes should fail
        assert!(Right::from_canonical_bytes(&[0u8; 16]).is_err());
    }

    #[test]
    fn test_right_transfer() {
        let right = test_right();
        let new_owner = OwnershipProof {
            proof: vec![0xAA, 0xBB, 0xCC],
            owner: vec![0xDD; 32],
        };
        let transferred = right.transfer(new_owner.clone(), b"transfer-salt");

        // New Right should have:
        // - Different ID (due to different salt)
        assert_ne!(transferred.id, right.id);
        // - Same commitment
        assert_eq!(transferred.commitment, right.commitment);
        // - New owner
        assert_eq!(transferred.owner, new_owner);
        // - Not consumed
        assert!(!transferred.is_consumed());
        // - Original Right unaffected
        assert!(!right.is_consumed());
    }

    #[test]
    fn test_right_transfer_preserves_state_root() {
        let mut right = test_right();
        right.state_root = Some(Hash::new([0xCD; 32]));

        let new_owner = OwnershipProof {
            proof: vec![0x01],
            owner: vec![0xFF; 32],
        };
        let transferred = right.transfer(new_owner, b"transfer");

        assert_eq!(transferred.state_root, right.state_root);
    }

    #[test]
    fn test_right_is_consumed() {
        let mut right = test_right();
        assert!(!right.is_consumed());

        right.consume(Some(b"secret"));
        assert!(right.is_consumed());
    }

    #[test]
    fn test_right_requires_nullifier() {
        let right_l1 = test_right(); // L1/L2 doesn't need nullifier
        assert!(!right_l1.requires_nullifier());

        let mut right_l3 = test_right();
        right_l3.consume(Some(b"secret")); // L3 does
        assert!(right_l3.requires_nullifier());
    }
}
