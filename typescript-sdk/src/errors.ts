// ============================================================================
// CSV Adapter SDK — Typed Error Classes
// ============================================================================
//
// Every error in the SDK extends CsvError and includes:
// - A machine-readable `code` from the ErrorCode enum
// - A human-readable `message`
// - Optional `details` with additional context
// - A `suggestedFix` with actionable guidance
// - A `docsUrl` linking to relevant documentation
//
// This design enables:
// 1. Programmatic error handling (switch on code)
// 2. Self-healing UX (apps can display suggestedFix)
// 3. Agent auto-recovery (MCP agents can read suggestedFix)
// 4. Developer debugging (docsUrl points to full explanation)
//
// ============================================================================

import { ErrorCode, CsvErrorBase, Chain, RightId } from "./types.js";

// Re-export ErrorCode so consumers can import it from errors
export { ErrorCode } from "./types.js";

// ---------------------------------------------------------------------------
// Base Error Class
// ---------------------------------------------------------------------------

/**
 * Base class for all CSV SDK errors.
 *
 * Every SDK error extends this class. It provides structured error data
 * that applications and AI agents can use for automated recovery.
 *
 * @example
 * ```ts
 * try {
 *   await csv.rights.create({ chain: Chain.Bitcoin, commitmentData: {} });
 * } catch (error) {
 *   if (error instanceof CsvError) {
 *     console.log(`Error ${error.code}: ${error.message}`);
 *     console.log(`Fix: ${error.suggestedFix}`);
 *   }
 * }
 * ```
 */
export class CsvError extends Error implements CsvErrorBase {
  /** Machine-readable error code */
  public readonly code: ErrorCode;

  /** Additional context about the error */
  public readonly details?: Record<string, unknown>;

  /** Suggested fix for the user */
  public readonly suggestedFix: string;

  /** URL to documentation about this error */
  public readonly docsUrl: string;

  constructor(
    code: ErrorCode,
    message: string,
    options?: {
      details?: Record<string, unknown>;
      suggestedFix?: string;
      docsUrl?: string;
    },
  ) {
    super(message);
    this.name = "CsvError";
    this.code = code;
    this.details = options?.details;
    this.suggestedFix =
      options?.suggestedFix ?? getDefaultFix(code, message);
    this.docsUrl =
      options?.docsUrl ?? `https://docs.csv.dev/errors/${codeToPath(code)}`;

    // Maintain proper prototype chain for instanceof checks
    Object.setPrototypeOf(this, CsvError.prototype);
  }

  /**
   * Convert this error to a plain object (useful for serialization).
   */
  toJSON(): CsvErrorBase {
    return {
      code: this.code,
      message: this.message,
      details: this.details,
      suggestedFix: this.suggestedFix,
      docsUrl: this.docsUrl,
    };
  }
}

// ---------------------------------------------------------------------------
// Specific Error Classes
// ---------------------------------------------------------------------------

/**
 * The wallet balance is insufficient for the requested operation.
 *
 * Includes the available and required amounts for programmatic handling.
 */
export class InsufficientFunds extends CsvError {
  public readonly available: bigint;
  public readonly required: bigint;
  public readonly chain: Chain;

  constructor(params: {
    available: bigint;
    required: bigint;
    chain: Chain;
    suggestedFix?: string;
  }) {
    const availableStr = params.available.toString();
    const requiredStr = params.required.toString();
    super(ErrorCode.INSUFFICIENT_FUNDS, `Insufficient funds on ${params.chain}: have ${availableStr}, need ${requiredStr}`, {
      details: {
        available: availableStr,
        required: requiredStr,
        chain: params.chain,
      },
      suggestedFix:
        params.suggestedFix ??
        `Fund your ${params.chain} wallet. You need ${requiredStr} but only have ${availableStr}.`,
    });
    this.name = "InsufficientFunds";
    this.available = params.available;
    this.required = params.required;
    this.chain = params.chain;
    Object.setPrototypeOf(this, InsufficientFunds.prototype);
  }
}

/**
 * The provided Right ID is malformed, does not exist, or is not accessible.
 */
export class InvalidRightId extends CsvError {
  public readonly rightId: string;

  constructor(rightId: string, reason?: string) {
    const message = reason
      ? `Invalid Right ID "${rightId}": ${reason}`
      : `Invalid Right ID: "${rightId}". Right IDs must be 32-byte hex strings (0x + 64 hex characters).`;
    super(ErrorCode.INVALID_RIGHT_ID, message, {
      details: { rightId },
      suggestedFix:
        "Verify the Right ID format. It should be a 32-byte hex string starting with '0x'. " +
        "If the format is correct, the Right may have been burned or never existed.",
    });
    this.name = "InvalidRightId";
    this.rightId = rightId;
    Object.setPrototypeOf(this, InvalidRightId.prototype);
  }
}

/**
 * The requested chain is not supported by this SDK version or configuration.
 */
export class ChainNotSupported extends CsvError {
  public readonly chain: string;

  constructor(chain: string) {
    super(ErrorCode.CHAIN_NOT_SUPPORTED, `Chain "${chain}" is not supported.`, {
      details: { chain },
      suggestedFix:
        `Supported chains are: bitcoin, ethereum, sui, aptos. ` +
        `If you need "${chain}" support, check for SDK updates at https://docs.csv.dev/chains.`,
    });
    this.name = "ChainNotSupported";
    this.chain = chain;
    Object.setPrototypeOf(this, ChainNotSupported.prototype);
  }
}

/**
 * Proof verification failed — the proof is invalid, expired, or tampered.
 */
export class ProofVerificationFailed extends CsvError {
  public readonly reason: string;

  constructor(reason: string) {
    super(ErrorCode.PROOF_VERIFICATION_FAILED, `Proof verification failed: ${reason}`, {
      details: { reason },
      suggestedFix:
        "The proof could not be verified. This may mean: " +
        "1) The source chain transaction was reorganized, " +
        "2) The proof data is corrupted, or " +
        "3) The Right was already consumed elsewhere (double-spend attempt). " +
        "Retry the transfer or contact support if the issue persists.",
    });
    this.name = "ProofVerificationFailed";
    this.reason = reason;
    Object.setPrototypeOf(this, ProofVerificationFailed.prototype);
  }
}

/**
 * An RPC call to a chain node timed out.
 */
export class RPCTimeout extends CsvError {
  public readonly chain: Chain;
  public readonly timeoutMs: number;
  public readonly rpcUrl?: string;

  constructor(params: { chain: Chain; timeoutMs: number; rpcUrl?: string }) {
    const urlInfo = params.rpcUrl ? ` (${params.rpcUrl})` : "";
    super(ErrorCode.RPC_TIMEOUT, `RPC timeout on ${params.chain} after ${params.timeoutMs}ms${urlInfo}`, {
      details: {
        chain: params.chain,
        timeoutMs: params.timeoutMs,
        rpcUrl: params.rpcUrl,
      },
      suggestedFix:
        `The ${params.chain} RPC node did not respond within ${params.timeoutMs}ms. ` +
        "Try again later, or configure a different RPC endpoint. " +
        "See https://docs.csv.dev/rpc-setup for recommended providers.",
    });
    this.name = "RPCTimeout";
    this.chain = params.chain;
    this.timeoutMs = params.timeoutMs;
    this.rpcUrl = params.rpcUrl;
    Object.setPrototypeOf(this, RPCTimeout.prototype);
  }
}

/**
 * The wallet is not connected. Operations requiring wallet access will fail.
 */
export class WalletNotConnected extends CsvError {
  constructor() {
    super(ErrorCode.WALLET_NOT_CONNECTED, "Wallet is not connected", {
      suggestedFix:
        "Connect a wallet using CSV.createDevWallet(), CSV.fromMnemonic(), " +
        "or CSV.connectExtension() before performing wallet operations.",
    });
    this.name = "WalletNotConnected";
    Object.setPrototypeOf(this, WalletNotConnected.prototype);
  }
}

/**
 * The Right does not exist or is not owned by the caller.
 */
export class RightNotFound extends CsvError {
  public readonly rightId: RightId;

  constructor(rightId: RightId) {
    super(ErrorCode.RIGHT_NOT_FOUND, `Right "${rightId}" not found`, {
      details: { rightId },
      suggestedFix:
        "The Right may have been burned, transferred, or never existed. " +
        "Check the Right history to verify its current state.",
    });
    this.name = "RightNotFound";
    this.rightId = rightId;
    Object.setPrototypeOf(this, RightNotFound.prototype);
  }
}

/**
 * The Right exists but is not on the expected chain.
 */
export class RightOnWrongChain extends CsvError {
  public readonly rightId: RightId;
  public readonly expected: Chain;
  public readonly actual: Chain;

  constructor(params: { rightId: RightId; expected: Chain; actual: Chain }) {
    super(
      ErrorCode.RIGHT_ON_WRONG_CHAIN,
      `Right "${params.rightId}" is on ${params.actual}, expected ${params.expected}`,
      {
        details: {
          rightId: params.rightId,
          expected: params.expected,
          actual: params.actual,
        },
        suggestedFix:
          `Transfer the Right from ${params.actual} to ${params.expected} first, ` +
          `or update your operation to use ${params.actual}.`,
      },
    );
    this.name = "RightOnWrongChain";
    this.rightId = params.rightId;
    this.expected = params.expected;
    this.actual = params.actual;
    Object.setPrototypeOf(this, RightOnWrongChain.prototype);
  }
}

/**
 * The destination address is invalid for the target chain.
 */
export class InvalidDestinationAddress extends CsvError {
  public readonly address: string;
  public readonly chain: Chain;

  constructor(address: string, chain: Chain) {
    super(
      ErrorCode.INVALID_DESTINATION_ADDRESS,
      `Invalid address "${address}" for ${chain}`,
      {
        details: { address, chain },
        suggestedFix: `Verify the address format for ${chain}. ` + getChainAddressHint(chain),
      },
    );
    this.name = "InvalidDestinationAddress";
    this.address = address;
    this.chain = chain;
    Object.setPrototypeOf(this, InvalidDestinationAddress.prototype);
  }
}

/**
 * The transfer has already been completed or failed and cannot be modified.
 */
export class TransferAlreadyFinalized extends CsvError {
  public readonly transferId: string;
  public readonly status: string;

  constructor(transferId: string, status: string) {
    super(
      ErrorCode.TRANSFER_ALREADY_FINALIZED,
      `Transfer "${transferId}" is already ${status}`,
      {
        details: { transferId, status },
        suggestedFix:
          "This transfer has reached a terminal state. " +
          "To move the Right again, initiate a new transfer.",
      },
    );
    this.name = "TransferAlreadyFinalized";
    this.transferId = transferId;
    this.status = status;
    Object.setPrototypeOf(this, TransferAlreadyFinalized.prototype);
  }
}

/**
 * The operation timed out waiting for transfer completion.
 */
export class TransferTimeout extends CsvError {
  public readonly transferId: string;
  public readonly timeoutMs: number;

  constructor(transferId: string, timeoutMs: number) {
    super(
      ErrorCode.TRANSFER_TIMEOUT,
      `Transfer "${transferId}" did not complete within ${timeoutMs}ms`,
      {
        details: { transferId, timeoutMs },
        suggestedFix:
          "The transfer is still in progress but exceeded the timeout. " +
          "Call transfer.waitForCompletion() with a longer timeout, " +
          "or check transfer.status() to see current progress.",
      },
    );
    this.name = "TransferTimeout";
    this.transferId = transferId;
    this.timeoutMs = timeoutMs;
    Object.setPrototypeOf(this, TransferTimeout.prototype);
  }
}

/**
 * A network error occurred during RPC communication.
 */
export class NetworkError extends CsvError {
  public readonly chain?: Chain;

  constructor(message: string, chain?: Chain) {
    super(ErrorCode.NETWORK_ERROR, message, {
      details: { chain },
      suggestedFix:
        "Check your internet connection and RPC endpoint configuration. " +
        "If using a local node, ensure it is running and accessible.",
    });
    this.name = "NetworkError";
    this.chain = chain;
    Object.setPrototypeOf(this, NetworkError.prototype);
  }
}

/**
 * The browser wallet extension is not installed or not accessible.
 */
export class ExtensionNotAvailable extends CsvError {
  constructor() {
    super(ErrorCode.EXTENSION_NOT_AVAILABLE, "CSV browser extension is not available", {
      suggestedFix:
        "Install the CSV browser extension from https://wallet.csv.dev " +
        "or use CSV.fromMnemonic() for programmatic access.",
    });
    this.name = "ExtensionNotAvailable";
    Object.setPrototypeOf(this, ExtensionNotAvailable.prototype);
  }
}

/**
 * The provided mnemonic phrase is invalid.
 */
export class InvalidMnemonic extends CsvError {
  constructor() {
    super(ErrorCode.INVALID_MNEMONIC, "Invalid BIP-39 mnemonic phrase", {
      suggestedFix:
        "Verify your mnemonic is a valid BIP-39 phrase (12 or 24 words). " +
        "Check for typos, extra spaces, or incorrect words. " +
        "Never share your mnemonic — it controls your wallet.",
    });
    this.name = "InvalidMnemonic";
    Object.setPrototypeOf(this, InvalidMnemonic.prototype);
  }
}

/**
 * An internal SDK error — this indicates a bug in the SDK itself.
 */
export class InternalError extends CsvError {
  constructor(message: string, details?: Record<string, unknown>) {
    super(ErrorCode.INTERNAL_ERROR, `Internal SDK error: ${message}`, {
      details,
      suggestedFix:
        "This is an internal SDK error. Please report it at " +
        "https://github.com/zorvan/csv-adapter/issues with the details above.",
      docsUrl: "https://github.com/zorvan/csv-adapter/issues",
    });
    this.name = "InternalError";
    Object.setPrototypeOf(this, InternalError.prototype);
  }
}

// ---------------------------------------------------------------------------
// Error factory function
// ---------------------------------------------------------------------------

/**
 * Create a CsvError from a plain object (useful for deserialization).
 */
export function fromJsonError(json: CsvErrorBase): CsvError {
  return new CsvError(json.code, json.message, {
    details: json.details,
    suggestedFix: json.suggestedFix,
    docsUrl: json.docsUrl,
  });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function codeToPath(code: ErrorCode): string {
  return code.toLowerCase().replace(/_/g, "-");
}

function getDefaultFix(code: ErrorCode, _message: string): string {
  const defaults: Partial<Record<ErrorCode, string>> = {
    [ErrorCode.INSUFFICIENT_FUNDS]: "Fund your wallet and try again.",
    [ErrorCode.INVALID_RIGHT_ID]: "Verify the Right ID is a valid 32-byte hex string.",
    [ErrorCode.CHAIN_NOT_SUPPORTED]: "Use one of: bitcoin, ethereum, sui, aptos.",
    [ErrorCode.PROOF_VERIFICATION_FAILED]: "Retry the transfer or contact support.",
    [ErrorCode.RPC_TIMEOUT]: "Check your RPC endpoint and try again.",
    [ErrorCode.WALLET_NOT_CONNECTED]: "Connect a wallet before performing operations.",
    [ErrorCode.RIGHT_NOT_FOUND]: "The Right may have been burned or never existed.",
    [ErrorCode.INTERNAL_ERROR]: "Report this bug at https://github.com/zorvan/csv-adapter/issues",
  };
  return defaults[code] ?? "See documentation for more details.";
}

function getChainAddressHint(chain: Chain): string {
  const hints: Record<Chain, string> = {
    [Chain.Bitcoin]: "Bitcoin addresses start with 1, 3, or bc1 (Bech32).",
    [Chain.Ethereum]: "Ethereum addresses is 0x followed by 40 hex characters.",
    [Chain.Sui]: "Sui addresses start with 0x followed by 64 hex characters.",
    [Chain.Aptos]: "Aptos addresses start with 0x followed by 64 hex characters.",
  };
  return hints[chain];
}
