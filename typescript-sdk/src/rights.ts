// ============================================================================
// CSV Adapter SDK — Right Lifecycle Management
// ============================================================================
//
// The RightsManager handles the full lifecycle of Rights:
// creation, retrieval, listing, transfer (same-chain), and burning.
//
// Key concepts:
// - A Right is created on a specific chain and anchored to its single-use seal.
// - The right_id is globally unique and derived from the commitment data.
// - To move a Right to another chain, use TransferManager.crossChain().
// - Burning a Right permanently consumes its seal — it can never be recreated.
//
// ============================================================================

import {
  Chain,
  Right,
  RightId,
  CreateRightRequest,
  CreateRightResponse,
  RightFilter,
} from "./types.js";
import { RightNotFound, RightOnWrongChain } from "./errors.js";
import { validateCommitmentData, validateRightId } from "./utils/validation.js";
import { Wallet } from "./wallet.js";

// ---------------------------------------------------------------------------
// RightsManager Class
// ---------------------------------------------------------------------------

/**
 * Manages the lifecycle of CSV Rights.
 *
 * Rights are the core primitive in the CSV system — transferrable claims
 * anchored to a chain's single-use seal.
 *
 * @example
 * ```ts
 * const csv = await CSV.createDevWallet();
 *
 * // Create a new Right
 * const right = await csv.rights.create({
 *   chain: Chain.Bitcoin,
 *   commitmentData: { tokenUri: "ipfs://..." },
 * });
 *
 * // List all Rights
 * const allRights = await csv.rights.list();
 *
 * // Get a specific Right
 * const fetched = await csv.rights.get(right.id);
 * ```
 */
export class RightsManager {
  /** Reference to the parent wallet (for address derivation) */
  private wallet: Wallet;

  /**
   * Cached Rights in memory.
   *
   * Rights exist in client state — they are not "on-chain" in the sense
   * of a contract. The chain only tracks seal consumption.
   */
  private rightsCache: Map<RightId, Right> = new Map();

  constructor(wallet: Wallet) {
    this.wallet = wallet;
  }

  /**
   * Create a new Right anchored to a specific chain.
   *
   * This is the entry point for creating any transferrable claim.
   * The commitment data is hashed to create the commitment hash that
   * conceals the actual transfer details.
   *
   * @param params - Creation parameters including chain and commitment data
   * @returns The created Right and its creation transaction hash
   *
   * @throws {WalletNotConnected} if the wallet is not initialized
   * @throws {CsvError} if creation fails (insufficient funds, RPC error, etc.)
   *
   * @example
   * ```ts
   * const right = await rightsManager.create({
   *   chain: Chain.Sui,
   *   commitmentData: {
   *     name: "My NFT",
   *     image: "ipfs://QmXyz...",
   *   },
   * });
   * ```
   */
  async create(params: CreateRightRequest): Promise<CreateRightResponse> {
    this.wallet.isConnected();
    validateCommitmentData(params.commitmentData);

    // Integration point: @csv-adapter/core
    // Creates the Right by:
    // 1. Hashing the commitment data to produce the commitment hash
    // 2. Deriving the right_id from the commitment
    // 3. Anchoring the Right to the chain's single-use seal
    // 4. Broadcasting the creation transaction

    throw new Error(
      "RightsManager.create() requires @csv-adapter/core integration. " +
        "This method calls the core library to anchor the Right to the chain.",
    );
  }

  /**
   * Fetch a Right by its ID.
   *
   * Rights exist in client state. This method checks the local cache first,
   * then queries the chain if needed.
   *
   * @param rightId - The 32-byte hex Right ID
   * @param expectedChain - Optionally verify the Right is on this chain
   * @returns The Right
   *
   * @throws {InvalidRightId} if the format is invalid
   * @throws {RightNotFound} if the Right doesn't exist
   * @throws {RightOnWrongChain} if the Right is not on the expected chain
   */
  async get(rightId: RightId, expectedChain?: Chain): Promise<Right> {
    validateRightId(rightId);

    // Check local cache first
    const cached = this.rightsCache.get(rightId);
    if (cached) {
      if (expectedChain && cached.chain !== expectedChain) {
        throw new RightOnWrongChain({
          rightId,
          expected: expectedChain,
          actual: cached.chain,
        });
      }
      return cached;
    }

    // Integration point: @csv-adapter/core
    // Queries the chain for the Right's seal state to verify existence
    // and reconstruct the Right object.
    throw new RightNotFound(rightId);
  }

  /**
   * List all Rights in the wallet, optionally filtered.
   *
   * @param filters - Optional filters to narrow the results
   * @returns Array of Rights matching the filters
   *
   * @example
   * ```ts
   * // All Rights
   * const all = await rightsManager.list();
   *
   * // Only Bitcoin Rights
   * const btcRights = await rightsManager.list({ chain: Chain.Bitcoin });
   * ```
   */
  async list(filters?: RightFilter): Promise<Right[]> {
    // Return from cache if available
    let rights = Array.from(this.rightsCache.values());

    // Apply filters
    if (filters) {
      if (filters.chain) {
        rights = rights.filter((r) => r.chain === filters.chain);
      }
      if (filters.ownerAddress) {
        rights = rights.filter((r) => r.ownerAddress === filters.ownerAddress);
      }
      if (filters.createdAfter) {
        rights = rights.filter((r) => r.createdAt >= filters.createdAfter!);
      }
      if (filters.createdBefore) {
        rights = rights.filter((r) => r.createdAt <= filters.createdBefore!);
      }
    }

    // Integration point: @csv-adapter/store
    // If cache is incomplete, query persistent storage for additional Rights
    // The store layer handles SQLite, RocksDB, or in-memory persistence.

    return rights;
  }

  /**
   * Transfer a Right to a new owner on the same chain.
   *
   * This is a same-chain transfer — the Right stays on the same chain
   * but changes ownership. For cross-chain transfers, use TransferManager.
   *
   * @param rightId - The Right to transfer
   * @param toAddress - The new owner's address on the same chain
   * @returns The updated Right
   *
   * @throws {RightNotFound} if the Right doesn't exist
   * @throws {RightOnWrongChain} if the Right is not on the expected chain
   */
  async transfer(rightId: RightId, _toAddress: string): Promise<Right> {
    await this.get(rightId);

    // Integration point: @csv-adapter/core
    // Same-chain transfer:
    // 1. Consume the current seal
    // 2. Create a new seal for the new owner
    // 3. Broadcast the transfer transaction

    throw new Error(
      "RightsManager.transfer() requires @csv-adapter/core integration. " +
        "For cross-chain transfers, use TransferManager.crossChain() instead.",
    );
  }

  /**
   * Burn a Right permanently.
   *
   * Burning consumes the seal with no new seal created — the Right
   * can never be recreated or transferred. This is useful for
   * redemption scenarios (e.g., burning a ticket Right at event entry).
   *
   * @param rightId - The Right to burn
   * @returns The burned Right (for confirmation)
   *
   * @throws {RightNotFound} if the Right doesn't exist
   *
   * @example
   * ```ts
   * // Burn a ticket Right at event entry
   * await rightsManager.burn(ticketRight.id);
   * console.log("Ticket consumed — cannot be reused");
   * ```
   */
  async burn(rightId: RightId): Promise<Right> {
    await this.get(rightId);

    // Integration point: @csv-adapter/core
    // Burning:
    // 1. Consume the seal without creating a new one
    // 2. Broadcast the burn transaction
    // 3. Remove from cache

    throw new Error(
      "RightsManager.burn() requires @csv-adapter/core integration.",
    );
  }

  /**
   * Update the internal cache with a Right.
   *
   * Called internally after successful create/transfer operations.
   */
  cacheRight(right: Right): void {
    this.rightsCache.set(right.id, right);
  }

  /**
   * Remove a Right from the cache.
   *
   * Called internally after burn operations.
   */
  uncacheRight(rightId: RightId): void {
    this.rightsCache.delete(rightId);
  }

  /**
   * Clear the entire cache.
   *
   * Useful when switching wallets or for testing.
   */
  clearCache(): void {
    this.rightsCache.clear();
  }
}
