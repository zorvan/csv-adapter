//! Encrypted IndexedDB storage for sensitive seal nullifier data
//!
//! CRITICAL FIX: Replaces unencrypted localStorage with AES-GCM encrypted IndexedDB.
//! All seal nullifiers and sensitive state are encrypted at rest with HMAC integrity.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use csv_core::mcp::{error_codes, FixAction, HasErrorSuggestion};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::Zeroize;

use crate::state::StorageError;

/// HMAC-SHA256 type alias
type HmacSha256 = Hmac<Sha256>;

/// Encryption error types
#[derive(Debug, thiserror::Error)]
pub enum EncryptedStorageError {
    /// Encryption/decryption error
    #[error("Crypto error: {0}")]
    CryptoError(String),
    /// Browser API error
    #[error("Browser API error: {0}")]
    BrowserError(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializeError(String),
    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),
    /// Integrity check failed
    #[error("Integrity check failed: {0}")]
    IntegrityError(String),
}

impl From<EncryptedStorageError> for StorageError {
    fn from(e: EncryptedStorageError) -> Self {
        StorageError::SerializeError(e.to_string())
    }
}

impl HasErrorSuggestion for EncryptedStorageError {
    fn error_code(&self) -> &'static str {
        error_codes::WALLET_BROWSER_STORAGE
    }

    fn description(&self) -> String {
        self.to_string()
    }

    fn suggested_fix(&self) -> String {
        match self {
            EncryptedStorageError::CryptoError(_) => "Encryption/decryption failed. \
                 Check: 1) Browser supports WebCrypto API, \
                 2) Storage key hasn't been corrupted. \
                 You may need to re-import your wallet."
                .to_string(),
            EncryptedStorageError::IntegrityError(_) => "Data integrity check failed. \
                 This could indicate tampering or storage corruption. \
                 Do not proceed with transactions - verify your system security."
                .to_string(),
            EncryptedStorageError::BrowserError(_) => "IndexedDB error. Check: \
                 1) Browser storage is enabled, \
                 2) Not in private/incognito mode, \
                 3) Storage quota available."
                .to_string(),
            EncryptedStorageError::SerializeError(_) => "Data serialization failed. \
                 Ensure data format is valid."
                .to_string(),
            EncryptedStorageError::NotFound(key) => {
                format!("Item '{}' not found in encrypted storage.", key)
            }
        }
    }

    fn docs_url(&self) -> String {
        error_codes::docs_url(self.error_code())
    }

    fn fix_action(&self) -> Option<FixAction> {
        match self {
            EncryptedStorageError::CryptoError(_) => Some(FixAction::CheckState {
                url: "https://docs.csv.dev/wallet/encrypted-storage".to_string(),
                what: "Verify WebCrypto API is available".to_string(),
            }),
            EncryptedStorageError::IntegrityError(_) => Some(FixAction::CheckState {
                url: "https://docs.csv.dev/wallet/security".to_string(),
                what: "SECURITY WARNING: Verify system integrity".to_string(),
            }),
            _ => None,
        }
    }
}

/// Encrypted envelope for stored data
#[derive(Serialize, Deserialize, Clone, Debug)]
struct EncryptedEnvelope {
    /// AES-GCM ciphertext
    ciphertext: Vec<u8>,
    /// Nonce (12 bytes for AES-GCM)
    nonce: Vec<u8>,
    /// HMAC-SHA256 for integrity
    hmac: Vec<u8>,
    /// Version for future compatibility
    version: u32,
}

impl EncryptedEnvelope {
    const CURRENT_VERSION: u32 = 1;

    /// Create new envelope from plaintext
    fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Self, EncryptedStorageError> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptedStorageError::CryptoError(format!("Key init failed: {}", e)))?;

        // Generate random nonce
        let nonce_bytes = aes_gcm::aead::rand_core::RngCore::next_u64(&mut OsRng);
        let nonce = Nonce::from_slice(&nonce_bytes.to_le_bytes()[..12]);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptedStorageError::CryptoError(format!("Encryption failed: {}", e)))?;

        // Compute HMAC for integrity
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|e| EncryptedStorageError::CryptoError(format!("HMAC init failed: {}", e)))?;
        mac.update(&ciphertext);
        mac.update(nonce.as_slice());
        let hmac = mac.finalize().into_bytes().to_vec();

        Ok(EncryptedEnvelope {
            ciphertext,
            nonce: nonce.as_slice().to_vec(),
            hmac,
            version: Self::CURRENT_VERSION,
        })
    }

    /// Decrypt envelope to plaintext
    fn decrypt(&self, key: &[u8; 32]) -> Result<Vec<u8>, EncryptedStorageError> {
        // Verify HMAC first (constant-time)
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|e| EncryptedStorageError::CryptoError(format!("HMAC init failed: {}", e)))?;
        mac.update(&self.ciphertext);
        mac.update(&self.nonce);
        let computed_hmac = mac.finalize().into_bytes();

        // Constant-time comparison to prevent timing attacks
        if !hmac::digest::Output::<HmacSha256>::from_slice(&self.hmac)
            .ct_eq(&computed_hmac)
            .into()
        {
            return Err(EncryptedStorageError::IntegrityError(
                "HMAC verification failed - data may be tampered".to_string(),
            ));
        }

        // Decrypt
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| EncryptedStorageError::CryptoError(format!("Key init failed: {}", e)))?;
        let nonce = Nonce::from_slice(&self.nonce);

        cipher
            .decrypt(nonce, self.ciphertext.as_ref())
            .map_err(|e| EncryptedStorageError::CryptoError(format!("Decryption failed: {}", e)))
            .map(|pt| pt.to_vec())
    }
}

/// Encrypted IndexedDB storage manager
pub struct EncryptedStorageManager {
    db_name: String,
    store_name: String,
    key: [u8; 32],
}

impl EncryptedStorageManager {
    /// Create new encrypted storage manager
    ///
    /// # Security Note
    /// The key must be derived from a user password or secure key material.
    /// Do NOT hardcode keys in production.
    pub fn new(db_name: &str, store_name: &str, key: [u8; 32]) -> Self {
        Self {
            db_name: db_name.to_string(),
            store_name: store_name.to_string(),
            key,
        }
    }

    /// Derive encryption key from password using PBKDF2
    #[cfg(feature = "wasm32")]
    pub async fn derive_key_from_password(
        password: &str,
        salt: &[u8],
    ) -> Result<[u8; 32], EncryptedStorageError> {
        use pbkdf2::pbkdf2_hmac;

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
        Ok(key)
    }

    /// Save encrypted item to IndexedDB
    #[cfg(feature = "wasm32")]
    pub async fn save<T: Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), EncryptedStorageError> {
        let plaintext = serde_json::to_vec(value)
            .map_err(|e| EncryptedStorageError::SerializeError(e.to_string()))?;

        let envelope = EncryptedEnvelope::encrypt(&plaintext, &self.key)?;

        let js_value = serde_wasm_bindgen::to_value(&envelope)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("JS conversion failed: {}", e)))?;

        self.set_indexeddb(key, js_value).await
    }

    /// Load and decrypt item from IndexedDB
    #[cfg(feature = "wasm32")]
    pub async fn load<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<T, EncryptedStorageError> {
        let js_value = self
            .get_indexeddb(key)
            .await?
            .ok_or_else(|| EncryptedStorageError::NotFound(key.to_string()))?;

        let envelope: EncryptedEnvelope = serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| EncryptedStorageError::SerializeError(format!("JS conversion failed: {}", e)))?;

        let plaintext = envelope.decrypt(&self.key)?;

        serde_json::from_slice(&plaintext)
            .map_err(|e| EncryptedStorageError::SerializeError(e.to_string()))
    }

    /// Delete item from IndexedDB
    #[cfg(feature = "wasm32")]
    pub async fn delete(&self, key: &str) -> Result<(), EncryptedStorageError> {
        self.delete_indexeddb(key).await
    }

    /// Check if key exists
    #[cfg(feature = "wasm32")]
    pub async fn contains(&self, key: &str) -> bool {
        self.get_indexeddb(key).await.ok().flatten().is_some()
    }

    /// IndexedDB set operation via web-sys
    #[cfg(feature = "wasm32")]
    async fn set_indexeddb(
        &self,
        key: &str,
        value: wasm_bindgen::JsValue,
    ) -> Result<(), EncryptedStorageError> {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{IdbDatabase, IdbTransactionMode};

        let window = web_sys::window()
            .ok_or_else(|| EncryptedStorageError::BrowserError("No window".to_string()))?;
        let indexed_db = window
            .indexed_db()
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?
            .ok_or_else(|| EncryptedStorageError::BrowserError("IndexedDB not available".to_string()))?;

        // Open database
        let open_request = indexed_db
            .open_with_u32(&self.db_name, 1)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        // Create object store if needed
        let db: IdbDatabase = JsFuture::from(open_request)
            .await
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?
            .dyn_into()
            .map_err(|_| EncryptedStorageError::BrowserError("Invalid DB".to_string()))?;

        // Start transaction
        let transaction = db
            .transaction_with_str_and_mode(&self.store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let store = transaction
            .object_store(&self.store_name)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        // Put value
        let put_request = store
            .put_with_key(&value, &wasm_bindgen::JsValue::from_str(key))
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        JsFuture::from(put_request)
            .await
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        Ok(())
    }

    /// IndexedDB get operation via web-sys
    #[cfg(feature = "wasm32")]
    async fn get_indexeddb(
        &self,
        key: &str,
    ) -> Result<Option<wasm_bindgen::JsValue>, EncryptedStorageError> {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{IdbDatabase, IdbTransactionMode};

        let window = web_sys::window()
            .ok_or_else(|| EncryptedStorageError::BrowserError("No window".to_string()))?;
        let indexed_db = window
            .indexed_db()
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?
            .ok_or_else(|| EncryptedStorageError::BrowserError("IndexedDB not available".to_string()))?;

        let open_request = indexed_db
            .open_with_u32(&self.db_name, 1)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let db: IdbDatabase = JsFuture::from(open_request)
            .await
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?
            .dyn_into()
            .map_err(|_| EncryptedStorageError::BrowserError("Invalid DB".to_string()))?;

        let transaction = db
            .transaction_with_str_and_mode(&self.store_name, IdbTransactionMode::Readonly)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let store = transaction
            .object_store(&self.store_name)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let get_request = store
            .get(&wasm_bindgen::JsValue::from_str(key))
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let result = JsFuture::from(get_request)
            .await
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        if result.is_null() || result.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// IndexedDB delete operation via web-sys
    #[cfg(feature = "wasm32")]
    async fn delete_indexeddb(&self, key: &str) -> Result<(), EncryptedStorageError> {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{IdbDatabase, IdbTransactionMode};

        let window = web_sys::window()
            .ok_or_else(|| EncryptedStorageError::BrowserError("No window".to_string()))?;
        let indexed_db = window
            .indexed_db()
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?
            .ok_or_else(|| EncryptedStorageError::BrowserError("IndexedDB not available".to_string()))?;

        let open_request = indexed_db
            .open_with_u32(&self.db_name, 1)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let db: IdbDatabase = JsFuture::from(open_request)
            .await
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?
            .dyn_into()
            .map_err(|_| EncryptedStorageError::BrowserError("Invalid DB".to_string()))?;

        let transaction = db
            .transaction_with_str_and_mode(&self.store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let store = transaction
            .object_store(&self.store_name)
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        let delete_request = store
            .delete(&wasm_bindgen::JsValue::from_str(key))
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        JsFuture::from(delete_request)
            .await
            .map_err(|e| EncryptedStorageError::BrowserError(format!("{:?}", e)))?;

        Ok(())
    }
}

impl Drop for EncryptedStorageManager {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

/// Factory for seal nullifier encrypted storage
pub fn seal_nullifier_storage(key: [u8; 32]) -> EncryptedStorageManager {
    EncryptedStorageManager::new("csv-seal-nullifiers", "nullifiers", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_encrypt_decrypt() {
        let key = [0x42u8; 32];
        let plaintext = b"test seal nullifier data";

        let envelope = EncryptedEnvelope::encrypt(plaintext, &key).unwrap();
        let decrypted = envelope.decrypt(&key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_integrity_failure() {
        let key = [0x42u8; 32];
        let plaintext = b"test data";

        let mut envelope = EncryptedEnvelope::encrypt(plaintext, &key).unwrap();
        // Tamper with ciphertext
        envelope.ciphertext[0] ^= 0xFF;

        let result = envelope.decrypt(&key);
        assert!(matches!(result, Err(EncryptedStorageError::IntegrityError(_))));
    }
}
