# CSV Adapter VS Code Extension

CSV Adapter integration for Visual Studio Code -- syntax highlighting, code snippets, IntelliSense, wallet explorer, and proof visualization for cross-chain development.

## Features

### Syntax Highlighting

- Highlight CSV-specific constructs in Rust and TypeScript files
- Recognizes Right IDs, chain names, proof types, and SDK calls
- Special comment highlighting (`// csv:`, `/* csv-adapter: */`)
- Works with both light and dark themes

### Code Snippets

Pre-built code templates for common CSV patterns:

| Prefix | Description |
|--------|-------------|
| `csv-transfer` | Cross-chain transfer template |
| `csv-right-create` | Create Right template |
| `csv-proof-verify` | Proof verification template |
| `csv-wallet-init` | Wallet initialization |
| `csv-error-handle` | Error handling pattern |
| `csv-client-builder` | Client builder pattern |

Each snippet is available for both Rust and TypeScript.

### Wallet Explorer

Sidebar view showing:

- Wallet balances per chain
- Rights tree grouped by chain
- Recent transfer history
- Click to copy Right ID or address
- Refresh button for live updates

### Proof Visualizer

Interactive panel for visualizing CSV proofs:

- Merkle tree diagrams rendered with mermaid.js
- Step-by-step verification display
- Color-coded valid/invalid paths
- Export capability for diagrams

### Inline Error Assistance

- Detects CSV-specific error patterns
- Quick fixes for common issues:
  - Add `@csv-adapter/sdk` dependency
  - Fix chain name typos (e.g., `eth` -> `ethereum`)
  - Add error handling wrappers
  - Import missing types

### Commands

Access via Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

| Command | Description |
|---------|-------------|
| `CSV: Create Right` | Opens input form to create a new Right |
| `CSV: Transfer Cross-Chain` | Wizard for cross-chain transfers |
| `CSV: Inspect Proof` | Opens the proof visualizer |
| `CSV: Check Wallet Balance` | Shows balance notification |
| `CSV: Open Documentation` | Opens docs.csv.dev |
| `CSV: Run Tutorial` | Opens interactive tutorial |

## Installation

### From VSIX

1. Download the latest `.vsix` file from releases
2. Open VS Code
3. Go to Extensions view (`Ctrl+Shift+X`)
4. Click "..." menu and select "Install from VSIX"
5. Select the downloaded file

### From Source

```bash
cd csv-vscode
npm install
npm run compile
# Press F5 to run in Extension Development Host
```

### Requirements

- VS Code 1.85.0 or later

## Configuration

Configure the extension in VS Code Settings (`Ctrl+,` / `Cmd+,`):

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `csv.chains` | array | `["ethereum", "aptos", "sui"]` | Configured chains |
| `csv.network` | string | `"testnet"` | Network: `mainnet`, `testnet`, `devnet` |
| `csv.rpc.ethereum` | string | `""` | Custom Ethereum RPC |
| `csv.rpc.bitcoin` | string | `""` | Custom Bitcoin RPC |
| `csv.rpc.aptos` | string | `""` | Custom Aptos RPC |
| `csv.rpc.sui` | string | `""` | Custom Sui RPC |
| `csv.rpc.solana` | string | `""` | Custom Solana RPC |
| `csv.wallet.path` | string | `""` | Wallet file path |
| `csv.logLevel` | string | `"info"` | Log level: `debug`, `info`, `warn`, `error` |

### Example settings.json

```json
{
  "csv.chains": ["ethereum", "aptos", "sui"],
  "csv.network": "testnet",
  "csv.rpc.aptos": "https://fullnode.testnet.aptoslabs.com/v1",
  "csv.wallet.path": "./wallet.json",
  "csv.logLevel": "debug"
}
```

## Usage

### Creating a Right

1. Open Command Palette
2. Run `CSV: Create Right`
3. Select chain, type, and enter owner address
4. A template file opens for editing

### Cross-Chain Transfer

1. Run `CSV: Transfer Cross-Chain`
2. Select source and destination chains
3. Enter amount and recipient
4. Edit and run the generated code

### Inspecting Proofs

1. Select a Right ID in your code
2. Run `CSV: Inspect Proof`
3. View the interactive Merkle tree visualization

## Supported Languages

- Rust (`.rs`)
- TypeScript (`.ts`, `.tsx`)
- JavaScript (`.js`, `.jsx`)

## Accessibility

- Full keyboard navigation support
- Screen reader compatible
- High contrast theme support
- Reduced motion preference respected

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

MIT
