//! CLI `sync` command — sync vault with remote provider

use keepassex_core::{
    Vault,
    sync::{SyncConfig, SyncProviderType, ConflictResolution},
    sync::providers::LocalFolderProvider,
    vault::operations::{open_vault, save_vault, VaultCredentials},
};
use colored::Colorize;
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
        "local" | "folder" => {
            sync_local(vault, vault_path, password, remote_path, direction).await
        }
        "webdav" => {
            eprintln!(
                "{} WebDAV sync requires the desktop app. Use: KeePassEx → Settings → Sync",
                "ℹ".blue()
            );
            Ok(())
        }
        _ => Err(anyhow::anyhow!(
            "Provider '{}' not supported in CLI. Supported: local, webdav",
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
            eprintln!("{} Pulled from {} (backup: {})", "✓".green(), remote_path.bold(), backup.dimmed());
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
