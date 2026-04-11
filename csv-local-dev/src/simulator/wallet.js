#!/usr/bin/env node
'use strict';

const crypto = require('crypto');

/**
 * Deterministic Wallet Manager
 * 
 * Generates deterministic wallets for each chain from a seed phrase.
 * Pre-funds wallets with test tokens.
 */
class WalletManager {
  constructor(options = {}) {
    this.seed = options.seed || 'csv-local-dev-test-seed-do-not-use-in-production-2024';
    this.wallets = new Map();
    this.initialized = false;
    
    // Chain-specific address derivation prefixes
    this.chainConfig = {
      bitcoin: {
        prefix: 'bcrt',
        addressFormat: 'bech32',
        initialBalance: '50.00000000 BTC',
        rawBalance: 50 * 1e8
      },
      ethereum: {
        prefix: '0x',
        addressLength: 42,
        initialBalance: '10000.0 ETH',
        rawBalance: BigInt(10000 * 1e18)
      },
      sui: {
        prefix: '0x',
        addressLength: 66,
        initialBalance: '1000.000000000 SUI',
        rawBalance: BigInt(1000 * 1e9)
      },
      aptos: {
        prefix: '0x',
        addressLength: 66,
        initialBalance: '100.00000000 APT',
        rawBalance: BigInt(100 * 1e8)
      }
    };
  }

  /**
   * Initialize wallets for all chains
   */
  async initialize() {
    if (this.initialized) return;

    for (const chain of Object.keys(this.chainConfig)) {
      await this.generateWallet(chain);
    }

    this.initialized = true;
  }

  /**
   * Generate a deterministic wallet for a specific chain
   */
  async generateWallet(chain) {
    const config = this.chainConfig[chain];
    if (!config) {
      throw new Error(`Unknown chain: ${chain}`);
    }

    // Derive deterministic key from seed + chain
    const chainSeed = `${this.seed}::${chain}`;
    const hash = crypto.createHash('sha256').update(chainSeed).digest();
    const privateKey = '0x' + hash.toString('hex');

    // Derive address based on chain format
    let address;
    switch (chain) {
      case 'bitcoin':
        // Bech32-like address (bcrt1...)
        address = this.deriveBitcoinAddress(hash);
        break;
      case 'ethereum':
        // 0x + last 20 bytes of keccak-like hash
        address = '0x' + hash.slice(12).toString('hex');
        break;
      case 'sui':
        // 0x + 32 bytes
        address = '0x' + hash.toString('hex');
        break;
      case 'aptos':
        // 0x + 32 bytes
        address = '0x' + hash.toString('hex');
        break;
      default:
        address = '0x' + hash.toString('hex');
    }

    const wallet = {
      chain,
      address,
      privateKey,
      publicKey: '0x' + crypto.createHash('sha256').update(hash).digest('hex'),
      balance: config.initialBalance,
      rawBalance: config.rawBalance,
      nonce: 0,
      createdAt: Date.now()
    };

    this.wallets.set(chain, wallet);
    return wallet;
  }

  /**
   * Derive a Bitcoin-like regtest address
   */
  deriveBitcoinAddress(hash) {
    // Simplified bech32-like address derivation
    const data = hash.slice(0, 20);
    const hrp = 'bcrt';
    const charset = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
    
    // Convert to bech32-like encoding (simplified)
    let address = hrp + '1';
    for (const byte of data) {
      address += charset[byte % 32];
    }
    
    return address;
  }

  /**
   * Get wallet for a specific chain
   */
  getWallet(chain) {
    return this.wallets.get(chain) || null;
  }

  /**
   * Get all wallets
   */
  getAllWallets() {
    return Object.fromEntries(this.wallets);
  }

  /**
   * Update wallet balance
   */
  updateBalance(chain, balance) {
    const wallet = this.wallets.get(chain);
    if (wallet) {
      wallet.balance = balance;
    }
  }

  /**
   * Get wallet address for a chain
   */
  getAddress(chain) {
    const wallet = this.wallets.get(chain);
    return wallet ? wallet.address : null;
  }

  /**
   * Export wallet keys (WARNING: dev only!)
   */
  exportWallet(chain) {
    const wallet = this.wallets.get(chain);
    if (!wallet) return null;

    return {
      chain: wallet.chain,
      address: wallet.address,
      privateKey: wallet.privateKey,
      publicKey: wallet.publicKey,
      warning: 'WARNING: Private key exposure! Do not use these keys in production!'
    };
  }

  /**
   * Reset all wallets
   */
  reset() {
    this.wallets.clear();
    this.initialized = false;
  }
}

module.exports = { WalletManager };
