//! Cross-platform Secure Credential Storage
//!
//! This module provides secure storage for API tokens using the system's
//! native credential manager:
//! - macOS: Keychain
//! - Windows: Windows Credential Manager
//! - Linux: Secret Service (via libsecret/GNOME Keyring)
//!
//! All credentials are stored securely and zeroed from memory when dropped.

#![allow(dead_code)]

use keyring::Entry;
use zeroize::Zeroize;

const SERVICE_NAME: &str = "com.d1manager.app";

/// Secure string that is zeroed on drop
#[derive(Clone)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureString([REDACTED])")
    }
}

impl Default for SecureString {
    fn default() -> Self {
        Self { inner: String::new() }
    }
}

/// Store API token in system credential store
/// - macOS: Keychain
/// - Windows: Credential Manager
/// - Linux: Secret Service
pub fn store_token(profile_id: &str, token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, profile_id)
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;

    entry.set_password(token)
        .map_err(|e| format!("Failed to store token: {}", e))
}

/// Retrieve API token from system credential store
pub fn get_token(profile_id: &str) -> Result<SecureString, String> {
    let entry = Entry::new(SERVICE_NAME, profile_id)
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;

    match entry.get_password() {
        Ok(password) => Ok(SecureString::new(password)),
        Err(keyring::Error::NoEntry) => Ok(SecureString::default()),
        Err(e) => Err(format!("Failed to get token: {}", e)),
    }
}

/// Delete API token from system credential store
pub fn delete_token(profile_id: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, profile_id)
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(format!("Failed to delete token: {}", e)),
    }
}

/// Generate a unique ID for a profile (used as keychain key)
pub fn generate_profile_id(name: &str, database_id: &str) -> String {
    format!("{}_{}", name.replace(' ', "_"), &database_id[..8.min(database_id.len())])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_zeroize() {
        let mut s = SecureString::new("secret".to_string());
        assert_eq!(s.as_str(), "secret");
        drop(s);
        // After drop, memory should be zeroed (can't easily test this)
    }

    #[test]
    fn test_generate_profile_id() {
        let id = generate_profile_id("My Database", "abc123def456");
        assert_eq!(id, "My_Database_abc123de");
    }
}
