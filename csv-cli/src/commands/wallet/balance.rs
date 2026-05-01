//! Wallet balance checking commands (Phase 5 Compliant).
//!
//! Uses csv-adapter facade APIs only - no direct chain adapter dependencies.

use crate::config::{Chain, Config};
use crate::output;
use crate::state::UnifiedStateManager;
use anyhow::Result;

use csv_adapter::CsvClient;

/// Check balance for a specific chain.
pub fn cmd_balance(
    chain: Chain,
    address: Option<String>,
    config: &Config,
    state: &mut UnifiedStateManager,
) -> Result<()> {
    let address = address.or_else(|| state.get_address(&chain).map(|s| s.to_string()));

    if let Some(addr) = address {
        output::header(&format!("{} Balance", chain));
        output::kv("Address", &addr);

        // Query balance from chain using csv-adapter facade
        match query_balance(&chain, &addr, config) {
            Ok(balance) => {
                output::kv("Balance", &format!("{} {}", balance, chain_symbol(&chain)));
            }
            Err(e) => {
                output::error(&format!("Failed to query balance: {}", e));
                output::info("Balance query requires chain RPC to be configured");
            }
        }
    } else {
        output::warning(&format!("No {} address found in wallet", chain));
        output::info(&format!(
            "Generate one with: csv wallet generate --chain {}",
            chain
        ));
    }

    Ok(())
}

/// List all wallets.
pub fn cmd_list(_config: &Config, state: &mut UnifiedStateManager) -> Result<()> {
    output::header("Wallet Addresses");

    let chains = vec![
        Chain::Bitcoin,
        Chain::Ethereum,
        Chain::Sui,
        Chain::Aptos,
        Chain::Solana,
    ];

    let mut found_any = false;
    for chain in chains {
        if let Some(address) = state.get_address(&chain) {
            output::kv(&format!("{}", chain), &address);
            found_any = true;
        }
    }

    if !found_any {
        output::warning("No wallets found");
        output::info("Generate wallets with: csv wallet generate --chain <chain>");
        output::info("Or use one-command setup: csv wallet init");
    }

    Ok(())
}

/// Query balance from chain using facade API.
fn query_balance(chain: &Chain, address: &str, config: &Config) -> Result<f64> {
    // Phase 5: Use facade client for balance queries
    let core_chain = match chain {
        Chain::Bitcoin => csv_adapter_core::Chain::Bitcoin,
        Chain::Ethereum => csv_adapter_core::Chain::Ethereum,
        Chain::Sui => csv_adapter_core::Chain::Sui,
        Chain::Aptos => csv_adapter_core::Chain::Aptos,
        Chain::Solana => csv_adapter_core::Chain::Solana,
    };

    // Create facade client
    let client = CsvClient::builder()
        .with_chain(core_chain)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create CSV client: {}", e))?;

    // Query balance via facade wallet manager if available
    match client.wallet() {
        Ok(wallet) => {
            // Use wallet manager to query balance
            // Note: This requires the wallet to be properly configured
            output::info(&format!("Querying balance for {} on {:?}", address, core_chain));
            output::info("Balance query requires chain adapter integration via facade.");

            // Return placeholder - actual implementation would call wallet.get_balance()
            Err(anyhow::anyhow!(
                "Balance query not yet implemented via facade. \
                Use direct RPC or chain explorer for now."
            ))
        }
        Err(_) => {
            // No wallet configured - return error
            Err(anyhow::anyhow!(
                "No wallet configured. Use 'csv wallet generate' to create one."
            ))
        }
    }
}

/// Get symbol for chain.
fn chain_symbol(chain: &Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "BTC",
        Chain::Ethereum => "ETH",
        Chain::Sui => "SUI",
        Chain::Aptos => "APT",
        Chain::Solana => "SOL",
    }
}
