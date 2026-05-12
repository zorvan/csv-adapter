//! Canonical Proof Validation Pipeline
//!
//! This module provides the ONLY allowed proof validation entrypoint for the CSV protocol.
//! All chain adapters MUST route through this pipeline to ensure consistent validation
//! ordering across all chains.
//!
//! ## Validation Order (Mandatory)
//!
//! The pipeline executes validation in exactly this order:
//! 1. structural validation
//! 2. domain validation
//! 3. inclusion proof validation
//! 4. zk proof validation
//! 5. finality validation
//! 6. replay validation
//! 7. seal registry validation
//! 8. transition legality validation
//! 9. signature validation
//! 10. acceptance decision
//!
//! No adapter may reorder or skip steps.

use alloc::vec::Vec;

use crate::error::{ProtocolError, Result};
use crate::hash::Hash;
use crate::proof::{FinalityProof, InclusionProof, ProofBundle};
use crate::protocol_version::ChainId;

/// Chain verifier trait that adapters must implement
///
/// Adapters provide chain-specific verification logic through this trait,
/// but the orchestration is handled by the canonical pipeline.
pub trait ChainVerifier {
    /// Verify inclusion proof for a transaction on this chain
    async fn verify_inclusion(
        &self,
        proof: &InclusionProof,
        expected_root: Hash,
    ) -> Result<bool>;

    /// Verify finality proof for a block on this chain
    async fn verify_finality(&self, proof: &FinalityProof) -> Result<bool>;

    /// Verify zero-knowledge proof (if applicable for this chain)
    async fn verify_zk(&self, proof: &[u8]) -> Result<bool>;
}

/// Validation step result
#[derive(Debug, Clone)]
pub struct ValidationStep {
    /// Step name
    pub name: &'static str,
    /// Whether this step passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall validation result
    pub accepted: bool,
    /// Individual step results
    pub steps: Vec<ValidationStep>,
    /// Final error if rejected
    pub error: Option<String>,
}

/// Validate a proof bundle through the canonical pipeline
///
/// This is the ONLY allowed proof validation entrypoint. All chain adapters
/// must route through this function to ensure consistent validation ordering.
///
/// # Arguments
///
/// * `bundle` - The proof bundle to validate
/// * `verifier` - Chain-specific verifier implementation
/// * `source_chain` - Source chain ID
/// * `destination_chain` - Destination chain ID
///
/// # Returns
///
/// Validation result indicating acceptance or rejection with step-by-step details
pub async fn validate_proof_bundle(
    bundle: &ProofBundle,
    verifier: &dyn ChainVerifier,
    source_chain: ChainId,
    destination_chain: ChainId,
) -> ValidationResult {
    let mut steps = Vec::with_capacity(10);

    // Step 1: Structural validation
    let step1 = validate_structural(bundle);
    steps.push(step1.clone());
    if !step1.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step1.error.unwrap_or_else(|| "Structural validation failed".to_string())),
        };
    }

    // Step 2: Domain validation
    let step2 = validate_domain(bundle, &source_chain, &destination_chain);
    steps.push(step2.clone());
    if !step2.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step2.error.unwrap_or_else(|| "Domain validation failed".to_string())),
        };
    }

    // Step 3: Inclusion proof validation
    let step3 = validate_inclusion_proof(bundle, verifier).await;
    steps.push(step3.clone());
    if !step3.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step3.error.unwrap_or_else(|| "Inclusion proof validation failed".to_string())),
        };
    }

    // Step 4: ZK proof validation
    let step4 = validate_zk_proof(bundle, verifier).await;
    steps.push(step4.clone());
    if !step4.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step4.error.unwrap_or_else(|| "ZK proof validation failed".to_string())),
        };
    }

    // Step 5: Finality validation
    let step5 = validate_finality(bundle, verifier).await;
    steps.push(step5.clone());
    if !step5.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step5.error.unwrap_or_else(|| "Finality validation failed".to_string())),
        };
    }

    // Step 6: Replay validation
    let step6 = validate_replay(bundle);
    steps.push(step6.clone());
    if !step6.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step6.error.unwrap_or_else(|| "Replay validation failed".to_string())),
        };
    }

    // Step 7: Seal registry validation
    let step7 = validate_seal_registry(bundle);
    steps.push(step7.clone());
    if !step7.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step7.error.unwrap_or_else(|| "Seal registry validation failed".to_string())),
        };
    }

    // Step 8: Transition legality validation
    let step8 = validate_transition_legality(bundle);
    steps.push(step8.clone());
    if !step8.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step8.error.unwrap_or_else(|| "Transition legality validation failed".to_string())),
        };
    }

    // Step 9: Signature validation
    let step9 = validate_signature(bundle);
    steps.push(step9.clone());
    if !step9.passed {
        return ValidationResult {
            accepted: false,
            steps,
            error: Some(step9.error.unwrap_or_else(|| "Signature validation failed".to_string())),
        };
    }

    // Step 10: Acceptance decision
    let step10 = ValidationStep {
        name: "acceptance_decision",
        passed: true,
        error: None,
    };
    steps.push(step10);

    ValidationResult {
        accepted: true,
        steps,
        error: None,
    }
}

/// Step 1: Structural validation
fn validate_structural(bundle: &ProofBundle) -> ValidationStep {
    // Check that all required fields are present
    if bundle.inclusion_proof.proof_bytes.is_empty() {
        return ValidationStep {
            name: "structural_validation",
            passed: false,
            error: Some("Inclusion proof bytes are empty".to_string()),
        };
    }

    if bundle.finality_proof.proof_bytes.is_empty() {
        return ValidationStep {
            name: "structural_validation",
            passed: false,
            error: Some("Finality proof bytes are empty".to_string()),
        };
    }

    ValidationStep {
        name: "structural_validation",
        passed: true,
        error: None,
    }
}

/// Step 2: Domain validation
fn validate_domain(
    bundle: &ProofBundle,
    source_chain: &ChainId,
    destination_chain: &ChainId,
) -> ValidationStep {
    // Verify that the proof is for the correct source/destination chains
    // This is a placeholder - actual implementation would check bundle metadata
    ValidationStep {
        name: "domain_validation",
        passed: true,
        error: None,
    }
}

/// Step 3: Inclusion proof validation
async fn validate_inclusion_proof(
    bundle: &ProofBundle,
    verifier: &dyn ChainVerifier,
) -> ValidationStep {
    match verifier
        .verify_inclusion(&bundle.inclusion_proof, bundle.block_hash)
        .await
    {
        Ok(true) => ValidationStep {
            name: "inclusion_proof_validation",
            passed: true,
            error: None,
        },
        Ok(false) => ValidationStep {
            name: "inclusion_proof_validation",
            passed: false,
            error: Some("Inclusion proof verification failed".to_string()),
        },
        Err(e) => ValidationStep {
            name: "inclusion_proof_validation",
            passed: false,
            error: Some(format!("Inclusion proof verification error: {}", e)),
        },
    }
}

/// Step 4: ZK proof validation
async fn validate_zk_proof(
    bundle: &ProofBundle,
    verifier: &dyn ChainVerifier,
) -> ValidationStep {
    if bundle.zk_proof.is_empty() {
        // ZK proof may be optional for some chains
        return ValidationStep {
            name: "zk_proof_validation",
            passed: true,
            error: None,
        };
    }

    match verifier.verify_zk(&bundle.zk_proof).await {
        Ok(true) => ValidationStep {
            name: "zk_proof_validation",
            passed: true,
            error: None,
        },
        Ok(false) => ValidationStep {
            name: "zk_proof_validation",
            passed: false,
            error: Some("ZK proof verification failed".to_string()),
        },
        Err(e) => ValidationStep {
            name: "zk_proof_validation",
            passed: false,
            error: Some(format!("ZK proof verification error: {}", e)),
        },
    }
}

/// Step 5: Finality validation
async fn validate_finality(
    bundle: &ProofBundle,
    verifier: &dyn ChainVerifier,
) -> ValidationStep {
    match verifier.verify_finality(&bundle.finality_proof).await {
        Ok(true) => ValidationStep {
            name: "finality_validation",
            passed: true,
            error: None,
        },
        Ok(false) => ValidationStep {
            name: "finality_validation",
            passed: false,
            error: Some("Finality proof verification failed".to_string()),
        },
        Err(e) => ValidationStep {
            name: "finality_validation",
            passed: false,
            error: Some(format!("Finality proof verification error: {}", e)),
        },
    }
}

/// Step 6: Replay validation
fn validate_replay(bundle: &ProofBundle) -> ValidationStep {
    // Placeholder - would check replay registry
    // This will be implemented when replay_registry.rs is created
    ValidationStep {
        name: "replay_validation",
        passed: true,
        error: None,
    }
}

/// Step 7: Seal registry validation
fn validate_seal_registry(bundle: &ProofBundle) -> ValidationStep {
    // Placeholder - would check seal registry
    // This will be implemented when replay_registry.rs is created
    ValidationStep {
        name: "seal_registry_validation",
        passed: true,
        error: None,
    }
}

/// Step 8: Transition legality validation
fn validate_transition_legality(bundle: &ProofBundle) -> ValidationStep {
    // Placeholder - would verify transition is legal per protocol rules
    ValidationStep {
        name: "transition_legality_validation",
        passed: true,
        error: None,
    }
}

/// Step 9: Signature validation
fn validate_signature(bundle: &ProofBundle) -> ValidationStep {
    // Placeholder - would verify signatures
    ValidationStep {
        name: "signature_validation",
        passed: true,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockVerifier;

    #[async_trait::async_trait]
    impl ChainVerifier for MockVerifier {
        async fn verify_inclusion(
            &self,
            _proof: &InclusionProof,
            _expected_root: Hash,
        ) -> Result<bool> {
            Ok(true)
        }

        async fn verify_finality(&self, _proof: &FinalityProof) -> Result<bool> {
            Ok(true)
        }

        async fn verify_zk(&self, _proof: &[u8]) -> Result<bool> {
            Ok(true)
        }
    }

    #[test]
    fn test_validation_step_creation() {
        let step = ValidationStep {
            name: "test_step",
            passed: true,
            error: None,
        };
        assert!(step.passed);
        assert_eq!(step.name, "test_step");
    }

    #[tokio::test]
    async fn test_validate_proof_bundle_success() {
        let bundle = ProofBundle {
            inclusion_proof: InclusionProof::new(vec![1, 2, 3], Hash::new([1u8; 32]), 0),
            finality_proof: FinalityProof::new(vec![4, 5, 6]),
            zk_proof: vec![],
            block_hash: Hash::new([2u8; 32]),
        };

        let verifier = MockVerifier;
        let result = validate_proof_bundle(
            &bundle,
            &verifier,
            ChainId::new("bitcoin"),
            ChainId::new("ethereum"),
        )
        .await;

        assert!(result.accepted);
        assert_eq!(result.steps.len(), 10);
        assert!(result.error.is_none());
    }
}
