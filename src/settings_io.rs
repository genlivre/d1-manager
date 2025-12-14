//! Settings export/import functionality
//! Allows users to backup and restore connection profiles

use crate::app::{EnvironmentType, ProfileMetadata};
use crate::secure_storage;
use serde::{Deserialize, Serialize};

/// Exportable connection profile (includes encrypted token if requested)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportableProfile {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub database_id: String,
    pub environment: EnvironmentType,
    pub read_only: bool,
    /// Base64 encoded encrypted token (only if include_tokens is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_token: Option<String>,
}

/// Full settings export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsExport {
    pub version: u32,
    pub exported_at: u64,
    pub profiles: Vec<ExportableProfile>,
    /// Whether tokens are included (encrypted)
    pub tokens_included: bool,
}

impl SettingsExport {
    pub const CURRENT_VERSION: u32 = 1;
}

/// Simple XOR-based encryption with key derivation
/// Not cryptographically strong, but sufficient for basic obfuscation
fn derive_key(password: &str, length: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(length);
    let password_bytes = password.as_bytes();

    if password_bytes.is_empty() {
        return vec![0u8; length];
    }

    // Simple key stretching using repeated hashing pattern
    let mut state: u32 = 0x12345678;
    for (i, &b) in password_bytes.iter().cycle().take(length * 4).enumerate() {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        state ^= (b as u32) << ((i % 4) * 8);
        if i % 4 == 3 {
            key.push((state >> 24) as u8);
        }
    }

    while key.len() < length {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        key.push((state >> 24) as u8);
    }

    key.truncate(length);
    key
}

/// Encrypt a string with password
fn encrypt_string(plaintext: &str, password: &str) -> String {
    let data = plaintext.as_bytes();
    let key = derive_key(password, data.len().max(32));

    // Add a simple checksum for validation
    let checksum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    let mut encrypted = Vec::with_capacity(data.len() + 1);
    encrypted.push(checksum);

    for (i, &byte) in data.iter().enumerate() {
        encrypted.push(byte ^ key[i % key.len()]);
    }

    base64_encode(&encrypted)
}

/// Decrypt a string with password
fn decrypt_string(encrypted: &str, password: &str) -> Result<String, &'static str> {
    let data = base64_decode(encrypted).map_err(|_| "Invalid base64")?;

    if data.is_empty() {
        return Err("Empty data");
    }

    let stored_checksum = data[0];
    let encrypted_data = &data[1..];

    let key = derive_key(password, encrypted_data.len().max(32));

    let mut decrypted = Vec::with_capacity(encrypted_data.len());
    for (i, &byte) in encrypted_data.iter().enumerate() {
        decrypted.push(byte ^ key[i % key.len()]);
    }

    // Verify checksum
    let computed_checksum: u8 = decrypted.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    if computed_checksum != stored_checksum {
        return Err("Checksum mismatch - wrong password?");
    }

    String::from_utf8(decrypted).map_err(|_| "Invalid UTF-8")
}

/// Simple base64 encoding (no external dependency)
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() { data[i + 1] as usize } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as usize } else { 0 };

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

/// Simple base64 decoding
fn base64_decode(encoded: &str) -> Result<Vec<u8>, &'static str> {
    const DECODE: [i8; 128] = [
        -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
        -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
        -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,62,-1,-1,-1,63,
        52,53,54,55,56,57,58,59,60,61,-1,-1,-1,-1,-1,-1,
        -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13,14,
        15,16,17,18,19,20,21,22,23,24,25,-1,-1,-1,-1,-1,
        -1,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,
        41,42,43,44,45,46,47,48,49,50,51,-1,-1,-1,-1,-1,
    ];

    let mut result = Vec::new();
    let bytes: Vec<u8> = encoded.bytes().filter(|&b| b != b'=' && b != b'\n' && b != b'\r').collect();

    let mut i = 0;
    while i < bytes.len() {
        let mut buf = [0u8; 4];
        let mut count = 0;

        while count < 4 && i < bytes.len() {
            let b = bytes[i];
            if b < 128 && DECODE[b as usize] >= 0 {
                buf[count] = DECODE[b as usize] as u8;
                count += 1;
            }
            i += 1;
        }

        if count >= 2 {
            result.push((buf[0] << 2) | (buf[1] >> 4));
        }
        if count >= 3 {
            result.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if count >= 4 {
            result.push((buf[2] << 6) | buf[3]);
        }
    }

    Ok(result)
}

/// Export profiles to JSON string
pub fn export_profiles(
    profiles: &[ProfileMetadata],
    include_tokens: bool,
    password: &str,
) -> Result<String, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let exported_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut exportable_profiles = Vec::new();

    for profile in profiles {
        let encrypted_token = if include_tokens {
            match secure_storage::get_token(&profile.id) {
                Ok(token) if !token.is_empty() => {
                    Some(encrypt_string(token.as_str(), password))
                }
                _ => None,
            }
        } else {
            None
        };

        exportable_profiles.push(ExportableProfile {
            id: profile.id.clone(),
            name: profile.name.clone(),
            account_id: profile.account_id.clone(),
            database_id: profile.database_id.clone(),
            environment: profile.environment,
            read_only: profile.read_only,
            encrypted_token,
        });
    }

    let export = SettingsExport {
        version: SettingsExport::CURRENT_VERSION,
        exported_at,
        profiles: exportable_profiles,
        tokens_included: include_tokens,
    };

    serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Failed to serialize: {}", e))
}

/// Import result
#[derive(Debug)]
pub struct ImportResult {
    pub profiles: Vec<ProfileMetadata>,
    pub tokens: Vec<(String, String)>, // (profile_id, decrypted_token)
}

/// Import profiles from JSON string
pub fn import_profiles(
    json: &str,
    password: &str,
) -> Result<ImportResult, String> {
    let export: SettingsExport = serde_json::from_str(json)
        .map_err(|e| format!("Invalid settings file: {}", e))?;

    if export.version > SettingsExport::CURRENT_VERSION {
        return Err(format!(
            "Settings file version {} is newer than supported version {}",
            export.version,
            SettingsExport::CURRENT_VERSION
        ));
    }

    let mut profiles = Vec::new();
    let mut tokens = Vec::new();

    for ep in export.profiles {
        profiles.push(ProfileMetadata {
            id: ep.id.clone(),
            name: ep.name,
            account_id: ep.account_id,
            database_id: ep.database_id,
            environment: ep.environment,
            read_only: ep.read_only,
        });

        if let Some(encrypted) = ep.encrypted_token {
            match decrypt_string(&encrypted, password) {
                Ok(token) => tokens.push((ep.id, token)),
                Err(e) => return Err(format!("Failed to decrypt token: {}", e)),
            }
        }
    }

    Ok(ImportResult {
        profiles,
        tokens,
    })
}

/// Save export to file using file dialog
pub fn save_export_to_file(content: &str) -> Result<std::path::PathBuf, String> {
    let file_path = rfd::FileDialog::new()
        .set_file_name("d1-manager-settings.json")
        .add_filter("JSON", &["json"])
        .add_filter("All Files", &["*"])
        .save_file()
        .ok_or_else(|| "Export cancelled".to_string())?;

    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(file_path)
}

/// Load import from file using file dialog
pub fn load_import_from_file() -> Result<String, String> {
    let file_path = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .ok_or_else(|| "Import cancelled".to_string())?;

    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "my-secret-api-token-12345";
        let password = "test-password";

        let encrypted = encrypt_string(plaintext, password);
        let decrypted = decrypt_string(&encrypted, password).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let plaintext = "my-secret-api-token-12345";
        let password = "correct-password";
        let wrong_password = "wrong-password";

        let encrypted = encrypt_string(plaintext, password);
        let result = decrypt_string(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World! \x00\xff\x80";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();

        assert_eq!(decoded, data);
    }

    #[test]
    fn test_empty_string() {
        let encrypted = encrypt_string("", "password");
        let decrypted = decrypt_string(&encrypted, "password").unwrap();
        assert_eq!(decrypted, "");
    }
}
