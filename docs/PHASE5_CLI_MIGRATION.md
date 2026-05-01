# Phase 5 CLI Migration Plan

## Current Violation

The CLI currently violates **Phase 5: CLI and Wallet Convergence (Unified Facade)**.

### Specific Violations

1. **Direct chain adapter dependencies** in `csv-cli/Cargo.toml`:
   ```toml
   csv-adapter-bitcoin = { path = "../csv-adapter-bitcoin" }
   csv-adapter-ethereum = { path = "../csv-adapter-ethereum" }
   csv-adapter-sui = { path = "../csv-adapter-sui" }
   csv-adapter-aptos = { path = "../csv-adapter-aptos" }
   csv-adapter-solana = { path = "../csv-adapter-solana" }
   ```

2. **Direct cryptographic dependencies** (should be via adapters only):
   ```toml
   bitcoin = { version = "0.32" }
   ed25519-dalek = { version = "2.0" }
   secp256k1 = { version = "0.28" }
   ```

3. **Chain-specific modules** implementing their own logic:
   - `csv-cli/src/commands/cross_chain/bitcoin.rs`
   - `csv-cli/src/commands/cross_chain/ethereum.rs`
   - `csv-cli/src/commands/cross_chain/aptos.rs`
   - `csv-cli/src/commands/cross_chain/sui.rs`
   - `csv-cli/src/commands/cross_chain/solana.rs`
   - `csv-cli/src/commands/cross_chain_impl.rs`

4. **Direct private key handling** in CLI instead of using keystore facade

## Required State

Per PRODUCTION_GUARANTEE_PLAN.md Phase 5:

> CLI and wallet must use csv-adapter facade APIs only. No direct chain adapter imports.

### Target Architecture

```
csv-cli
└── csv-adapter (facade only)
    ├── csv-adapter-core
    ├── csv-adapter-bitcoin
    ├── csv-adapter-ethereum
    ├── csv-adapter-sui
    ├── csv-adapter-aptos
    └── csv-adapter-solana
```

CLI should only import:
- `csv-adapter` (unified facade)
- `csv-adapter-core` (core types)
- `csv-adapter-store` (persistence)
- `csv-adapter-keystore` (key management)

## Migration Steps

### Step 1: Move Chain-Specific Logic to Adapters

Move implementation from CLI modules to chain adapter facade methods:

| CLI Module | Target Adapter Method |
|------------|----------------------|
| `cross_chain/bitcoin.rs` | `csv_adapter_bitcoin::BitcoinAnchorLayer::lock_seal()` |
| `cross_chain/ethereum.rs` | `csv_adapter_ethereum::EthereumAnchorLayer::mint_from_lock()` |
| `cross_chain/aptos.rs` | `csv_adapter_aptos::AptosAnchorLayer::mint_from_lock()` |
| `cross_chain_impl.rs` | `csv_adapter::transfers::CrossChainManager` |

### Step 2: Refactor CLI to Use Facade

Replace direct chain calls with facade API:

```rust
// BEFORE (violates Phase 5)
use csv_adapter_bitcoin::BitcoinAnchorLayer;
let adapter = BitcoinAnchorLayer::new(config)?;
adapter.publish(seal).await?;

// AFTER (Phase 5 compliant)
use csv_adapter::CsvClient;
let client = CsvClient::builder()
    .with_chain(Chain::Bitcoin)
    .build()?;
client.transfers().lock_seal(seal).await?;
```

### Step 3: Remove Direct Dependencies

Update `csv-cli/Cargo.toml`:

```toml
[dependencies]
# Keep only these:
csv-adapter = { path = "../csv-adapter" }
csv-adapter-core = { path = "../csv-adapter-core" }
csv-adapter-store = { path = "../csv-adapter-store" }
csv-adapter-keystore = { path = "../csv-adapter-keystore" }

# Remove all direct chain adapters:
# csv-adapter-bitcoin = ...
# csv-adapter-ethereum = ...
# etc.

# Remove all direct crypto crates:
# bitcoin = ...
# ed25519-dalek = ...
# secp256k1 = ...
```

### Step 4: Private Key Handling

Move all private key operations to `csv-adapter-keystore`:

```rust
// BEFORE
let key_bytes = hex::decode(private_key_hex)?;
let signing_key = SigningKey::from_bytes(&key_array);

// AFTER
let keystore = client.keystore();
let signer = keystore.get_signer(chain, address)?;
```

## Compliance Status

| Requirement | Status |
|-------------|--------|
| CLI uses facade only | ❌ Violation - direct adapter deps |
| No duplicate chain logic | ❌ Violation - `cross_chain_impl.rs` |
| No direct crypto in CLI | ❌ Violation - `ed25519-dalek`, `secp256k1` in CLI |
| Keystore facade for keys | ⚠️ Partial - dev mode fallback removed |

## Temporary Exception

Until the full migration is complete, the CLI **documents** this violation with:

1. Deprecation warnings on all chain-specific modules
2. Comments marking violations with `// PHASE5-TODO: Migrate to facade`
3. This migration plan document

## Review Schedule

- **Target:** Full compliance by v0.5.0
- **Review Date:** 2026-06-01
- **Owner:** Core maintainer team

---
*Document Version: 1.0*
*Created: 2026-05-01*
