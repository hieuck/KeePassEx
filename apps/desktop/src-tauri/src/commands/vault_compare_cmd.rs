//! Vault Compare Tauri commands

use crate::state::AppState;
use keepassex_core::vault::operations::{open_vault, VaultCredentials};
use keepassex_core::vault_compare::{compare_vaults, merge_vaults, MergeStrategy, VaultDiff};
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn compare_vaults_cmd(
    vault1_path: String,
    vault2_path: String,
    password1: String,
    password2: String,
) -> Result<VaultDiff, String> {
    let creds1 = VaultCredentials::password_only(&password1);
    let creds2 = VaultCredentials::password_only(&password2);

    let vault1 = open_vault(std::path::Path::new(&vault1_path), &creds1)
        .await
        .map_err(|e| format!("Cannot open first vault: {e}"))?;

    let vault2 = open_vault(std::path::Path::new(&vault2_path), &creds2)
        .await
        .map_err(|e| format!("Cannot open second vault: {e}"))?;

    Ok(compare_vaults(&vault1, &vault2))
}

#[tauri::command]
pub async fn merge_vaults_cmd(
    target_vault_path: String,
    source_vault_path: String,
    password: String,
    strategy: String,
) -> Result<(), String> {
    let creds = VaultCredentials::password_only(&password);

    let mut target = open_vault(std::path::Path::new(&target_vault_path), &creds)
        .await
        .map_err(|e| format!("Cannot open target vault: {e}"))?;

    let source = open_vault(std::path::Path::new(&source_vault_path), &creds)
        .await
        .map_err(|e| format!("Cannot open source vault: {e}"))?;

    let merge_strategy = match strategy.as_str() {
        "KeepFirst" => MergeStrategy::KeepFirst,
        "KeepSecond" => MergeStrategy::KeepSecond,
        "KeepBoth" => MergeStrategy::KeepBoth,
        _ => MergeStrategy::KeepNewer,
    };

    merge_vaults(&mut target, &source, merge_strategy).map_err(|e| e.to_string())?;

    // Save merged vault
    keepassex_core::vault::operations::save_vault(
        &target,
        std::path::Path::new(&target_vault_path),
        &creds,
    )
    .await
    .map_err(|e| format!("Cannot save merged vault: {e}"))?;

    Ok(())
}
