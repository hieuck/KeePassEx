//! Credential tile — the UI shown on the Windows login screen
//!
//! The tile has two fields:
//! 1. A label: "KeePassEx — Vault Master Password"
//! 2. A password input field
//!
//! When the user submits, we:
//! 1. Run ZKPV pre-check (fast, no Argon2id)
//! 2. If ZKPV passes, retrieve Windows credentials from vault
//! 3. Return credentials to Windows for authentication

#![cfg(windows)]

use crate::credential::WindowsCredential;
use keepassex_core::zkpv::ZkpvCommitment;
use zeroize::Zeroize;

/// Field indices for our credential tile
pub const FIELD_LABEL: u32 = 0;
pub const FIELD_PASSWORD: u32 = 1;
pub const FIELD_SUBMIT: u32 = 2;
pub const FIELD_COUNT: u32 = 3;

/// The credential tile shown on the Windows login screen
pub struct KeePassExCredentialTile {
    /// The password entered by the user (zeroed after use)
    password: String,
    /// ZKPV commitment loaded from vault header (for fast pre-check)
    zkpv_commitment: Option<ZkpvCommitment>,
    /// Path to the vault file
    vault_path: Option<String>,
}

impl KeePassExCredentialTile {
    pub fn new() -> Self {
        Self {
            password: String::new(),
            zkpv_commitment: None,
            vault_path: None,
        }
    }

    /// Load vault configuration from the KeePassEx app data directory
    pub fn load_vault_config(&mut self) -> bool {
        let config_path = get_keepassex_config_path();
        if let Some(path) = config_path {
            self.vault_path = Some(path);
            // In production: load ZKPV commitment from vault header
            // self.zkpv_commitment = load_zkpv_from_vault(&path);
            true
        } else {
            false
        }
    }

    /// Called when the user types in the password field
    pub fn set_password(&mut self, password: &str) {
        self.password.zeroize();
        self.password = password.to_string();
    }

    /// Called when the user clicks Submit
    /// Returns Windows credentials if the vault password is correct
    pub fn submit(&mut self) -> Result<WindowsCredential, SubmitError> {
        if self.password.is_empty() {
            return Err(SubmitError::EmptyPassword);
        }

        // Step 1: ZKPV pre-check (fast, no Argon2id overhead)
        if let Some(ref commitment) = self.zkpv_commitment {
            match commitment.verify(&self.password) {
                Ok(false) => {
                    self.password.zeroize();
                    return Err(SubmitError::WrongPassword);
                }
                Err(_) => {
                    self.password.zeroize();
                    return Err(SubmitError::VaultError("ZKPV verification failed".into()));
                }
                Ok(true) => {} // Pre-check passed, proceed
            }
        }

        // Step 2: Retrieve Windows credentials from vault
        // In production: open vault with Argon2id, find Windows credential entry
        let credential = self.retrieve_windows_credential()?;

        // Zero out the password
        self.password.zeroize();

        Ok(credential)
    }

    fn retrieve_windows_credential(&self) -> Result<WindowsCredential, SubmitError> {
        // In production:
        // 1. Open vault with self.password using keepassex_core
        // 2. Find entry tagged "windows-credential" or in "Windows" group
        // 3. Return username + password from that entry
        //
        // For now, return a placeholder that demonstrates the flow
        let vault_path = self
            .vault_path
            .as_deref()
            .ok_or_else(|| SubmitError::VaultError("No vault configured".into()))?;

        tracing::info!("Retrieving Windows credentials from vault: {}", vault_path);

        // TODO: Implement full vault open + credential retrieval
        // This requires async runtime integration with the COM synchronous interface
        Err(SubmitError::VaultError(
            "Vault credential retrieval not yet implemented".into(),
        ))
    }

    /// Get the label text for a field
    pub fn get_field_label(&self, field_id: u32) -> &str {
        match field_id {
            FIELD_LABEL => "KeePassEx",
            FIELD_PASSWORD => "Vault Master Password",
            FIELD_SUBMIT => "Unlock",
            _ => "",
        }
    }

    /// Get the field type for Windows credential provider API
    /// Returns CPFT_* constants
    pub fn get_field_type(&self, field_id: u32) -> u32 {
        match field_id {
            FIELD_LABEL => 1,    // CPFT_LARGE_TEXT
            FIELD_PASSWORD => 3, // CPFT_PASSWORD_TEXT
            FIELD_SUBMIT => 7,   // CPFT_SUBMIT_BUTTON
            _ => 0,
        }
    }
}

impl Drop for KeePassExCredentialTile {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}

#[derive(Debug)]
pub enum SubmitError {
    EmptyPassword,
    WrongPassword,
    VaultError(String),
    WindowsAuthError(String),
}

impl std::fmt::Display for SubmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPassword => write!(f, "Password cannot be empty"),
            Self::WrongPassword => write!(f, "Wrong vault master password"),
            Self::VaultError(e) => write!(f, "Vault error: {}", e),
            Self::WindowsAuthError(e) => write!(f, "Windows authentication error: {}", e),
        }
    }
}

/// Get the KeePassEx app data directory path
fn get_keepassex_config_path() -> Option<String> {
    // %APPDATA%\KeePassEx\config.json
    std::env::var("APPDATA")
        .ok()
        .map(|appdata| format!("{}\\KeePassEx\\config.json", appdata))
}
