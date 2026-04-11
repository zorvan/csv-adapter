// ============================================================================
// CSV Adapter SDK — Bitcoin Chain Provider
// ============================================================================
//
// Bitcoin-specific utilities for CSV Adapter.
//
// Bitcoin's single-use seal is based on UTXO consumption. A Right on Bitcoin
// is anchored to a specific UTXO. Transferring the Right spends that UTXO,
// which cryptographically proves the Right was consumed.
//
// Proof format: Bitcoin Merkle proofs — the lock transaction is proven to be
// included in a block via a Merkle branch from the transaction to the block's
// Merkle root.
//
// ============================================================================

import { Chain, ChainProvider, WalletBalance, Network } from "../types.js";
import { chainSymbol, chainDecimals, chainConfirmations, chainName } from "../utils/format.js";

/**
 * Default RPC endpoints for Bitcoin networks.
 */
const DEFAULT_RPC: Record<Network, string> = {
  [Network.Mainnet]: "https://mempool.space/api",
  [Network.Testnet]: "https://mempool.space/testnet/api",
  [Network.Devnet]: "http://127.0.0.1:18443",
  [Network.Regtest]: "http://127.0.0.1:18443",
};

/**
 * Configuration for the Bitcoin provider.
 */
export interface BitcoinConfig {
  /** Network to connect to */
  network?: Network;

  /** Custom RPC URL */
  rpcUrl?: string;
}

/**
 * Bitcoin chain provider.
 *
 * Handles Bitcoin-specific operations including:
 * - Address validation (Legacy, SegWit, Native SegWit)
 * - Balance checking via UTXO set
 * - Block height and confirmation tracking
 * - Merkle proof generation for CSV proofs
 */
export class BitcoinProvider implements ChainProvider {
  public readonly chain = Chain.Bitcoin;
  public readonly name = chainName(Chain.Bitcoin);
  public readonly symbol = chainSymbol(Chain.Bitcoin);
  public readonly decimals = chainDecimals(Chain.Bitcoin);
  public readonly defaultConfirmations = chainConfirmations(Chain.Bitcoin);

  private readonly rpcUrl: string;
  private readonly network: Network;

  constructor(config?: BitcoinConfig) {
    this.network = config?.network ?? Network.Mainnet;
    this.rpcUrl = config?.rpcUrl ?? DEFAULT_RPC[this.network];
  }

  /**
   * Get the current Bitcoin block height.
   *
   * @integration Uses @csv-adapter/bitcoin to fetch tip height
   *   from the configured RPC endpoint.
   */
  async getBlockNumber(): Promise<bigint> {
    // Integration point: @csv-adapter/bitcoin
    // Calls RPC endpoint to get current block height
    // For mempool.space: GET /blocks/tip/height
    throw new Error(
      "Bitcoin provider requires @csv-adapter/bitcoin integration. " +
        "Configure the provider with the core adapter.",
    );
  }

  /**
   * Get the balance for a Bitcoin address.
   *
   * Sum of all unspent outputs (UTXOs) for the address.
   *
   * @integration Uses @csv-adapter/bitcoin to scan the UTXO set
   *   for the given address.
   */
  async getBalance(_address: string): Promise<WalletBalance> {
    // Integration point: @csv-adapter/bitcoin
    // Scans UTXO set for the address
    // For mempool.space: GET /address/{address}/utxo
    throw new Error(
      "Bitcoin provider requires @csv-adapter/bitcoin integration. " +
        "Configure the provider with the core adapter.",
    );
  }

  /**
   * Validate a Bitcoin address format.
   *
   * Supports:
   * - Legacy addresses (P2PKH, P2SH): start with 1 or 3
   * - Native SegWit (Bech32): start with bc1
   * - Testnet addresses: start with tb or bcrt
   */
  isValidAddress(address: string): boolean {
    // Legacy P2PKH
    if (/^1[a-km-zA-HJ-NP-Z1-9]{25,34}$/.test(address)) return true;
    // P2SH
    if (/^3[a-km-zA-HJ-NP-Z1-9]{25,34}$/.test(address)) return true;
    // Bech32 (Native SegWit)
    if (/^bc1[a-z0-9]{39,59}$/.test(address)) return true;
    // Testnet/Signet
    if (/^(tb|bcrt)1[a-z0-9]{39,59}$/.test(address)) return true;
    return false;
  }

  /**
   * Format a Bitcoin address for display.
   */
  formatAddress(address: string): string {
    if (address.length <= 14) return address;
    return `${address.slice(0, 7)}...${address.slice(-4)}`;
  }

  /**
   * Get the RPC URL in use.
   */
  getRpcUrl(): string {
    return this.rpcUrl;
  }

  /**
   * Get the configured network.
   */
  getNetwork(): Network {
    return this.network;
  }
}
