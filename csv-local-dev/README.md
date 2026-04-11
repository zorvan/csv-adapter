# CSV Local Dev Environment

Local chain simulator for CSV Adapter - enables the **"5-minute rule"**: developers can start a full local dev environment with simulated chains, no network needed.

## Features

- **Deterministic**: Same state every time - reproducible development
- **Fast**: Blocks every 1s (not 10s+) with `--fast-mode`
- **Pre-funded**: No need to request faucets - dev wallets come loaded
- **Realistic**: Simulates real chain behavior (Bitcoin, Ethereum, Sui, Aptos)
- **Resettable**: One command to start fresh
- **Scriptable**: Can be used in CI/CD pipelines

## Quick Start

```bash
# Install dependencies
npm install

# Start all chains with dashboard
npx csv-local start

# Start in fast mode (1s blocks)
npx csv-local start --fast-mode

# Check chain health
npx csv-local status

# Stop all chains
npx csv-local stop

# Reset to clean state
npx csv-local reset
```

## Commands

### `csv-local start`

Start the local dev environment with simulated chains.

```bash
# Start all chains (default)
csv-local start

# Start specific chains only
csv-local start --chains bitcoin,ethereum

# Fast mode (1s block intervals)
csv-local start --fast-mode

# Custom port
csv-local start --port 9000

# Run in background (no dashboard)
csv-local start --background

# Disable dashboard
csv-local start --no-dashboard
```

**Options:**
| Option | Description | Default |
|--------|-------------|---------|
| `--chains` | Comma-separated chains to start | `all` |
| `--fast-mode` | 1s block intervals | `false` |
| `--port` | Base port for RPC endpoints | `8545` |
| `--background` | Run without dashboard | `false` |
| `--no-dashboard` | Disable TUI dashboard | `false` |

### `csv-local status`

Show chain health dashboard.

```bash
# Interactive dashboard view
csv-local status

# JSON output (for scripts)
csv-local status --json
```

### `csv-local stop`

Stop all simulated chains and clean up.

```bash
# Graceful shutdown
csv-local stop

# Force kill (if graceful fails)
csv-local stop --force
```

### `csv-local reset`

Reset to initial clean state - stops, clears state, and restarts.

```bash
# Reset all chains
csv-local reset

# Reset specific chains
csv-local reset --chains bitcoin,ethereum

# Reset with fast mode
csv-local reset --fast-mode
```

### `csv-local wallet`

Show dev wallet information.

```bash
# Show all wallets
csv-local wallet

# Show specific chain wallet
csv-local wallet --chain ethereum

# Export private keys (DEV ONLY!)
csv-local wallet --chain bitcoin --export
```

### `csv-local scenario`

Run a pre-built scenario.

```bash
# Basic cross-chain transfer
csv-local scenario basic-transfer

# Double-spend prevention demo
csv-local scenario double-spend
```

## Chain Endpoints

When running, the following endpoints are available:

| Chain | Endpoint | Port |
|-------|----------|------|
| Bitcoin | `POST /bitcoin` | 8545 |
| Ethereum | `POST /ethereum` or `POST /eth-rpc` | 8545 |
| Sui | `POST /sui` | 8545 |
| Aptos | `POST /aptos` | 8545 |
| Unified RPC | `POST /rpc/:chain` | 8545 |

### Management Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Health check with all chain statuses |
| `GET /dashboard` | Dashboard data (JSON) |
| `GET /registry/rights` | List all registered Rights |
| `GET /registry/rights/:id` | Get specific Right |
| `POST /registry/transfer` | Register a cross-chain transfer |
| `POST /shutdown` | Graceful server shutdown |
| `GET /debug/requests` | Request log |
| `GET /debug/stats` | RPC proxy statistics |
| `POST /debug/inject-error` | Configure error injection |

## Dev Wallets

All wallets are pre-funded for testing:

| Chain | Balance | Address Format |
|-------|---------|----------------|
| Bitcoin | 50 BTC | `bcrt1...` |
| Ethereum | 10,000 ETH | `0x...` |
| Sui | 1,000 SUI | `0x...` |
| Aptos | 100 APT | `0x...` |

## Simulated Chain Features

### Bitcoin Simulator
- Block generation at configurable intervals
- UTXO set management
- Transaction validation
- Merkle proof generation
- JSON-RPC compatible (Bitcoin Core style)

### Ethereum Simulator
- Block generation with EVM-like execution
- Account state management (balance, nonce, code, storage)
- Gas estimation and limits
- Transaction receipts
- JSON-RPC compatible (Ethereum style)
- Anvil-like methods (`evm_mine`, `evm_snapshot`, etc.)

### Sui Simulator
- Epoch management
- Object state management
- Checkpoint generation
- Transaction finality
- JSON-RPC compatible (Sui style)

### Aptos Simulator
- Block generation
- Account state management
- Ledger info
- Transaction finality
- REST/JSON-RPC compatible (Aptos style)

### Cross-Chain Registry
- Track Rights across all chains
- Enforce single-use (prevent double-spend)
- Transfer tracking with full audit trail
- Proof verification simulation

## Error Injection

For testing failure scenarios:

```bash
# Inject errors into a specific chain
curl -X POST http://localhost:8545/debug/inject-error \
  -H "Content-Type: application/json" \
  -d '{"chain": "ethereum", "errorRate": 0.1}'

# Add artificial delay
curl -X POST http://localhost:8545/debug/inject-error \
  -H "Content-Type: application/json" \
  -d '{"delayMs": 1000}'

# Disable error injection
curl -X DELETE http://localhost:8545/debug/inject-error
```

## Docker Alternative

If you prefer Docker containers with real chain implementations:

```bash
docker-compose -f docker-compose.dev.yml up -d
```

This starts:
- Bitcoin Core (regtest)
- Foundry Anvil (Ethereum)
- Sui Test Validator
- Aptos Node
- CSV Local Simulator

## Programmatic Usage

```javascript
// Import simulators directly
const { BitcoinSimulator } = require('./src/simulator/bitcoin');
const { EthereumSimulator } = require('./src/simulator/ethereum');
const { SuiSimulator } = require('./src/simulator/sui');
const { AptosSimulator } = require('./src/simulator/aptos');
const { CrossChainRegistry } = require('./src/simulator/registry');
const { WalletManager } = require('./src/simulator/wallet');

// Use in tests
const registry = new CrossChainRegistry();
const right = registry.createRight({
  name: 'My Right',
  owner: '0xowner',
  chain: 'ethereum'
});

// Transfer cross-chain
const transfer = await registry.registerTransfer({
  rightId: right.id,
  fromChain: 'ethereum',
  toChain: 'sui',
  fromOwner: '0xowner',
  toOwner: '0xrecipient'
});
```

## Scenarios

### Basic Transfer
Demonstrates transferring a Right from Sui to Ethereum:
```bash
csv-local scenario basic-transfer
```

### Double-Spend Prevention
Demonstrates CSV's double-spend prevention mechanism:
```bash
csv-local scenario double-spend
```

## Architecture

```
csv-local-dev/
├── src/
│   ├── index.js              # CLI entry point
│   ├── dashboard.js          # Terminal dashboard
│   ├── commands/
│   │   ├── start.js          # Start command
│   │   ├── stop.js           # Stop command
│   │   ├── status.js         # Status command
│   │   └── reset.js          # Reset command
│   ├── simulator/
│   │   ├── bitcoin.js        # Bitcoin simulator
│   │   ├── ethereum.js       # Ethereum simulator
│   │   ├── sui.js            # Sui simulator
│   │   ├── aptos.js          # Aptos simulator
│   │   ├── registry.js       # Cross-chain registry
│   │   ├── wallet.js         # Wallet manager
│   │   └── rpc-proxy.js      # RPC proxy
│   └── scenarios/
│       ├── basic-transfer.js # Transfer scenario
│       └── double-spend.js   # Security scenario
├── package.json
├── docker-compose.dev.yml
└── README.md
```

## License

MIT
