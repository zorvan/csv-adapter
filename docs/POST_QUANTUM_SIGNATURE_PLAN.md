# Post-Quantum Signature Plan for CSV Sanads and Seals

## Executive Summary

This document outlines the migration path to post-quantum cryptography (PQC) for the CSV (Client-Side Validation) protocol's Sanad and seal mechanisms.

## Current State

### Existing Signature Support

- **Secp256k1**: Bitcoin/Ethereum compatible ECDSA
- **Ed25519**: Solana/Sui/Aptos compatible EdDSA
- Both are **NOT** post-quantum secure

### ZK Proof Infrastructure (STARK-based)

The protocol currently uses STARK proofs via:

- **SP1** (Succinct Labs RISC-V zkVM) - STARK-based
- **Risc0** (RISC-V zkVM) - STARK-based
- **Groth16** - SNARK-based (for compatibility)

STARKs are quantum-resistant (rely on hash functions, not elliptic curves).

## Post-Quantum Signature Algorithms

### Recommended: CRYSTALS-Dilithium (FIPS 204)

- **Standard**: NIST FIPS 204 (finalized August 2024)
- **Security Level**: 128-bit (NIST Level 2), 256-bit (NIST Level 5)
- **Signature Size**: ~2-4 KB
- **Public Key Size**: ~1-2 KB
- **Rust Implementation**: `pqcrypto-dilithium`

### Alternative: SPHINCS+ (FIPS 205)

- **Standard**: NIST FIPS 205 (stateless hash-based)
- **Security Level**: 128-bit, 256-bit
- **Signature Size**: ~8-50 KB (larger, slower)
- **Advantage**: Minimal assumptions (only hash functions)

### Hybrid Approach (Recommended for Transition)

Combine classical + PQC during transition period:

```
Signature = ECDSA(Secp256k1) + Dilithium
```

## Implementation Plan

### Phase 1: PQC Signature Extension (1-2 weeks)

1. **Add PQC Dependencies**

   ```toml
   [dependencies]
   pqcrypto-dilithium = "0.5"
   pqcrypto-traits = "0.3"
   ```

2. **Extend SignatureScheme Enum**

   ```rust
   pub enum SignatureScheme {
       Secp256k1,
       Ed25519,
       // Post-quantum schemes
       Dilithium2,      // NIST Level 2
       Dilithium3,      // NIST Level 3
       Dilithium5,      // NIST Level 5
       SphincsPlus128f, // Hash-based fallback
   }
   ```

3. **Implement PQC Signer Trait**

   ```rust
   pub trait PqcSigner {
       fn sign_pqc(&self, message: &[u8], scheme: PqcScheme) -> Result<Vec<u8>, SignError>;
       fn verify_pqc(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, SignError>;
   }
   ```

### Phase 2: Sanad PQC Binding (2-3 weeks)

1. **Extend OwnershipProof**

   ```rust
   pub struct OwnershipProof {
       pub proof: Vec<u8>,
       pub owner: Vec<u8>,
       pub scheme: Option<SignatureScheme>,
       // Post-quantum extension
       pub pqc_proof: Option<PqcProof>,
   }

   pub struct PqcProof {
       pub scheme: PqcScheme,
       pub signature: Vec<u8>,
       pub public_key: Vec<u8>,
   }
   ```

2. **PQC Commitment Integration**
   - Include PQC public key in Sanad genesis commitment
   - Support hybrid commitments (classical + PQC)

### Phase 3: Seal Protocol PQC Extension (2-3 weeks)

1. **Update SealProtocol Trait**

   ```rust
   pub trait SealProtocol {
       // Existing methods...

       /// Verify seal with PQC signature
       fn verify_pqc_seal(&self, seal: &Self::SealPoint, proof: &PqcProof) -> CoreResult<()>;

       /// Check if seal supports PQC
       fn has_pqc_capability(&self) -> bool;
   }
   ```

2. **Chain-Specific PQC Support**
   - **Bitcoin**: Taproot + PQC via OP_CHECKSIGADD (future soft fork)
   - **Ethereum**: Precompile contracts for Dilithium
   - **Sui/Aptos**: Move contracts for PQC verification
   - **Solana**: Native program for PQC signatures

### Phase 4: Migration Strategy (4-6 weeks)

1. **Dual-Signature Period**
   - Accept both classical and PQC signatures
   - Gradual migration of Sanads to PQC

2. **ZK Proof Integration**
   - STARK proofs naturally support PQC (hash-based)
   - Guest programs can verify Dilithium/SPHINCS+ in zkVM

3. **Timeline**

   ```
   Month 1-2: PQC signature implementation
   Month 3-4: Chain adapter updates
   Month 5-6: Testing and audit
   Month 7+: Production deployment
   ```

## Technical Considerations

### Signature Size Impact

- Dilithium signatures: ~2-4 KB vs 64-72 bytes (ECDSA/EdDSA)
- Bundle size increase: ~5-10x
- Mitigation: Use signature aggregation (MPC trees)

### Performance

- Dilithium signing: ~100-300 μs (faster than ECDSA)
- Dilithium verification: ~50-100 μs
- SPHINCS+: Slower but acceptable for high-security use cases

### ZK Circuit Compatibility

- STARKs verify hash functions efficiently
- Dilithium uses symmetric primitives (shake256)
- SPHINCS+ is entirely hash-based

## Recommended Rust Libraries

### Production-Ready

```toml
# NIST FIPS 204/205 implementations
pqcrypto-dilithium = "0.5"
pqcrypto-sphincsplus = "0.6"
pqcrypto-traits = "0.3"

# OR: Pure Rust implementation
# dilithium = "0.2" # Alternative implementation
```

### STARK Provers (Quantum-Resistant)

```toml
# Already in use - STARKs are PQC
sp1-sdk = "0.4"
risc0-zkvm = "1.0"
```

## Security Analysis

### Pre-Quantum Threats

- ✅ Secp256k1/Ed25519 provide 128-bit security
- ⚠️ Vulnerable to Shor's algorithm on quantum computers

### Post-Quantum Threats

- ✅ Dilithium: Lattice-based, NIST approved
- ✅ SPHINCS+: Hash-based, conservative security
- ✅ STARKs: Hash-based, no quantum speedup known

### Hybrid Security

- ✅ Defense in depth during transition
- ✅ Backward compatibility maintained
- ✅ Forward secrecy preserved

## Conclusion

The CSV protocol should adopt **CRYSTALS-Dilithium** as the primary PQC signature scheme, with **SPHINCS+** as a conservative fallback. The existing STARK-based ZK infrastructure is already quantum-resistant.

### Immediate Next Steps

1. Add `pqcrypto-dilithium` dependency to csv-keys
2. Extend `SignatureScheme` enum with PQC variants
3. Implement hybrid signature mode for transition period
4. Update ZK guest programs to verify PQC signatures

### Priority: HIGH

Quantum computers capable of breaking ECDSA may emerge within 10-15 years. Early adoption ensures long-term protocol security.
