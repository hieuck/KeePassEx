//! Scheduled Backup — automatic vault backup on schedule
//!
//! Unique feature: no competitor has built-in scheduled backup.
//! Supports backup to local folder, with configurable retention policy.
//!
//! # How it works
//! 1. User configures backup schedule (daily/weekly/monthly)
//! 2. On each vault save, check if backup is due
//! 3. If due, copy the encrypted vault file to the backup destination
//! 4. Apply retention policy (keep last N backups)
//! 5. Record backup in history
//!
//! The backup is always the encrypted KDBX file — no plaintext data is written.

use crate::error::{KeePassExError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ─── Types ────────────────────────────────────────────────────────────────────

/// Backup schedule frequency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupFrequency {
    /// Backup every time the vault is saved
    OnSave,
    /// Backup once per day (at most)
    Daily,
    /// Backup once per week (at most)
    Weekly,
    /// Backup once per month (at most)
    Monthly,
}

impl std::fmt::Display for BackupFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnSave => write!(f, "On Save"),
            Self::Daily => write!(f, "Daily"),
            Self::Weekly => write!(f, "Weekly"),
            Self::Monthly => write!(f, "Monthly"),
        }
    }
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub frequency: BackupFrequency,
    /// Destination directory for backups
    pub destination: String,
    /// Maximum number of backup files to keep (0 = unlimited)
    pub max_backups: usize,
    /// Whether to include a timestamp in the backup filename
    pub timestamp_in_filename: bool,
    /// Last backup timestamp (ISO 8601)
    pub last_backup_at: Option<String>,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: BackupFrequency::Daily,
            destination: String::new(),
            max_backups: 10,
            timestamp_in_filename: true,
            last_backup_at: None,
        }
    }
}

/// A single backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub path: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub vault_name: String,
}

/// Result of a backup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub backup_path: Option<String>,
    pub error: Option<String>,
    pub backups_pruned: usize,
}

// ─── Backup Engine ────────────────────────────────────────────────────────────

/// Check if a backup is due based on the configuration and last backup time.
pub fn is_backup_due(config: &BackupConfig) -> bool {
    if !config.enabled {
        return false;
    }
    if config.destination.is_empty() {
        return false;
    }

    let last = match &config.last_backup_at {
        None => return true, // Never backed up
        Some(ts) => match chrono::DateTime::parse_from_rfc3339(ts) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(_) => return true,
        },
    };

    let now = chrono::Utc::now();
    let elapsed = now - last;

    match config.frequency {
        BackupFrequency::OnSave => true,
        BackupFrequency::Daily => elapsed.num_hours() >= 24,
        BackupFrequency::Weekly => elapsed.num_days() >= 7,
        BackupFrequency::Monthly => elapsed.num_days() >= 30,
    }
}

/// Perform a backup of the vault file.
///
/// Copies the encrypted vault file to the backup destination with an optional
/// timestamp in the filename. Applies retention policy after copying.
pub async fn perform_backup(vault_path: &Path, config: &mut BackupConfig) -> Result<BackupResult> {
    if !config.enabled {
        return Ok(BackupResult {
            success: false,
            backup_path: None,
            error: Some("Backup is disabled".to_string()),
            backups_pruned: 0,
        });
    }

    let dest_dir = Path::new(&config.destination);
    if !dest_dir.exists() {
        tokio::fs::create_dir_all(dest_dir)
            .await
            .map_err(|e| KeePassExError::Other(format!("Cannot create backup directory: {e}")))?;
    }

    // Build backup filename
    let vault_stem = vault_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("vault");

    let backup_filename = if config.timestamp_in_filename {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        format!("{vault_stem}_backup_{ts}.kdbx")
    } else {
        format!("{vault_stem}_backup.kdbx")
    };

    let backup_path = dest_dir.join(&backup_filename);

    // Copy vault file
    match tokio::fs::copy(vault_path, &backup_path).await {
        Ok(_) => {
            config.last_backup_at = Some(chrono::Utc::now().to_rfc3339());

            // Apply retention policy
            let pruned = if config.max_backups > 0 {
                prune_old_backups(dest_dir, vault_stem, config.max_backups).await
            } else {
                0
            };

            Ok(BackupResult {
                success: true,
                backup_path: Some(backup_path.to_string_lossy().to_string()),
                error: None,
                backups_pruned: pruned,
            })
        }
        Err(e) => Ok(BackupResult {
            success: false,
            backup_path: None,
            error: Some(format!("Backup failed: {e}")),
            backups_pruned: 0,
        }),
    }
}

/// List all backup files for a vault in the backup directory.
pub async fn list_backups(backup_dir: &Path, vault_stem: &str) -> Result<Vec<BackupRecord>> {
    if !backup_dir.exists() {
        return Ok(vec![]);
    }

    let mut records = Vec::new();
    let mut dir = tokio::fs::read_dir(backup_dir).await?;

    while let Some(entry) = dir.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&format!("{vault_stem}_backup")) && name.ends_with(".kdbx") {
            let meta = entry.metadata().await?;
            let created_at = meta
                .modified()
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

            records.push(BackupRecord {
                path: entry.path().to_string_lossy().to_string(),
                created_at,
                size_bytes: meta.len(),
                vault_name: vault_stem.to_string(),
            });
        }
    }

    // Sort by creation time, newest first
    records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(records)
}

/// Restore a vault from a backup file.
///
/// Copies the backup file over the current vault file.
/// The caller is responsible for re-opening the vault after restoration.
pub async fn restore_from_backup(backup_path: &Path, vault_path: &Path) -> Result<()> {
    // Create a safety copy of the current vault before overwriting
    let safety_path = vault_path.with_extension("kdbx.before_restore");
    if vault_path.exists() {
        tokio::fs::copy(vault_path, &safety_path)
            .await
            .map_err(|e| KeePassExError::Other(format!("Cannot create safety copy: {e}")))?;
    }

    tokio::fs::copy(backup_path, vault_path)
        .await
        .map_err(|e| KeePassExError::Other(format!("Cannot restore backup: {e}")))?;

    Ok(())
}

/// Delete a specific backup file.
pub async fn delete_backup(backup_path: &Path) -> Result<()> {
    tokio::fs::remove_file(backup_path)
        .await
        .map_err(|e| KeePassExError::Other(format!("Cannot delete backup: {e}")))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Remove old backup files, keeping only the `max_keep` most recent.
/// Returns the number of files deleted.
async fn prune_old_backups(backup_dir: &Path, vault_stem: &str, max_keep: usize) -> usize {
    let records = match list_backups(backup_dir, vault_stem).await {
        Ok(r) => r,
        Err(_) => return 0,
    };

    if records.len() <= max_keep {
        return 0;
    }

    let to_delete = &records[max_keep..];
    let mut deleted = 0;

    for record in to_delete {
        if tokio::fs::remove_file(&record.path).await.is_ok() {
            deleted += 1;
        }
    }

    deleted
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_due_never_backed_up() {
        let config = BackupConfig {
            enabled: true,
            frequency: BackupFrequency::Daily,
            destination: "/tmp/backups".to_string(),
            max_backups: 10,
            timestamp_in_filename: true,
            last_backup_at: None,
        };
        assert!(is_backup_due(&config));
    }

    #[test]
    fn test_backup_not_due_when_disabled() {
        let config = BackupConfig {
            enabled: false,
            ..BackupConfig::default()
        };
        assert!(!is_backup_due(&config));
    }

    #[test]
    fn test_backup_not_due_empty_destination() {
        let config = BackupConfig {
            enabled: true,
            destination: String::new(),
            ..BackupConfig::default()
        };
        assert!(!is_backup_due(&config));
    }

    #[test]
    fn test_backup_due_on_save() {
        let config = BackupConfig {
            enabled: true,
            frequency: BackupFrequency::OnSave,
            destination: "/tmp/backups".to_string(),
            last_backup_at: Some(chrono::Utc::now().to_rfc3339()),
            ..BackupConfig::default()
        };
        assert!(is_backup_due(&config));
    }

    #[test]
    fn test_backup_not_due_daily_recent() {
        let recent = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let config = BackupConfig {
            enabled: true,
            frequency: BackupFrequency::Daily,
            destination: "/tmp/backups".to_string(),
            last_backup_at: Some(recent),
            ..BackupConfig::default()
        };
        assert!(!is_backup_due(&config));
    }

    #[test]
    fn test_backup_due_daily_old() {
        let old = (chrono::Utc::now() - chrono::Duration::hours(25)).to_rfc3339();
        let config = BackupConfig {
            enabled: true,
            frequency: BackupFrequency::Daily,
            destination: "/tmp/backups".to_string(),
            last_backup_at: Some(old),
            ..BackupConfig::default()
        };
        assert!(is_backup_due(&config));
    }

    #[test]
    fn test_frequency_display() {
        assert_eq!(BackupFrequency::Daily.to_string(), "Daily");
        assert_eq!(BackupFrequency::OnSave.to_string(), "On Save");
    }

    #[test]
    fn test_default_config() {
        let config = BackupConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_backups, 10);
        assert!(config.timestamp_in_filename);
    }
}
