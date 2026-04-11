// ============================================================================
// CSV Adapter SDK — Main Exports
// ============================================================================
//
// This is the primary entry point for the SDK.
//
// Usage:
//   import { CSV, Chain, Right, Transfer } from '@csv-adapter/sdk';
//
// Progressive API:
//   - CSV.createDevWallet()    → Quick dev setup
//   - CSV.fromMnemonic()       → Production init with existing wallet
//   - CSV.connectExtension()   → Browser wallet integration
//
// ============================================================================

// ---------------------------------------------------------------------------
// Primary Entry Point
// ---------------------------------------------------------------------------

export { CSV } from "./csv.js";

// ---------------------------------------------------------------------------
// Core Types (type-only exports)
// ---------------------------------------------------------------------------

export type {
  ChainId,
  NetworkId,
  Right,
  RightId,
  TransferState,
  TransferStatus,
  ProofBundle,
  InclusionProof,
  FinalityProof,
  SealConsumptionProof,
  WalletState,
  WalletBalance,
  WalletOptions,
  CreateRightRequest,
  CreateRightResponse,
  TransferRequest,
  TransferResponse,
  RightFilter,
  TransferFilter,
  ChainProvider,
  TransferEvent,
  TransferEventCallback,
  ErrorCode,
  CsvErrorBase,
  WaitForCompletionOptions,
  CsvInitOptions,
  ProofStrategy,
} from "./types.js";

/** @deprecated Use TransferState for the data type. The Transfer class is exported from transfers. */
export type { Transfer as TransferType } from "./types.js";

// Re-export the Chain enum (value, not type)
export { Chain, Network, ChainSchema } from "./types.js";

// ---------------------------------------------------------------------------
// Error Classes
// ---------------------------------------------------------------------------

export {
  CsvError,
  InsufficientFunds,
  InvalidRightId,
  ChainNotSupported,
  ProofVerificationFailed,
  RPCTimeout,
  WalletNotConnected,
  RightNotFound,
  RightOnWrongChain,
  InvalidDestinationAddress,
  TransferAlreadyFinalized,
  TransferTimeout,
  NetworkError,
  ExtensionNotAvailable,
  InvalidMnemonic,
  InternalError,
  fromJsonError,
} from "./errors.js";

// ---------------------------------------------------------------------------
// Wallet
// ---------------------------------------------------------------------------

export { Wallet } from "./wallet.js";
export type { WalletExtension } from "./wallet.js";

// ---------------------------------------------------------------------------
// Managers (for advanced/direct usage)
// ---------------------------------------------------------------------------

export { RightsManager } from "./rights.js";
export { TransferManager, Transfer } from "./transfers.js";
export { ProofManager } from "./proofs.js";

// ---------------------------------------------------------------------------
// Chain Providers (re-export)
// ---------------------------------------------------------------------------

export {
  registerProvider,
  getProvider,
  getAllProviders,
  hasProvider,
} from "./chains/index.js";

export {
  BitcoinProvider,
} from "./chains/bitcoin.js";
export type { BitcoinConfig } from "./chains/bitcoin.js";

export {
  EthereumProvider,
} from "./chains/ethereum.js";
export type { EthereumConfig } from "./chains/ethereum.js";

export {
  SuiProvider,
} from "./chains/sui.js";
export type { SuiConfig } from "./chains/sui.js";

export {
  AptosProvider,
} from "./chains/aptos.js";
export type { AptosConfig } from "./chains/aptos.js";

// ---------------------------------------------------------------------------
// Utilities (re-export)
// ---------------------------------------------------------------------------

export {
  formatAddress,
  formatAddressWithChain,
  formatAmount,
  parseAmount,
  formatRightId,
  formatBalance,
  chainSymbol,
  chainDecimals,
  chainConfirmations,
  chainName,
  parseTimeout,
  formatDuration,
} from "./utils/format.js";

export {
  validateRightId,
  validateAddress,
  validateChain,
  isChainSupported,
  requireNonEmpty,
  requirePositive,
  validateMnemonicFormat,
  validateCommitmentData,
  validateRpcEndpoint,
  rightIdSchema,
  ethAddressSchema,
  btcAddressSchema,
  suiAddressSchema,
  aptosAddressSchema,
} from "./utils/validation.js";
