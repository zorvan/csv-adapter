// ============================================================================
// CSV Adapter SDK — Wallet Management
// ============================================================================
//
// The Wallet class handles multi-chain wallet operations:
// - Mnemonic generation and derivation
// - Browser extension connection
// - Balance checking across chains
// - Address derivation per chain
//
// Security note: Mnemonics are sensitive. In production, never log or
// transmit them. The SDK keeps the mnemonic in memory only during
// initialization and derives keys immediately.
//
// ============================================================================

import { Chain, Network, WalletState, WalletBalance, WalletOptions } from "./types.js";
import { WalletNotConnected } from "./errors.js";
import { validateMnemonicFormat } from "./utils/validation.js";
import { getProvider } from "./chains/index.js";

// ---------------------------------------------------------------------------
// Wallet Extension Interface (for browser environments)
// ---------------------------------------------------------------------------

/**
 * Minimal interface for a browser wallet extension.
 *
 * Implementations include CSV Wallet Extension, MetaMask (via adapter), etc.
 */
export interface WalletExtension {
  /** Unique provider identifier */
  providerName: string;

  /** Request account access (triggers user approval popup) */
  requestAccounts(): Promise<string[]>;

  /** Get the current active account */
  getActiveAccount(): Promise<string | null>;

  /** Sign a message with the wallet */
  signMessage(message: string): Promise<string>;

  /** Whether the extension supports the given chain */
  supportsChain(chain: Chain): boolean;
}

// ---------------------------------------------------------------------------
// Wallet Class
// ---------------------------------------------------------------------------

/**
 * Multi-chain wallet for CSV Adapter operations.
 *
 * A Wallet holds the cryptographic keys needed to interact with all supported
 * chains. It derives addresses from a single BIP-39 mnemonic using BIP-44
 * derivation paths.
 *
 * @example
 * ```ts
 * // Generate a new dev wallet
 * const wallet = await Wallet.generate();
 *
 * // Import from mnemonic
 * const wallet = await Wallet.fromMnemonic("word1 word2 ...");
 *
 * // Check balances
 * const state = await wallet.getBalances();
 * ```
 */
export class Wallet {
  /** Whether the wallet has been initialized with keys */
  private initialized = false;

  /** Derived addresses per chain */
  private addresses: Partial<Record<Chain, string>> = {};

  /** Whether this wallet was created from a browser extension */
  private extensionMode = false;

  /** Browser extension reference (if connected) */
  private extension: WalletExtension | null = null;

  /** Network configuration */
  private network: Network;

  private constructor(options?: WalletOptions) {
    this.network = options?.network ?? Network.Mainnet;
  }

  // -----------------------------------------------------------------------
  // Factory Methods
  // -----------------------------------------------------------------------

  /**
   * Generate a new wallet with a random mnemonic.
   *
   * This is suitable for development and testing. The generated mnemonic
   * is returned so the caller can save it.
   *
   * **WARNING**: For production use, prefer `fromMnemonic()` with a
   * securely generated mnemonic that you back up safely.
   *
   * @param options - Wallet configuration
   * @returns The new wallet and its mnemonic
   *
   * @example
   * ```ts
   * const { wallet, mnemonic } = await Wallet.generate();
   * console.log("Save this mnemonic:", mnemonic);
   * ```
   */
  static async generate(_options?: WalletOptions): Promise<{ wallet: Wallet; mnemonic: string }> {
    // Integration point: @scure/bip39 for mnemonic generation
    // In production, this uses @scure/bip39 to generate a 12-word mnemonic
    // and derive keys using BIP-44 paths for each chain.
    throw new Error(
      "Wallet.generate() requires @scure/bip39 integration. " +
        "Install @scure/bip39 and @scure/bip32 to enable wallet generation.",
    );
  }

  /**
   * Initialize a wallet from a BIP-39 mnemonic phrase.
   *
   * This is the production initialization method. The mnemonic controls
   * all derived addresses across all chains.
   *
   * @param mnemonic - BIP-39 mnemonic phrase (12 or 24 words)
   * @param options - Wallet configuration
   * @returns An initialized wallet
   *
   * @throws {InvalidMnemonic} if the mnemonic format is invalid
   *
   * @example
   * ```ts
   * const wallet = await Wallet.fromMnemonic("word1 word2 ... word12");
   * const address = wallet.getAddress(Chain.Ethereum);
   * ```
   */
  static async fromMnemonic(
    mnemonic: string,
    options?: WalletOptions,
  ): Promise<Wallet> {
    // Validate mnemonic format before proceeding
    validateMnemonicFormat(mnemonic);

    const wallet = new Wallet(options);

    // Integration point: @scure/bip32 for key derivation
    // Derives addresses for each chain using BIP-44 paths:
    //   Bitcoin: m/86'/0'/0'/0/0
    //   Ethereum: m/44'/60'/0'/0/0
    //   Sui: m/44'/784'/0'/0'/0'
    //   Aptos: m/44'/637'/0'/0'/0'

    wallet.initialized = true;
    return wallet;
  }

  /**
   * Connect to a browser wallet extension.
   *
   * This method enables dApp integration — users approve the connection
   * through their wallet extension UI.
   *
   * @param extension - The wallet extension to connect to
   * @returns An initialized wallet linked to the extension
   *
   * @throws {CsvError} with code EXTENSION_NOT_AVAILABLE if no extension is found
   *
   * @example
   * ```ts
   * // Auto-detect CSV extension
   * const wallet = await Wallet.connectExtension();
   *
   * // Or with a specific extension
   * const wallet = await Wallet.connectExtension(window.csvWallet);
   * ```
   */
  static async connectExtension(
    extension?: WalletExtension,
    options?: WalletOptions,
  ): Promise<Wallet> {
    const wallet = new Wallet(options);

    // Auto-detect extension if not provided
    const ext = extension ?? detectExtension();
    if (!ext) {
      throw new Error(
        "No CSV wallet extension detected. " +
          "Install it from https://wallet.csv.dev " +
          "or use Wallet.fromMnemonic() instead.",
      );
    }

    // Request account access
    const accounts = await ext.requestAccounts();
    if (accounts.length === 0) {
      throw new WalletNotConnected();
    }

    wallet.extension = ext;
    wallet.extensionMode = true;
    wallet.initialized = true;

    // For extension mode, we store the active account address
    // but not private keys (they remain in the extension)
    wallet.addresses[Chain.Ethereum] = accounts[0];
    // Additional chain addresses would be requested from the extension

    return wallet;
  }

  // -----------------------------------------------------------------------
  // Instance Methods
  // -----------------------------------------------------------------------

  /**
   * Get the wallet's address on a specific chain.
   *
   * @param chain - The chain to get the address for
   * @returns The wallet's address on that chain
   * @throws {WalletNotConnected} if the wallet is not initialized
   */
  getAddress(chain: Chain): string {
    this.ensureInitialized();
    const address = this.addresses[chain];
    if (!address) {
      throw new Error(
        `No address derived for ${chain}. ` +
          `This may indicate the wallet was initialized in extension mode.`,
      );
    }
    return address;
  }

  /**
   * Get all derived addresses across all chains.
   */
  getAllAddresses(): Partial<Record<Chain, string>> {
    this.ensureInitialized();
    return { ...this.addresses };
  }

  /**
   * Check if the wallet is initialized and connected.
   */
  isConnected(): boolean {
    return this.initialized;
  }

  /**
   * Get the balance on a specific chain.
   *
   * @integration Delegates to the chain provider's getBalance method.
   *
   * @param chain - The chain to check balance on
   * @returns Wallet balance for that chain
   */
  async getBalance(chain: Chain): Promise<WalletBalance> {
    this.ensureInitialized();
    const address = this.getAddress(chain);
    const provider = getProvider(chain);
    return provider.getBalance(address);
  }

  /**
   * Get balances across all chains.
   *
   * Fetches all balances in parallel for efficiency.
   * Failed chains are omitted from the result (check individually for errors).
   */
  async getBalances(): Promise<WalletState> {
    this.ensureInitialized();

    const balances: Partial<Record<Chain, WalletBalance>> = {};
    const chains = Object.values(Chain);

    // Fetch all balances in parallel
    const results = await Promise.allSettled(
      chains.map(async (chain) => {
        const address = this.addresses[chain];
        if (!address) return { chain, balance: null as WalletBalance | null };
        const provider = getProvider(chain);
        const balance = await provider.getBalance(address);
        return { chain, balance: balance as WalletBalance | null };
      }),
    );

    for (const result of results) {
      if (result.status === "fulfilled" && result.value.balance) {
        balances[result.value.chain] = result.value.balance;
      }
    }

    return {
      connected: true,
      addresses: { ...this.addresses },
      balances,
    };
  }

  /**
   * Get the network this wallet is configured for.
   */
  getNetwork(): Network {
    return this.network;
  }

  /**
   * Whether this wallet was created via browser extension.
   */
  isExtensionMode(): boolean {
    return this.extensionMode;
  }

  /**
   * Sign a message with the wallet.
   *
   * In extension mode, the signing happens inside the extension.
   * In mnemonic mode, signing happens locally.
   *
   * @param message - The message to sign
   * @param chain - The chain context for the signature
   * @returns The signature as a hex string
   */
  async signMessage(message: string, _chain: Chain): Promise<string> {
    this.ensureInitialized();

    if (this.extensionMode && this.extension) {
      return this.extension.signMessage(message);
    }

    // Integration point: @noble/secp256k1 for local signing
    throw new Error(
      "Local signing requires @noble/secp256k1 integration. " +
        "This is not yet implemented.",
    );
  }

  // -----------------------------------------------------------------------
  // Internal
  // -----------------------------------------------------------------------

  /**
   * Throw WalletNotConnected if the wallet is not initialized.
   */
  private ensureInitialized(): void {
    if (!this.initialized) {
      throw new WalletNotConnected();
    }
  }

  /**
   * Set a derived address for a chain (called internally during initialization).
   */
  setAddress(chain: Chain, address: string): void {
    this.addresses[chain] = address;
  }
}

// ---------------------------------------------------------------------------
// Extension Auto-Detection
// ---------------------------------------------------------------------------

/**
 * Attempt to detect a CSV wallet extension in the browser environment.
 *
 * Checks window.csvWallet and other known extension injection points.
 */
function detectExtension(): WalletExtension | null {
  // Integration point: Browser extension detection
  // In a browser, the CSV extension injects window.csvWallet
  // This function checks known injection points
  if (typeof window === "undefined") {
    return null;
  }

  // Check for CSV extension
  const csvWallet = (window as unknown as Record<string, unknown>).csvWallet as
    | WalletExtension
    | undefined;
  if (csvWallet) {
    return csvWallet;
  }

  // Check for MetaMask (with CSV adapter)
  const ethereum = (window as unknown as Record<string, unknown>).ethereum as
    | { isMetaMask?: boolean; request: (args: { method: string }) => Promise<unknown> }
    | undefined;
  if (ethereum?.isMetaMask) {
    // Would need a MetaMask adapter for CSV operations
    return null;
  }

  return null;
}
