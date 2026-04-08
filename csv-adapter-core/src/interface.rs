// Define core AnchorLayer interface for blockchain adapters
pub trait AnchorLayer {
    // Associated types for blockchain-specific references and proofs
    type SealRef: std::fmt::Debug + Clone + Eq + PartialEq;
    type AnchorRef: std::fmt::Debug + Clone + Eq + PartialEq;
    type InclusionProof: std::fmt::Debug + Clone;
    type FinalityProof: std::fmt::Debug + Clone;

    // Core methods required for all blockchain adapters
    fn publish(
        &self,
        commitment: &[u8],
        seal: Self::SealRef,
    ) -> Result<Self::AnchorRef, crate::error::AdapterError>;

    fn verify_inclusion(
        &self,
        anchor: Self::AnchorRef,
    ) -> Result<Self::InclusionProof, crate::error::AdapterError>;

    fn verify_finality(
        &self,
        anchor: Self::AnchorRef,
    ) -> Result<Self::FinalityProof, crate::error::AdapterError>;

    fn enforce_seal(&self, seal: Self::SealRef) -> Result<(), crate::error::AdapterError>;

    fn create_seal(&self, value: Option<u64>) -> Result<Self::SealRef, crate::error::AdapterError>;

    fn hash_commitment(
        &self,
        contract_id: &[u8],
        previous_commitment: &[u8],
        transition_payload_hash: &[u8],
        seal_ref: &Self::SealRef,
    ) -> Hash;

    fn build_proof_bundle(
        &self,
        anchor: Self::AnchorRef,
        transition_dag: &crate::dag::DAGSegment,
    ) -> Result<crate::proof::ProofBundle, crate::error::AdapterError>;

    fn rollback(&self, anchor: Self::AnchorRef) -> Result<(), crate::error::AdapterError>;

    fn domain_separator(&self) -> [u8; 32];

    fn signature_scheme(&self) -> csv_adapter_core::SignatureScheme;
}