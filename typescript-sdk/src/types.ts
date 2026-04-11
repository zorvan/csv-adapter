// ============================================================================
// CSV Adapter SDK — Core Type Definitions
// ============================================================================
//
// This module defines every TypeScript type used across the SDK.
//
// Key concepts:
// - **Right**: A transferrable claim anchored to a chain's single-use seal.
//   A Right exists on exactly one chain at a time.
// - **Seal**: The chain-specific mechanism that enforces single-use
//   (UTXO on Bitcoin, object deletion on Sui, etc.).
// - **Commitment**: A hash that hides transfer details. Only the seal
//   consumption and proof verification are on-chain.
// - **Proof Bundle**: Cryptographic evidence that a seal was consumed
//   on the source chain, verifiable on the destination chain.
//
// ============================================================================

import { z } from "zod";

// ---------------------------------------------------------------------------
// Chain
// ---------------------------------------------------------------------------

/**
 * Supported blockchain networks.
 *
 * The SDK provides a chain-agnostic API — the same methods work regardless
 * of which chain a Right is anchored to.
 */
export enum Chain {
  Bitcoin = "bitcoin",
  Ethereum = "ethereum",
  Sui = "sui",
  Aptos = "aptos",
}

/** Type-level union of all chain string values */
export type ChainId = (typeof Chain)[keyof typeof Chain];

/** Zod schema for validating chain values at runtime */
export const ChainSchema = z.nativeEnum(Chain);

// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

/**
 * Network identifier — distinguishes mainnet from test/dev networks.
 */
export enum Network {
  Mainnet = "mainnet",
  Testnet = "testnet",
  Devnet = "devnet",
  Regtest = "regtest",
}

export type NetworkId = (typeof Network)[keyof typeof Network];

// ---------------------------------------------------------------------------
// Right
// ---------------------------------------------------------------------------

/**
 * A Right is a transferrable claim anchored to a chain's single-use seal.
 *
 * Invariants:
 * - A Right exists on exactly one chain at a time.
 * - `right_id` is globally unique and derived from the committed data.
 * - To transfer a Right, the seal on the source chain must be consumed.
 * - The invariant `sum(minted) - sum(burned) = 1` always holds.
 */
export interface Right {
  /** Globally unique identifier (32-byte hex, e.g. "0xabc123...") */
  id: RightId;

  /** Commitment hash — conceals the actual data transferred */
  commitment: string;

  /** Chain that currently enforces this Right's single-use seal */
  chain: Chain;

  /** Address that currently owns this Right on `chain` */
  ownerAddress: string;

  /** When the Right was created */
  createdAt: Date;

  /** When the Right was last transferred (undefined if never moved) */
  transferredAt?: Date;

  /** Optional metadata attached by the application layer */
  metadata?: Record<string, unknown>;
}

/** A 32-byte hex string identifying a Right (e.g. "0x" + 64 hex chars) */
export type RightId = `0x${string}`;

// ---------------------------------------------------------------------------
// Transfer
// ---------------------------------------------------------------------------

/**
 * Status of a cross-chain transfer.
 *
 * Lifecycle:
 * initiated → locking → locked → generating-proof → proof-generated
 *   → submitting-proof → verifying → minting → completed
 *
 * Any state can transition to `failed` on error.
 */
export enum TransferStatus {
  Initiated = "initiated",
  Locking = "locking",
  Locked = "locked",
  GeneratingProof = "generating-proof",
  ProofGenerated = "proof-generated",
  SubmittingProof = "submitting-proof",
  Verifying = "verifying",
  Minting = "minting",
  Completed = "completed",
  Failed = "failed",
}

/**
 * A cross-chain transfer moves a Right from one chain to another.
 *
 * The transfer consumes the seal on the source chain, generates a
 * cryptographic proof, and re-anchors the Right on the destination chain.
 */
export interface TransferState {
  /** Unique transfer identifier */
  id: string;

  /** Right being transferred */
  rightId: RightId;

  /** Source chain */
  from: Chain;

  /** Destination chain */
  to: Chain;

  /** Destination owner address on the `to` chain */
  toAddress: string;

  /** Current status in the transfer lifecycle */
  status: TransferStatus;

  /** Proof bundle generated during the transfer (available after proof generation) */
  proof?: ProofBundle;

  /** When the transfer was initiated */
  createdAt: Date;

  /** When the transfer completed (undefined if not yet completed) */
  completedAt?: Date;

  /** Source chain transaction hash (available after locking) */
  sourceTxHash?: string;

  /** Destination chain transaction hash (available after minting) */
  destinationTxHash?: string;

  /** Human-readable error if status is Failed */
  error?: string;
}

/** @deprecated Use TransferState instead. Kept for backward compatibility. */
export type Transfer = TransferState;

// ---------------------------------------------------------------------------
// Proof Bundle
// ---------------------------------------------------------------------------

/**
 * A cryptographic proof that a seal was consumed on the source chain.
 *
 * A ProofBundle contains three components:
 * 1. **Inclusion proof** — proves the lock event is in a block/ledger.
 * 2. **Finality proof** — proves the block has reached sufficient confirmations.
 * 3. **Seal consumption proof** — proves the single-use seal was consumed.
 */
export interface ProofBundle {
  /** Proves the lock event is included in a block */
  inclusion: InclusionProof;

  /** Proves the block has reached finality */
  finality: FinalityProof;

  /** Proves the single-use seal was consumed */
  sealConsumption: SealConsumptionProof;
}

/**
 * Inclusion proof — cryptographic evidence that an event is in a block.
 *
 * Format varies by chain:
 * - Bitcoin: Merkle branch
 * - Ethereum: MPT (Merkle Patricia Trie) proof
 * - Sui: Checkpoint certification
 * - Aptos: Ledger info proof
 */
export interface InclusionProof {
  /** Source chain */
  chain: Chain;

  /** Block/checkpoint number */
  blockNumber: bigint;

  /** Transaction index within the block */
  txIndex: number;

  /** Merkle branch or chain-specific inclusion data */
  merkleProof: Uint8Array;

  /** Block hash */
  blockHash: string;
}

/**
 * Finality proof — evidence that the block has sufficient confirmations.
 */
export interface FinalityProof {
  /** Number of confirmations */
  confirmations: number;

  /** Required confirmations for this chain/network */
  requiredConfirmations: number;

  /** Tip block number at time of proof generation */
  tipBlockNumber: bigint;

  /** Chain-specific finality data (e.g. checkpoint epoch for Sui) */
  chainData?: Record<string, unknown>;
}

/**
 * Seal consumption proof — evidence that the single-use seal was consumed.
 */
export interface SealConsumptionProof {
  /** Transaction hash that consumed the seal */
  txHash: string;

  /** Seal identifier (UTXO outpoint, object ID, etc.) */
  sealId: string;

  /** Chain-specific seal consumption data */
  chainData?: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Wallet
// ---------------------------------------------------------------------------

/**
 * Wallet state — holds addresses and balances across chains.
 */
export interface WalletState {
  /** Whether the wallet is connected */
  connected: boolean;

  /** Addresses derived from the wallet on each chain */
  addresses: Partial<Record<Chain, string>>;

  /** Balances on each chain (raw amounts in smallest unit) */
  balances: Partial<Record<Chain, WalletBalance>>;
}

/** Balance information for a single chain */
export interface WalletBalance {
  /** Raw balance in the chain's smallest unit (satoshis, wei, etc.) */
  amount: bigint;

  /** Formatted balance as a human-readable string */
  formatted: string;

  /** Currency symbol (BTC, ETH, SUI, APT) */
  symbol: string;

  /** Number of decimals for the currency */
  decimals: number;

  /** Estimated USD value (if available) */
  usdValue?: number;
}

/**
 * Wallet initialization options.
 */
export interface WalletOptions {
  /** Network to connect to (default: mainnet) */
  network?: Network;

  /** Custom RPC URLs per chain */
  rpcUrls?: Partial<Record<Chain, string>>;

  /** BIP-44 derivation path override */
  derivationPath?: string;
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/**
 * Parameters for creating a new Right.
 */
export interface CreateRightRequest {
  /** Chain that will enforce this Right's single-use seal */
  chain: Chain;

  /** Data to commit (will be hashed into the commitment) */
  commitmentData: Record<string, unknown>;

  /** Owner address on the chain (default: wallet's own address) */
  ownerAddress?: string;

  /** Optional application-layer metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Response from creating a Right.
 */
export interface CreateRightResponse {
  /** The newly created Right */
  right: Right;

  /** Transaction hash that created the Right */
  transactionHash: string;
}

/**
 * Parameters for a cross-chain transfer.
 */
export interface TransferRequest {
  /** Right ID to transfer */
  rightId: RightId;

  /** Source chain (where the Right currently exists) */
  from: Chain;

  /** Destination chain */
  to: Chain;

  /** Destination owner address on the `to` chain */
  toAddress: string;

  /** Optional custom proof configuration */
  proof?: ProofConfig;
}

/**
 * Custom proof configuration for advanced users.
 *
 * Allows overriding default confirmation thresholds or providing
 * a custom verification function.
 */
export interface ProofConfig {
  /** Number of confirmations to wait for (overrides chain default) */
  confirmations?: number;

  /** Whether to include the full Merkle proof in the bundle */
  includeMerkleProof?: boolean;

  /** Custom verification function called before submitting proof */
  customVerifier?: (proof: ProofBundle) => Promise<boolean>;
}

/**
 * Response from initiating a transfer.
 */
export interface TransferResponse {
  /** The transfer object */
  transfer: Transfer;

  /** Estimated time until completion */
  estimatedCompletion?: Date;
}

/**
 * Filter options for listing Rights.
 */
export interface RightFilter {
  /** Filter by chain */
  chain?: Chain;

  /** Filter by owner address */
  ownerAddress?: string;

  /** Only Rights created after this date */
  createdAfter?: Date;

  /** Only Rights created before this date */
  createdBefore?: Date;
}

/**
 * Filter options for listing transfers.
 */
export interface TransferFilter {
  /** Filter by source chain */
  from?: Chain;

  /** Filter by destination chain */
  to?: Chain;

  /** Filter by status */
  status?: TransferStatus;

  /** Only transfers after this date */
  createdAfter?: Date;

  /** Only transfers before this date */
  createdBefore?: Date;

  /** Maximum number of results (default: 20) */
  limit?: number;
}

// ---------------------------------------------------------------------------
// Chain Provider
// ---------------------------------------------------------------------------

/**
 * Interface that each chain provider must implement.
 *
 * Chain providers handle chain-specific RPC calls, address formatting,
 * proof generation, and seal mechanics.
 */
export interface ChainProvider {
  /** The chain this provider supports */
  chain: Chain;

  /** Human-readable name (e.g. "Bitcoin", "Ethereum") */
  name: string;

  /** Currency symbol (e.g. "BTC", "ETH") */
  symbol: string;

  /** Number of decimals for the native currency */
  decimals: number;

  /** Default number of confirmations required for finality */
  defaultConfirmations: number;

  /** Get the current block number */
  getBlockNumber(): Promise<bigint>;

  /** Get balance for an address */
  getBalance(address: string): Promise<WalletBalance>;

  /** Validate an address format */
  isValidAddress(address: string): boolean;

  /** Format an address for display */
  formatAddress(address: string): string;
}

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

/**
 * Events emitted by the Transfer class during its lifecycle.
 */
export type TransferEvent =
  | { type: "initiated"; transfer: Transfer }
  | { type: "locking"; chain: Chain; confirmations: number; required: number }
  | { type: "locked"; txHash: string }
  | {
      type: "generating-proof";
      progress: number; // 0-100
    }
  | { type: "proof-generated"; proof: ProofBundle }
  | { type: "submitting-proof"; chain: Chain }
  | { type: "verifying" }
  | { type: "minting"; chain: Chain }
  | { type: "completed"; transfer: Transfer }
  | { type: "failed"; error: CsvErrorBase };

/**
 * Event callback type for transfer progress listeners.
 */
export type TransferEventCallback = (event: TransferEvent) => void;

// ---------------------------------------------------------------------------
// Error types (forward-declared; see errors.ts for implementations)
// ---------------------------------------------------------------------------

/**
 * Error codes for all CSV SDK errors.
 *
 * Each code maps to a specific error class with actionable suggestions.
 */
export enum ErrorCode {
  /** Wallet balance is insufficient for the requested operation */
  INSUFFICIENT_FUNDS = "INSUFFICIENT_FUNDS",

  /** The provided Right ID is malformed or does not exist */
  INVALID_RIGHT_ID = "INVALID_RIGHT_ID",

  /** The requested chain is not supported by this SDK version */
  CHAIN_NOT_SUPPORTED = "CHAIN_NOT_SUPPORTED",

  /** Proof verification failed — the proof is invalid or tampered */
  PROOF_VERIFICATION_FAILED = "PROOF_VERIFICATION_FAILED",

  /** RPC call timed out */
  RPC_TIMEOUT = "RPC_TIMEOUT",

  /** Wallet is not connected — operation requires an active wallet */
  WALLET_NOT_CONNECTED = "WALLET_NOT_CONNECTED",

  /** The Right does not exist or is not owned by the caller */
  RIGHT_NOT_FOUND = "RIGHT_NOT_FOUND",

  /** The Right is not on the expected chain */
  RIGHT_ON_WRONG_CHAIN = "RIGHT_ON_WRONG_CHAIN",

  /** The destination address is invalid for the target chain */
  INVALID_DESTINATION_ADDRESS = "INVALID_DESTINATION_ADDRESS",

  /** The transfer has already been completed or failed */
  TRANSFER_ALREADY_FINALIZED = "TRANSFER_ALREADY_FINALIZED",

  /** The operation timed out waiting for completion */
  TRANSFER_TIMEOUT = "TRANSFER_TIMEOUT",

  /** Network error during RPC communication */
  NETWORK_ERROR = "NETWORK_ERROR",

  /** The wallet extension is not installed or not accessible */
  EXTENSION_NOT_AVAILABLE = "EXTENSION_NOT_AVAILABLE",

  /** Invalid mnemonic phrase */
  INVALID_MNEMONIC = "INVALID_MNEMONIC",

  /** Internal SDK error — this indicates a bug */
  INTERNAL_ERROR = "INTERNAL_ERROR",
}

/**
 * Base type for all CSV SDK errors.
 *
 * Every error includes machine-actionable metadata so that agents and
 * applications can provide self-healing UX.
 */
export interface CsvErrorBase {
  /** Machine-readable error code */
  code: ErrorCode;

  /** Human-readable error message */
  message: string;

  /** Additional context about the error */
  details?: Record<string, unknown>;

  /** Suggested fix for the user */
  suggestedFix?: string;

  /** URL to documentation about this error */
  docsUrl?: string;
}

// ---------------------------------------------------------------------------
// WaitForCompletion options
// ---------------------------------------------------------------------------

/**
 * Options for `transfer.waitForCompletion()`.
 */
export interface WaitForCompletionOptions {
  /**
   * Maximum time to wait before throwing a timeout error.
   * Accepts human-readable strings like "5m", "30s", "1h".
   * Default: "10m"
   */
  timeout?: string;

  /**
   * How often to poll for status updates.
   * Accepts human-readable strings like "5s", "30s".
   * Default: "10s"
   */
  pollInterval?: string;
}

/**
 * Options for CSV initialization methods.
 */
export interface CsvInitOptions {
  /** Network to connect to */
  network?: Network;

  /** Custom RPC URLs per chain */
  rpcUrls?: Partial<Record<Chain, string>>;

  /** BIP-44 derivation path */
  derivationPath?: string;
}

// ---------------------------------------------------------------------------
// Proof Strategy (for advanced users)
// ---------------------------------------------------------------------------

/**
 * Predefined proof strategies that balance speed vs. security.
 */
export enum ProofStrategy {
  /** Fastest: minimal confirmations, suitable for low-value transfers */
  Fast = "fast",

  /** Default balance of speed and security */
  Standard = "standard",

  /** Maximum security: waits for 6+ confirmations, full Merkle proofs */
  MaxSecurity = "max-security",
}
