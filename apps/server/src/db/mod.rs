//! Database layer — SQLite via sqlx
//!
//! Schema:
//! - users: user accounts (email, password hash, created_at)
//! - vaults: encrypted vault blobs (user_id, data, version, uploaded_at)
//! - sessions: JWT refresh tokens (user_id, token_hash, expires_at)
//! - audit_log: server-side audit events (user_id, action, ip, timestamp)

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Open (or create) the SQLite database
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let url = format!("sqlite://{}?mode=rwc", path);
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&url)
            .await?;
        Ok(Self { pool })
    }

    /// Run database migrations (create tables if they don't exist)
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id          TEXT PRIMARY KEY,
                email       TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                last_login  TEXT
            );

            CREATE TABLE IF NOT EXISTS vaults (
                id          TEXT PRIMARY KEY,
                user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                data        BLOB NOT NULL,
                version     INTEGER NOT NULL DEFAULT 1,
                size_bytes  INTEGER NOT NULL,
                uploaded_at TEXT NOT NULL,
                client_hash TEXT,
                UNIQUE(user_id, version)
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash  TEXT NOT NULL UNIQUE,
                expires_at  TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                ip_address  TEXT
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id          TEXT PRIMARY KEY,
                user_id     TEXT REFERENCES users(id) ON DELETE SET NULL,
                action      TEXT NOT NULL,
                ip_address  TEXT,
                user_agent  TEXT,
                details     TEXT,
                created_at  TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_vaults_user_id ON vaults(user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_audit_user_id ON audit_log(user_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Database migrations complete");
        Ok(())
    }

    // ─── User operations ──────────────────────────────────────────────────────

    pub async fn create_user(&self, email: &str, password_hash: &str) -> anyhow::Result<User> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(email)
        .bind(password_hash)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(User {
            id,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            created_at: Utc::now(),
            last_login: None,
        })
    }

    pub async fn find_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, created_at, last_login FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_user_by_id(&self, id: &str) -> anyhow::Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, created_at, last_login FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn update_last_login(&self, user_id: &str) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
            .bind(&now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_users(&self) -> anyhow::Result<Vec<User>> {
        let rows = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, created_at, last_login FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(User::from).collect())
    }

    pub async fn delete_user(&self, user_id: &str) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ─── Vault operations ─────────────────────────────────────────────────────

    pub async fn get_vault_meta(&self, user_id: &str) -> anyhow::Result<Option<VaultMeta>> {
        let row = sqlx::query_as::<_, VaultMetaRow>(
            "SELECT id, user_id, version, size_bytes, uploaded_at, client_hash
             FROM vaults WHERE user_id = ? ORDER BY version DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(VaultMeta::from))
    }

    pub async fn upload_vault(
        &self,
        user_id: &str,
        data: &[u8],
        client_hash: Option<&str>,
        max_history: u32,
    ) -> anyhow::Result<VaultMeta> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let size = data.len() as i64;

        // Get current max version
        let current_version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM vaults WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let new_version = current_version + 1;

        sqlx::query(
            "INSERT INTO vaults (id, user_id, data, version, size_bytes, uploaded_at, client_hash)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(data)
        .bind(new_version)
        .bind(size)
        .bind(&now)
        .bind(client_hash)
        .execute(&self.pool)
        .await?;

        // Prune old versions beyond max_history
        sqlx::query(
            "DELETE FROM vaults WHERE user_id = ? AND version <= (
                SELECT MAX(version) - ? FROM vaults WHERE user_id = ?
             )",
        )
        .bind(user_id)
        .bind(max_history as i64)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(VaultMeta {
            id,
            user_id: user_id.to_string(),
            version: new_version as u32,
            size_bytes: size as u64,
            uploaded_at: Utc::now(),
            client_hash: client_hash.map(String::from),
        })
    }

    pub async fn download_vault(&self, user_id: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM vaults WHERE user_id = ? ORDER BY version DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(data,)| data))
    }

    pub async fn get_vault_history(&self, user_id: &str) -> anyhow::Result<Vec<VaultMeta>> {
        let rows = sqlx::query_as::<_, VaultMetaRow>(
            "SELECT id, user_id, version, size_bytes, uploaded_at, client_hash
             FROM vaults WHERE user_id = ? ORDER BY version DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(VaultMeta::from).collect())
    }

    pub async fn get_vault_version(
        &self,
        user_id: &str,
        version: u32,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM vaults WHERE user_id = ? AND version = ?",
        )
        .bind(user_id)
        .bind(version as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(data,)| data))
    }

    // ─── Session operations ───────────────────────────────────────────────────

    pub async fn create_session(
        &self,
        user_id: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
        ip_address: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO sessions (id, user_id, token_hash, expires_at, created_at, ip_address)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at.to_rfc3339())
        .bind(&now)
        .bind(ip_address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_session(&self, token_hash: &str) -> anyhow::Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT user_id FROM sessions WHERE token_hash = ? AND expires_at > ?",
        )
        .bind(token_hash)
        .bind(Utc::now().to_rfc3339())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(user_id,)| user_id))
    }

    pub async fn delete_session(&self, token_hash: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ─── Audit log ────────────────────────────────────────────────────────────

    pub async fn log_event(
        &self,
        user_id: Option<&str>,
        action: &str,
        ip_address: Option<&str>,
        details: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO audit_log (id, user_id, action, ip_address, details, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(action)
        .bind(ip_address)
        .bind(details)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ─── Stats ────────────────────────────────────────────────────────────────

    pub async fn get_stats(&self) -> anyhow::Result<ServerStats> {
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        let vault_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT user_id) FROM vaults",
        )
        .fetch_one(&self.pool)
        .await?;

        let total_storage: i64 =
            sqlx::query_scalar("SELECT COALESCE(SUM(size_bytes), 0) FROM vaults")
                .fetch_one(&self.pool)
                .await?;

        Ok(ServerStats {
            user_count: user_count as u64,
            vault_count: vault_count as u64,
            total_storage_bytes: total_storage as u64,
        })
    }
}

// ─── Domain models ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct VaultMeta {
    pub id: String,
    pub user_id: String,
    pub version: u32,
    pub size_bytes: u64,
    pub uploaded_at: DateTime<Utc>,
    pub client_hash: Option<String>,
}

#[derive(Debug)]
pub struct ServerStats {
    pub user_count: u64,
    pub vault_count: u64,
    pub total_storage_bytes: u64,
}

// ─── sqlx row types ───────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    password_hash: String,
    created_at: String,
    last_login: Option<String>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            created_at: row.created_at.parse().unwrap_or_else(|_| Utc::now()),
            last_login: row.last_login.and_then(|s| s.parse().ok()),
        }
    }
}

#[derive(sqlx::FromRow)]
struct VaultMetaRow {
    id: String,
    user_id: String,
    version: i64,
    size_bytes: i64,
    uploaded_at: String,
    client_hash: Option<String>,
}

impl From<VaultMetaRow> for VaultMeta {
    fn from(row: VaultMetaRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            version: row.version as u32,
            size_bytes: row.size_bytes as u64,
            uploaded_at: row.uploaded_at.parse().unwrap_or_else(|_| Utc::now()),
            client_hash: row.client_hash,
        }
    }
}
