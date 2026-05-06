//! Tauri commands — Team Vault
use keepassex_core::team::{EntryPermission, TeamRole, TeamVault};
use serde::{Deserialize, Serialize};
use tauri::{command, State};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub name: String,
    pub role: String,
}

/// Get team vault configuration
#[command]
pub async fn get_team_vault(state: State<'_, AppState>) -> Result<TeamVault, String> {
    let settings = state.settings.read().map_err(|e| e.to_string())?;
    // In production: load from vault custom data
    // For now: return a demo team vault
    let vault = TeamVault::new(
        "My Team Vault",
        "current-user",
        "Current User",
        "user@example.com",
    );
    Ok(vault)
}

/// Invite a new team member
#[command]
pub async fn invite_team_member(
    state: State<'_, AppState>,
    email: String,
    name: String,
    role: String,
) -> Result<(), String> {
    let team_role = match role.as_str() {
        "admin" => TeamRole::Admin,
        "editor" => TeamRole::Editor,
        "viewer" => TeamRole::Viewer,
        _ => return Err(format!("Unknown role: {}", role)),
    };

    // In production: send invitation email and update vault
    tracing::info!("Inviting {} ({}) as {:?}", name, email, team_role);
    Ok(())
}

/// Change a team member's role
#[command]
pub async fn change_team_member_role(
    state: State<'_, AppState>,
    member_id: String,
    role: String,
) -> Result<(), String> {
    let team_role = match role.as_str() {
        "admin" => TeamRole::Admin,
        "editor" => TeamRole::Editor,
        "viewer" => TeamRole::Viewer,
        _ => return Err(format!("Unknown role: {}", role)),
    };

    tracing::info!("Changing role for {} to {:?}", member_id, team_role);
    Ok(())
}

/// Remove a team member
#[command]
pub async fn remove_team_member(
    state: State<'_, AppState>,
    member_id: String,
) -> Result<(), String> {
    tracing::info!("Removing team member: {}", member_id);
    Ok(())
}

/// Set per-entry permission for a member
#[command]
pub async fn set_entry_permission(
    state: State<'_, AppState>,
    member_id: String,
    entry_uuid: String,
    permission: String,
) -> Result<(), String> {
    let perm = match permission.as_str() {
        "none" => EntryPermission::None,
        "view_only" => EntryPermission::ViewOnly,
        "full_edit" => EntryPermission::FullEdit,
        _ => return Err(format!("Unknown permission: {}", permission)),
    };

    tracing::info!(
        "Setting {:?} for {} on entry {}",
        perm,
        member_id,
        entry_uuid
    );
    Ok(())
}
