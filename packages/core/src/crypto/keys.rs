//! Composite key construction from master password + key file + hardware key

use crate::error::{KeePassExError, Result};
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Composite key = SHA256(SHA256(password) || SHA256(keyfile) || ...)
#[derive(ZeroizeOnDrop)]
pub struct CompositeKey {
    components: Vec<Vec<u8>>,
}

impl CompositeKey {
    pub fn new() -> Self {
        Self { components: Vec::new() }
    }

    /// Add master password component
    pub fn add_password(&mut self, password: &str) {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        self.components.push(hasher.finalize().to_vec());
    }

    /// Add key file component
    pub fn add_key_file(&mut self, key_file: &KeyFile) -> Result<()> {
        let hash = key_file.hash()?;
        self.components.push(hash);
        Ok(())
    }

    /// Add hardware key (YubiKey challenge-response) component
    pub fn add_hardware_key(&mut self, response: Vec<u8>) {
        let mut hasher = Sha256::new();
        hasher.update(&response);
        self.components.push(hasher.finalize().to_vec());
    }

    /// Build the final composite key hash
    pub fn build(&self) -> Result<Vec<u8>> {
        if self.components.is_empty() {
            return Err(KeePassExError::InvalidCredentials);
        }

        let mut hasher = Sha256::new();
        for component in &self.components {
            hasher.update(component);
        }
        Ok(hasher.finalize().to_vec())
    }
}

impl Default for CompositeKey {
    fn default() -> Self {
        Self::new()
    }
}

/// Master key derived from composite key + KDF
#[derive(ZeroizeOnDrop)]
pub struct MasterKey {
    pub key: Vec<u8>,
}

impl MasterKey {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    /// Derive encryption key and HMAC key from master key + master seed
    pub fn derive_keys(&self, master_seed: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // Encryption key: SHA256(master_seed || transformed_key || 0x01)
        let mut enc_hasher = Sha256::new();
        enc_hasher.update(master_seed);
        enc_hasher.update(&self.key);
        enc_hasher.update(&[0x01u8]);
        let enc_key = enc_hasher.finalize().to_vec();

        // HMAC key: SHA512(master_seed || transformed_key || 0x01) — first 32 bytes
        use sha2::Sha512;
        let mut hmac_hasher = Sha512::new();
        hmac_hasher.update(master_seed);
        hmac_hasher.update(&self.key);
        hmac_hasher.update(&[0x01u8]);
        let hmac_full = hmac_hasher.finalize();
        let hmac_key = hmac_full[..32].to_vec();

        (enc_key, hmac_key)
    }
}

/// Key file formats supported
#[derive(Debug, Clone)]
pub enum KeyFile {
    /// XML key file (KeePass 2.x format)
    Xml { data: Vec<u8> },
    /// Binary key file (32 bytes)
    Binary { data: Vec<u8> },
    /// Hex-encoded key file (64 hex chars)
    Hex { data: Vec<u8> },
    /// Any file used as key (hashed)
    Arbitrary { data: Vec<u8> },
}

impl KeyFile {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        // Try to detect format
        if data.starts_with(b"<?xml") || data.starts_with(b"<KeyFile") {
            return KeyFile::Xml { data };
        }
        if data.len() == 32 {
            return KeyFile::Binary { data };
        }
        if data.len() == 64 {
            // Check if valid hex
            if data.iter().all(|b| b.is_ascii_hexdigit()) {
                return KeyFile::Hex { data };
            }
        }
        KeyFile::Arbitrary { data }
    }

    pub fn hash(&self) -> Result<Vec<u8>> {
        match self {
            KeyFile::Xml { data } => parse_xml_key_file(data),
            KeyFile::Binary { data } => Ok(data.clone()),
            KeyFile::Hex { data } => {
                let hex_str = std::str::from_utf8(data)
                    .map_err(|_| KeePassExError::Other("Invalid hex key file".into()))?;
                hex::decode(hex_str)
                    .map_err(|_| KeePassExError::Other("Invalid hex key file".into()))
            }
            KeyFile::Arbitrary { data } => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
        }
    }

    /// Generate a new random XML key file
    pub fn generate() -> (Self, Vec<u8>) {
        use rand::RngCore;
        let mut key_bytes = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);

        let hex_key = hex::encode(&key_bytes);
        let xml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<KeyFile>
    <Meta>
        <Version>2.0</Version>
    </Meta>
    <Key>
        <Data Hash="{}">{}</Data>
    </Key>
</KeyFile>"#,
            &hex_key[..8].to_uppercase(),
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &key_bytes)
        );

        let xml_bytes = xml.into_bytes();
        (KeyFile::Xml { data: xml_bytes.clone() }, xml_bytes)
    }
}

fn parse_xml_key_file(data: &[u8]) -> Result<Vec<u8>> {
    // Simple XML key file parser
    let content = std::str::from_utf8(data)
        .map_err(|_| KeePassExError::Other("Invalid XML key file encoding".into()))?;

    // Extract <Data> content
    let start = content
        .find("<Data")
        .ok_or_else(|| KeePassExError::Other("Invalid XML key file: no <Data> element".into()))?;
    let data_section = &content[start..];
    let content_start = data_section
        .find('>')
        .ok_or_else(|| KeePassExError::Other("Invalid XML key file: malformed <Data>".into()))?
        + 1;
    let content_end = data_section
        .find("</Data>")
        .ok_or_else(|| KeePassExError::Other("Invalid XML key file: no </Data>".into()))?;

    let key_data = data_section[content_start..content_end].trim();

    // Try base64 decode first
    if let Ok(decoded) = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        key_data,
    ) {
        return Ok(decoded);
    }

    // Try hex decode
    if let Ok(decoded) = hex::decode(key_data) {
        return Ok(decoded);
    }

    Err(KeePassExError::Other("Cannot decode XML key file data".into()))
}
