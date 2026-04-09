# Bitcoin Adapter: Wiring Real RPCs Guide

## Overview

This guide explains how to wire the Bitcoin adapter to broadcast real transactions on Bitcoin Signet (or other networks). The adapter is **fully implemented** and ready for live network testing.

## Current Status

✅ **Complete Implementation:**
- Real RPC client (`RealBitcoinRpc`) using `bitcoincore-rpc`
- Transaction building and signing (`tx_builder`)
- Taproot commitment scripts (`tapret`)
- HD wallet with BIP-86 derivation (`wallet`)
- UTXO discovery and management
- Merkle proof extraction and verification
- End-to-end publishing pipeline

⚠️ **Requires Manual Setup:**
- Bitcoin Core node access (Signet/Testnet/Mainnet)
- Funded wallet with confirmed UTXOs
- Environment configuration

## Architecture Flow

```
User Application
    ↓
1. Create/Load Wallet (SealWallet)
    ↓
2. Discover/Fund UTXOs
    ↓
3. fund_seal(outpoint) → Creates seal from real UTXO
    ↓
4. publish(commitment, seal) → Builds + Signs + Broadcasts tx
    ↓
5. Returns real txid and anchor reference
    ↓
6. verify_inclusion(anchor) → Fetches real Merkle proof
    ↓
7. verify_finality(anchor) → Checks confirmation depth
```

## Step-by-Step Setup

### 1. Bitcoin Core Node Setup

You need access to a Bitcoin Core node with RPC enabled.

**For Signet (recommended for testing):**
```bash
bitcoind -signet \
  -server=1 \
  -rpcuser=testuser \
  -rpcpassword=testpass \
  -rpcbind=127.0.0.1 \
  -rpcport=38332 \
  -rpcallowip=127.0.0.1
```

**RPC URL:** `http://127.0.0.1:38332`

### 2. Create and Fund a Wallet

**Option A: Use Bitcoin Core Wallet**
```bash
# Create wallet
bitcoin-cli -signet createwallet "csv_test"

# Get address
bitcoin-cli -signet getnewaddress "csv_test" "bech32m"

# Fund the wallet (use Signet faucet or mine blocks)
# Wait for confirmation

# Export UTXOs
bitcoin-cli -signet listunspent
```

**Option B: Use External Wallet with XPub**
```bash
# Generate wallet with bitcoin-cli or hardware wallet
# Export the XPub
bitcoin-cli -signet getwalletinfo
```

### 3. Configure Environment Variables

```bash
# Required: RPC endpoint
export CSV_TESTNET_BITCOIN_RPC_URL="http://127.0.0.1:38332"

# Optional: Authentication (if configured)
export CSV_TESTNET_BITCOIN_RPC_USER="testuser"
export CSV_TESTNET_BITCOIN_RPC_PASS="testpass"

# Optional: XPub for funded wallet (if not using random wallet)
export CSV_TESTNET_BITCOIN_XPUB="xpub6CUG..."
```

### 4. Code Example: Real Transaction

```rust
use csv_adapter_bitcoin::{
    BitcoinAnchorLayer, BitcoinConfig, Network, 
    RealBitcoinRpc, BitcoinRpc,
};
use csv_adapter_bitcoin::wallet::SealWallet;
use csv_adapter_core::{Hash, AnchorLayer};
use bitcoin::{Network as BtcNetwork, OutPoint, Txid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create RPC client
    let rpc = RealBitcoinRpc::with_auth(
        "http://127.0.0.1:38332",
        "testuser",
        "testpass",
        BtcNetwork::Signet,
    )?;

    // 2. Get current height
    let height = rpc.get_block_count()?;
    println!("Current Signet height: {}", height);

    // 3. Create wallet (from XPub or random)
    let xpub = std::env::var("CSV_TESTNET_BITCOIN_XPUB")?;
    let wallet = SealWallet::from_xpub(&xpub, BtcNetwork::Signet)?;

    // 4. Create adapter with RPC
    let config = BitcoinConfig {
        network: Network::Signet,
        finality_depth: 6,
        publication_timeout_seconds: 300,
        rpc_url: "http://127.0.0.1:38332".to_string(),
    };

    let adapter = BitcoinAnchorLayer::with_wallet(config, wallet)?
        .with_rpc(rpc);

    // 5. Scan wallet for UTXOs (optional)
    let utxos_found = adapter.scan_wallet_for_utxos(0, 20)?;
    println!("Found {} UTXOs", utxos_found);

    // 6. List UTXOs in wallet
    let utxos = adapter.wallet().list_utxos();
    if utxos.is_empty() {
        panic!("No UTXOs found! Fund the wallet first.");
    }

    // 7. Create seal from real UTXO
    let first_utxo = utxos[0].outpoint;
    let (seal, path) = adapter.fund_seal(first_utxo)?;
    println!("Created seal from UTXO:");
    println!("  TXID: {}", seal.txid_hex());
    println!("  Value: {} sat", seal.nonce.unwrap_or(0));

    // 8. Publish commitment (BROADCASTS REAL TRANSACTION)
    let commitment = Hash::new([0xAB; 32]);
    let anchor = adapter.publish(commitment, seal.clone())?;
    println!("Published commitment!");
    println!("  TXID: {}", hex::encode(anchor.txid));
    println!("  Verify at: https://mempool.space/signet/tx/{}", 
             hex::encode(anchor.txid));

    // 9. Verify finality (after confirmation)
    let finality = adapter.verify_finality(anchor.clone())?;
    println!("Finality: {} confirmations", finality.confirmations);

    Ok(())
}
```

### 5. Manual UTXO Registration

If automatic UTXO discovery fails (e.g., using external wallet), manually register UTXOs:

```rust
use csv_adapter_bitcoin::wallet::Bip86Path;
use bitcoin::{OutPoint, Txid};

// After funding your address and waiting for confirmation:
let txid = Txid::from_str("abc123...")?; // Your funding tx
let outpoint = OutPoint::new(txid, 0);    // vout 0
let amount_sat = 100_000;                 // Amount received
let path = Bip86Path::external(0, 0);     // Derivation path used

adapter.wallet().add_utxo(outpoint, amount_sat, path);

// Now you can create a seal from this UTXO
let (seal, _) = adapter.fund_seal(outpoint)?;
```

### 6. Running the Tests

**Test 1: Real Transaction Lifecycle**
```bash
cargo test -p csv-adapter-bitcoin \
  --test signet_real_tx \
  --features rpc \
  -- --ignored --nocapture
```

**Test 2: UTXO Discovery**
```bash
cargo test -p csv-adapter-bitcoin \
  --test signet_real_tx \
  --test signet_utxo_discovery \
  --features rpc \
  -- --ignored --nocapture
```

**Test 3: Block Verification**
```bash
cargo test -p csv-adapter-bitcoin \
  --test signet_real_tx \
  --test signet_real_block_verification \
  --features rpc \
  -- --ignored --nocapture
```

## Key Methods

### BitcoinAnchorLayer

| Method | Description | Status |
|--------|-------------|--------|
| `with_wallet(config, wallet)` | Create adapter with wallet | ✅ Complete |
| `with_rpc(rpc)` | Attach RPC client for broadcasting | ✅ Complete |
| `fund_seal(outpoint)` | Create seal from real UTXO | ✅ Complete |
| `scan_wallet_for_utxos(account, gap)` | Discover UTXOs on-chain | ✅ Complete |
| `publish(commitment, seal)` | Build + sign + broadcast tx | ✅ Complete |
| `verify_inclusion(anchor)` | Fetch real Merkle proof | ✅ Complete |
| `verify_finality(anchor)` | Check confirmation depth | ✅ Complete |

### SealWallet

| Method | Description | Status |
|--------|-------------|--------|
| `from_xpub(xpub, network)` | Create wallet from XPub | ✅ Complete |
| `generate_random(network)` | Create random wallet | ✅ Complete |
| `add_utxo(outpoint, amount, path)` | Register UTXO | ✅ Complete |
| `scan_chain_for_utxos(fetch_cb, account, gap)` | Scan chain | ✅ Complete |
| `derive_key(path)` | Derive Taproot key | ✅ Complete |
| `list_utxos()` | List all UTXOs | ✅ Complete |
| `balance()` | Total wallet balance | ✅ Complete |

### RealBitcoinRpc

| Method | Description | Status |
|--------|-------------|--------|
| `new(url, network)` | Create RPC client (no auth) | ✅ Complete |
| `with_auth(url, user, pass, network)` | Create with auth | ✅ Complete |
| `get_block_count()` | Get current height | ✅ Complete |
| `get_block_hash(height)` | Get block hash | ✅ Complete |
| `get_block(hash)` | Get full block | ✅ Complete |
| `get_address_utxos(address)` | Get UTXOs for address | ✅ Complete |
| `send_raw_transaction(tx)` | Broadcast transaction | ✅ Complete |
| `extract_merkle_proof(txid, block_hash)` | Get Merkle proof | ✅ Complete |
| `wait_for_confirmation(txid, required, timeout)` | Poll for confirmations | ✅ Complete |

## Common Issues

### Issue: "UTXO not found in wallet"
**Cause:** The seal was created with `create_seal()` which generates synthetic UTXOs.
**Solution:** Use `fund_seal(outpoint)` with a real UTXO that was added to the wallet.

### Issue: "No RPC client configured"
**Cause:** Adapter created without calling `with_rpc()`.
**Solution:** 
```rust
let rpc = RealBitcoinRpc::new(url, network)?;
let adapter = BitcoinAnchorLayer::with_wallet(config, wallet)?
    .with_rpc(rpc);
```

### Issue: UTXO discovery returns 0 UTXOs
**Causes:**
1. Wallet has no on-chain history
2. Bitcoin Core wallet not watching the addresses
3. UTXOs not confirmed yet

**Solutions:**
1. Fund the wallet and wait for confirmation
2. Import addresses into Bitcoin Core:
   ```bash
   bitcoin-cli -signet importdescriptors '[{
     "desc": "addr(bcrt1q...)#checksum",
     "timestamp": "now"
   }]'
   ```
3. Manually add UTXOs with `wallet.add_utxo()`

### Issue: Transaction broadcast fails
**Possible causes:**
1. Invalid signature (wrong key derivation)
2. UTXO already spent
3. Insufficient fees
4. Network connectivity issues

**Debug steps:**
```rust
// Check UTXO is unspent
let is_unspent = rpc.is_utxo_unspent(seal.txid, seal.vout)?;
println!("UTXO unspent: {}", is_unspent);

// Get current mempool minimum fee
let mempool_info = rpc.client.get_mempool_info()?;
println!("Min relay fee: {}", mempool_info.mempoolminfee);
```

## Production Checklist

Before using on mainnet:

- [ ] **Security audit** of transaction signing logic
- [ ] **Fee estimation** implementation (currently uses fixed fees)
- [ ] **Change address handling** (currently no change output)
- [ ] **Multi-UTXO selection** for larger amounts
- [ ] **Transaction retry logic** for failed broadcasts
- [ ] **Proper error handling** for network failures
- [ ] **Wallet backup** and recovery procedures
- [ ] **Multi-signature support** (if needed)
- [ ] **Hardware wallet integration** (if needed)
- [ ] **Mainnet testing** with small amounts first

## Next Steps

After wiring real RPCs:

1. **Sprint 2: Client-Side Validation** - Implement the validation engine
2. **Sprint 3: E2E Testing** - Full integration tests on testnets
3. **Sprint 4: Cross-Chain** - Transfer Rights between chains
4. **Sprint 5: RGB Verification** - Compare against RGB reference
5. **Sprint 6: Security Hardening** - Fuzzing and audit

## Support

- **Documentation:** `README.md` in project root
- **Production Plan:** `docs/PRODUCTION_PLAN.md`
- **API Reference:** `cargo doc --open -p csv-adapter-bitcoin --features rpc`
- **Tests:** `csv-adapter-bitcoin/tests/signet_real_tx.rs`
