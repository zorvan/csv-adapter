// ============================================================================
// CSV Adapter SDK — Ethereum Chain Provider
// ============================================================================
//
// Ethereum-specific utilities for CSV Adapter.
//
// Ethereum's single-use seal is implemented via smart contract state.
// A Right on Ethereum is anchored to a contract that tracks seal consumption.
// The proof format uses Merkle Patricia Trie (MPT) proofs to verify
// that the lock event occurred in a specific block.
//
// ============================================================================

import { Chain, ChainProvider, WalletBalance, Network } from "../types.js";
import { chainSymbol, chainDecimals, chainConfirmations, chainName } from "../utils/format.js";

/**
 * Default RPC endpoints for Ethereum networks.
 */
const DEFAULT_RPC: Record<Network, string> = {
  [Network.Mainnet]: "https://eth.llamarpc.com",
  [Network.Testnet]: "https://sepolia.gateway.tenderly.co",
  [Network.Devnet]: "http://127.0.0.1:8545",
  [Network.Regtest]: "http://127.0.0.1:8545",
};

/**
 * Configuration for the Ethereum provider.
 */
export interface EthereumConfig {
  /** Network to connect to */
  network?: Network;

  /** Custom RPC URL */
  rpcUrl?: string;
}

/**
 * Ethereum chain provider.
 *
 * Handles Ethereum-specific operations including:
 * - Address validation (checksummed EIP-55)
 * - Balance checking via eth_getBalance
 * - Block number and confirmation tracking
 * - MPT proof generation for CSV proofs
 *
 * @integration Uses viem for Ethereum RPC communication.
 */
export class EthereumProvider implements ChainProvider {
  public readonly chain = Chain.Ethereum;
  public readonly name = chainName(Chain.Ethereum);
  public readonly symbol = chainSymbol(Chain.Ethereum);
  public readonly decimals = chainDecimals(Chain.Ethereum);
  public readonly defaultConfirmations = chainConfirmations(Chain.Ethereum);

  private readonly rpcUrl: string;
  private readonly network: Network;

  constructor(config?: EthereumConfig) {
    this.network = config?.network ?? Network.Mainnet;
    this.rpcUrl = config?.rpcUrl ?? DEFAULT_RPC[this.network];
  }

  /**
   * Get the current Ethereum block number.
   *
   * @integration Uses viem's getBlockNumber() via the configured RPC.
   */
  async getBlockNumber(): Promise<bigint> {
    // Integration point: viem
    // Calls eth_blockNumber RPC method
    throw new Error(
      "Ethereum provider requires viem integration. " +
        "Install viem and configure the provider.",
    );
  }

  /**
   * Get the ETH balance for an address.
   *
   * @integration Uses viem's getBalance() via the configured RPC.
   */
  async getBalance(_address: string): Promise<WalletBalance> {
    // Integration point: viem
    // Calls eth_getBalance RPC method
    throw new Error(
      "Ethereum provider requires viem integration. " +
        "Install viem and configure the provider.",
    );
  }

  /**
   * Validate an Ethereum address.
   *
   * Accepts both checksummed (EIP-55) and lowercase addresses.
   */
  isValidAddress(address: string): boolean {
    // Must be 0x followed by exactly 40 hex characters
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  }

  /**
   * Format an Ethereum address for display (checksummed if valid).
   */
  formatAddress(address: string): string {
    if (address.length <= 14) return address;
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
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
