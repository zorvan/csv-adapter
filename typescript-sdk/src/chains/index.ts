// ============================================================================
// CSV Adapter SDK — Chain Provider Registry
// ============================================================================
//
// The chain provider registry maps each Chain to its provider implementation.
// This enables the SDK's chain-agnostic API — consumers call the same methods
// regardless of which chain they're working with.
//
// ============================================================================

import { Chain, ChainProvider } from "../types.js";
import { BitcoinProvider } from "./bitcoin.js";
import { EthereumProvider } from "./ethereum.js";
import { SuiProvider } from "./sui.js";
import { AptosProvider } from "./aptos.js";

/**
 * Registry of all chain providers.
 */
const registry: Map<Chain, ChainProvider> = new Map();

/**
 * Register a chain provider.
 *
 * Called automatically during SDK initialization.
 */
export function registerProvider(provider: ChainProvider): void {
  registry.set(provider.chain, provider);
}

/**
 * Get the provider for a specific chain.
 *
 * @throws {Error} if the chain is not registered
 */
export function getProvider(chain: Chain): ChainProvider {
  const provider = registry.get(chain);
  if (!provider) {
    throw new Error(
      `No provider registered for "${chain}". ` +
        `This is an SDK configuration error.`,
    );
  }
  return provider;
}

/**
 * Get all registered providers.
 */
export function getAllProviders(): ChainProvider[] {
  return Array.from(registry.values());
}

/**
 * Check if a provider is registered for the given chain.
 */
export function hasProvider(chain: Chain): boolean {
  return registry.has(chain);
}

// Register all built-in providers on module load
registerProvider(new BitcoinProvider());
registerProvider(new EthereumProvider());
registerProvider(new SuiProvider());
registerProvider(new AptosProvider());
