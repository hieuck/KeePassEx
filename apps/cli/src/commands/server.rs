//! CLI `server` command — manage KeePassEx self-hosted sync server
//!
//! Usage:
//!   kpx server status --url https://sync.example.com
//!   kpx server login --url https://sync.example.com --email you@example.com
//!   kpx server register --url https://sync.example.com --email you@example.com
//!   kpx server history --url https://sync.example.com --token <jwt>
//!
//! Environment variables:
//!   KPX_SERVER_URL      — server base URL
//!   KPX_SERVER_EMAIL    — email for login/register
//!   KPX_SERVER_PASSWORD — password for login/register
//!   KPX_SERVER_TOKEN    — JWT access token

use colored::Colorize;
use serde::Deserialize;

// ─── Response types ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: String,
    #[allow(dead_code)]
    user_id: String,
    email: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Deserialize)]
struct ServerInfoResponse {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version: String,
    user_count: u64,
    vault_count: u64,
    total_storage_mb: f64,
}

#[derive(Deserialize)]
struct VaultMetaResponse {
    version: u32,
    size_bytes: u64,
    uploaded_at: String,
    client_hash: Option<String>,
}

#[derive(Deserialize)]
struct VaultHistoryResponse {
    versions: Vec<VaultMetaResponse>,
}

// ─── Entry point ──────────────────────────────────────────────────────────────

/// Run a server subcommand.
/// `action` is the `ServerAction` enum from main.rs, passed by reference.
pub async fn run_status(url: &str) -> anyhow::Result<()> {
    let client = build_client()?;
    let base = url.trim_end_matches('/');

    let health: HealthResponse = client
        .get(format!("{}/health", base))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Cannot reach server: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Invalid health response: {}", e))?;

    println!("{} KeePassEx Server", "🔐".bold());
    println!("  URL:     {}", url.cyan());
    println!("  Status:  {}", health.status.green().bold());
    println!("  Version: {}", health.version.dimmed());

    if let Ok(resp) = client
        .get(format!("{}/api/v1/server/info", base))
        .send()
        .await
    {
        if let Ok(info) = resp.json::<ServerInfoResponse>().await {
            println!("  Users:   {}", info.user_count.to_string().bold());
            println!("  Vaults:  {}", info.vault_count.to_string().bold());
            println!("  Storage: {:.1} MB", info.total_storage_mb);
        }
    }

    Ok(())
}

pub async fn run_login(url: &str, email: &str, password: Option<&str>) -> anyhow::Result<()> {
    let password = get_password(password)?;
    let client = build_client()?;
    let base = url.trim_end_matches('/');

    let response = client
        .post(format!("{}/api/v1/auth/login", base))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Cannot reach server: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body: serde_json::Value = response.json().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Login failed ({}): {}",
            status,
            body["error"].as_str().unwrap_or("Unknown error")
        ));
    }

    let auth: AuthResponse = response.json().await?;

    println!("{} Logged in as {}", "✓".green().bold(), auth.email.cyan());
    println!();
    println!(
        "{}",
        format!("Access token (expires in {}s):", auth.expires_in).dimmed()
    );
    println!("{}", auth.access_token.yellow());
    println!();
    println!("{}", "Set as environment variable:".dimmed());
    println!("  export KPX_SERVER_TOKEN={}", auth.access_token.dimmed());
    println!();
    println!("{}", "Then sync your vault:".dimmed());
    println!(
        "  kpx sync --provider keepassex-server --remote {} push",
        url.dimmed()
    );

    Ok(())
}

pub async fn run_register(url: &str, email: &str, password: Option<&str>) -> anyhow::Result<()> {
    let password = get_password(password)?;

    let confirm = rpassword::prompt_password("Confirm password: ")?;
    if password != confirm {
        return Err(anyhow::anyhow!("Passwords do not match"));
    }

    let client = build_client()?;
    let base = url.trim_end_matches('/');

    let response = client
        .post(format!("{}/api/v1/auth/register", base))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Cannot reach server: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body: serde_json::Value = response.json().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Registration failed ({}): {}",
            status,
            body["error"].as_str().unwrap_or("Unknown error")
        ));
    }

    let auth: AuthResponse = response.json().await?;

    println!(
        "{} Account created: {}",
        "✓".green().bold(),
        auth.email.cyan()
    );
    println!();
    println!("{}", "Access token:".dimmed());
    println!("{}", auth.access_token.yellow());
    println!();
    println!("{}", "Set as environment variable:".dimmed());
    println!("  export KPX_SERVER_TOKEN={}", auth.access_token.dimmed());

    Ok(())
}

pub async fn run_history(url: &str, token: &str) -> anyhow::Result<()> {
    let client = build_client()?;
    let base = url.trim_end_matches('/');

    let response = client
        .get(format!("{}/api/v1/vault/history", base))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Cannot reach server: {}", e))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(anyhow::anyhow!(
            "Invalid or expired token. Run: kpx server login --url {} --email <email>",
            url
        ));
    }

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Server error: HTTP {}", response.status()));
    }

    let history: VaultHistoryResponse = response.json().await?;

    if history.versions.is_empty() {
        println!("{} No vault versions found on server", "ℹ".blue());
        return Ok(());
    }

    println!("{} Vault history on {}", "📋".bold(), url.cyan());
    println!();
    println!(
        "{:<8} {:<12} {:<30} {}",
        "Version".bold(),
        "Size".bold(),
        "Uploaded".bold(),
        "Hash".bold()
    );
    println!("{}", "─".repeat(70).dimmed());

    for v in &history.versions {
        let size = format_bytes(v.size_bytes);
        let hash = v.client_hash.as_deref().unwrap_or("—");
        let hash_short = if hash.len() > 12 { &hash[..12] } else { hash };
        println!(
            "{:<8} {:<12} {:<30} {}",
            v.version.to_string().cyan(),
            size.dimmed(),
            v.uploaded_at.dimmed(),
            hash_short.dimmed()
        );
    }

    println!();
    println!(
        "To restore a version: kpx sync --provider keepassex-server --remote {} pull",
        url.dimmed()
    );

    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn build_client() -> anyhow::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("kpx-cli/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))
}

fn get_password(password: Option<&str>) -> anyhow::Result<String> {
    if let Some(pw) = password {
        Ok(pw.to_string())
    } else {
        rpassword::prompt_password("Server password: ")
            .map_err(|e| anyhow::anyhow!("Failed to read password: {}", e))
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
