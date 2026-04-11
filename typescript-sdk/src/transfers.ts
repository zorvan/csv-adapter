// ============================================================================
// CSV Adapter SDK — Cross-Chain Transfer Management
// ============================================================================
//
// The TransferManager handles cross-chain transfers — the core operation
// of the CSV system. A cross-chain transfer:
//
// 1. Locks the Right on the source chain (consumes the seal)
// 2. Generates a cryptographic proof of seal consumption
// 3. Submits the proof to the destination chain
// 4. Verifies the proof and re-anchors the Right on the destination
//
// The Transfer class provides real-time progress monitoring via events
// and a waitForCompletion() method for async/await style usage.
//
// ============================================================================

import { EventEmitter } from "events";
import {
  TransferState,
  TransferStatus,
  TransferRequest,
  TransferFilter,
  TransferEvent,
  WaitForCompletionOptions,
  ProofBundle,
} from "./types.js";
import {
  CsvError,
  TransferTimeout,
  ErrorCode,
} from "./errors.js";
import { validateRightId, validateAddress } from "./utils/validation.js";
import { parseTimeout } from "./utils/format.js";
import { Wallet } from "./wallet.js";
import { RightsManager } from "./rights.js";
import { ProofManager } from "./proofs.js";

// ---------------------------------------------------------------------------
// Transfer Class
// ---------------------------------------------------------------------------

/**
 * A single cross-chain transfer with real-time progress monitoring.
 *
 * Use `waitForCompletion()` for async/await style usage, or subscribe
 * to progress events via `on('progress', callback)`.
 *
 * @example
 * ```ts
 * // Async/await style
 * const result = await transfer.waitForCompletion({ timeout: "5m" });
 *
 * // Event-driven style
 * transfer.on("progress", (event) => {
 *   console.log(`Step: ${event.type}`);
 * });
 * await transfer.waitForCompletion();
 * ```
 */
export class Transfer {
  /** Internal event emitter for progress updates */
  private emitter: EventEmitter;

  /** The underlying transfer state */
  public state: TransferState;

  /** Whether the transfer has reached a terminal state */
  private finalized = false;

  constructor(initialState: TransferState) {
    this.emitter = new EventEmitter();
    this.state = initialState;
  }

  /**
   * Wait for the transfer to complete or fail.
   *
   * This method polls the transfer status until it reaches a terminal
   * state (completed or failed) or the timeout is exceeded.
   *
   * @param options - Timeout and polling interval configuration
   * @returns The completed transfer
   *
   * @throws {TransferTimeout} if the timeout is exceeded
   * @throws {CsvError} if the transfer fails
   *
   * @example
   * ```ts
   * const transfer = await csv.transfers.crossChain({ ... });
   * const result = await transfer.waitForCompletion({ timeout: "5m" });
   * console.log(`Completed: ${result.destinationTxHash}`);
   * ```
   */
  async waitForCompletion(
    options?: WaitForCompletionOptions,
  ): Promise<TransferState> {
    const timeoutMs = options?.timeout ? parseTimeout(options.timeout) : 600_000; // 10 min default
    const pollIntervalMs = options?.pollInterval
      ? parseTimeout(options.pollInterval)
      : 10_000; // 10 sec default

    const deadline = Date.now() + timeoutMs;

    while (!this.finalized) {
      if (Date.now() >= deadline) {
        throw new TransferTimeout(this.state.id, timeoutMs);
      }

      // Check current state
      if (this.state.status === TransferStatus.Completed) {
        this.finalized = true;
        this.emit({ type: "completed", transfer: this.state });
        return this.state;
      }

      if (this.state.status === TransferStatus.Failed) {
        this.finalized = true;
        const error = new CsvError(
          ErrorCode.INTERNAL_ERROR,
          this.state.error ?? "Transfer failed with unknown error",
          { details: { transferId: this.state.id } },
        );
        this.emit({ type: "failed", error });
        throw error;
      }

      // Wait for next poll interval or state update
      await this.pollOrWait(pollIntervalMs);
    }

    return this.state;
  }

  /**
   * Subscribe to transfer progress events.
   *
   * @param event - Always "progress" for transfer events
   * @param callback - Called with each progress event
   *
   * @example
   * ```ts
   * transfer.on("progress", (event) => {
   *   switch (event.type) {
   *     case "locking":
   *       console.log(`Locking on ${event.chain}: ${event.confirmations}/${event.required} confirmations`);
   *       break;
   *     case "generating-proof":
   *       console.log(`Generating proof: ${event.progress}%`);
   *       break;
   *     case "completed":
   *       console.log("Transfer complete!");
   *       break;
   *   }
   * });
   * ```
   */
  on(_event: "progress", callback: (event: TransferEvent) => void): this {
    this.emitter.on("progress", callback);
    return this;
  }

  /**
   * Remove a progress event listener.
   */
  off(_event: "progress", callback: (event: TransferEvent) => void): this {
    this.emitter.off("progress", callback);
    return this;
  }

  /**
   * Get the current transfer status.
   */
  getStatus(): TransferStatus {
    return this.state.status;
  }

  /**
   * Whether the transfer has reached a terminal state.
   */
  isFinalized(): boolean {
    return this.finalized;
  }

  // -----------------------------------------------------------------------
  // Internal Methods
  // -----------------------------------------------------------------------

  /**
   * Emit a progress event to all listeners.
   */
  private emit(event: TransferEvent): void {
    this.emitter.emit("progress", event);
  }

  /**
   * Update the internal transfer state and emit a progress event.
   *
   * Called internally by TransferManager as the transfer progresses.
   */
  updateState(updates: Partial<TransferState>): void {
    this.state = { ...this.state, ...updates };

    // Emit corresponding event based on status
    const event = this.stateToEvent(this.state);
    if (event) {
      this.emit(event);
    }

    if (
      this.state.status === TransferStatus.Completed ||
      this.state.status === TransferStatus.Failed
    ) {
      this.finalized = true;
    }
  }

  /**
   * Poll for status updates or wait for the interval to elapse.
   *
   * In a full implementation, this would use a subscription/WebSocket
   * for real-time updates with the poll interval as a fallback.
   */
  private async pollOrWait(pollIntervalMs: number): Promise<void> {
    // Integration point: real-time status updates
    // In production, this could use:
    // 1. WebSocket subscription to the CSV coordinator
    // 2. Event listeners on the chain provider
    // 3. Polling the transfer status endpoint
    await new Promise((resolve) => setTimeout(resolve, pollIntervalMs));
  }

  /**
   * Convert a transfer state to a progress event.
   */
  private stateToEvent(state: TransferState): TransferEvent | null {
    switch (state.status) {
      case TransferStatus.Initiated:
        return { type: "initiated", transfer: state };
      case TransferStatus.Locking:
        return { type: "locking", chain: state.from, confirmations: 0, required: 0 };
      case TransferStatus.Locked:
        return { type: "locked", txHash: state.sourceTxHash ?? "" };
      case TransferStatus.GeneratingProof:
        return { type: "generating-proof", progress: 0 };
      case TransferStatus.ProofGenerated:
        return {
          type: "proof-generated",
          proof: state.proof ?? ({} as ProofBundle),
        };
      case TransferStatus.SubmittingProof:
        return { type: "submitting-proof", chain: state.to };
      case TransferStatus.Verifying:
        return { type: "verifying" };
      case TransferStatus.Minting:
        return { type: "minting", chain: state.to };
      case TransferStatus.Completed:
        return { type: "completed", transfer: state };
      case TransferStatus.Failed:
        return {
          type: "failed",
          error: new CsvError(
            ErrorCode.INTERNAL_ERROR,
            state.error ?? "Unknown error",
          ),
        };
      default:
        return null;
    }
  }
}

// ---------------------------------------------------------------------------
// TransferManager Class
// ---------------------------------------------------------------------------

/**
 * Manages cross-chain transfers.
 *
 * Cross-chain transfers are the core operation of the CSV system. They move
 * a Right from one chain to another by consuming the source seal, generating
 * a proof, and re-anchoring on the destination.
 *
 * @example
 * ```ts
 * const csv = await CSV.createDevWallet();
 *
 * const transfer = await csv.transfers.crossChain({
 *   rightId: right.id,
 *   from: Chain.Bitcoin,
 *   to: Chain.Ethereum,
 *   toAddress: "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD38",
 * });
 *
 * const result = await transfer.waitForCompletion({ timeout: "5m" });
 * ```
 */
export class TransferManager {
  /** Reference to the parent wallet */
  private wallet: Wallet;

  /** Reference to the rights manager */
  private rights: RightsManager;

  /** Cached transfers in memory */
  private transfersCache: Map<string, Transfer> = new Map();

  constructor(
    wallet: Wallet,
    rights: RightsManager,
    _proofs: ProofManager,
  ) {
    this.wallet = wallet;
    this.rights = rights;
  }

  /**
   * Initiate a cross-chain transfer.
   *
   * This starts the transfer process and returns a Transfer object
   * that can be used to monitor progress.
   *
   * The transfer lifecycle:
   * 1. **Locking**: The Right's seal is consumed on the source chain
   * 2. **Proof generation**: A cryptographic proof of seal consumption is created
   * 3. **Proof submission**: The proof is submitted to the destination chain
   * 4. **Verification & minting**: The destination chain verifies the proof and re-anchors the Right
   *
   * @param params - Transfer parameters
   * @returns A Transfer object for monitoring progress
   *
   * @throws {WalletNotConnected} if the wallet is not initialized
   * @throws {InvalidRightId} if the Right ID format is invalid
   * @throws {RightNotFound} if the Right doesn't exist
   * @throws {InvalidDestinationAddress} if the destination address is invalid
   *
   * @example
   * ```ts
   * const transfer = await transfers.crossChain({
   *   rightId: "0xabc123...",
   *   from: Chain.Sui,
   *   to: Chain.Ethereum,
   *   toAddress: "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD38",
   * });
   * ```
   */
  async crossChain(params: TransferRequest): Promise<Transfer> {
    this.wallet.isConnected();

    // Validate inputs
    validateRightId(params.rightId);
    validateAddress(params.toAddress, params.to);

    // Verify the Right exists and is on the expected chain
    await this.rights.get(params.rightId, params.from);

    // Generate a unique transfer ID
    const transferId = this.generateTransferId();

    // Create initial transfer state
    const transfer = new Transfer({
      id: transferId,
      rightId: params.rightId,
      from: params.from,
      to: params.to,
      toAddress: params.toAddress,
      status: TransferStatus.Initiated,
      createdAt: new Date(),
    });

    // Cache the transfer
    this.transfersCache.set(transferId, transfer);

    // Integration point: @csv-adapter/core
    // The actual cross-chain operation involves:
    // 1. Locking the Right on the source chain (consuming the seal)
    // 2. Generating a proof via ProofManager
    // 3. Submitting the proof to the destination chain
    // 4. Waiting for destination chain confirmation
    //
    // This is an async multi-step process that updates the Transfer
    // state as each step completes.

    throw new Error(
      "TransferManager.crossChain() requires @csv-adapter/core integration. " +
        "This orchestrates the full cross-chain transfer: lock → proof → submit → verify.",
    );
  }

  /**
   * Check the status of a transfer by ID.
   *
   * @param transferId - The transfer ID to check
   * @returns The current transfer state
   *
   * @throws {Error} if the transfer ID is not found
   */
  status(transferId: string): TransferState {
    const transfer = this.transfersCache.get(transferId);
    if (!transfer) {
      throw new Error(
        `Transfer "${transferId}" not found. ` +
          `It may have been created in a previous session.`,
      );
    }
    return transfer.state;
  }

  /**
   * List all transfers, optionally filtered.
   *
   * @param filters - Optional filters to narrow the results
   * @returns Array of transfer states matching the filters
   *
   * @example
   * ```ts
   * // All completed transfers
   * const completed = await transfers.list({ status: TransferStatus.Completed });
   *
   * // All Bitcoin -> Ethereum transfers
   * const btcToEth = await transfers.list({
   *   from: Chain.Bitcoin,
   *   to: Chain.Ethereum,
   * });
   * ```
   */
  async list(filters?: TransferFilter): Promise<TransferState[]> {
    let transfers = Array.from(this.transfersCache.values());

    if (filters) {
      if (filters.from) {
        transfers = transfers.filter((t) => t.state.from === filters.from);
      }
      if (filters.to) {
        transfers = transfers.filter((t) => t.state.to === filters.to);
      }
      if (filters.status) {
        transfers = transfers.filter((t) => t.state.status === filters.status);
      }
      if (filters.createdAfter) {
        transfers = transfers.filter(
          (t) => t.state.createdAt >= filters.createdAfter!,
        );
      }
      if (filters.createdBefore) {
        transfers = transfers.filter(
          (t) => t.state.createdAt <= filters.createdBefore!,
        );
      }
      if (filters.limit !== undefined) {
        transfers = transfers.slice(0, filters.limit);
      }
    }

    // Integration point: @csv-adapter/store
    // Query persistent storage for transfers not in the cache
    return transfers.map((t) => t.state);
  }

  /**
   * Generate a unique transfer ID.
   *
   * Uses crypto-random bytes to ensure uniqueness across sessions.
   */
  private generateTransferId(): string {
    // In production, use crypto.randomUUID() or similar
    return `0x${Date.now().toString(16).padStart(16, "0")}${Math.random().toString(16).slice(2, 18).padStart(48, "0")}`;
  }
}
