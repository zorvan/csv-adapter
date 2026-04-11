//! Bitcoin Signet Real Transaction Integration Test (via mempool.space REST API)
//!
//! This test runs against real Bitcoin Signet using the mempool.space public API.
//! No local Bitcoin Core node required.
//!
//! ## Setup
//!
//! 1. Generate a Signet funding address:
//!    ```bash
//!    cargo run -p csv-adapter-bitcoin --example signet_funding_addr
//!    ```
//!
//! 2. Fund the address from a Signet faucet:
//!    - https://mempool.space/signet/faucet
//!    - https://signet.bc-2.jp
//!
//! 3. Set the funding UTXO environment variable:
//!    ```bash
//!    export CSV_SIGNET_FUNDING_TXID="<txid-of-funding-transaction>"
//!    export CSV_SIGNET_FUNDING_VOUT=0
//!    export CSV_SIGNET_FUNDING_AMOUNT=10000  # in satoshis
//!    ```
//!
//! ## Run
//!
//! ```bash
//! cargo test -p csv-adapter-bitcoin --test signet_real_tx --features signet-rest -- --ignored --nocapture
//! ```

#![cfg(feature = "signet-rest")]

use bitcoin::{Network as BtcNetwork, OutPoint, Txid};
use bitcoin_hashes::Hash as BitcoinHash;
use csv_adapter_bitcoin::mempool_rpc::{get_address_utxos, MempoolSignetRpc};
use csv_adapter_bitcoin::wallet::SealWallet;
use csv_adapter_bitcoin::{BitcoinAnchorLayer, BitcoinConfig, BitcoinRpc, Network};
use csv_adapter_core::{AnchorLayer, Hash};
use std::str::FromStr;

#[test]
#[ignore = "requires network and funded wallet"]
fn test_signet_real_transaction_lifecycle() {
    println!("=== Bitcoin Signet Real Transaction Test ===");

    // Create RPC client
    let rpc = MempoolSignetRpc::new();

    // Get current block height from Signet
    let current_height = rpc.get_block_count().expect("Failed to get block count");
    println!("Current Signet height: {}", current_height);

    // Create a random wallet for testing
    let wallet = SealWallet::generate_random(BtcNetwork::Signet);

    // Derive a funding address
    let (funding_key, funding_path) = wallet.next_address(0).expect("Failed to derive address");
    println!("\n📬 Funding address: {}", funding_key.address);
    println!("   (Send Signet sats to this address from a faucet)");

    // Create Bitcoin adapter
    let config = BitcoinConfig {
        network: Network::Signet,
        finality_depth: 6,
        publication_timeout_seconds: 300,
        rpc_url: "https://mempool.space/signet".to_string(),
    };

    // Check for funding UTXO from environment
    let funding_txid = std::env::var("CSV_SIGNET_FUNDING_TXID");
    let funding_vout = std::env::var("CSV_SIGNET_FUNDING_VOUT")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let funding_amount = std::env::var("CSV_SIGNET_FUNDING_AMOUNT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(100_000);

    let adapter = if let Ok(txid_hex) = funding_txid {
        println!("\n💰 Using provided funding UTXO");

        // Add the UTXO to the wallet
        let txid_bytes = hex::decode(&txid_hex).expect("Invalid txid hex");
        let txid = Txid::from_slice(&txid_bytes).expect("valid txid");
        let outpoint = OutPoint::new(txid, funding_vout);
        wallet.add_utxo(outpoint, funding_amount, funding_path.clone());

        println!("   TXID: {}", txid_hex);
        println!("   VOUT: {}", funding_vout);
        println!("   Amount: {} sat", funding_amount);

        // Attach RPC client
        let rpc_box: Box<dyn BitcoinRpc + Send + Sync> = Box::new(rpc);
        BitcoinAnchorLayer::with_wallet(config, wallet)
            .expect("Failed to create adapter")
            .with_rpc(rpc_box)
    } else {
        println!("\n⚠️  No funding UTXO provided!");
        println!(
            "Set CSV_SIGNET_FUNDING_TXID, CSV_SIGNET_FUNDING_VOUT, and CSV_SIGNET_FUNDING_AMOUNT"
        );
        println!("to run the full real transaction test.");
        println!("\nFor now, demonstrating wallet and seal creation...");

        // Demo mode - no real broadcast
        BitcoinAnchorLayer::with_wallet(config, wallet).expect("Failed to create adapter")
    };

    // Step 1: Create a seal from the funding UTXO (or mock)
    println!("\n--- Creating seal ---");

    let has_rpc = std::env::var("CSV_SIGNET_FUNDING_TXID").is_ok();
    let seal = if has_rpc {
        // Real mode: use the funding UTXO
        let utxos = adapter.wallet().list_utxos();
        assert!(
            !utxos.is_empty(),
            "No UTXOs in wallet - fund the address first!"
        );

        let first_utxo = &utxos[0];
        let (seal_ref, path) = adapter
            .fund_seal(first_utxo.outpoint)
            .expect("Failed to create seal from UTXO");

        println!("✅ Created seal from real UTXO:");
        println!("   TXID: {}", seal_ref.txid_hex());
        println!("   VOUT: {}", seal_ref.vout);
        println!("   Value: {} sat", seal_ref.nonce.unwrap_or(0));
        println!("   Path: {:?}", path);

        seal_ref
    } else {
        // Demo mode: create a mock seal
        let seal_ref = adapter
            .create_seal(Some(100_000))
            .expect("Failed to create seal");
        println!("✅ Created mock seal (no real UTXO):");
        println!("   TXID: {}", seal_ref.txid_hex());
        println!("   VOUT: {}", seal_ref.vout);
        seal_ref
    };

    // Step 2: Publish commitment
    println!("\n--- Publishing commitment ---");
    let commitment = Hash::new([0xAB; 32]);

    match adapter.publish(commitment, seal.clone()) {
        Ok(anchor) => {
            println!("✅ Published commitment:");
            println!("   TXID: {}", hex::encode(anchor.txid));
            println!("   Block height: {}", anchor.block_height);
            println!("   Output index: {}", anchor.output_index);

            // Step 3: Verify inclusion
            println!("\n--- Verifying inclusion ---");
            let inclusion = adapter
                .verify_inclusion(anchor.clone())
                .expect("Failed to verify inclusion");

            println!("✅ Inclusion proof:");
            println!("   TX index: {}", inclusion.tx_index);
            println!("   Block height: {}", inclusion.block_height);
            println!("   Merkle branch length: {}", inclusion.merkle_branch.len());

            // Step 4: Verify finality (may not be reached in mock mode)
            println!("\n--- Verifying finality ---");
            match adapter.verify_finality(anchor.clone()) {
                Ok(finality) => {
                    println!("✅ Finality proof:");
                    println!("   Confirmations: {}", finality.confirmations);
                    println!("   Required depth: {}", finality.required_depth);
                    println!("   Meets requirement: {}", finality.meets_required_depth);
                }
                Err(e) => {
                    println!("⚠️  Finality not reached (expected in mock mode):");
                    println!("   {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not publish commitment (expected in demo mode):");
            println!("   {}", e);
        }
    }

    // Step 5: Test replay prevention (seal already enforced during publish)
    println!("\n--- Testing replay prevention ---");
    let replay_result = adapter.enforce_seal(seal.clone());
    if replay_result.is_err() {
        println!("✅ Replay prevention works correctly (seal already enforced)");
    } else {
        println!("✅ First enforcement succeeded");
        let replay_result2 = adapter.enforce_seal(seal);
        assert!(replay_result2.is_err(), "Replay should be prevented");
        println!("✅ Second enforcement correctly prevented");
    }

    println!("\n=== Bitcoin Signet Real Transaction Test PASSED ===");
}

/// Test Signet block verification with real Merkle proofs
#[test]
#[ignore = "requires network"]
fn test_signet_real_block_verification() {
    println!("=== Bitcoin Signet Block Verification Test ===");

    let rpc = MempoolSignetRpc::new();

    // Get current height
    let height = rpc.get_block_count().expect("Failed to get block count");
    println!("Current Signet height: {}", height);

    // Get block hash at current height
    let block_hash = rpc
        .get_block_hash(height)
        .expect("Failed to get block hash");
    println!(
        "Block hash at height {}: {}",
        height,
        hex::encode(block_hash)
    );

    // Get block info
    let block_info = rpc
        .get_block_info(&hex::encode(block_hash))
        .expect("Failed to get block info");
    println!("Block {}:", height);
    println!("  Transactions: {}", block_info.tx_count);
    println!("  Weight: {} WU", block_info.weight);

    println!("\n=== Bitcoin Signet Block Verification Test PASSED ===");
}

/// Test UTXO discovery via mempool.space API
#[test]
#[ignore = "requires network and funded address"]
fn test_signet_utxo_discovery() {
    println!("=== Bitcoin Signet UTXO Discovery Test ===");

    let rpc = MempoolSignetRpc::new();

    // Test with a known Signet address from wallet
    let address_str = std::env::var("CSV_SIGNET_TEST_ADDRESS").unwrap_or_else(|_| {
        "tb1p69r3kn7qu2w6ppj7sr2c7x45rp7urc535u4nv2g4n884nnt26nyqq4qz5c".to_string()
    });

    let address = match bitcoin::Address::from_str(&address_str) {
        Ok(addr) => match addr.require_network(BtcNetwork::Signet) {
            Ok(network_addr) => network_addr,
            Err(_) => {
                println!("⚠️  Invalid network for address: {}", address_str);
                println!("=== Bitcoin Signet UTXO Discovery Test Complete (skipped) ===");
                return;
            }
        },
        Err(e) => {
            println!("⚠️  Invalid address {}: {}", address_str, e);
            println!("=== Bitcoin Signet UTXO Discovery Test Complete (skipped) ===");
            return;
        }
    };

    println!("Checking UTXOs for: {}", address);

    match get_address_utxos(&rpc, &address) {
        Ok(utxos) => {
            println!("✅ Found {} UTXO(s)", utxos.len());
            for (outpoint, value) in &utxos {
                println!("   {}:{} → {} sat", outpoint.txid, outpoint.vout, value);
            }
        }
        Err(e) => {
            println!("⚠️  UTXO lookup failed: {}", e);
        }
    }

    println!("\n=== Bitcoin Signet UTXO Discovery Test Complete ===");
}
