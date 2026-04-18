# Scalable Chain Architecture for 20+ Chains

## Current Problem (Demonstrated by Solana Integration)

Adding **one chain (Solana)** required touching **15+ files** with **40+ manual edits**:

```
csv-cli/src/config.rs              - Chain enum + Display
csv-cli/src/commands/wallet.rs     - 8 match arms + 4 functions
csv-cli/src/commands/contracts.rs  - 1 match arm + deploy_solana()
csv-cli/src/commands/chain.rs      - 1 match arm
csv-cli/src/commands/proofs.rs     - 2 match arms
csv-cli/src/commands/seals.rs      - 2 match arms
csv-cli/src/commands/tests.rs      - 1 match arm
csv-cli/src/commands/cross_chain.rs - 6 match arms
csv-wallet/src/services/chain_api.rs - ChainConfig + get_balance
csv-wallet/src/core/key_manager.rs   - derive_solana_keys()
csv-wallet/src/pages/mod.rs        - chain_color, chain_icon, chain_name
```

**For 20 chains**: 300+ match arms to maintain. This doesn't scale.

---

## Proposed Architecture: Plugin-Based Chain System

### 1. Chain Registration via Trait System

Replace hardcoded `Chain` enum with a **dynamic registry**:

```rust
// csv-adapter-core/src/chain_plugin.rs
pub trait ChainPlugin: Send + Sync {
    fn chain_id(&self) -> &'static str;
    fn chain_name(&self) -> &'static str;
    fn coin_type(&self) -> u32; // BIP-44 coin type
    
    // Wallet integration
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, KeyError>;
    
    // Chain API integration
    fn api_config(&self, network: NetworkType) -> ChainApiConfig;
    fn fetch_balance(&self, address: &str) -> BoxFuture<'_, Result<f64, ChainApiError>>;
    
    // Contract deployment (optional)
    fn deploy_contract(&self, config: &Config) -> Result<(), DeployError>;
    fn has_contracts(&self) -> bool;
    
    // UI styling
    fn ui_style(&self) -> ChainUiStyle;
}

pub struct ChainUiStyle {
    pub color_hex: &'static str,
    pub icon_emoji: &'static str,
    pub badge_class: &'static str,
}

pub struct ChainRegistry {
    plugins: HashMap<String, Box<dyn ChainPlugin>>,
}

impl ChainRegistry {
    pub fn register(&mut self, plugin: Box<dyn ChainPlugin>) {
        self.plugins.insert(plugin.chain_id().to_string(), plugin);
    }
    
    pub fn get(&self, chain_id: &str) -> Option<&dyn ChainPlugin> {
        self.plugins.get(chain_id).map(|b| b.as_ref())
    }
    
    pub fn all_chains(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}
```

### 2. Auto-Discovered Chain Modules

Use Rust's `inventory` or `linkme` crates for auto-registration:

```rust
// csv-adapter-solana/src/lib.rs
inventory::submit! {
    ChainPluginRegistration(SolanaPlugin::new)
}

// In main adapter, auto-collect all plugins:
fn load_all_plugins() -> ChainRegistry {
    let mut registry = ChainRegistry::new();
    for registration in inventory::iter::<ChainPluginRegistration> {
        registry.register((registration.0)());
    }
    registry
}
```

### 3. Data-Driven CLI Configuration

Replace manual `Chain` enum with **configuration-driven** chains:

```rust
// csv-cli/src/config.rs - Simplified
pub struct ChainConfig {
    pub chain_id: String,  // "solana", "ethereum", etc.
    pub rpc_url: String,
    pub network: Network,
}

// CLI uses dynamic dispatch instead of match arms
pub async fn cmd_balance(chain_id: &str, address: Option<String>) -> Result<()> {
    let plugin = REGISTRY.get(chain_id)
        .ok_or_else(|| anyhow!("Unknown chain: {}", chain_id))?;
    
    let addr = address.unwrap_or_else(|| {
        // Use plugin to derive from stored mnemonic
        plugin.derive_address(&load_seed())
    })?;
    
    let balance = plugin.fetch_balance(&addr).await?;
    output::success(&format!("Balance: {} {}", balance, plugin.chain_id()));
    Ok(())
}
```

### 4. Feature-Gated Chain Compilation

Keep compile-time feature flags:

```toml
# csv-adapter/Cargo.toml
[features]
default = []
bitcoin = ["dep:csv-adapter-bitcoin"]
etereum = ["dep:csv-adapter-ethereum"]
solana = ["dep:csv-adapter-solana"]
all-chains = ["bitcoin", "ethereum", "sui", "aptos", "solana"]
```

```rust
// csv-adapter/src/lib.rs
#[cfg(feature = "solana")]
use csv_adapter_solana::SolanaPlugin;

fn register_enabled_chains(registry: &mut ChainRegistry) {
    #[cfg(feature = "bitcoin")]
    registry.register(Box::new(BitcoinPlugin::new()));
    #[cfg(feature = "ethereum")]
    registry.register(Box::new(EthereumPlugin::new()));
    #[cfg(feature = "solana")]
    registry.register(Box::new(SolanaPlugin::new()));
    // ... etc
}
```

### 5. Chains.toml Configuration

Support runtime chain addition via config:

```toml
# chains.toml - Can add new chains without recompiling
[[chain]]
id = "solana"
name = "Solana"
coin_type = 501
color = "#9945FF"
icon = "◈"
has_contracts = true

[chain.api.devnet]
url = "https://api.devnet.solana.com"
[chain.api.mainnet]
url = "https://api.mainnet-beta.solana.com"

[[chain]]
id = "near"
name = "NEAR Protocol"
coin_type = 397
# ... dynamically loaded
```

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
1. Create `ChainPlugin` trait in `csv-adapter-core`
2. Implement `ChainRegistry` with plugin storage
3. Add `inventory` dependency for auto-registration

### Phase 2: Migrate Existing Chains (Week 3-4)
1. Convert `csv-adapter-bitcoin` to `BitcoinPlugin`
2. Convert `csv-adapter-ethereum` to `EthereumPlugin`
3. Convert `csv-adapter-solana` to `SolanaPlugin`
4. Convert `csv-adapter-sui` to `SuiPlugin`
5. Convert `csv-adapter-aptos` to `AptosPlugin`

### Phase 3: CLI Migration (Week 5-6)
1. Replace `Chain` enum with `String` chain_id
2. Convert all `match chain { ... }` to registry dispatch
3. Remove 40+ match arms, replace with 5 plugin impls

### Phase 4: Wallet Integration (Week 7-8)
1. Migrate `key_manager.rs` to use plugins
2. Migrate `chain_api.rs` to use plugins
3. Migrate UI helpers to use `ChainUiStyle`

### Phase 5: New Chain Onboarding (Week 9+)
1. Add a 6th chain (e.g., NEAR, Avalanche) to validate
2. Measure: should require only **2 file changes**:
   - Add feature flag in Cargo.toml
   - Add `register!(NearPlugin)` in plugin init

---

## Comparison: Old vs New Architecture

| Task | Old (Hardcoded) | New (Plugin) |
|------|-----------------|--------------|
| Add 1 new chain | 15 files, 40 edits | 2 files, 3 edits |
| Add 20 chains | 300 match arms | Same 2 files |
| Remove a chain | Delete from 15 files | Remove 1 feature flag |
| Chain-specific RPC | Match arm per chain | Plugin method |
| Key derivation | Match arm per chain | Plugin method |
| UI styling | Match arm per chain | Plugin data |

---

## Migration Example: Solana

**Before (Current)**:
```rust
// 8 different files with match arms:
match chain {
    Chain::Bitcoin => ...,
    Chain::Ethereum => ...,
    Chain::Solana => { /* added manually */ },
}
```

**After (Plugin System)**:
```rust
// csv-adapter-solana/src/plugin.rs only
pub struct SolanaPlugin;

impl ChainPlugin for SolanaPlugin {
    fn chain_id(&self) -> &'static str { "solana" }
    fn chain_name(&self) -> &'static str { "Solana" }
    fn coin_type(&self) -> u32 { 501 }
    
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, KeyError> {
        // Solana-specific ed25519 derivation
        let keypair = ed25519_dalek::SigningKey::from_bytes(&seed[..32]);
        Ok(bs58::encode(keypair.verifying_key()).into_string())
    }
    
    fn ui_style(&self) -> ChainUiStyle {
        ChainUiStyle {
            color_hex: "#9945FF",
            icon_emoji: "◈",
            badge_class: "solana-badge",
        }
    }
}

inventory::submit!(ChainPluginRegistration(|| Box::new(SolanaPlugin)));
```

---

## Files to Modify for Migration

### Core (csv-adapter-core)
- [ ] `src/chain_plugin.rs` - NEW: Define traits
- [ ] `src/lib.rs` - Export traits

### Adapter (csv-adapter)
- [ ] `src/lib.rs` - Auto-register all enabled plugins
- [ ] `Cargo.toml` - Add inventory dependency

### Per-Chain Adapter (csv-adapter-solana, etc.)
- [ ] `src/plugin.rs` - NEW: Implement ChainPlugin
- [ ] `src/lib.rs` - Submit to inventory

### CLI (csv-cli)
- [ ] `src/config.rs` - Remove Chain enum, use String
- [ ] `src/commands/*.rs` - Replace match arms with registry calls

### Wallet (csv-wallet)
- [ ] `src/services/chain_api.rs` - Use plugin for balance/API
- [ ] `src/core/key_manager.rs` - Use plugin for derivation
- [ ] `src/pages/mod.rs` - Use plugin for styling

---

## Success Metrics

- [ ] Add NEAR chain in **< 30 minutes** (currently takes days)
- [ ] Total lines of code **reduced by 30%**
- [ ] Adding/removing chains requires **zero changes** to CLI/wallet code
- [ ] New chains can be **runtime-loaded** from chains.toml (optional)

---

## Conclusion

The current architecture requires **O(n*m)** changes where n=chains and m=integration points. The plugin architecture requires **O(n)** changes - just implement the trait for each new chain.

**For 20 chains: 300 edits → 20 plugin implementations.**
