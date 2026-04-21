# CSV Wallet Test Plan

## Current Test Coverage

### Native Rust Tests ✅
- **Unit tests**: Chain API, serialization, business logic
- **Smoke tests**: Chain enum validation, trait checks
- **Build checks**: Clippy, compilation

### What Native Tests CANNOT Catch ⚠️

Native tests run in a standard Rust environment and **cannot detect**:

1. **Context Provider Panics** - Missing `BalanceProvider`, `WalletProvider`, etc.
2. **Component Rendering Errors** - Dioxus component tree issues
3. **Signal/Hook Misuse** - Borrow checker violations in WASM
4. **Browser API Failures** - localStorage, web-sys, etc.
5. **Provider Hierarchy Issues** - Context not found at runtime

## Why Runtime Crashes Happen

The crash you experienced:
```
Could not find context csv_wallet::hooks::use_balance::BalanceContext
```

This happens because:
1. `BalanceContext` is provided by `BalanceProvider` component
2. If a component calls `use_balance()` but no ancestor provided `BalanceProvider`, Dioxus panics
3. This only happens at **runtime in WASM**, not at compile time

## Future: Browser Testing Strategy

### Option 1: wasm-pack test (Recommended)

Add to `Cargo.toml`:
```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
wasm-pack = "0.12"
```

Create `tests/wasm_integration_test.rs`:
```rust
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_app_renders_without_context_panic() {
    // This will panic if providers are missing
    dioxus::launch(csv_wallet::App);
}

#[wasm_bindgen_test]
fn test_dashboard_loads() {
    // Navigate to dashboard and verify it loads
    // Use web-sys to query DOM
}
```

Run with:
```bash
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
```

### Option 2: Playwright E2E Tests

Create `e2e/` directory with Playwright tests:
```javascript
// e2e/wallet.spec.js
test('wallet loads without runtime errors', async ({ page }) => {
  // Capture console errors
  const errors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') errors.push(msg.text());
  });

  await page.goto('http://localhost:8080');
  
  // Wait for app to load
  await page.waitForSelector('[data-testid="dashboard"]', { timeout: 5000 });
  
  // Assert no panics
  expect(errors).not.toContain(expect.stringMatching(/panicked/));
});
```

### Option 3: Trunk + Web Driver Testing

For Dioxus Web specifically:
```bash
cargo install trunk wasm-bindgen-cli

# Build for test
trunk build --release

# Run webdriver tests
cargo test --target wasm32-unknown-unknown
```

## Implementation Checklist

### Phase 1: wasm-bindgen-test (Immediate)
- [ ] Add `wasm-bindgen-test` to dev-dependencies
- [ ] Create `tests/wasm_render_test.rs`
- [ ] Add CI job for `wasm-pack test --headless --chrome`

### Phase 2: E2E Testing (Short-term)
- [ ] Set up Playwright
- [ ] Create basic navigation tests
- [ ] Add console error monitoring
- [ ] Test provider context availability

### Phase 3: Full Integration (Medium-term)
- [ ] Test wallet import/export
- [ ] Test balance fetching
- [ ] Test cross-chain transfer flow
- [ ] Test all chain integrations

## Current Smoke Test Status

The current smoke tests in `tests/render_smoke_test.rs` provide:

| Test | Coverage | Runtime |
|------|----------|---------|
| Chain enum validation | ✅ Native | Instant |
| Trait implementations | ✅ Native | Instant |
| Chain ID strings | ✅ Native | Instant |
| Chain parsing | ✅ Native | Instant |
| Component rendering | ❌ Requires WASM | N/A |
| Context providers | ❌ Requires WASM | N/A |
| Browser APIs | ❌ Requires WASM | N/A |

## Running Tests

```bash
# Native tests (fast, runs in CI)
cargo test -p csv-wallet

# Smoke tests
cargo test -p csv-wallet --test render_smoke_test

# Future: Browser tests (catches runtime panics)
wasm-pack test --headless --chrome
cd e2e && npx playwright test
```

## Key Takeaway

**Rust unit tests alone cannot catch Dioxus context/Provider errors.**

You need browser-based testing for:
- Provider hierarchy validation
- Component lifecycle issues  
- WASM-specific runtime panics
- Browser API integration

The runtime panic you experienced (`Could not find context BalanceContext`) requires a WASM test environment to catch before production.
