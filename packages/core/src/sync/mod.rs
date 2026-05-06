//! Vault sync engine — multi-provider, conflict-aware

pub mod merge;
pub mod providers;

use crate::error::Result;
use async_trait::async_trait;

/// Sync provider trait
#[async_trait]
pub trait SyncProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata>;
    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)>;
    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata>;
    async fn list(&self, path: &str) -> Result<Vec<SyncEntry>>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn test_connection(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct SyncMetadata {
    pub path: String,
    pub size: u64,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub etag: Option<String>,
    pub revision: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub metadata: Option<SyncMetadata>,
}

/// Sync configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncConfig {
    pub provider: SyncProviderType,
    pub remote_path: String,
    pub auto_sync: bool,
    pub sync_interval_seconds: u64,
    pub conflict_resolution: ConflictResolution,
    /// Provider-specific credentials (stored encrypted in vault settings)
    pub credentials: Option<SyncCredentials>,
}

/// Provider-specific credentials — all fields optional; each provider uses
/// only the fields relevant to it.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SyncCredentials {
    pub username: Option<String>,
    /// Password or app-specific password (never logged)
    pub password: Option<String>,
    /// OAuth2 access token
    pub token: Option<String>,
    /// AWS / S3-compatible access key ID
    pub access_key_id: Option<String>,
    /// AWS / S3-compatible secret access key (never logged)
    pub secret_access_key: Option<String>,
    /// AWS region (e.g. "us-east-1")
    pub region: Option<String>,
    /// S3 bucket name
    pub bucket: Option<String>,
    /// Custom S3-compatible endpoint URL
    pub endpoint: Option<String>,
    /// Path to PEM private key file (SFTP)
    pub private_key_path: Option<String>,
    /// Known host fingerprint for SFTP host verification (SHA-256 base64)
    pub host_fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SyncProviderType {
    WebDav,
    ICloudDrive,
    GoogleDrive,
    OneDrive,
    Dropbox,
    S3,
    SftpServer,
    LocalFolder,
    /// KeePassEx self-hosted server (zero-knowledge, end-to-end encrypted)
    KeePassExServer,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ConflictResolution {
    /// Keep local version
    KeepLocal,
    /// Keep remote version
    KeepRemote,
    /// Merge (CRDT-inspired, keeps newest modification per entry)
    Merge,
    /// Ask user
    AskUser,
}

/// Sync result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub status: SyncStatus,
    pub entries_uploaded: usize,
    pub entries_downloaded: usize,
    pub conflicts: Vec<SyncConflict>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Success,
    PartialSuccess,
    Conflict,
    Error,
    NoChanges,
}

#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub entry_uuid: String,
    pub local_modified: chrono::DateTime<chrono::Utc>,
    pub remote_modified: chrono::DateTime<chrono::Utc>,
    pub resolution: Option<ConflictResolution>,
}
