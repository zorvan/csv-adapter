// ============================================================================
// CSV Adapter SDK — Address and Amount Formatting Utilities
// ============================================================================

import { Chain, WalletBalance } from "../types.js";

// ---------------------------------------------------------------------------
// Address Formatting
// ---------------------------------------------------------------------------

/**
 * Format an address for human-readable display.
 *
 * Truncates the middle of the address while preserving the prefix and suffix.
 *
 * @example
 * ```ts
 * formatAddress("0x742d35Cc6634C0532925a3b844Bc9e7595f2bD38", Chain.Ethereum)
 * // => "0x742d...bD38"
 * ```
 */
export function formatAddress(address: string, _chain: Chain, chars = 4): string {
  const normalized = address.trim();
  if (normalized.length <= chars * 2 + 3) {
    return normalized; // Address is already short enough
  }
  return `${normalized.slice(0, chars + 2)}...${normalized.slice(-chars)}`;
}

/**
 * Format an address with its chain label for clarity.
 *
 * @example
 * ```ts
 * formatAddressWithChain("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", Chain.Bitcoin)
 * // => "BTC: bc1qw5...v8f3t4"
 * ```
 */
export function formatAddressWithChain(address: string, chain: Chain): string {
  const symbol = chainSymbol(chain);
  return `${symbol}: ${formatAddress(address, chain)}`;
}

// ---------------------------------------------------------------------------
// Amount Formatting
// ---------------------------------------------------------------------------

/**
 * Convert a raw amount (in the chain's smallest unit) to a human-readable string.
 *
 * @param rawAmount - Amount in smallest unit (satoshis, wei, etc.)
 * @param decimals - Number of decimals for the currency
 * @returns Formatted string (e.g. "0.001 BTC")
 *
 * @example
 * ```ts
 * formatAmount(100000n, 8, "BTC")  // Bitcoin: 100k satoshis
 * // => "0.001 BTC"
 *
 * formatAmount(1000000000000000000n, 18, "ETH")  // 1 ETH in wei
 * // => "1.0 ETH"
 * ```
 */
export function formatAmount(
  rawAmount: bigint,
  decimals: number,
  symbol: string,
): string {
  if (rawAmount === 0n) return `0 ${symbol}`;

  const divisor = 10n ** BigInt(decimals);
  const whole = rawAmount / divisor;
  const fractional = rawAmount % divisor;

  if (fractional === 0n) {
    return `${whole.toString()} ${symbol}`;
  }

  // Format fractional part with leading zeros
  const fractionalStr = fractional.toString().padStart(decimals, "0");
  // Remove trailing zeros
  const trimmed = fractionalStr.replace(/0+$/, "");

  return `${whole.toString()}.${trimmed} ${symbol}`;
}

/**
 * Parse a human-readable amount string to the raw value in smallest unit.
 *
 * @param amount - Human-readable amount (e.g. "0.001" or "1.5")
 * @param decimals - Number of decimals for the currency
 * @returns Raw amount in smallest unit
 *
 * @example
 * ```ts
 * parseAmount("0.001", 8)  // 0.001 BTC in satoshis
 * // => 100000n
 * ```
 */
export function parseAmount(amount: string, decimals: number): bigint {
  const [wholePart = "0", fractionalPart = ""] = amount.split(".");

  // Validate fractional length doesn't exceed decimals
  if (fractionalPart.length > decimals) {
    throw new Error(
      `Amount has ${fractionalPart.length} decimal places but currency only supports ${decimals}`,
    );
  }

  const whole = BigInt(wholePart) * 10n ** BigInt(decimals);
  const fractional = BigInt(fractionalPart.padEnd(decimals, "0"));

  return whole + fractional;
}

// ---------------------------------------------------------------------------
// Right ID Formatting
// ---------------------------------------------------------------------------

/**
 * Format a Right ID for display.
 *
 * @example
 * ```ts
 * formatRightId("0xabc123...")  // => "0xabc...123"
 * ```
 */
export function formatRightId(rightId: string): string {
  return formatAddress(rightId, Chain.Ethereum, 3);
}

// ---------------------------------------------------------------------------
// Chain Metadata
// ---------------------------------------------------------------------------

/**
 * Get the native currency symbol for a chain.
 */
export function chainSymbol(chain: Chain): string {
  const symbols: Record<Chain, string> = {
    [Chain.Bitcoin]: "BTC",
    [Chain.Ethereum]: "ETH",
    [Chain.Sui]: "SUI",
    [Chain.Aptos]: "APT",
  };
  return symbols[chain];
}

/**
 * Get the number of decimals for a chain's native currency.
 */
export function chainDecimals(chain: Chain): number {
  const decimals: Record<Chain, number> = {
    [Chain.Bitcoin]: 8, // satoshis
    [Chain.Ethereum]: 18, // wei
    [Chain.Sui]: 9, // MIST
    [Chain.Aptos]: 8, // octas
  };
  return decimals[chain];
}

/**
 * Get the default number of confirmations required for finality on a chain.
 */
export function chainConfirmations(chain: Chain): number {
  const confirmations: Record<Chain, number> = {
    [Chain.Bitcoin]: 6,
    [Chain.Ethereum]: 12,
    [Chain.Sui]: 1, // Sui has instant finality
    [Chain.Aptos]: 1, // Aptos has instant finality
  };
  return confirmations[chain];
}

/**
 * Get a human-readable name for a chain.
 */
export function chainName(chain: Chain): string {
  const names: Record<Chain, string> = {
    [Chain.Bitcoin]: "Bitcoin",
    [Chain.Ethereum]: "Ethereum",
    [Chain.Sui]: "Sui",
    [Chain.Aptos]: "Aptos",
  };
  return names[chain];
}

// ---------------------------------------------------------------------------
// Balance Formatting
// ---------------------------------------------------------------------------

/**
 * Format a WalletBalance for display.
 *
 * @example
 * ```ts
 * formatBalance({ amount: 100000n, formatted: "0.001", symbol: "BTC", decimals: 8 })
 * // => "0.001 BTC"
 * ```
 */
export function formatBalance(balance: WalletBalance): string {
  if (balance.usdValue !== undefined) {
    return `${balance.formatted} ${balance.symbol} (~$${balance.usdValue.toFixed(2)})`;
  }
  return `${balance.formatted} ${balance.symbol}`;
}

// ---------------------------------------------------------------------------
// Timeout String Parsing
// ---------------------------------------------------------------------------

/**
 * Parse a human-readable timeout string to milliseconds.
 *
 * Supported formats: "30s", "5m", "1h", "1000" (raw ms)
 *
 * @example
 * ```ts
 * parseTimeout("5m")  // => 300000
 * parseTimeout("30s") // => 30000
 * parseTimeout("1h")  // => 3600000
 * parseTimeout("5000") // => 5000
 * ```
 */
export function parseTimeout(timeout: string): number {
  const match = timeout.match(/^(\d+)([smh]?)$/);
  if (!match) {
    throw new Error(
      `Invalid timeout format: "${timeout}". Use formats like "30s", "5m", "1h".`,
    );
  }

  const value = Number.parseInt(match[1], 10);
  const unit = match[2] as "" | "s" | "m" | "h";

  switch (unit) {
    case "s":
      return value * 1_000;
    case "m":
      return value * 60 * 1_000;
    case "h":
      return value * 60 * 60 * 1_000;
    case "":
      return value; // Already in milliseconds
    default:
      throw new Error(`Unknown timeout unit: "${unit}"`);
  }
}

/**
 * Format milliseconds as a human-readable string.
 */
export function formatDuration(ms: number): string {
  if (ms < 1_000) return `${ms}ms`;
  if (ms < 60_000) return `${(ms / 1_000).toFixed(1)}s`;
  if (ms < 3_600_000) return `${(ms / 60_000).toFixed(1)}m`;
  return `${(ms / 3_600_000).toFixed(1)}h`;
}
