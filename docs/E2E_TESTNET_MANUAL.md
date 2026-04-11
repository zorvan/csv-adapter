# CSV Adapter — End-to-End Testnet User Manual

This guide walks you through the complete end-to-end test of the CSV Adapter across all four supported chains: **Bitcoin**, **Ethereum**, **Sui**, and **Aptos**. It covers wallet generation, funding, contract deployment, and validation testing.

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.80+ | `rustup install stable` |
| Foundry (forge) | latest | `curl -L https://foundry.paradigm.xyz \| bash && foundryup` |
| Sui CLI | 1.69+ | `cargo install --locked --git https://github.com/MystenLabs/sui.git --bin sui` |
| Aptos CLI | 9.1+ | `cargo install --git https://github.com/aptos-labs/aptos-core.git aptos` |

---

## Step 1: Build the CLI

```bash
cargo build --release -p csv-cli
```

Verify:

```bash
./target/release/csv --version
```

---

## Step 2: Generate Wallets

Generate one wallet per chain. **Save every private key and address** — they cannot be recovered.

### Bitcoin (Signet)

```bash
./target/release/csv wallet generate bitcoin
```

### Ethereum (Sepolia)

```bash
./target/release/csv wallet generate ethereum
```

### Sui (Testnet)

```bash
./target/release/csv wallet generate sui
```

### Aptos (Testnet)

```bash
./target/release/csv wallet generate aptos
```

### Verify all wallets

```bash
./target/release/csv wallet list
```

---

## Step 3: Fund Wallets

Use the addresses from Step 2 to request testnet tokens.

### Bitcoin Signet

Visit: <https://mempool.space/signet/faucet> or <https://signet.bc-2.jp>

### Ethereum Sepolia

Visit: <https://sepoliafaucet.com> or any Sepolia faucet.

### Sui Testnet

```bash
./target/release/csv wallet fund sui
```

Or use the web UI: <https://faucet.sui.io/>

### Aptos Testnet

```bash
./target/release/csv wallet fund aptos
```

Or use the web UI: <https://aptos.dev/network/faucet>

### Verify Balances

```bash
./target/release/csv wallet balance bitcoin
./target/release/csv wallet balance ethereum
./target/release/csv wallet balance sui
./target/release/csv wallet balance aptos
```

---

## Step 4: Import Wallets into Chain CLIs

The `csv-cli` manages addresses independently. Chain-specific CLIs need their own wallet import.

### Ethereum — no import needed

Foundry uses the `--private-key` flag or `DEPLOYER_KEY` env var directly.

### Sui

Convert the hex private key to Sui's Bech32 format, then import:

```bash
# Convert hex → Bech32
sui keytool convert <hex-private-key>

# Import
sui keytool import "suiprivkey1..." ed25519

# Switch to the imported address
sui client switch --address <sui-address>

# Ensure testnet environment
sui client switch --env testnet
```

### Aptos

```bash
aptos init --network testnet --private-key 0x<hex-private-key> --assume-yes
```

### Bitcoin

```bash
./target/release/csv wallet import bitcoin <hex-seed>
```

The seed is 64 bytes (128 hex chars) generated during wallet creation.

---

## Step 5: Deploy Contracts

### Bitcoin Signet

Bitcoin Signet uses **real on-chain UTXOs** via the mempool.space public REST API (no local node needed).

```bash
cargo run -p csv-adapter-bitcoin --example signet_real_tx_demo --features signet-rest
```

This performs the complete Bitcoin flow:

1. **Scans** your funded address for UTXOs via mempool.space API
2. **Creates a seal** from a real UTXO
3. **Builds and signs** a Taproot commitment transaction with Schnorr signatures
4. **Broadcasts** the transaction to the Signet network
5. **Waits for confirmation** on-chain
6. **Verifies inclusion** and finality
7. **Tests replay prevention** (seal cannot be reused)

You can monitor your transaction at: `https://mempool.space/signet/tx/<txid>`

### Ethereum (Sepolia)

```bash
./target/release/csv contract deploy ethereum
```

This runs `forge script script/Deploy.s.sol --broadcast` internally. It deploys:

| Contract | Description |
|----------|-------------|
| `CSVLock` | Locks Rights, registers nullifiers, supports refunds |
| `CSVMint` | Verifies proofs and mints Rights |

**Manual deployment** (if needed):

```bash
cd csv-adapter-ethereum/contracts
DEPLOYER_KEY="0x<your-private-key>" ~/.foundry/bin/forge script script/Deploy.s.sol \
  --rpc-url https://ethereum-sepolia-rpc.publicnode.com \
  --broadcast
```

### Sui (Testnet)

```bash
./target/release/csv contract deploy sui
```

This publishes the Move package and creates the shared `LockRegistry`.

**Manual deployment:**

```bash
# Publish package
sui client publish csv-adapter-sui/contracts --gas-budget 500000000 --json

# Note the packageId from output, then create registry:
sui client call \
  --package <package-id> \
  --module csv_seal \
  --function create_registry \
  --gas-budget 10000000
```

### Aptos (Testnet)

```bash
./target/release/csv contract deploy aptos
```

**Manual deployment:**

```bash
# Compile
aptos move compile --package-dir csv-adapter-aptos/contracts

# Publish
aptos move publish \
  --package-dir csv-adapter-aptos/contracts \
  --profile default \
  --assume-yes

# Initialize registry
aptos move run \
  --function-id "<account>::csv_seal::init_registry" \
  --profile default \
  --assume-yes
```

### Verify Deployments

```bash
./target/release/csv contract status ethereum
./target/release/csv contract status sui
./target/release/csv contract status aptos
./target/release/csv contract list
```

---

## Step 6: Run End-to-End Tests

### All Chain Pairs

```bash
./target/release/csv test run --all
```

This runs 4 cross-chain transfer tests:

| # | Source → Dest | Status |
|---|--------------|--------|
| 1 | Bitcoin → Sui | ✅ |
| 2 | Bitcoin → Ethereum | ✅ |
| 3 | Sui → Aptos | ✅ |
| 4 | Ethereum → Sui | ✅ |

### Single Chain Pair

```bash
./target/release/csv test run -p ethereum:sui
```

### Specific Scenario

```bash
./target/release/csv test scenario double_spend
./target/release/csv test scenario invalid_proof
./target/release/csv test scenario ownership_transfer
```

### View Results

```bash
./target/release/csv test results
```

---

## Step 7: Validate On-Chain State

### Ethereum — Verify Contract Code

```bash
curl -s --data '{
  "jsonrpc":"2.0",
  "method":"eth_getCode",
  "params":["0x906734c85644152fAA9EAE4Ce6179C5C5E3D866c","latest"],
  "id":1
}' https://ethereum-sepolia-rpc.publicnode.com
```

The result should be non-empty bytecode (`"0x60806040..."`).

### Sui — Verify Package

```bash
sui client object 0xa972ca52e0c69118471a755aee0efd89993649b6d1f32a4fc9e186c1458694c2
```

Should show `objType: package`, `owner: Immutable`.

### Bitcoin — Verify Commitment Transaction

```bash
curl -s "https://mempool.space/signet/api/tx/<your-txid>/status"
```

Should show `"confirmed": true` with block height.

### Aptos — Verify Account Resources

```bash
curl -s "https://fullnode.testnet.aptoslabs.com/v1/accounts/<account>/resources"
```

Should return `LockRegistry` and `RegistrySingleton` resources.

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `forge` not found | Run `foundryup` and add `~/.foundry/bin` to PATH |
| Sui faucet rate-limited | Wait 60s or use web UI |
| Aptos faucet returns 500 | Use web UI: <https://aptos.dev/network/faucet> |
| Sui `No gas coins` | Fund address or check `sui client gas` |
| `csv wallet fund` 404 | Faucet endpoint may have changed; check `~/.csv/config.toml` |
| Contract deployment fails | Check balance, RPC connectivity, and gas budget |
| Bitcoin tx rejected | Verify UTXO is unspent at <https://mempool.space/signet/address/><addr> |

---

## Architecture Summary

```
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│  Bitcoin   │    │  Ethereum  │    │    Sui     │    │   Aptos    │
│  (Signet)  │    │  (Sepolia) │    │ (Testnet)  │    │ (Testnet)  │
├────────────┤    ├────────────┤    ├────────────┤    ├────────────┤
│ UTXO seals │    │ CSVLock    │    │ csv_seal   │    │ csv_seal   │
│ Tapret ctx │    │ CSVMint    │    │ LockRegistry│   │ LockRegistry│
└────────────┘    └────────────┘    └────────────┘    └────────────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │    csv-cli        │
                    │  Wallet / Test    │
                    │  Deploy / Verify  │
                    └───────────────────┘
```
