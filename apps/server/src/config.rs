//! Server configuration

/// Runtime configuration for the KeePassEx server
pub struct ServerConfig {
    /// JWT signing secret
    pub jwt_secret: String,
    /// Maximum vault file size in bytes
    pub max_vault_bytes: u64,
    /// Whether the admin API is enabled
    pub admin_enabled: bool,
    /// Admin API key (required if admin_enabled)
    pub admin_key: Option<String>,
}

impl ServerConfig {
    /// JWT token expiry in seconds (24 hours)
    pub const TOKEN_EXPIRY_SECS: u64 = 86_400;
    /// Refresh token expiry in seconds (30 days)
    pub const REFRESH_TOKEN_EXPIRY_SECS: u64 = 30 * 86_400;
    /// Maximum vault history versions to keep per user
    pub const MAX_VAULT_HISTORY: u32 = 10;
}
