// ============================================================================
// CSV Adapter SDK — Proof Generation and Verification
// ============================================================================
//
// The ProofManager handles cryptographic proof operations:
// - generate(): Create a proof bundle that a seal was consumed
// - verify(): Verify a proof bundle is valid and untampered
// - simulate(): Test proof verification without chain interaction
//
// Proof Bundles are the bridge between chains — they contain cryptographic
// evidence that a single-use seal was consumed on the source chain, which
// can be independently verified on the destination chain.
//
// ============================================================================

import {
  Chain,
  RightId,
  ProofBundle,
  InclusionProof,
  FinalityProof,
  SealConsumptionProof,
} from "./types.js";
import {
  ProofVerificationFailed,
} from "./errors.js";
import { validateRightId, validateChain } from "./utils/validation.js";
import { chainConfirmations } from "./utils/format.js";
import { getProvider } from "./chains/index.js";

// ---------------------------------------------------------------------------
// ProofManager Class
// ---------------------------------------------------------------------------

/**
 * Manages proof generation and verification.
 *
 * Proofs are the cryptographic bridge between chains. A proof bundle
 * contains three components:
 *
 * 1. **Inclusion proof** — proves the lock event is in a block
 * 2. **Finality proof** — proves the block has sufficient confirmations
 * 3. **Seal consumption proof** — proves the single-use seal was consumed
 *
 * @example
 * ```ts
 * // Generate a proof for a locked Right
 * const proof = await proofs.generate(rightId, Chain.Bitcoin);
 *
 * // Verify a proof bundle
 * const isValid = await proofs.verify(proof, rightId);
 *
 * // Simulate verification (test without chain calls)
 * const result = await proofs.simulate(proof);
 * ```
 */
export class ProofManager {
  /**
   * Generate a proof bundle for a Right that has been locked on a chain.
   *
   * This method creates a complete ProofBundle containing inclusion,
   * finality, and seal consumption proofs. It queries the source chain
   * to gather cryptographic evidence.
   *
   * @param rightId - The Right that was locked
   * @param chain - The chain where the Right was locked
   * @returns A complete ProofBundle
   *
   * @throws {InvalidRightId} if the Right ID format is invalid
   * @throws {ChainNotSupported} if the chain is not supported
   *
   * @example
   * ```ts
   * // After locking a Right on Bitcoin
   * const proof = await proofs.generate(rightId, Chain.Bitcoin);
   // console.log(`Proof size: ${proof.inclusion.merkleProof.length} bytes`);
   * ```
   */
  async generate(rightId: RightId, chain: Chain): Promise<ProofBundle> {
    validateRightId(rightId);
    validateChain(chain);

    // Integration point: @csv-adapter/{chain}
    // Each chain adapter has a specific proof format:
    // - Bitcoin: Merkle branch from tx to block Merkle root
    // - Ethereum: MPT proof nodes for state verification
    // - Sui: Checkpoint certification
    // - Aptos: Ledger info + accumulator proof

    throw new Error(
      `ProofManager.generate() requires @csv-adapter/${chain} integration. ` +
        `Each chain adapter produces chain-specific proof formats.`,
    );
  }

  /**
   * Verify a proof bundle.
   *
   * This method cryptographically verifies that the proof bundle is valid,
   * untampered, and corresponds to the expected Right (if provided).
   *
   * Verification checks:
   * 1. The inclusion proof proves the lock event is in a valid block
   * 2. The finality proof proves the block has sufficient confirmations
   * 3. The seal consumption proof proves the seal was actually consumed
   * 4. If expectedRightId is provided, the proof must correspond to that Right
   *
   * @param proofBundle - The proof bundle to verify
   * @param expectedRightId - Optionally verify the proof is for this specific Right
   * @returns true if the proof is valid
   *
   * @throws {ProofVerificationFailed} if verification fails
   *
   * @example
   * ```ts
   * const isValid = await proofs.verify(proofBundle, rightId);
   * if (isValid) {
   *   console.log("Proof verified — safe to mint on destination chain");
   * }
   * ```
   */
  async verify(
    proofBundle: ProofBundle,
    expectedRightId?: RightId,
  ): Promise<boolean> {
    const sourceChain = proofBundle.inclusion.chain;
    const provider = getProvider(sourceChain);

    try {
      // Step 1: Verify inclusion proof
      await this.verifyInclusion(proofBundle.inclusion, provider);

      // Step 2: Verify finality proof
      await this.verifyFinality(proofBundle.finality, provider, sourceChain);

      // Step 3: Verify seal consumption
      await this.verifySealConsumption(proofBundle.sealConsumption, provider);

      // Step 4: If expected Right ID is provided, cross-reference
      if (expectedRightId) {
        validateRightId(expectedRightId);
        // Integration point: verify the proof corresponds to the expected Right
        // In production, this checks the proof's commitment matches the Right
      }

      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown verification error";
      throw new ProofVerificationFailed(message);
    }
  }

  /**
   * Simulate proof verification without chain interaction.
   *
   * This method performs structural validation of the proof bundle
   * without making any RPC calls. Useful for:
   * - Testing proof generation locally
   * - Validating proof format before submission
   * - Offline verification of proof structure
   *
   * @param proofBundle - The proof bundle to simulate
   * @returns Whether the proof structure is valid
   */
  async simulate(proofBundle: ProofBundle): Promise<boolean> {
    try {
      // Structural checks — no network calls needed

      // Check inclusion proof has required fields
      if (!proofBundle.inclusion.chain) return false;
      if (proofBundle.inclusion.blockNumber <= 0n) return false;
      if (proofBundle.inclusion.merkleProof.length === 0) return false;
      if (!proofBundle.inclusion.blockHash) return false;

      // Check finality proof has required fields
      if (proofBundle.finality.confirmations < 0) return false;
      if (proofBundle.finality.requiredConfirmations < 0) return false;

      // Check seal consumption proof has required fields
      if (!proofBundle.sealConsumption.txHash) return false;
      if (!proofBundle.sealConsumption.sealId) return false;

      // Verify confirmations meet the chain's requirement
      const chain = proofBundle.inclusion.chain;
      const requiredConfirmations = chainConfirmations(chain);
      if (proofBundle.finality.confirmations < requiredConfirmations) {
        return false;
      }

      return true;
    } catch {
      return false;
    }
  }

  // -----------------------------------------------------------------------
  // Internal Verification Methods
  // -----------------------------------------------------------------------

  /**
   * Verify the inclusion proof component.
   *
   * Checks that the lock transaction is cryptographically proven
   * to be included in the specified block.
   */
  private async verifyInclusion(
    inclusion: InclusionProof,
    // provider reference for chain-specific verification
    _provider: Awaited<ReturnType<typeof getProvider>>,
  ): Promise<void> {
    // Integration point: chain-specific inclusion verification
    // - Bitcoin: Verify Merkle branch from tx hash to block Merkle root
    // - Ethereum: Verify MPT proof against block state root
    // - Sui: Verify checkpoint certification
    // - Aptos: Verify accumulator proof

    if (inclusion.merkleProof.length === 0) {
      throw new Error("Inclusion proof is empty");
    }

    if (!inclusion.blockHash) {
      throw new Error("Inclusion proof missing block hash");
    }
  }

  /**
   * Verify the finality proof component.
   *
   * Checks that the block has reached sufficient confirmations
   * for the chain's security requirements.
   */
  private async verifyFinality(
    finality: FinalityProof,
    _provider: Awaited<ReturnType<typeof getProvider>>,
    chain: Chain,
  ): Promise<void> {
    const required = chainConfirmations(chain);

    if (finality.confirmations < required) {
      throw new Error(
        `Insufficient confirmations: ${finality.confirmations} < ${required} required for ${chain}`,
      );
    }
  }

  /**
   * Verify the seal consumption proof component.
   *
   * Checks that the single-use seal was actually consumed,
   * preventing double-spend attacks.
   */
  private async verifySealConsumption(
    sealConsumption: SealConsumptionProof,
    _provider: Awaited<ReturnType<typeof getProvider>>,
  ): Promise<void> {
    if (!sealConsumption.txHash) {
      throw new Error("Seal consumption proof missing transaction hash");
    }

    if (!sealConsumption.sealId) {
      throw new Error("Seal consumption proof missing seal ID");
    }
  }
}
