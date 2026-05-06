//! Vault Comparison — diff two KDBX vaults and merge changes

use crate::error::Result;
use crate::types::Entry;
use crate::vault::Vault;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDiff {
    pub only_in_first: Vec<EntryDiffRef>,
    pub only_in_second: Vec<EntryDiffRef>,
    pub modified: Vec<EntryModification>,
    pub identical_count: usize,
    pub first_total: usize,
    pub second_total: usize,
}

impl VaultDiff {
    pub fn is_identical(&self) -> bool {
        self.only_in_first.is_empty() && self.only_in_second.is_empty() && self.modified.is_empty()
    }
    pub fn diff_count(&self) -> usize {
        self.only_in_first.len() + self.only_in_second.len() + self.modified.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryDiffRef {
    pub uuid: String,
    pub title: String,
    pub username: String,
    pub url: String,
    pub modified_at: String,
    pub group_name: Option<String>,
}

impl From<&Entry> for EntryDiffRef {
    fn from(e: &Entry) -> Self {
        Self {
            uuid: e.uuid.to_string(),
            title: e.title.get().to_string(),
            username: e.username.get().to_string(),
            url: e.url.clone(),
            modified_at: e.modified_at.to_rfc3339(),
            group_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryModification {
    pub uuid: String,
    pub title: String,
    pub newer_in: WhichVault,
    pub first_modified: String,
    pub second_modified: String,
    pub changed_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhichVault {
    First,
    Second,
    Same,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    KeepFirst,
    KeepSecond,
    KeepNewer,
    KeepBoth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub entries_added: usize,
    pub entries_updated: usize,
    pub entries_kept: usize,
    pub conflicts_resolved: usize,
}

pub fn compare_vaults(first: &Vault, second: &Vault) -> VaultDiff {
    let first_map: std::collections::HashMap<String, &Entry> = first
        .all_entries()
        .map(|e| (e.uuid.to_string(), e))
        .collect();
    let second_map: std::collections::HashMap<String, &Entry> = second
        .all_entries()
        .map(|e| (e.uuid.to_string(), e))
        .collect();

    let mut only_in_first = Vec::new();
    let mut only_in_second = Vec::new();
    let mut modified = Vec::new();
    let mut identical_count = 0;

    for (uuid, entry) in &first_map {
        if let Some(second_entry) = second_map.get(uuid) {
            let changed = diff_fields(entry, second_entry);
            if changed.is_empty() {
                identical_count += 1;
            } else {
                let first_ts = entry.modified_at.to_rfc3339();
                let second_ts = second_entry.modified_at.to_rfc3339();
                let newer_in = if first_ts > second_ts {
                    WhichVault::First
                } else if second_ts > first_ts {
                    WhichVault::Second
                } else {
                    WhichVault::Same
                };
                modified.push(EntryModification {
                    uuid: uuid.clone(),
                    title: entry.title.get().to_string(),
                    newer_in,
                    first_modified: first_ts,
                    second_modified: second_ts,
                    changed_fields: changed,
                });
            }
        } else {
            only_in_first.push(EntryDiffRef::from(*entry));
        }
    }

    for (uuid, entry) in &second_map {
        if !first_map.contains_key(uuid) {
            only_in_second.push(EntryDiffRef::from(*entry));
        }
    }

    VaultDiff {
        only_in_first,
        only_in_second,
        modified,
        identical_count,
        first_total: first_map.len(),
        second_total: second_map.len(),
    }
}

fn diff_fields(a: &Entry, b: &Entry) -> Vec<String> {
    let mut changed = Vec::new();
    if a.title != b.title {
        changed.push("title".into());
    }
    if a.username != b.username {
        changed.push("username".into());
    }
    if a.url != b.url {
        changed.push("url".into());
    }
    if a.notes != b.notes {
        changed.push("notes".into());
    }
    if a.expiry != b.expiry {
        changed.push("expiry".into());
    }
    if a.tags != b.tags {
        changed.push("tags".into());
    }

    let a_cf: std::collections::HashMap<&str, &str> = a
        .custom_fields
        .values()
        .filter(|f| !f.protected)
        .map(|f| (f.key.as_str(), f.value.get()))
        .collect();
    let b_cf: std::collections::HashMap<&str, &str> = b
        .custom_fields
        .values()
        .filter(|f| !f.protected)
        .map(|f| (f.key.as_str(), f.value.get()))
        .collect();
    if a_cf != b_cf {
        changed.push("customFields".into());
    }

    if a.has_password != b.has_password {
        changed.push("password".into());
    }
    changed
}

pub fn merge_vaults(
    target: &mut Vault,
    source: &Vault,
    strategy: MergeStrategy,
) -> Result<MergeResult> {
    let diff = compare_vaults(target, source);
    let mut result = MergeResult {
        entries_added: 0,
        entries_updated: 0,
        entries_kept: 0,
        conflicts_resolved: 0,
    };

    for entry_ref in &diff.only_in_second {
        if let Some(entry) = source.get_entry_by_str(&entry_ref.uuid) {
            // Use root group if source group doesn't exist in target
            let group_uuid = if target.get_group(&entry.group_uuid).is_some() {
                entry.group_uuid
            } else {
                target.root_group_uuid
            };
            let new_uuid = target.create_entry(group_uuid)?;
            if let Some(e) = target.get_entry_mut(&new_uuid) {
                e.title = entry.title.clone();
                e.username = entry.username.clone();
                e.password = entry.password.clone();
                e.url = entry.url.clone();
                e.notes = entry.notes.clone();
            }
            result.entries_added += 1;
        }
    }

    for modification in &diff.modified {
        let should_update = match &strategy {
            MergeStrategy::KeepFirst => false,
            MergeStrategy::KeepSecond => true,
            MergeStrategy::KeepNewer => modification.newer_in == WhichVault::Second,
            MergeStrategy::KeepBoth => {
                if let Some(entry) = source.get_entry_by_str(&modification.uuid) {
                    let group_uuid = entry.group_uuid;
                    let new_uuid = target.create_entry(group_uuid)?;
                    if let Some(e) = target.get_entry_mut(&new_uuid) {
                        e.title = entry.title.clone();
                        e.title.set(format!("{} (merged)", e.title.get()));
                        e.username = entry.username.clone();
                        e.password = entry.password.clone();
                        e.url = entry.url.clone();
                    }
                }
                result.conflicts_resolved += 1;
                false
            }
        };

        if should_update {
            if let Some(source_entry) = source.get_entry_by_str(&modification.uuid) {
                target.update_entry(source_entry.clone())?;
                result.entries_updated += 1;
            }
            result.conflicts_resolved += 1;
        } else if strategy != MergeStrategy::KeepBoth {
            result.entries_kept += 1;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_empty_vaults() {
        let v1 = Vault::new("V1");
        let v2 = Vault::new("V2");
        let diff = compare_vaults(&v1, &v2);
        assert!(diff.is_identical());
        assert_eq!(diff.diff_count(), 0);
    }

    #[test]
    fn test_merge_result_default() {
        let r = MergeResult {
            entries_added: 0,
            entries_updated: 0,
            entries_kept: 0,
            conflicts_resolved: 0,
        };
        assert_eq!(r.entries_added, 0);
    }

    #[test]
    fn test_which_vault_enum() {
        assert_ne!(WhichVault::First, WhichVault::Second);
        assert_eq!(WhichVault::Same, WhichVault::Same);
    }

    #[test]
    fn test_vault_diff_is_identical() {
        let diff = VaultDiff {
            only_in_first: vec![],
            only_in_second: vec![],
            modified: vec![],
            identical_count: 5,
            first_total: 5,
            second_total: 5,
        };
        assert!(diff.is_identical());
        assert_eq!(diff.diff_count(), 0);
    }

    #[test]
    fn test_vault_diff_not_identical() {
        let diff = VaultDiff {
            only_in_first: vec![EntryDiffRef {
                uuid: "u1".into(),
                title: "T".into(),
                username: "u".into(),
                url: "".into(),
                modified_at: "".into(),
                group_name: None,
            }],
            only_in_second: vec![],
            modified: vec![],
            identical_count: 0,
            first_total: 1,
            second_total: 0,
        };
        assert!(!diff.is_identical());
        assert_eq!(diff.diff_count(), 1);
    }
}
