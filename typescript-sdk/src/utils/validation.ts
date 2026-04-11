// ============================================================================
// CSV Adapter SDK — Input Validation Utilities
// ============================================================================
//
// All validation functions throw descriptive errors on failure.
// Use these before making RPC calls to fail fast with actionable messages.
//
// ============================================================================

import { z } from "zod";
import { Chain, ChainId, RightId } from "../types.js";
import {
  InvalidRightId,
  InvalidDestinationAddress,
  ChainNotSupported,
} from "../errors.js";

// ---------------------------------------------------------------------------
// Zod Schemas
// ---------------------------------------------------------------------------

/** Schema for a valid Right ID (32-byte hex) */
export const rightIdSchema = z.string().regex(
  /^0x[a-fA-F0-9]{64}$/,
  "Right ID must be a 32-byte hex string (0x + 64 hex characters)",
) as z.ZodType<RightId>;

/** Schema for a valid Ethereum-style address (0x + 40 hex) */
export const ethAddressSchema = z.string().regex(
  /^0x[a-fA-F0-9]{40}$/,
  "Ethereum address must be 0x followed by 40 hex characters",
);

/** Schema for a valid Bitcoin address (legacy, segwit, or native segwit) */
export const btcAddressSchema = z.string().refine(
  (addr) => {
    // Legacy: starts with 1
    if (/^1[a-km-zA-HJ-NP-Z1-9]{25,34}$/.test(addr)) return true;
    // P2SH: starts with 3
    if (/^3[a-km-zA-HJ-NP-Z1-9]{25,34}$/.test(addr)) return true;
    // Bech32 (SegWit): starts with bc1
    if (/^bc1[a-z0-9]{39,59}$/.test(addr)) return true;
    // Testnet/Signet: starts with tb or bcrt
    if (/^(tb|bcrt)1[a-z0-9]{39,59}$/.test(addr)) return true;
    return false;
  },
  { message: "Invalid Bitcoin address format" },
);

/** Schema for a valid Sui/Aptos address (0x + 64 hex) */
export const suiAddressSchema = z.string().regex(
  /^0x[a-fA-F0-9]{1,64}$/,
  "Sui address must be 0x followed by up to 64 hex characters",
);

/** Schema for a valid Aptos address (0x + 64 hex) */
export const aptosAddressSchema = z.string().regex(
  /^0x[a-fA-F0-9]{1,64}$/,
  "Aptos address must be 0x followed by up to 64 hex characters",
);

// ---------------------------------------------------------------------------
// Validation Functions
// ---------------------------------------------------------------------------

/**
 * Validate a Right ID string.
 *
 * @throws {InvalidRightId} if the format is invalid
 */
export function validateRightId(rightId: string): RightId {
  const result = rightIdSchema.safeParse(rightId);
  if (!result.success) {
    throw new InvalidRightId(rightId, result.error.errors[0]?.message);
  }
  return result.data;
}

/**
 * Validate an address for a specific chain.
 *
 * @throws {InvalidDestinationAddress} if the address format is invalid for the chain
 */
export function validateAddress(address: string, chain: Chain): void {
  switch (chain) {
    case Chain.Bitcoin: {
      const result = btcAddressSchema.safeParse(address);
      if (!result.success) {
        throw new InvalidDestinationAddress(address, chain);
      }
      break;
    }
    case Chain.Ethereum: {
      const result = ethAddressSchema.safeParse(address);
      if (!result.success) {
        throw new InvalidDestinationAddress(address, chain);
      }
      break;
    }
    case Chain.Sui: {
      const result = suiAddressSchema.safeParse(address);
      if (!result.success) {
        throw new InvalidDestinationAddress(address, chain);
      }
      break;
    }
    case Chain.Aptos: {
      const result = aptosAddressSchema.safeParse(address);
      if (!result.success) {
        throw new InvalidDestinationAddress(address, chain);
      }
      break;
    }
    default: {
      // Exhaustiveness check — this should never be reached
      const exhaustiveCheck: never = chain;
      throw new ChainNotSupported(exhaustiveCheck);
    }
  }
}

/**
 * Validate that a chain value is supported.
 *
 * @throws {ChainNotSupported} if the chain is not recognized
 */
export function validateChain(chain: string): Chain {
  if (!isChainSupported(chain)) {
    throw new ChainNotSupported(chain);
  }
  return chain as Chain;
}

/**
 * Check if a string is a supported chain identifier.
 */
export function isChainSupported(chain: string): chain is ChainId {
  return Object.values(Chain).includes(chain as Chain);
}

/**
 * Validate that a string is a non-empty, trimmed value.
 */
export function requireNonEmpty(value: string, fieldName: string): string {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    throw new Error(`"${fieldName}" must not be empty`);
  }
  return trimmed;
}

/**
 * Validate that a number is positive.
 */
export function requirePositive(value: number, fieldName: string): number {
  if (value <= 0 || !Number.isFinite(value)) {
    throw new Error(`"${fieldName}" must be a positive number, got ${value}`);
  }
  return value;
}

/**
 * Validate a BIP-39 mnemonic phrase (basic format check).
 *
 * Checks word count (12 or 24) and that words are from the BIP-39 wordlist.
 * Note: This does NOT verify the checksum — use a proper BIP-39 library for that.
 *
 * @throws {Error} if the mnemonic format is invalid
 */
export function validateMnemonicFormat(mnemonic: string): void {
  const words = mnemonic.trim().toLowerCase().split(/\s+/);

  if (words.length !== 12 && words.length !== 24) {
    throw new Error(
      `Mnemonic must be 12 or 24 words, got ${words.length} words`,
    );
  }

  // Basic check: all words should be alphabetic (BIP-39 wordlist entries)
  const invalidWords = words.filter((w) => !/^[a-z]+$/.test(w));
  if (invalidWords.length > 0) {
    throw new Error(
      `Mnemonic contains invalid words: ${invalidWords.slice(0, 3).join(", ")}`,
    );
  }
}

/**
 * Validate a commitment data object.
 *
 * Must be a non-null object with at least one key.
 */
export function validateCommitmentData(
  data: Record<string, unknown>,
): void {
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new Error("Commitment data must be a plain object");
  }
  if (Object.keys(data).length === 0) {
    throw new Error("Commitment data must have at least one field");
  }
}

// ---------------------------------------------------------------------------
// Async Validators (for operations that need network access)
// ---------------------------------------------------------------------------

/**
 * Validate that an RPC endpoint is reachable.
 *
 * @returns true if the endpoint responds
 */
export async function validateRpcEndpoint(url: string): Promise<boolean> {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5_000);

    await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "noop" }),
      signal: controller.signal,
    });

    clearTimeout(timeoutId);
    // Even an error response means the endpoint is reachable
    return true;
  } catch {
    return false;
  }
}
