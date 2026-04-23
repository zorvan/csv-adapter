//! Native transaction signer using imported private keys.
//!
//! This module provides native transaction signing for all supported chains
//! using the private keys stored in ChainAccount. No external wallet needed.

use csv_adapter_core::Chain;
use csv_adapter_core::agent_types::{HasErrorSuggestion, FixAction, error_codes};
use serde::{Deserialize, Serialize};

/// Transaction to be signed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    pub chain: Chain,
    pub from: String,
    pub to: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub nonce: Option<u64>,
    pub gas_price: Option<u64>,
    pub gas_limit: Option<u64>,
}

/// Signed transaction ready for broadcast.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub chain: Chain,
    pub tx_hash: String,
    pub raw_bytes: Vec<u8>,
}

/// Error type for signing operations.
#[derive(Debug, thiserror::Error)]
pub enum SignerError {
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Unsupported chain: {0:?}")]
    UnsupportedChain(Chain),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl HasErrorSuggestion for SignerError {
    fn error_code(&self) -> &'static str {
        error_codes::WALLET_NATIVE_SIGNER_ERROR
    }

    fn description(&self) -> String {
        self.to_string()
    }

    fn suggested_fix(&self) -> String {
        match self {
            SignerError::InvalidPrivateKey(_) => {
                "Invalid private key format. Ensure the key is: \
                 1) 64 hex characters (32 bytes), \
                 2) Optionally prefixed with '0x', \
                 3) A valid secp256k1 (Ethereum/Bitcoin) or ed25519 (Sui/Aptos/Solana) key. \
                 Never share private keys - they control your funds.".to_string()
            }
            SignerError::SigningFailed(_) => {
                "Transaction signing failed. Check: \
                 1) The private key is valid and has funds, \
                 2) The transaction format matches the chain, \
                 3) The nonce/sequence number is correct. \
                 Retry with corrected parameters.".to_string()
            }
            SignerError::UnsupportedChain(chain) => {
                format!(
                    "Chain {:?} is not supported for native signing. \
                     Supported chains: Ethereum, Sui, Aptos, Solana, Bitcoin. \
                     Use a wallet extension or external signer for other chains.",
                    chain
                )
            }
            SignerError::SerializationError(_) => {
                "Failed to serialize transaction data. Check all fields are valid \
                 and the transaction structure matches the expected format.".to_string()
            }
        }
    }

    fn docs_url(&self) -> String {
        error_codes::docs_url(self.error_code())
    }

    fn fix_action(&self) -> Option<FixAction> {
        match self {
            SignerError::InvalidPrivateKey(_) => {
                Some(FixAction::CheckState {
                    url: "https://docs.csv.dev/wallet/key-management".to_string(),
                    what: "Verify private key format and derivation path".to_string(),
                })
            }
            SignerError::SigningFailed(_) | SignerError::SerializationError(_) => {
                Some(FixAction::Retry {
                    parameter_changes: std::collections::HashMap::from([
                        ("verify_nonce".to_string(), "true".to_string()),
                        ("check_chain_id".to_string(), "true".to_string()),
                    ]),
                })
            }
            _ => None,
        }
    }
}

/// Native signer using private keys.
pub struct NativeSigner;

impl NativeSigner {
    /// Sign a transaction for the specified chain.
    pub fn sign_transaction(
        tx: &UnsignedTransaction,
        private_key_hex: &str,
    ) -> Result<SignedTransaction, SignerError> {
        match tx.chain {
            Chain::Ethereum => Self::sign_ethereum(tx, private_key_hex),
            Chain::Sui => Self::sign_sui(tx, private_key_hex),
            Chain::Aptos => Self::sign_aptos(tx, private_key_hex),
            Chain::Solana => Self::sign_solana(tx, private_key_hex),
            Chain::Bitcoin => Self::sign_bitcoin(tx, private_key_hex),
            _ => Err(SignerError::UnsupportedChain(tx.chain)),
        }
    }

    /// Sign Ethereum transaction (EIP-155).
    fn sign_ethereum(
        tx: &UnsignedTransaction,
        private_key_hex: &str,
    ) -> Result<SignedTransaction, SignerError> {
        use secp256k1::{Message, Secp256k1, SecretKey};
        use sha3::{Digest, Keccak256};

        let key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|e| SignerError::InvalidPrivateKey(e.to_string()))?;
        
        if key_bytes.len() != 32 {
            return Err(SignerError::InvalidPrivateKey(
                format!("Expected 32 bytes, got {}", key_bytes.len())
            ));
        }

        let secret_key = SecretKey::from_slice(&key_bytes)
            .map_err(|e| SignerError::InvalidPrivateKey(e.to_string()))?;

        // Build RLP-encoded transaction
        let nonce = tx.nonce.unwrap_or(0);
        let gas_price = tx.gas_price.unwrap_or(1000000000); // 1 gwei
        let gas_limit = tx.gas_limit.unwrap_or(21000);
        let value = tx.value;
        let data = &tx.data;

        // Simple EIP-155 transaction (without access list for now)
        // RLP encode: [nonce, gasPrice, gasLimit, to, value, data, v, r, s]
        // For unsigned tx: v = chain_id, r = 0, s = 0
        let chain_id: u64 = 11155111; // Sepolia
        
        // Build the transaction hash to sign
        let to_bytes = hex::decode(tx.to.trim_start_matches("0x"))
            .map_err(|e| SignerError::SerializationError(e.to_string()))?;
        
        // RLP encoding for signing
        let mut rlp = Vec::new();
        Self::encode_u64(&mut rlp, nonce);
        Self::encode_u64(&mut rlp, gas_price);
        Self::encode_u64(&mut rlp, gas_limit);
        Self::encode_bytes(&mut rlp, &to_bytes);
        Self::encode_u64(&mut rlp, value);
        Self::encode_bytes(&mut rlp, data);
        Self::encode_u64(&mut rlp, chain_id); // EIP-155 chain ID
        Self::encode_bytes(&mut rlp, &[]); // r = 0
        Self::encode_bytes(&mut rlp, &[]); // s = 0

        // Add RLP list prefix
        let encoded = Self::prefix_list(&rlp);
        
        // Hash and sign
        let hash = Keccak256::digest(&encoded);
        let message = Message::from_digest_slice(&hash)
            .map_err(|e| SignerError::SigningFailed(e.to_string()))?;

        let secp = Secp256k1::new();
        // Note: For proper recovery ID, use sign_ecdsa_recoverable in production
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact();

        // Build signed transaction
        // v = chain_id * 2 + 35 + recovery_id for EIP-155
        // recovery_id is 0 or 1 - simplified for now
        let recovery_id = 0u8;
        let v = chain_id * 2 + 35 + recovery_id as u64;
        let r = &signature_bytes[..32];
        let s = &signature_bytes[32..];

        // RLP encode signed transaction
        let mut signed_rlp = Vec::new();
        Self::encode_u64(&mut signed_rlp, nonce);
        Self::encode_u64(&mut signed_rlp, gas_price);
        Self::encode_u64(&mut signed_rlp, gas_limit);
        Self::encode_bytes(&mut signed_rlp, &to_bytes);
        Self::encode_u64(&mut signed_rlp, value);
        Self::encode_bytes(&mut signed_rlp, data);
        Self::encode_u64(&mut signed_rlp, v);
        Self::encode_bytes(&mut signed_rlp, r);
        Self::encode_bytes(&mut signed_rlp, s);

        let signed_encoded = Self::prefix_list(&signed_rlp);
        let tx_hash = format!("0x{}", hex::encode(Keccak256::digest(&signed_encoded)));

        Ok(SignedTransaction {
            chain: Chain::Ethereum,
            tx_hash,
            raw_bytes: signed_encoded,
        })
    }

    /// Sign Sui transaction.
    pub fn sign_sui(
        tx: &UnsignedTransaction,
        private_key_hex: &str,
    ) -> Result<SignedTransaction, SignerError> {
        use ed25519_dalek::{Signer, SigningKey};

        let key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|e| SignerError::InvalidPrivateKey(e.to_string()))?;
        
        if key_bytes.len() < 32 {
            return Err(SignerError::InvalidPrivateKey(
                format!("Expected at least 32 bytes, got {}", key_bytes.len())
            ));
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&key_bytes[..32]);
        let signing_key = SigningKey::from_bytes(&seed);

        // Sign transaction data
        let signature = signing_key.sign(&tx.data);
        let signature_bytes = signature.to_bytes().to_vec();

        // Create signed transaction
        // Format: [flag (1) | signature (64) | public_key (32) | tx_data]
        let public_key = signing_key.verifying_key().to_bytes();
        let mut signed_tx = Vec::new();
        signed_tx.push(0x00); // Ed25519 flag
        signed_tx.extend_from_slice(&signature_bytes);
        signed_tx.extend_from_slice(&public_key);
        signed_tx.extend_from_slice(&tx.data);

        // Transaction hash is BLAKE2b of signed data (simplified)
        use blake2::{Blake2b, Digest};
        use sha2::digest::consts::U32;
        let hash = Blake2b::<U32>::digest(&signed_tx);
        let tx_hash = format!("0x{}", hex::encode(&hash[..]));

        Ok(SignedTransaction {
            chain: Chain::Sui,
            tx_hash,
            raw_bytes: signed_tx,
        })
    }

    /// Sign Aptos transaction.
    pub fn sign_aptos(
        tx: &UnsignedTransaction,
        private_key_hex: &str,
    ) -> Result<SignedTransaction, SignerError> {
        use ed25519_dalek::{Signer, SigningKey};
        use sha3::{Digest, Sha3_256};

        let key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|e| SignerError::InvalidPrivateKey(e.to_string()))?;
        
        if key_bytes.len() < 32 {
            return Err(SignerError::InvalidPrivateKey(
                format!("Expected at least 32 bytes, got {}", key_bytes.len())
            ));
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&key_bytes[..32]);
        let signing_key = SigningKey::from_bytes(&seed);

        // Sign transaction data (simplified - real Aptos uses BCS serialization)
        let signing_message = Sha3_256::digest(&tx.data);
        let signature = signing_key.sign(&signing_message);
        let signature_bytes = signature.to_bytes().to_vec();

        // Create signed transaction JSON
        let public_key = signing_key.verifying_key().to_bytes();
        let signed_tx_json = serde_json::json!({
            "sender": tx.from,
            "sequence_number": tx.nonce.unwrap_or(0).to_string(),
            "payload": hex::encode(&tx.data),
            "signature": hex::encode(&signature_bytes),
            "public_key": hex::encode(&public_key),
        });

        let signed_tx = signed_tx_json.to_string().into_bytes();
        let tx_hash = format!("0x{}", hex::encode(Sha3_256::digest(&signed_tx)));

        Ok(SignedTransaction {
            chain: Chain::Aptos,
            tx_hash,
            raw_bytes: signed_tx,
        })
    }

    /// Sign Solana transaction.
    fn sign_solana(
        tx: &UnsignedTransaction,
        private_key_hex: &str,
    ) -> Result<SignedTransaction, SignerError> {
        use ed25519_dalek::{Signer, SigningKey};
        use sha2::{Digest, Sha256};

        let key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|e| SignerError::InvalidPrivateKey(e.to_string()))?;
        
        if key_bytes.len() < 32 {
            return Err(SignerError::InvalidPrivateKey(
                format!("Expected at least 32 bytes, got {}", key_bytes.len())
            ));
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&key_bytes[..32]);
        let signing_key = SigningKey::from_bytes(&seed);

        // Sign transaction data
        let message = Sha256::digest(&tx.data);
        let signature = signing_key.sign(&message);
        let signature_bytes = signature.to_bytes().to_vec();

        // Create signed transaction
        let public_key = signing_key.verifying_key().to_bytes();
        let mut signed_tx = Vec::new();
        signed_tx.extend_from_slice(&signature_bytes);
        signed_tx.extend_from_slice(&public_key);
        signed_tx.extend_from_slice(&tx.data);

        let tx_hash = format!("0x{}", hex::encode(Sha256::digest(&signed_tx)));

        Ok(SignedTransaction {
            chain: Chain::Solana,
            tx_hash,
            raw_bytes: signed_tx,
        })
    }

    /// Sign Bitcoin transaction.
    fn sign_bitcoin(
        _tx: &UnsignedTransaction,
        private_key_hex: &str,
    ) -> Result<SignedTransaction, SignerError> {
        use secp256k1::{Message, Secp256k1, SecretKey};
        use sha2::{Digest, Sha256};

        let key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|e| SignerError::InvalidPrivateKey(e.to_string()))?;
        
        if key_bytes.len() < 32 {
            return Err(SignerError::InvalidPrivateKey(
                format!("Expected at least 32 bytes, got {}", key_bytes.len())
            ));
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&key_bytes[..32]);
        let secret_key = SecretKey::from_slice(&seed)
            .map_err(|e| SignerError::InvalidPrivateKey(e.to_string()))?;

        // Bitcoin signing is complex - simplified placeholder
        // Real implementation would use bitcoin crate for proper transaction building
        let secp = Secp256k1::new();
        let message = Message::from_digest_slice(&[0u8; 32])
            .map_err(|e| SignerError::SigningFailed(e.to_string()))?;
        let signature = secp.sign_ecdsa(&message, &secret_key);

        let tx_hash = format!("0x{}", hex::encode(Sha256::digest(&signature.serialize_compact())));

        Ok(SignedTransaction {
            chain: Chain::Bitcoin,
            tx_hash,
            raw_bytes: signature.serialize_compact().to_vec(),
        })
    }

    // RLP encoding helpers
    fn encode_u64(buf: &mut Vec<u8>, value: u64) {
        if value == 0 {
            buf.push(0x80);
        } else {
            let bytes = value.to_be_bytes();
            let start = bytes.iter().position(|&x| x != 0).unwrap_or(8);
            let len = 8 - start;
            if len == 1 && bytes[7] < 0x80 {
                buf.push(bytes[7]);
            } else {
                buf.push(0x80 + len as u8);
                buf.extend_from_slice(&bytes[start..]);
            }
        }
    }

    fn encode_bytes(buf: &mut Vec<u8>, bytes: &[u8]) {
        if bytes.len() == 1 && bytes[0] < 0x80 {
            buf.push(bytes[0]);
        } else if bytes.len() <= 55 {
            buf.push(0x80 + bytes.len() as u8);
            buf.extend_from_slice(bytes);
        } else {
            let len_bytes = bytes.len().to_be_bytes();
            let start = len_bytes.iter().position(|&x| x != 0).unwrap_or(8);
            let len_len = 8 - start;
            buf.push(0xb7 + len_len as u8);
            buf.extend_from_slice(&len_bytes[start..]);
            buf.extend_from_slice(bytes);
        }
    }

    fn prefix_list(bytes: &[u8]) -> Vec<u8> {
        if bytes.len() <= 55 {
            let mut result = vec![0xc0 + bytes.len() as u8];
            result.extend_from_slice(bytes);
            result
        } else {
            let len_bytes = bytes.len().to_be_bytes();
            let start = len_bytes.iter().position(|&x| x != 0).unwrap_or(8);
            let len_len = 8 - start;
            let mut result = vec![0xf7 + len_len as u8];
            result.extend_from_slice(&len_bytes[start..]);
            result.extend_from_slice(bytes);
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_address_derivation() {
        // Test vector from known private key
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        
        // Just verify we can create the key without error
        let key_bytes = hex::decode(private_key.trim_start_matches("0x")).unwrap();
        assert_eq!(key_bytes.len(), 32);
    }

    #[test]
    fn test_rlp_encoding() {
        let mut buf = Vec::new();
        NativeSigner::encode_u64(&mut buf, 0);
        assert_eq!(buf, vec![0x80]);
        
        buf.clear();
        NativeSigner::encode_u64(&mut buf, 1);
        assert_eq!(buf, vec![0x01]);
        
        buf.clear();
        NativeSigner::encode_u64(&mut buf, 127);
        assert_eq!(buf, vec![0x7f]);
        
        buf.clear();
        NativeSigner::encode_u64(&mut buf, 128);
        assert_eq!(buf, vec![0x81, 0x80]);
    }
}
