//! KeePassEx error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, KeePassExError>;

#[derive(Debug, Error)]
pub enum KeePassExError {
    // --- Vault errors ---
    #[error("Invalid master password or key file")]
    InvalidCredentials,

    #[error("Vault file not found: {path}")]
    VaultNotFound { path: String },

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Vault is already open")]
    VaultAlreadyOpen,

    #[error("Unsupported KDBX version: {version}")]
    UnsupportedVersion { version: u32 },

    #[error("Corrupted vault: {reason}")]
    CorruptedVault { reason: String },

    // --- Crypto errors ---
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed — wrong password or corrupted data")]
    DecryptionFailed,

    #[error("Key derivation failed: {0}")]
    KdfFailed(String),

    #[error("HMAC verification failed — vault may be tampered")]
    HmacVerificationFailed,

    // --- Entry errors ---
    #[error("Entry not found: {uuid}")]
    EntryNotFound { uuid: String },

    #[error("Group not found: {uuid}")]
    GroupNotFound { uuid: String },

    #[error("Duplicate entry UUID: {uuid}")]
    DuplicateEntry { uuid: String },

    // --- Sync errors ---
    #[error("Sync conflict: {details}")]
    SyncConflict { details: String },

    #[error("Sync provider error: {0}")]
    SyncProviderError(String),

    // --- OTP errors ---
    #[error("Invalid OTP secret")]
    InvalidOtpSecret,

    #[error("OTP generation failed: {0}")]
    OtpFailed(String),

    // --- Passkey errors ---
    #[error("Passkey not found for origin: {origin}")]
    PasskeyNotFound { origin: String },

    #[error("Passkey operation failed: {0}")]
    PasskeyFailed(String),

    // --- SSH errors ---
    #[error("SSH key parse error: {0}")]
    SshKeyError(String),

    // --- Hardware key errors ---
    #[error("Hardware key not connected")]
    HardwareKeyNotConnected,

    #[error("Hardware key operation failed: {0}")]
    HardwareKeyFailed(String),

    #[error("Hardware key type not supported on this platform: {0}")]
    HardwareKeyNotSupported(String),

    #[error("Hardware key touch required — please touch your key")]
    HardwareKeyTouchRequired,

    #[error("Hardware key timeout — no response received")]
    HardwareKeyTimeout,

    // --- Plugin errors ---
    #[error("Plugin not found: {id}")]
    PluginNotFound { id: String },

    #[error("Plugin execution failed: {0}")]
    PluginFailed(String),

    #[error("Plugin permission denied: {permission}")]
    PluginPermissionDenied { permission: String },

    // --- Import/Export errors ---
    #[error("Unsupported import format: {format}")]
    UnsupportedImportFormat { format: String },

    #[error("Import parse error: {0}")]
    ImportParseFailed(String),

    // --- I/O errors ---
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // --- Serialization errors ---
    #[error("Serialization error: {0}")]
    Serialization(String),

    // --- Generic ---
    #[error("{0}")]
    Other(String),
}
