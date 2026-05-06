//! High-level vault operations: open, save, lock, merge

use crate::error::{KeePassExError, Result};
use crate::vault::Vault;
use crate::kdbx::{KdbxReader, KdbxWriter};
use crate::crypto::keys::CompositeKey;
use std::path::Path;

/// Credentials for opening a vault
pub struct VaultCredentials {
    pub password: Option<String>,
    pub key_file_data: Option<Vec<u8>>,
    pub hardware_key_response: Option<Vec<u8>>,
}

impl VaultCredentials {
    pub fn password_only(password: impl Into<String>) -> Self {
        Self {
            password: Some(password.into()),
            key_file_data: None,
            hardware_key_response: None,
        }
    }

    pub fn build_composite_key(&self) -> Result<CompositeKey> {
        let mut key = CompositeKey::new();

        if let Some(ref pw) = self.password {
            key.add_password(pw);
        }

        if let Some(ref kf_data) = self.key_file_data {
            use crate::crypto::keys::KeyFile;
            let kf = KeyFile::from_bytes(kf_data.clone());
            key.add_key_file(&kf)?;
        }

        if let Some(ref hw) = self.hardware_key_response {
            key.add_hardware_key(hw.clone());
        }

        Ok(key)
    }
}

/// Open a KDBX vault from file
pub async fn open_vault(path: &Path, credentials: &VaultCredentials) -> Result<Vault> {
    let data = tokio::fs::read(path).await?;
    let composite_key = credentials.build_composite_key()?;
    let raw_key = composite_key.build()?;

    let reader = KdbxReader::new();
    reader.read(&data, &raw_key)
}

/// Save a vault to file
pub async fn save_vault(vault: &Vault, path: &Path, credentials: &VaultCredentials) -> Result<()> {
    let composite_key = credentials.build_composite_key()?;
    let raw_key = composite_key.build()?;

    let writer = KdbxWriter::new();
    let data = writer.write(vault, &raw_key)?;

    // Atomic write: write to temp file then rename
    let tmp_path = path.with_extension("kdbx.tmp");
    tokio::fs::write(&tmp_path, &data).await?;
    tokio::fs::rename(&tmp_path, path).await?;

    Ok(())
}

/// Change vault master password
pub async fn change_credentials(
    vault: &Vault,
    path: &Path,
    old_credentials: &VaultCredentials,
    new_credentials: &VaultCredentials,
) -> Result<()> {
    // Verify old credentials by attempting to open
    let data = tokio::fs::read(path).await?;
    let old_key = old_credentials.build_composite_key()?.build()?;
    let reader = KdbxReader::new();
    reader.read(&data, &old_key)?; // Will fail if wrong credentials

    // Save with new credentials
    save_vault(vault, path, new_credentials).await
}
