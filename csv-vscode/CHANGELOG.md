# Changelog

All notable changes to the CSV Adapter VS Code extension are documented in this file.

## [0.1.0] - 2026-04-12

### Added
- Syntax highlighting for CSV proof constructs in Rust and TypeScript
- Code snippets for common CSV patterns:
  - `csv-transfer` -- Cross-chain transfer template
  - `csv-right-create` -- Create Right template
  - `csv-proof-verify` -- Proof verification template
  - `csv-wallet-init` -- Wallet initialization
  - `csv-error-handle` -- Error handling pattern
  - `csv-client-builder` -- Client builder pattern
- Wallet Explorer sidebar with balances, Rights, and recent transfers
- Proof Visualizer panel with mermaid.js Merkle tree diagrams
- Inline error assistance with quick fixes for CSV-specific patterns
- Commands:
  - `CSV: Create Right`
  - `CSV: Transfer Cross-Chain`
  - `CSV: Inspect Proof`
  - `CSV: Check Wallet Balance`
  - `CSV: Open Documentation`
  - `CSV: Run Tutorial`
- Configuration settings for chains, network, RPC endpoints, wallet path, and log level
- Welcome message on first activation with quick start options
- Keyboard navigation and screen reader accessibility support
- Light and dark theme compatibility
