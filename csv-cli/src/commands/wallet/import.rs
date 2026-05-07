//! Wallet import from mnemonic phrase.
//!
//! Imports a mnemonic phrase (from csv-wallet or other source) and derives
//! all chain accounts using BIP-44 derivation. Keys are stored in the
//! encrypted file keystore.

use crate::config::{Chain, Config, Network};
use crate::output;
use crate::state::UnifiedStateManager;
use anyhow::Result;
use csv_core::Chain as CoreChain;
use csv_keys::{
    file_keystore::FileKeystore,
    bip39::Mnemonic,
    bip44::{derive_all_chain_keys, derive_address_from_key},
    memory::Passphrase,
};
use csv_store::state::WalletAccount;

/// Import wallet from mnemonic phrase.
pub fn cmd_import(
    phrase: &str,
    _network: Network,
    account: u32,
    _config: &Config,
    state: &mut UnifiedStateManager,
) -> Result<()> {
    output::header("Importing Wallet from Mnemonic");

    // Prompt for passphrase (to encrypt imported keys)
    let passphrase = prompt_passphrase("Enter keystore passphrase (min 8 chars)")?;
    if passphrase.len() < 8 {
        anyhow::bail!("Passphrase must be at least 8 characters");
    }
    let passphrase = Passphrase::new(passphrase);

    // Validate and parse mnemonic
    let mnemonic = Mnemonic::from_phrase(phrase)
        .map_err(|e| anyhow::anyhow!("Invalid mnemonic phrase: {}", e))?;

    output::success("Mnemonic validated");

    // Convert to seed
    let seed = mnemonic.to_seed(None);
    let mut seed_array = [0u8; 64];
    seed_array.copy_from_slice(seed.as_bytes());

    // Initialize file keystore
    let mut keystore = FileKeystore::new(None)?;

    // Derive keys for all chains
    let chain_keys = derive_all_chain_keys(&seed_array, account);

    output::info("Deriving addresses for all chains...");

    let mut imported = 0u32;
    for (core_chain, secret_key) in chain_keys {
        // Derive address
        let address = derive_address_from_key(secret_key.as_bytes(), core_chain)
            .map_err(|e| anyhow::anyhow!("Failed to derive address for {:?}: {}", core_chain, e))?;

        // Get coin type for derivation path
        let coin_type = match core_chain {
            CoreChain::Bitcoin => "0",
            CoreChain::Ethereum => "60",
            CoreChain::Sui => "784",
            CoreChain::Aptos => "637",
            CoreChain::Solana => "501",
        };
        let derivation_path = format!("m/44'/{}'/{}'/0/0", coin_type, account);

        // Store account
        let store_chain = match core_chain {
            CoreChain::Bitcoin => csv_store::state::Chain::Bitcoin,
            CoreChain::Ethereum => csv_store::state::Chain::Ethereum,
            CoreChain::Sui => csv_store::state::Chain::Sui,
            CoreChain::Aptos => csv_store::state::Chain::Aptos,
            CoreChain::Solana => csv_store::state::Chain::Solana,
        };

        // Store private key in encrypted file keystore
        let key_id = format!("{}-{}", store_chain.to_string().to_lowercase(), account);
        keystore.store_key(
            &key_id,
            &store_chain.to_string().to_lowercase(),
            Some(&format!("{} Account (imported)", store_chain)),
            &secret_key,
            &passphrase,
        )?;

        state.set_account(WalletAccount {
            id: format!("imported-{}", store_chain),
            chain: store_chain.clone(),
            name: format!("{} Account (imported)", store_chain),
            address: address.clone(),
            keystore_ref: Some(key_id.clone()),
            xpub: None,
            derivation_path: Some(derivation_path),
        });

        output::success(&format!("{}: {}", store_chain, address));
        imported += 1;
    }

    // Store mnemonic in unified storage (encrypted in keystore)
    state.storage.wallet.mnemonic = Some(phrase.to_string());

    // Save state
    state.save()?;

    output::success(&format!("Imported {} chain accounts", imported));
    output::info("Wallet imported successfully. Keys are encrypted in the file keystore.");

    Ok(())
}

/// Prompt user for a passphrase with confirmation.
fn prompt_passphrase(prompt: &str) -> Result<String> {
    use std::io::{self, Write};

    print!("{}: ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
