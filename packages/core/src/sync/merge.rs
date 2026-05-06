//! CRDT-inspired vault merge for conflict resolution

use crate::error::Result;
use crate::types::{Entry, Group};
use crate::vault::Vault;
use std::collections::HashMap;
use uuid::Uuid;

/// Merge two vault versions (local and remote) into a single vault
/// Strategy: last-write-wins per entry, based on modified_at timestamp
pub fn merge_vaults(local: &Vault, remote: &Vault) -> Result<Vault> {
    let mut merged = Vault::new(&local.meta.name);

    // Merge metadata — use most recently modified
    if remote.meta.modified_at > local.meta.modified_at {
        merged.meta = remote.meta.clone();
    } else {
        merged.meta = local.meta.clone();
    }

    // Collect all entry UUIDs from both vaults
    let mut all_entry_uuids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    for entry in local.all_entries() {
        all_entry_uuids.insert(entry.uuid);
    }
    for entry in remote.all_entries() {
        all_entry_uuids.insert(entry.uuid);
    }

    // For each entry, pick the most recently modified version
    for uuid in &all_entry_uuids {
        let local_entry = local.get_entry(uuid);
        let remote_entry = remote.get_entry(uuid);

        let winner = match (local_entry, remote_entry) {
            (Some(l), Some(r)) => {
                if r.modified_at > l.modified_at {
                    r
                } else {
                    l
                }
            }
            (Some(l), None) => l,
            (None, Some(r)) => r,
            (None, None) => continue,
        };

        // Ensure the group exists in merged vault
        ensure_group_path(&mut merged, local, remote, &winner.group_uuid)?;

        // Add entry to merged vault — use root if group not found
        let target_group = if merged.get_group(&winner.group_uuid).is_some() {
            winner.group_uuid
        } else {
            merged.root_group_uuid
        };

        let new_uuid = merged.create_entry(target_group)?;
        if let Some(entry) = merged.get_entry_mut(&new_uuid) {
            *entry = winner.clone();
            entry.uuid = new_uuid;
            entry.group_uuid = target_group;
        }
    }

    Ok(merged)
}

/// Ensure a group (and its ancestors) exists in the merged vault
fn ensure_group_path(
    merged: &mut Vault,
    local: &Vault,
    remote: &Vault,
    group_uuid: &Uuid,
) -> Result<()> {
    if merged.get_group(group_uuid).is_some() {
        return Ok(());
    }

    // Find group in local or remote
    let group = local
        .get_group(group_uuid)
        .or_else(|| remote.get_group(group_uuid));

    if let Some(group) = group {
        // Ensure parent exists first
        if let Some(parent_uuid) = group.parent_uuid {
            ensure_group_path(merged, local, remote, &parent_uuid)?;
        }

        let parent = group.parent_uuid.unwrap_or(merged.root_group_uuid);
        merged.create_group(&group.name, parent)?;
    }

    Ok(())
}

/// Compute a diff between two vault versions
pub struct VaultDiff {
    pub added_entries: Vec<Uuid>,
    pub modified_entries: Vec<Uuid>,
    pub deleted_entries: Vec<Uuid>,
    pub added_groups: Vec<Uuid>,
    pub modified_groups: Vec<Uuid>,
    pub deleted_groups: Vec<Uuid>,
}

pub fn diff_vaults(base: &Vault, current: &Vault) -> VaultDiff {
    let mut added_entries = Vec::new();
    let mut modified_entries = Vec::new();
    let mut deleted_entries = Vec::new();

    // Find added and modified entries
    for entry in current.all_entries() {
        match base.get_entry(&entry.uuid) {
            None => added_entries.push(entry.uuid),
            Some(base_entry) => {
                if entry.modified_at != base_entry.modified_at {
                    modified_entries.push(entry.uuid);
                }
            }
        }
    }

    // Find deleted entries
    for entry in base.all_entries() {
        if current.get_entry(&entry.uuid).is_none() {
            deleted_entries.push(entry.uuid);
        }
    }

    let mut added_groups = Vec::new();
    let mut modified_groups = Vec::new();
    let mut deleted_groups = Vec::new();

    for group in current.all_groups() {
        match base.get_group(&group.uuid) {
            None => added_groups.push(group.uuid),
            Some(base_group) => {
                if group.modified_at != base_group.modified_at {
                    modified_groups.push(group.uuid);
                }
            }
        }
    }

    for group in base.all_groups() {
        if current.get_group(&group.uuid).is_none() {
            deleted_groups.push(group.uuid);
        }
    }

    VaultDiff {
        added_entries,
        modified_entries,
        deleted_entries,
        added_groups,
        modified_groups,
        deleted_groups,
    }
}
