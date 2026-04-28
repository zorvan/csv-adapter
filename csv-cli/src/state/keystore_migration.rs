//! Keystore migration module (stub - no migration functionality).

/// Stub migration manager.
pub struct KeystoreMigration;

impl KeystoreMigration {
    /// Create a new stub instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for KeystoreMigration {
    fn default() -> Self {
        Self::new()
    }
}
