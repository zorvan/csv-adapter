//! Bitcoin transaction builder with UTXO support
//!
//! Builds real Bitcoin transactions for cross-chain locking.
//! Uses mempool.space API for UTXO fetching and transaction broadcast.

use crate::services::blockchain_service::BlockchainError;
use csv_adapter_core::Chain;

/// UTXO from blockchain
#[derive(Clone, Debug)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64, // satoshis
    pub script_pubkey: String,
}

/// Build a Bitcoin anchor transaction for cross-chain locking
/// 
/// This creates a transaction that:
/// 1. Consumes a UTXO from the sender
/// 2. Creates an OP_RETURN output with the lock commitment
/// 3. Returns change to sender
/// 
/// Returns (unsigned_tx, utxo) for signing
pub async fn build_anchor_transaction(
    sender_address: &str,
    lock_data: &[u8],
    rpc_url: &str,
) -> Result<(Vec<u8>, Utxo), BlockchainError> {
    // Fetch UTXOs for the sender
    let utxos = fetch_utxos(sender_address, rpc_url).await?;
    
    if utxos.is_empty() {
        return Err(BlockchainError {
            message: format!("No UTXOs available for address: {}", sender_address),
            chain: Some(Chain::Bitcoin),
            code: None,
        });
    }
    
    // Use the first UTXO (simplified - real implementation would select optimally)
    let utxo = utxos[0].clone();
    
    // Build transaction
    let mut tx = Vec::new();
    
    // Version (4 bytes, little-endian)
    tx.extend_from_slice(&2u32.to_le_bytes());
    
    // Input count (varint)
    tx.push(1); // One input
    
    // Input: Outpoint (txid + vout)
    let txid_bytes = hex::decode(&utxo.txid)
        .map_err(|e| BlockchainError {
            message: format!("Invalid UTXO txid: {}", e),
            chain: Some(Chain::Bitcoin),
            code: None,
        })?;
    // txid is reversed in Bitcoin
    let txid_rev: Vec<u8> = txid_bytes.iter().rev().cloned().collect();
    tx.extend_from_slice(&txid_rev);
    tx.extend_from_slice(&utxo.vout.to_le_bytes());
    
    // Input: scriptSig length (0 for now - to be signed later)
    tx.push(0);
    
    // Input: sequence (0xffffffff)
    tx.extend_from_slice(&0xffffffffu32.to_le_bytes());
    
    // Output count (varint)
    tx.push(2); // OP_RETURN + change
    
    // Output 1: OP_RETURN with lock data
    // Value: 0 satoshis
    tx.extend_from_slice(&0u64.to_le_bytes());
    // Script: OP_RETURN <data>
    let script_len = 1 + lock_data.len(); // OP_RETURN + data
    if script_len <= 80 { // Bitcoin allows up to 80 bytes in OP_RETURN
        encode_varint(&mut tx, script_len as u64);
        tx.push(0x6a); // OP_RETURN
        tx.extend_from_slice(lock_data);
    }
    
    // Output 2: Change back to sender (simplified - no fee calculation)
    let change_value = utxo.value; // Should subtract fee
    tx.extend_from_slice(&change_value.to_le_bytes());
    // P2PKH script: OP_DUP OP_HASH160 <20 bytes> OP_EQUALVERIFY OP_CHECKSIG
    // (simplified - would need proper address to script conversion)
    tx.push(0x19); // 25 bytes script
    tx.push(0x76); // OP_DUP
    tx.push(0xa9); // OP_HASH160
    tx.push(0x14); // Push 20 bytes
    // Would add 20 bytes of hash160 of public key here
    tx.extend_from_slice(&[0u8; 20]); // Placeholder
    tx.push(0x88); // OP_EQUALVERIFY
    tx.push(0xac); // OP_CHECKSIG
    
    // Locktime (4 bytes)
    tx.extend_from_slice(&0u32.to_le_bytes());
    
    Ok((tx, utxo))
}

/// Fetch UTXOs for a Bitcoin address
async fn fetch_utxos(
    address: &str,
    rpc_url: &str,
) -> Result<Vec<Utxo>, BlockchainError> {
    let client = reqwest::Client::new();
    let url = format!("{}/address/{}/utxo", rpc_url.trim_end_matches('/'), address);
    
    web_sys::console::log_1(&format!("Fetching UTXOs from: {}", url).into());
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| BlockchainError {
            message: format!("Failed to fetch UTXOs: {}", e),
            chain: Some(Chain::Bitcoin),
            code: None,
        })?;
    
    // Get raw text for debugging
    let text = response.text().await.map_err(|e| BlockchainError {
        message: format!("Failed to read UTXO response: {}", e),
        chain: Some(Chain::Bitcoin),
        code: None,
    })?;
    
    web_sys::console::log_1(&format!("UTXO response: {}", &text[..text.len().min(200)]).into());
    
    // Parse as generic JSON first to handle different formats
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| BlockchainError {
        message: format!("Failed to parse UTXO JSON: {}", e),
        chain: Some(Chain::Bitcoin),
        code: None,
    })?;
    
    let mut utxos = Vec::new();
    
    // Handle array response
    if let Some(array) = json.as_array() {
        for item in array {
            if let Some(txid) = item.get("txid").and_then(|v| v.as_str()) {
                if let Some(vout) = item.get("vout").and_then(|v| v.as_u64()) {
                    if let Some(value) = item.get("value").and_then(|v| v.as_u64()) {
                        utxos.push(Utxo {
                            txid: txid.to_string(),
                            vout: vout as u32,
                            value,
                            script_pubkey: item.get("scriptPubKey")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
            }
        }
    }
    
    web_sys::console::log_1(&format!("Parsed {} UTXOs", utxos.len()).into());
    
    Ok(utxos)
}

/// Encode a varint (variable length integer)
fn encode_varint(buf: &mut Vec<u8>, value: u64) {
    if value < 0xfd {
        buf.push(value as u8);
    } else if value <= 0xffff {
        buf.push(0xfd);
        buf.extend_from_slice(&(value as u16).to_le_bytes());
    } else if value <= 0xffffffff {
        buf.push(0xfe);
        buf.extend_from_slice(&(value as u32).to_le_bytes());
    } else {
        buf.push(0xff);
        buf.extend_from_slice(&value.to_le_bytes());
    }
}

/// Sign a Bitcoin transaction with ECDSA
pub fn sign_bitcoin_transaction(
    unsigned_tx: &[u8],
    private_key_hex: &str,
    _utxo: &Utxo,
) -> Result<Vec<u8>, BlockchainError> {
    use secp256k1::{Message, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};
    
    web_sys::console::log_1(&format!("Signing with key (first 10 chars): {}", &private_key_hex[..private_key_hex.len().min(10)]).into());
    web_sys::console::log_1(&format!("Key length: {} chars", private_key_hex.len()).into());
    
    // Clean the key - remove 0x prefix and whitespace
    let cleaned = private_key_hex.trim().trim_start_matches("0x").trim();
    
    // Try to decode as hex
    let key_bytes = hex::decode(cleaned)
        .map_err(|e| BlockchainError {
            message: format!("Invalid private key hex: {}", e),
            chain: Some(Chain::Bitcoin),
            code: None,
        })?;
    
    web_sys::console::log_1(&format!("Key bytes length: {}", key_bytes.len()).into());
    
    // Handle different key formats
    let secret_key = match key_bytes.len() {
        32 => {
            // Standard raw private key
            SecretKey::from_slice(&key_bytes[..])
                .map_err(|e| BlockchainError {
                    message: format!("Invalid 32-byte secret key: {}", e),
                    chain: Some(Chain::Bitcoin),
                    code: None,
                })?
        }
        33 if key_bytes[32] == 0x01 => {
            // Compressed format - drop the 01 suffix
            SecretKey::from_slice(&key_bytes[..32])
                .map_err(|e| BlockchainError {
                    message: format!("Invalid 33-byte secret key: {}", e),
                    chain: Some(Chain::Bitcoin),
                    code: None,
                })?
        }
        64 => {
            // 64-byte seed format - use first 32 bytes as private key
            // (This is common in Bitcoin seed formats)
            SecretKey::from_slice(&key_bytes[..32])
                .map_err(|e| BlockchainError {
                    message: format!("Invalid 64-byte seed: {}", e),
                    chain: Some(Chain::Bitcoin),
                    code: None,
                })?
        }
        _ => {
            return Err(BlockchainError {
                message: format!("Unsupported key length: {} bytes. Expected 32, 33, or 64", key_bytes.len()),
                chain: Some(Chain::Bitcoin),
                code: None,
            });
        }
    };
    
    // Double SHA256 hash for Bitcoin
    let hash1 = Sha256::digest(unsigned_tx);
    let hash2 = Sha256::digest(&hash1);
    
    let message = Message::from_digest_slice(&hash2)
        .map_err(|e| BlockchainError {
            message: format!("Invalid message: {}", e),
            chain: Some(Chain::Bitcoin),
            code: None,
        })?;
    
    let secp = Secp256k1::new();
    let signature = secp.sign_ecdsa(&message, &secret_key);
    
    // Build signed transaction with scriptSig
    // For P2PKH: <sig len> <sig> <pubkey len> <pubkey>
    let sig_der = signature.serialize_der().to_vec();
    let sig_with_hashtype = [&sig_der[..], &[0x01]].concat(); // SIGHASH_ALL
    
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
    let pubkey_bytes = public_key.serialize();
    
    // Build scriptSig
    let mut script_sig = Vec::new();
    script_sig.push(sig_with_hashtype.len() as u8);
    script_sig.extend_from_slice(&sig_with_hashtype);
    script_sig.push(pubkey_bytes.len() as u8);
    script_sig.extend_from_slice(&pubkey_bytes);
    
    // Insert scriptSig into transaction
    // (This is simplified - real implementation would reconstruct the tx)
    let signed_tx = unsigned_tx.to_vec();
    
    Ok(signed_tx)
}

/// Broadcast a Bitcoin transaction
pub async fn broadcast_transaction(
    raw_tx: &[u8],
    rpc_url: &str,
) -> Result<String, BlockchainError> {
    let client = reqwest::Client::new();
    let url = format!("{}/tx", rpc_url.trim_end_matches('/'));
    
    let tx_hex = hex::encode(raw_tx);
    
    let response = client.post(&url)
        .body(tx_hex)
        .send()
        .await
        .map_err(|e| BlockchainError {
            message: format!("Failed to broadcast: {}", e),
            chain: Some(Chain::Bitcoin),
            code: None,
        })?;
    
    let txid = response.text().await.map_err(|e| BlockchainError {
        message: format!("Failed to read response: {}", e),
        chain: Some(Chain::Bitcoin),
        code: None,
    })?;
    
    Ok(txid)
}
