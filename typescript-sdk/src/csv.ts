// ============================================================================
// CSV Adapter SDK — Main CSV Class
// ============================================================================
//
// The CSV class is the primary entry point for all SDK operations.
// It follows the progressive disclosure principle:
//
// Simple path (development):
//   const csv = await CSV.createDevWallet();
//
// Production path:
//   const csv = await CSV.fromMnemonic(process.env.MNEMONIC);
//
// Browser dApp path:
//   const csv = await CSV.connectExtension();
//
// All paths yield the same CSV instance with identical APIs.
//
// ============================================================================

import {
  Chain,
  Network,
  CsvInitOptions,
  WalletState,
  WalletBalance,
  WalletOptions,
} from "./types.js";
import { Wallet } from "./wallet.js";
import { RightsManager } from "./rights.js";
import { TransferManager } from "./transfers.js";
import { ProofManager } from "./proofs.js";

// Re-export chain utilities for convenience on CSV instances
import * as chainUtils from "./chains/index.js";

// ---------------------------------------------------------------------------
// CSV Class
// ---------------------------------------------------------------------------

/**
 * Main entry point for the CSV Adapter SDK.
 *
 * A CSV instance provides access to all cross-chain operations through
 * a unified, chain-agnostic API. Initialize it using one of the
 * factory methods, then use the `.rights`, `.transfers`, and `.proofs`
 * managers to perform operations.
 *
 * @example
 * ```ts
 * // Quick dev setup
 * const csv = await CSV.createDevWallet();
 *
 * // Production setup with existing mnemonic
 * const csv = await CSV.fromMnemonic(process.env.MNEMONIC);
 *
 * // Browser dApp setup
 * const csv = await CSV.connectExtension();
 *
 * // All instances have the same API:
 * const right = await csv.rights.create({ ... });
 * const transfer = await csv.transfers.crossChain({ ... });
 * const proof = await csv.proofs.generate(right.id, Chain.Bitcoin);
 * ```
 */
export class CSV {
  /** The underlying wallet */
  private walletInstance: Wallet;

  /** Rights lifecycle manager */
  public readonly rights: RightsManager;

  /** Cross-chain transfer manager */
  public readonly transfers: TransferManager;

  /** Proof generation and verification manager */
  public readonly proofs: ProofManager;

  /**
   * Chain provider utilities.
   *
   * Access chain provider functions like `csv.chains.getProvider(Chain.Bitcoin)`.
   */
  public readonly chains = chainUtils;

  /**
   * Private constructor — use factory methods to create instances.
   */
  private constructor(wallet: Wallet) {
    this.walletInstance = wallet;

    // Initialize managers
    this.proofs = new ProofManager();
    this.rights = new RightsManager(wallet);
    this.transfers = new TransferManager(wallet, this.rights, this.proofs);
  }

  // -----------------------------------------------------------------------
  // Factory Methods (Progressive Disclosure)
  // -----------------------------------------------------------------------

  /**
   * Quick development wallet setup.
   *
   * Generates a new random wallet and returns it with the mnemonic.
   * Suitable for development and testing — **not for production**.
   *
   * The returned mnemonic should be logged so the developer can fund
   * the generated addresses on testnets.
   *
   * @param options - Optional configuration (network, RPC URLs)
   * @returns A CSV instance ready for development
   *
   * @example
   * ```ts
   * const csv = await CSV.createDevWallet();
   * console.log("Dev wallet created");
   * console.log("Address:", csv.wallet.getAddress(Chain.Ethereum));
   * ```
   */
  static async createDevWallet(_options?: CsvInitOptions): Promise<CSV> {
    // Integration point: @scure/bip39 for wallet generation
    // In production: const { wallet } = await Wallet.generate(walletOptions);

    throw new Error(
      "CSV.createDevWallet() requires @scure/bip39 integration. " +
        "Install @scure/bip39 to enable wallet generation, " +
        "or use CSV.fromMnemonic() with an existing mnemonic.",
    );
  }

  /**
   * Initialize from an existing BIP-39 mnemonic phrase.
   *
   * This is the production initialization method. The mnemonic
   * controls all derived addresses across all chains.
   *
   * @param mnemonic - BIP-39 mnemonic phrase (12 or 24 words)
   * @param options - Optional configuration (network, RPC URLs)
   * @returns A CSV instance ready for production use
   *
   * @throws {InvalidMnemonic} if the mnemonic format is invalid
   *
   * @example
   * ```ts
   * const csv = await CSV.fromMnemonic(
   *   process.env.MNEMONIC,
   *   { network: Network.Mainnet }
   * );
   * ```
   */
  static async fromMnemonic(
    mnemonic: string,
    options?: CsvInitOptions,
  ): Promise<CSV> {
    const walletOptions: WalletOptions = {
      network: options?.network ?? Network.Mainnet,
      rpcUrls: options?.rpcUrls,
      derivationPath: options?.derivationPath,
    };

    const wallet = await Wallet.fromMnemonic(mnemonic, walletOptions);
    return new CSV(wallet);
  }

  /**
   * Connect to a browser wallet extension.
   *
   * This method enables dApp integration — users approve the
   * connection through their wallet extension UI.
   *
   * @param options - Optional configuration
   * @returns A CSV instance connected to the browser extension
   *
   * @throws {ExtensionNotAvailable} if no CSV wallet extension is detected
   *
   * @example
   * ```ts
   * // In a browser dApp
   * const csv = await CSV.connectExtension();
   * const address = csv.wallet.getAddress(Chain.Ethereum);
   * ```
   */
  static async connectExtension(options?: CsvInitOptions): Promise<CSV> {
    const walletOptions: WalletOptions = {
      network: options?.network ?? Network.Mainnet,
      rpcUrls: options?.rpcUrls,
      derivationPath: options?.derivationPath,
    };

    const wallet = await Wallet.connectExtension(undefined, walletOptions);
    return new CSV(wallet);
  }

  // -----------------------------------------------------------------------
  // Instance Properties
  // -----------------------------------------------------------------------

  /**
   * Access the underlying wallet instance.
   *
   * Use this for direct wallet operations like getting addresses,
   * checking balances, and signing messages.
   */
  get wallet(): Wallet {
    return this.walletInstance;
  }

  /**
   * Get the wallet's state including addresses and balances.
   *
   * @returns Current wallet state with all chain addresses and balances
   */
  async getWalletState(): Promise<WalletState> {
    return this.walletInstance.getBalances();
  }

  /**
   * Get the balance on a specific chain.
   *
   * @param chain - The chain to check balance on
   * @returns Wallet balance for that chain
   */
  async getBalance(chain: Chain): Promise<WalletBalance> {
    return this.walletInstance.getBalance(chain);
  }

  /**
   * Get the wallet's address on a specific chain.
   *
   * @param chain - The chain to get the address for
   * @returns The wallet's address on that chain
   */
  getAddress(chain: Chain): string {
    return this.walletInstance.getAddress(chain);
  }

  /**
   * Whether the wallet is connected and initialized.
   */
  isConnected(): boolean {
    return this.walletInstance.isConnected();
  }
}
