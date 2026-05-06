//! CLI `sync` command — sync vault with remote provider

use colored::Colorize;
use keepassex_core::{
    sync::providers::LocalFolderProvider,
    sync::{ConflictResolution, SyncConfig, SyncProviderType},
    vault::operations::{open_vault, save_vault, VaultCredentials},
    Vault,
};
use std::path::Path;

pub async fn run(
    vault: &Vault,
    vault_path: &str,
    password: &str,
    provider: &str,
    remote_path: &str,
    direction: &str,
) -> anyhow::Result<()> {
    eprintln!(
        "Syncing with {} ({})...",
        provider.bold(),
        remote_path.dimmed()
    );

    match provider.to_lowercase().as_str() {
        "local" | "folder" => sync_local(vault, vault_path, password, remote_path, direction).await,
        "webdav" => {
            eprintln!(
                "{} WebDAV sync requires the desktop app. Use: KeePassEx → Settings → Sync",
                "ℹ".blue()
            );
            Ok(())
        }
        "keepassex-server" | "keepassex_server" | "kpx-server" | "server" => {
            sync_keepassex_server(vault, vault_path, password, remote_path, direction).await
        }
        _ => Err(anyhow::anyhow!(
            "Provider '{}' not supported in CLI. Supported: local, keepassex-server",
            provider
        )),
    }
}

async fn sync_local(
    vault: &Vault,
    vault_path: &str,
    password: &str,
    remote_path: &str,
    direction: &str,
) -> anyhow::Result<()> {
    let remote = Path::new(remote_path);

    match direction {
        "push" | "upload" => {
            // Copy local vault to remote
            std::fs::copy(vault_path, remote)?;
            eprintln!("{} Pushed to {}", "✓".green(), remote_path.bold());
        }
        "pull" | "download" => {
            // Copy remote vault to local (backup first)
            let backup = format!("{}.bak", vault_path);
            std::fs::copy(vault_path, &backup)?;
            std::fs::copy(remote, vault_path)?;
            eprintln!(
                "{} Pulled from {} (backup: {})",
                "✓".green(),
                remote_path.bold(),
                backup.dimmed()
            );
        }
        "merge" | "both" => {
            // Open remote vault and merge
            let credentials = VaultCredentials::password_only(password);
            let remote_vault = open_vault(remote, &credentials).await?;

            let merged = keepassex_core::sync::merge::merge_vaults(vault, &remote_vault)?;
            save_vault(&merged, Path::new(vault_path), &credentials).await?;
            save_vault(&merged, remote, &credentials).await?;

            eprintln!("{} Merged successfully", "✓".green());
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown direction '{}'. Use: push, pull, merge",
                direction
            ));
        }
    }

    Ok(())
}

/// Sync with KeePassEx self-hosted server
///
/// Usage:
///   kpx sync --provider keepassex-server --remote https://sync.example.com push
///   kpx sync --provider keepassex-server --remote https://sync.example.com pull
///
/// Authentication: set KPX_SERVER_TOKEN env var with your JWT access token.
/// Get a token: POST https://sync.example.com/api/v1/auth/login
async fn sync_keepassex_server(
    vault: &Vault,
    vault_path: &str,
    password: &str,
    server_url: &str,
    direction: &str,
) -> anyhow::Result<()> {
    use keepassex_core::sync::providers::KeePassExServerProvider;
    use keepassex_core::sync::SyncProvider;

    // Get JWT token from environment variable
    let access_token = std::env::var("KPX_SERVER_TOKEN").map_err(|_| {
        anyhow::anyhow!(
            "KPX_SERVER_TOKEN environment variable not set.\n\
             Get a token by logging in:\n\
             curl -X POST {}/api/v1/auth/login \\\n\
             -H 'Content-Type: application/json' \\\n\
             -d '{{\"email\":\"you@example.com\",\"password\":\"yourpassword\"}}'\n\
             Then: export KPX_SERVER_TOKEN=<access_token>",
            server_url
        )
    })?;

    let provider = KeePassExServerProvider {
        server_url: server_url.to_string(),
        access_token,
    };

    // Test connection first
    provider
        .test_connection()
        .await
        .map_err(|e| anyhow::anyhow!("Cannot connect to KeePassEx Server: {}", e))?;

    match direction {
        "push" | "upload" => {
            // Read vault file and upload
            let vault_data = std::fs::read(vault_path)?;
            let meta = provider
                .upload("/vault.kdbx", &vault_data)
                .await
                .map_err(|e| anyhow::anyhow!("Upload failed: {}", e))?;
            eprintln!(
                "{} Pushed to {} (version {})",
                "✓".green(),
                server_url.bold(),
                meta.revision.unwrap_or_else(|| "?".to_string()).dimmed()
            );
        }
        "pull" | "download" => {
            // Download and save vault
            let (data, meta) = provider
                .download("/vault.kdbx")
                .await
                .map_err(|e| anyhow::anyhow!("Download failed: {}", e))?;

            // Backup existing vault
            let backup = format!("{}.bak", vault_path);
            std::fs::copy(vault_path, &backup)?;

            std::fs::write(vault_path, &data)?;
            eprintln!(
                "{} Pulled from {} (version {}, backup: {})",
                "✓".green(),
                server_url.bold(),
                meta.revision.unwrap_or_else(|| "?".to_string()).dimmed(),
                backup.dimmed()
            );
        }
        "status" => {
            let meta = provider
                .get_metadata("/vault.kdbx")
                .await
                .map_err(|e| anyhow::anyhow!("Cannot get metadata: {}", e))?;
            eprintln!("{} Server vault status:", "ℹ".blue());
            eprintln!(
                "  Version:     {}",
                meta.revision.unwrap_or_else(|| "?".to_string())
            );
            eprintln!("  Size:        {} bytes", meta.size);
            eprintln!(
                "  Last sync:   {}",
                meta.modified_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown direction '{}'. Use: push, pull, status",
                direction
            ));
        }
    }

    Ok(())
}
