// ============================================================================
// CSV Adapter SDK — Aptos Chain Provider
// ============================================================================
//
// Aptos-specific utilities for CSV Adapter.
//
// Aptos's single-use seal is based on resource deletion/move. A Right on
// Aptos is anchored to an Aptos resource. Transferring the Right deletes
// or moves that resource, which proves the Right was consumed.
//
// Like Sui, Aptos has instant finality — once a transaction is committed
// to the ledger, it cannot be reorganized.
//
// Proof format: Ledger info proof — the lock transaction is proven via
// Aptos's ledger info and accumulator proofs.
//
// ============================================================================

import { Chain, ChainProvider, WalletBalance, Network } from "../types.js";
import { chainSymbol, chainDecimals, chainConfirmations, chainName } from "../utils/format.js";

/**
 * Default RPC endpoints for Aptos networks.
 */
const DEFAULT_RPC: Record<Network, string> = {
  [Network.Mainnet]: "https://fullnode.mainnet.aptoslabs.com/v1",
  [Network.Testnet]: "https://fullnode.testnet.aptoslabs.com/v1",
  [Network.Devnet]: "https://fullnode.devnet.aptoslabs.com/v1",
  [Network.Regtest]: "http://127.0.0.1:8080/v1",
};

/**
 * Configuration for the Aptos provider.
 */
export interface AptosConfig {
  /** Network to connect to */
  network?: Network;

  /** Custom RPC URL */
  rpcUrl?: string;
}

/**
 * Aptos chain provider.
 *
 * Handles Aptos-specific operations including:
 * - Address validation (Aptos hex addresses)
 * - Balance checking via Aptos REST API
 * - Ledger version tracking
 * - Ledger info proof generation for CSV proofs
 *
 * @integration Uses the aptos package for Aptos API communication.
 */
export class AptosProvider implements ChainProvider {
  public readonly chain = Chain.Aptos;
  public readonly name = chainName(Chain.Aptos);
  public readonly symbol = chainSymbol(Chain.Aptos);
  public readonly decimals = chainDecimals(Chain.Aptos);
  public readonly defaultConfirmations = chainConfirmations(Chain.Aptos);

  private readonly rpcUrl: string;
  private readonly network: Network;

  constructor(config?: AptosConfig) {
    this.network = config?.network ?? Network.Mainnet;
    this.rpcUrl = config?.rpcUrl ?? DEFAULT_RPC[this.network];
  }

  /**
   * Get the current Aptos ledger version.
   *
   * Maps to the SDK's "block number" concept.
   *
   * @integration Uses the aptos package's getLedgerInfo() via the REST API.
   */
  async getBlockNumber(): Promise<bigint> {
    // Integration point: aptos
    // Calls GET /v1 endpoint for ledger info
    throw new Error(
      "Aptos provider requires aptos package integration. " +
        "Install aptos and configure the provider.",
    );
  }

  /**
   * Get the APT balance for an address.
   *
   * @integration Uses the aptos package's getAccountAPTAmount().
   */
  async getBalance(_address: string): Promise<WalletBalance> {
    // Integration point: aptos
    // Calls GET /v1/accounts/{address}/balance
    throw new Error(
      "Aptos provider requires aptos package integration. " +
        "Install aptos and configure the provider.",
    );
  }

  /**
   * Validate an Aptos address.
   *
   * Aptos addresses are hex strings, optionally prefixed with 0x.
   */
  isValidAddress(address: string): boolean {
    const normalized = address.startsWith("0x") ? address.slice(2) : address;
    return /^[a-fA-F0-9]{1,64}$/.test(normalized);
  }

  /**
   * Format an Aptos address for display.
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
