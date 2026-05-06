//! Team vault tests — uses the public TeamVault, TeamRole, TeamAction APIs

use crate::team::{EntryComment, EntryPermission, MemberStatus, TeamAction, TeamRole, TeamVault};

fn make_vault() -> TeamVault {
    TeamVault::new("Test Vault", "owner-1", "Alice", "alice@example.com")
}

#[test]
fn test_create_team_vault() {
    let vault = make_vault();
    assert_eq!(vault.members.len(), 1);
    assert_eq!(vault.members[0].role, TeamRole::Admin);
    assert_eq!(vault.members[0].status, MemberStatus::Active);
    assert_eq!(vault.owner_id, "owner-1");
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
    assert_eq!(vault.members[1].role, TeamRole::Editor);
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
fn test_admin_role_permissions() {
    assert!(TeamRole::Admin.can(TeamAction::ManageMembers));
    assert!(TeamRole::Admin.can(TeamAction::EditEntry));
    assert!(TeamRole::Admin.can(TeamAction::DeleteEntry));
    assert!(TeamRole::Admin.can(TeamAction::ViewEntry));
}

#[test]
fn test_editor_role_permissions() {
    assert!(TeamRole::Editor.can(TeamAction::EditEntry));
    assert!(TeamRole::Editor.can(TeamAction::CreateEntry));
    assert!(TeamRole::Editor.can(TeamAction::DeleteEntry));
    assert!(TeamRole::Editor.can(TeamAction::ViewEntry));
    assert!(!TeamRole::Editor.can(TeamAction::ManageMembers));
    assert!(!TeamRole::Editor.can(TeamAction::ManagePermissions));
}

#[test]
fn test_viewer_role_permissions() {
    assert!(TeamRole::Viewer.can(TeamAction::ViewEntry));
    assert!(TeamRole::Viewer.can(TeamAction::CopyPassword));
    assert!(!TeamRole::Viewer.can(TeamAction::EditEntry));
    assert!(!TeamRole::Viewer.can(TeamAction::DeleteEntry));
    assert!(!TeamRole::Viewer.can(TeamAction::ManageMembers));
}

#[test]
fn test_entry_permission_override_restricts_editor() {
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

    // Restrict Bob to ViewOnly on "secret-entry"
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
fn test_entry_permission_override_grants_viewer_edit() {
    let mut vault = make_vault();
    vault
        .invite_member(
            "carol@example.com",
            "Carol",
            TeamRole::Viewer,
            "owner-1",
            vec![],
        )
        .unwrap();
    let carol_id = vault.members[1].id.clone();
    vault.members[1].accept();

    // Grant Carol FullEdit on "shared-entry"
    vault
        .get_member_mut(&carol_id)
        .unwrap()
        .set_entry_permission("shared-entry", EntryPermission::FullEdit);

    assert!(vault.check_permission(&carol_id, TeamAction::EditEntry, Some("shared-entry")));
    // Other entries: still Viewer
    assert!(!vault.check_permission(&carol_id, TeamAction::EditEntry, Some("other-entry")));
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
fn test_active_members_count() {
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

#[test]
fn test_add_and_delete_comment() {
    let mut vault = make_vault();
    let comment = EntryComment::new(
        "entry-1",
        "owner-1",
        "Alice",
        b"encrypted_text".to_vec(),
        b"nonce12345678901".to_vec(),
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
        b"nonce".to_vec(),
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
    assert_eq!(vault.activity_log[0].action, "entry_viewed");
}

#[test]
fn test_inactive_member_cannot_act() {
    let mut vault = make_vault();
    vault
        .invite_member("bob@example.com", "Bob", TeamRole::Admin, "owner-1", vec![])
        .unwrap();
    let bob_id = vault.members[1].id.clone();
    // Bob is Invited (not Active) — should not be able to act
    assert!(!vault.check_permission(&bob_id, TeamAction::ViewEntry, None));
}
