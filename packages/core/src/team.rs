//! Team Vault — Collaborative vault with RBAC
//!
//! Enables sharing vault access with team members using role-based access control.
//! All data is end-to-end encrypted — the server never sees plaintext.
//!
//! # Roles
//! - **Admin**: Full access, can manage members and permissions
//! - **Editor**: Can create, edit, and delete entries
//! - **Viewer**: Can view entries and copy passwords (read-only)
//!
//! # Per-Entry Permissions
//! Override role-level permissions for specific entries:
//! - Alice (Editor) can be restricted to Viewer on "CEO Credentials"
//! - Bob (Viewer) can be granted Editor on "Shared Dev Keys"
//!
//! # Encrypted Comments
//! Team members can add encrypted comments to entries.
//! Comments are encrypted with the team vault key.

use crate::error::{KeePassExError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Team member role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    Admin,
    Editor,
    Viewer,
}

impl TeamRole {
    /// Check if this role can perform an action
    pub fn can(&self, action: TeamAction) -> bool {
        match (self, action) {
            // Admin can do everything
            (TeamRole::Admin, _) => true,
            // Editor can view, edit, create, delete entries
            (TeamRole::Editor, TeamAction::ViewEntry) => true,
            (TeamRole::Editor, TeamAction::EditEntry) => true,
            (TeamRole::Editor, TeamAction::CreateEntry) => true,
            (TeamRole::Editor, TeamAction::DeleteEntry) => true,
            (TeamRole::Editor, TeamAction::CopyPassword) => true,
            (TeamRole::Editor, TeamAction::AddComment) => true,
            // Viewer can only view and copy
            (TeamRole::Viewer, TeamAction::ViewEntry) => true,
            (TeamRole::Viewer, TeamAction::CopyPassword) => true,
            (TeamRole::Viewer, TeamAction::AddComment) => true,
            _ => false,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TeamRole::Admin => "Admin",
            TeamRole::Editor => "Editor",
            TeamRole::Viewer => "Viewer",
        }
    }
}

/// Actions that can be performed on a team vault
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamAction {
    ViewEntry,
    EditEntry,
    CreateEntry,
    DeleteEntry,
    CopyPassword,
    ManageMembers,
    ManagePermissions,
    ExportVault,
    AddComment,
    DeleteComment,
}

/// Team member status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberStatus {
    /// Invitation sent, not yet accepted
    Invited,
    /// Active member
    Active,
    /// Suspended (cannot access)
    Suspended,
    /// Removed from team
    Removed,
}

/// A team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: TeamRole,
    pub status: MemberStatus,
    pub joined_at: Option<DateTime<Utc>>,
    pub invited_at: DateTime<Utc>,
    pub invited_by: String,
    /// Per-entry permission overrides: entry_uuid → permission
    pub entry_overrides: HashMap<String, EntryPermission>,
    /// Public key for E2E encryption
    pub public_key: Vec<u8>,
}

impl TeamMember {
    /// Create a new team member invitation
    pub fn new_invitation(
        email: &str,
        name: &str,
        role: TeamRole,
        invited_by: &str,
        public_key: Vec<u8>,
    ) -> Self {
        TeamMember {
            id: Uuid::new_v4().to_string(),
            email: email.to_string(),
            name: name.to_string(),
            role,
            status: MemberStatus::Invited,
            joined_at: None,
            invited_at: Utc::now(),
            invited_by: invited_by.to_string(),
            entry_overrides: HashMap::new(),
            public_key,
        }
    }

    /// Accept invitation and activate member
    pub fn accept(&mut self) {
        self.status = MemberStatus::Active;
        self.joined_at = Some(Utc::now());
    }

    /// Check if member can perform action on a specific entry
    pub fn can_on_entry(&self, action: TeamAction, entry_uuid: &str) -> bool {
        if self.status != MemberStatus::Active {
            return false;
        }
        // Check per-entry override first
        if let Some(override_perm) = self.entry_overrides.get(entry_uuid) {
            return override_perm.allows(action);
        }
        // Fall back to role-level permission
        self.role.can(action)
    }

    /// Set per-entry permission override
    pub fn set_entry_permission(&mut self, entry_uuid: &str, permission: EntryPermission) {
        self.entry_overrides
            .insert(entry_uuid.to_string(), permission);
    }

    /// Remove per-entry permission override (revert to role default)
    pub fn clear_entry_permission(&mut self, entry_uuid: &str) {
        self.entry_overrides.remove(entry_uuid);
    }
}

/// Per-entry permission override
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryPermission {
    /// No access to this entry
    None,
    /// View only (override Editor → Viewer for this entry)
    ViewOnly,
    /// Full edit access (override Viewer → Editor for this entry)
    FullEdit,
}

impl EntryPermission {
    pub fn allows(&self, action: TeamAction) -> bool {
        match (self, action) {
            (EntryPermission::None, _) => false,
            (EntryPermission::ViewOnly, TeamAction::ViewEntry) => true,
            (EntryPermission::ViewOnly, TeamAction::CopyPassword) => true,
            (EntryPermission::ViewOnly, TeamAction::AddComment) => true,
            (EntryPermission::ViewOnly, _) => false,
            (EntryPermission::FullEdit, _) => true,
        }
    }
}

/// An encrypted comment on an entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryComment {
    pub id: String,
    pub entry_uuid: String,
    pub author_id: String,
    pub author_name: String,
    /// Encrypted comment text (ChaCha20-Poly1305 with team key)
    pub encrypted_text: Vec<u8>,
    /// Nonce for decryption
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

impl EntryComment {
    pub fn new(
        entry_uuid: &str,
        author_id: &str,
        author_name: &str,
        encrypted_text: Vec<u8>,
        nonce: Vec<u8>,
    ) -> Self {
        EntryComment {
            id: Uuid::new_v4().to_string(),
            entry_uuid: entry_uuid.to_string(),
            author_id: author_id.to_string(),
            author_name: author_name.to_string(),
            encrypted_text,
            nonce,
            created_at: Utc::now(),
            edited_at: None,
        }
    }
}

/// Team activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamActivity {
    pub id: String,
    pub member_id: String,
    pub member_name: String,
    pub action: String,
    pub entry_uuid: Option<String>,
    pub entry_title: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

/// Team vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamVault {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub members: Vec<TeamMember>,
    pub comments: Vec<EntryComment>,
    pub activity_log: Vec<TeamActivity>,
    pub created_at: DateTime<Utc>,
    pub real_time_sync: bool,
    /// Encrypted team key (one per member, encrypted with their public key)
    pub encrypted_keys: HashMap<String, Vec<u8>>,
}

impl TeamVault {
    /// Create a new team vault
    pub fn new(name: &str, owner_id: &str, owner_name: &str, owner_email: &str) -> Self {
        let owner = TeamMember {
            id: owner_id.to_string(),
            email: owner_email.to_string(),
            name: owner_name.to_string(),
            role: TeamRole::Admin,
            status: MemberStatus::Active,
            joined_at: Some(Utc::now()),
            invited_at: Utc::now(),
            invited_by: owner_id.to_string(),
            entry_overrides: HashMap::new(),
            public_key: vec![],
        };

        TeamVault {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            owner_id: owner_id.to_string(),
            members: vec![owner],
            comments: vec![],
            activity_log: vec![],
            created_at: Utc::now(),
            real_time_sync: true,
            encrypted_keys: HashMap::new(),
        }
    }

    /// Add a new member (invite)
    pub fn invite_member(
        &mut self,
        email: &str,
        name: &str,
        role: TeamRole,
        invited_by: &str,
        public_key: Vec<u8>,
    ) -> Result<&TeamMember> {
        // Check for duplicate email
        if self
            .members
            .iter()
            .any(|m| m.email == email && m.status != MemberStatus::Removed)
        {
            return Err(KeePassExError::Other(format!(
                "Member with email {} already exists",
                email
            )));
        }

        let member = TeamMember::new_invitation(email, name, role, invited_by, public_key);
        self.members.push(member);
        Ok(self.members.last().unwrap())
    }

    /// Get active members
    pub fn active_members(&self) -> Vec<&TeamMember> {
        self.members
            .iter()
            .filter(|m| m.status == MemberStatus::Active)
            .collect()
    }

    /// Get member by ID
    pub fn get_member(&self, member_id: &str) -> Option<&TeamMember> {
        self.members.iter().find(|m| m.id == member_id)
    }

    /// Get mutable member by ID
    pub fn get_member_mut(&mut self, member_id: &str) -> Option<&mut TeamMember> {
        self.members.iter_mut().find(|m| m.id == member_id)
    }

    /// Remove a member
    pub fn remove_member(&mut self, member_id: &str) -> Result<()> {
        // Check owner first (immutable borrow)
        let is_owner = self.owner_id == member_id;
        if is_owner {
            return Err(KeePassExError::Other("Cannot remove vault owner".into()));
        }

        let member = self
            .get_member_mut(member_id)
            .ok_or_else(|| KeePassExError::Other("Member not found".into()))?;

        member.status = MemberStatus::Removed;
        Ok(())
    }

    /// Change member role
    pub fn change_role(&mut self, member_id: &str, new_role: TeamRole) -> Result<()> {
        let owner_id = self.owner_id.clone();
        let member = self
            .get_member_mut(member_id)
            .ok_or_else(|| KeePassExError::Other("Member not found".into()))?;

        if member.id == owner_id && new_role != TeamRole::Admin {
            return Err(KeePassExError::Other("Cannot demote vault owner".into()));
        }

        member.role = new_role;
        Ok(())
    }

    /// Add a comment to an entry
    pub fn add_comment(&mut self, comment: EntryComment) {
        self.comments.push(comment);
    }

    /// Get comments for an entry
    pub fn get_comments(&self, entry_uuid: &str) -> Vec<&EntryComment> {
        self.comments
            .iter()
            .filter(|c| c.entry_uuid == entry_uuid)
            .collect()
    }

    /// Delete a comment
    pub fn delete_comment(&mut self, comment_id: &str, requester_id: &str) -> Result<()> {
        let comment = self
            .comments
            .iter()
            .find(|c| c.id == comment_id)
            .ok_or_else(|| KeePassExError::Other("Comment not found".into()))?;

        // Only author or admin can delete
        let is_admin = self
            .get_member(requester_id)
            .map(|m| m.role == TeamRole::Admin)
            .unwrap_or(false);

        if comment.author_id != requester_id && !is_admin {
            return Err(KeePassExError::Other(
                "Permission denied: cannot delete another member's comment".into(),
            ));
        }

        self.comments.retain(|c| c.id != comment_id);
        Ok(())
    }

    /// Log an activity
    pub fn log_activity(
        &mut self,
        member_id: &str,
        action: &str,
        entry_uuid: Option<&str>,
        entry_title: Option<&str>,
        details: Option<&str>,
    ) {
        let member_name = self
            .get_member(member_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        self.activity_log.push(TeamActivity {
            id: Uuid::new_v4().to_string(),
            member_id: member_id.to_string(),
            member_name,
            action: action.to_string(),
            entry_uuid: entry_uuid.map(|s| s.to_string()),
            entry_title: entry_title.map(|s| s.to_string()),
            timestamp: Utc::now(),
            details: details.map(|s| s.to_string()),
        });

        // Keep only last 1000 activities
        if self.activity_log.len() > 1000 {
            self.activity_log.drain(0..100);
        }
    }

    /// Get recent activity (last N entries)
    pub fn recent_activity(&self, limit: usize) -> Vec<&TeamActivity> {
        self.activity_log.iter().rev().take(limit).collect()
    }

    /// Check if a member can perform an action on an entry
    pub fn check_permission(
        &self,
        member_id: &str,
        action: TeamAction,
        entry_uuid: Option<&str>,
    ) -> bool {
        let member = match self.get_member(member_id) {
            Some(m) => m,
            None => return false,
        };

        if let Some(uuid) = entry_uuid {
            member.can_on_entry(action, uuid)
        } else {
            member.status == MemberStatus::Active && member.role.can(action)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vault() -> TeamVault {
        TeamVault::new("Test Vault", "owner-1", "Alice", "alice@example.com")
    }

    #[test]
    fn test_create_team_vault() {
        let vault = make_vault();
        assert_eq!(vault.members.len(), 1);
        assert_eq!(vault.members[0].role, TeamRole::Admin);
        assert_eq!(vault.members[0].status, MemberStatus::Active);
    }

    #[test]
    fn test_invite_member() {
        let mut vault = make_vault();
        vault
            .invite_member(
                "bob@example.com",
                "Bob",
                TeamRole::Editor,
                "owner-1",
                vec![],
            )
            .unwrap();
        assert_eq!(vault.members.len(), 2);
        assert_eq!(vault.members[1].status, MemberStatus::Invited);
    }

    #[test]
    fn test_duplicate_invite_fails() {
        let mut vault = make_vault();
        vault
            .invite_member(
                "bob@example.com",
                "Bob",
                TeamRole::Editor,
                "owner-1",
                vec![],
            )
            .unwrap();
        let result = vault.invite_member(
            "bob@example.com",
            "Bob2",
            TeamRole::Viewer,
            "owner-1",
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_role_permissions() {
        assert!(TeamRole::Admin.can(TeamAction::ManageMembers));
        assert!(TeamRole::Editor.can(TeamAction::EditEntry));
        assert!(!TeamRole::Editor.can(TeamAction::ManageMembers));
        assert!(TeamRole::Viewer.can(TeamAction::ViewEntry));
        assert!(!TeamRole::Viewer.can(TeamAction::EditEntry));
    }

    #[test]
    fn test_entry_permission_override() {
        let mut vault = make_vault();
        vault
            .invite_member(
                "bob@example.com",
                "Bob",
                TeamRole::Editor,
                "owner-1",
                vec![],
            )
            .unwrap();
        let bob_id = vault.members[1].id.clone();
        vault.members[1].accept();

        // Bob is Editor, but restrict to ViewOnly on "secret-entry"
        vault
            .get_member_mut(&bob_id)
            .unwrap()
            .set_entry_permission("secret-entry", EntryPermission::ViewOnly);

        assert!(!vault.check_permission(&bob_id, TeamAction::EditEntry, Some("secret-entry")));
        assert!(vault.check_permission(&bob_id, TeamAction::ViewEntry, Some("secret-entry")));
        // Other entries: still Editor
        assert!(vault.check_permission(&bob_id, TeamAction::EditEntry, Some("other-entry")));
    }

    #[test]
    fn test_remove_member() {
        let mut vault = make_vault();
        vault
            .invite_member(
                "bob@example.com",
                "Bob",
                TeamRole::Editor,
                "owner-1",
                vec![],
            )
            .unwrap();
        let bob_id = vault.members[1].id.clone();
        vault.remove_member(&bob_id).unwrap();
        assert_eq!(vault.members[1].status, MemberStatus::Removed);
    }

    #[test]
    fn test_cannot_remove_owner() {
        let mut vault = make_vault();
        let owner_id = vault.owner_id.clone();
        let result = vault.remove_member(&owner_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_delete_comment() {
        let mut vault = make_vault();
        let comment = EntryComment::new(
            "entry-1",
            "owner-1",
            "Alice",
            b"encrypted".to_vec(),
            b"nonce123".to_vec(),
        );
        let comment_id = comment.id.clone();
        vault.add_comment(comment);

        assert_eq!(vault.get_comments("entry-1").len(), 1);

        vault.delete_comment(&comment_id, "owner-1").unwrap();
        assert_eq!(vault.get_comments("entry-1").len(), 0);
    }

    #[test]
    fn test_delete_comment_permission_denied() {
        let mut vault = make_vault();
        vault
            .invite_member(
                "bob@example.com",
                "Bob",
                TeamRole::Editor,
                "owner-1",
                vec![],
            )
            .unwrap();
        vault.members[1].accept();
        let bob_id = vault.members[1].id.clone();

        // Alice adds a comment
        let comment = EntryComment::new(
            "entry-1",
            "owner-1",
            "Alice",
            b"encrypted".to_vec(),
            b"nonce123".to_vec(),
        );
        let comment_id = comment.id.clone();
        vault.add_comment(comment);

        // Bob tries to delete Alice's comment — should fail
        let result = vault.delete_comment(&comment_id, &bob_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_activity_log() {
        let mut vault = make_vault();
        vault.log_activity(
            "owner-1",
            "entry_viewed",
            Some("entry-1"),
            Some("GitHub"),
            None,
        );
        assert_eq!(vault.activity_log.len(), 1);
        assert_eq!(vault.recent_activity(10).len(), 1);
    }

    #[test]
    fn test_change_role() {
        let mut vault = make_vault();
        vault
            .invite_member(
                "bob@example.com",
                "Bob",
                TeamRole::Editor,
                "owner-1",
                vec![],
            )
            .unwrap();
        let bob_id = vault.members[1].id.clone();
        vault.change_role(&bob_id, TeamRole::Viewer).unwrap();
        assert_eq!(vault.members[1].role, TeamRole::Viewer);
    }

    #[test]
    fn test_active_members() {
        let mut vault = make_vault();
        vault
            .invite_member(
                "bob@example.com",
                "Bob",
                TeamRole::Editor,
                "owner-1",
                vec![],
            )
            .unwrap();
        // Bob is Invited, not Active
        assert_eq!(vault.active_members().len(), 1);
        vault.members[1].accept();
        assert_eq!(vault.active_members().len(), 2);
    }
}
