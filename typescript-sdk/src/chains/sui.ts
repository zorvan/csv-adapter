// ============================================================================
// CSV Adapter SDK — Sui Chain Provider
// ============================================================================
//
// Sui-specific utilities for CSV Adapter.
//
// Sui's single-use seal is based on object deletion. A Right on Sui is
// anchored to a Sui object. Transferring the Right deletes (or transfers)
// that object, which cryptographically proves the Right was consumed.
//
// Sui has instant finality — once a checkpoint is finalized, the transaction
// cannot be reorganized. This means CSV proofs on Sui require only 1
// confirmation.
//
// Proof format: Checkpoint certification — the lock transaction is proven
// to be included in a finalized checkpoint.
//
// ============================================================================

import { Chain, ChainProvider, WalletBalance, Network } from "../types.js";
import { chainSymbol, chainDecimals, chainConfirmations, chainName } from "../utils/format.js";

/**
 * Default RPC endpoints for Sui networks.
 */
const DEFAULT_RPC: Record<Network, string> = {
  [Network.Mainnet]: "https://fullnode.mainnet.sui.io:443",
  [Network.Testnet]: "https://fullnode.testnet.sui.io:443",
  [Network.Devnet]: "https://fullnode.devnet.sui.io:443",
  [Network.Regtest]: "http://127.0.0.1:9000",
};

/**
 * Configuration for the Sui provider.
 */
export interface SuiConfig {
  /** Network to connect to */
  network?: Network;

  /** Custom RPC URL */
  rpcUrl?: string;
}

/**
 * Sui chain provider.
 *
 * Handles Sui-specific operations including:
 * - Address validation (Sui hex addresses)
 * - Balance checking via Sui RPC
 * - Epoch and checkpoint tracking
 * - Checkproof generation for CSV proofs
 *
 * @integration Uses @mysten/sui.js for Sui RPC communication.
 */
export class SuiProvider implements ChainProvider {
  public readonly chain = Chain.Sui;
  public readonly name = chainName(Chain.Sui);
  public readonly symbol = chainSymbol(Chain.Sui);
  public readonly decimals = chainDecimals(Chain.Sui);
  public readonly defaultConfirmations = chainConfirmations(Chain.Sui);

  private readonly rpcUrl: string;
  private readonly network: Network;

  constructor(config?: SuiConfig) {
    this.network = config?.network ?? Network.Mainnet;
    this.rpcUrl = config?.rpcUrl ?? DEFAULT_RPC[this.network];
  }

  /**
   * Get the current Sui epoch/sequence number.
   *
   * Sui doesn't have traditional block numbers — it uses sequence numbers
   * within epochs. We map this to a "block number" concept for the SDK.
   *
   * @integration Uses @mysten/sui.js's getLatestSuiSystemState().
   */
  async getBlockNumber(): Promise<bigint> {
    // Integration point: @mysten/sui.js
    // Calls suix_getLatestSuiSystemState or similar RPC
    throw new Error(
      "Sui provider requires @mysten/sui.js integration. " +
        "Install @mysten/sui.js and configure the provider.",
    );
  }

  /**
   * Get the SUI balance for an address.
   *
   * @integration Uses @mysten/sui.js's getBalance() via the configured RPC.
   */
  async getBalance(_address: string): Promise<WalletBalance> {
    // Integration point: @mysten/sui.js
    // Calls suix_getBalance RPC method
    throw new Error(
      "Sui provider requires @mysten/sui.js integration. " +
        "Install @mysten/sui.js and configure the provider.",
    );
  }

  /**
   * Validate a Sui address.
   *
   * Sui addresses are hex strings, optionally prefixed with 0x.
   */
  isValidAddress(address: string): boolean {
    const normalized = address.startsWith("0x") ? address.slice(2) : address;
    return /^[a-fA-F0-9]{1,64}$/.test(normalized);
  }

  /**
   * Format a Sui address for display.
   */
  formatAddress(address: string): string {
    const normalized = address.startsWith("0x") ? address : `0x${address}`;
    if (normalized.length <= 14) return normalized;
    return `${normalized.slice(0, 6)}...${normalized.slice(-4)}`;
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
