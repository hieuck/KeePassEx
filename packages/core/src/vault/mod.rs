//! Vault — the main in-memory representation of an open KDBX database

pub mod operations;
pub mod pqc_migration;
pub mod search;

use crate::error::{KeePassExError, Result};
use crate::types::*;
use std::collections::HashMap;
use uuid::Uuid;

/// An open, decrypted vault
pub struct Vault {
    pub meta: VaultMeta,
    pub root_group_uuid: Uuid,
    groups: HashMap<Uuid, Group>,
    entries: HashMap<Uuid, Entry>,
    attachments: HashMap<Uuid, Attachment>,
    custom_icons: HashMap<Uuid, Vec<u8>>,
    /// Audit log — tracks all security-relevant operations
    pub audit_log: crate::audit_log::AuditLog,
    /// Whether the vault has unsaved changes
    pub dirty: bool,
}

impl Vault {
    /// Create a new empty vault
    pub fn new(name: impl Into<String>) -> Self {
        let vault_name: String = name.into();
        let root = Group::new(vault_name.clone(), None);
        let root_uuid = root.uuid;

        let mut groups = HashMap::new();
        groups.insert(root_uuid, root);

        let mut meta = VaultMeta::default();
        meta.name = vault_name;

        Self {
            meta,
            root_group_uuid: root_uuid,
            groups,
            entries: HashMap::new(),
            attachments: HashMap::new(),
            custom_icons: HashMap::new(),
            audit_log: crate::audit_log::AuditLog::new(1000),
            dirty: false,
        }
    }

    // ─── Group operations ─────────────────────────────────────────────────────

    pub fn get_group(&self, uuid: &Uuid) -> Option<&Group> {
        self.groups.get(uuid)
    }

    pub fn get_group_mut(&mut self, uuid: &Uuid) -> Option<&mut Group> {
        self.groups.get_mut(uuid)
    }

    pub fn create_group(&mut self, name: impl Into<String>, parent_uuid: Uuid) -> Result<Uuid> {
        if !self.groups.contains_key(&parent_uuid) {
            return Err(KeePassExError::GroupNotFound {
                uuid: parent_uuid.to_string(),
            });
        }
        let group = Group::new(name, Some(parent_uuid));
        let uuid = group.uuid;
        self.groups.insert(uuid, group);
        self.dirty = true;
        Ok(uuid)
    }

    pub fn delete_group(&mut self, uuid: &Uuid, permanent: bool) -> Result<()> {
        if *uuid == self.root_group_uuid {
            return Err(KeePassExError::Other("Cannot delete root group".into()));
        }

        if permanent {
            // Recursively delete all children
            let children: Vec<Uuid> = self
                .groups
                .values()
                .filter(|g| g.parent_uuid == Some(*uuid))
                .map(|g| g.uuid)
                .collect();

            for child_uuid in children {
                self.delete_group(&child_uuid, true)?;
            }

            // Delete all entries in this group
            let entry_uuids: Vec<Uuid> = self
                .entries
                .values()
                .filter(|e| e.group_uuid == *uuid)
                .map(|e| e.uuid)
                .collect();

            for entry_uuid in entry_uuids {
                self.entries.remove(&entry_uuid);
            }

            self.groups.remove(uuid);
        } else {
            // Move to recycle bin
            self.move_group_to_recycle_bin(uuid)?;
        }

        self.dirty = true;
        Ok(())
    }

    pub fn move_group(&mut self, uuid: &Uuid, new_parent_uuid: Uuid) -> Result<()> {
        if !self.groups.contains_key(&new_parent_uuid) {
            return Err(KeePassExError::GroupNotFound {
                uuid: new_parent_uuid.to_string(),
            });
        }

        // Check for circular reference
        if self.is_ancestor(uuid, &new_parent_uuid) {
            return Err(KeePassExError::Other(
                "Cannot move group into its own descendant".into(),
            ));
        }

        if let Some(group) = self.groups.get_mut(uuid) {
            group.parent_uuid = Some(new_parent_uuid);
            self.dirty = true;
            Ok(())
        } else {
            Err(KeePassExError::GroupNotFound {
                uuid: uuid.to_string(),
            })
        }
    }

    /// Get all direct children of a group
    pub fn get_child_groups(&self, parent_uuid: &Uuid) -> Vec<&Group> {
        self.groups
            .values()
            .filter(|g| g.parent_uuid == Some(*parent_uuid))
            .collect()
    }

    /// Get all entries in a group (non-recursive)
    pub fn get_group_entries(&self, group_uuid: &Uuid) -> Vec<&Entry> {
        self.entries
            .values()
            .filter(|e| e.group_uuid == *group_uuid)
            .collect()
    }

    /// Get all entries in a group and its descendants
    pub fn get_group_entries_recursive(&self, group_uuid: &Uuid) -> Vec<&Entry> {
        let mut result = self.get_group_entries(group_uuid);
        for child in self.get_child_groups(group_uuid) {
            result.extend(self.get_group_entries_recursive(&child.uuid));
        }
        result
    }

    // ─── Entry operations ─────────────────────────────────────────────────────

    pub fn get_entry(&self, uuid: &Uuid) -> Option<&Entry> {
        self.entries.get(uuid)
    }

    pub fn get_entry_mut(&mut self, uuid: &Uuid) -> Option<&mut Entry> {
        self.entries.get_mut(uuid)
    }

    /// Get entry by string UUID (convenience for importers/commands)
    pub fn get_entry_by_str(&self, uuid_str: &str) -> Option<&Entry> {
        uuid::Uuid::parse_str(uuid_str)
            .ok()
            .and_then(|u| self.entries.get(&u))
    }

    pub fn create_entry(&mut self, group_uuid: Uuid) -> Result<Uuid> {
        if !self.groups.contains_key(&group_uuid) {
            return Err(KeePassExError::GroupNotFound {
                uuid: group_uuid.to_string(),
            });
        }
        let entry = Entry::new(group_uuid);
        let uuid = entry.uuid;
        self.entries.insert(uuid, entry);
        self.dirty = true;
        Ok(uuid)
    }

    pub fn update_entry(&mut self, entry: Entry) -> Result<()> {
        if !self.entries.contains_key(&entry.uuid) {
            return Err(KeePassExError::EntryNotFound {
                uuid: entry.uuid.to_string(),
            });
        }

        // Save history snapshot before update
        let old_entry = self.entries.get(&entry.uuid).unwrap().clone();
        let history_entry = HistoryEntry {
            modified_at: old_entry.modified_at,
            entry_snapshot: Box::new(old_entry),
        };

        let entry_uuid = entry.uuid;
        self.entries.insert(entry_uuid, entry);

        // Add to history and sync computed fields
        if let Some(e) = self.entries.get_mut(&entry_uuid) {
            // Sync is_expired from expiry date
            e.is_expired = e.check_expired();
            e.has_password = !e.password.get().is_empty();
            e.has_otp = e.otp.is_some();
            e.has_passkey = !e.passkeys.is_empty();
            e.has_ssh_key = e.ssh_key.is_some();
            e.has_attachments = !e.attachments.is_empty();

            e.history.push(history_entry);
            // Trim history
            let max = self.meta.max_history_items as usize;
            if e.history.len() > max {
                let drain_count = e.history.len() - max;
                e.history.drain(..drain_count);
            }
        }

        self.dirty = true;
        Ok(())
    }

    pub fn delete_entry(&mut self, uuid: &Uuid, permanent: bool) -> Result<()> {
        if permanent {
            self.entries
                .remove(uuid)
                .ok_or_else(|| KeePassExError::EntryNotFound {
                    uuid: uuid.to_string(),
                })?;
        } else {
            self.move_entry_to_recycle_bin(uuid)?;
        }
        self.dirty = true;
        Ok(())
    }

    pub fn move_entry(&mut self, uuid: &Uuid, new_group_uuid: Uuid) -> Result<()> {
        if !self.groups.contains_key(&new_group_uuid) {
            return Err(KeePassExError::GroupNotFound {
                uuid: new_group_uuid.to_string(),
            });
        }
        if let Some(entry) = self.entries.get_mut(uuid) {
            entry.group_uuid = new_group_uuid;
            self.dirty = true;
            Ok(())
        } else {
            Err(KeePassExError::EntryNotFound {
                uuid: uuid.to_string(),
            })
        }
    }

    pub fn duplicate_entry(&mut self, uuid: &Uuid) -> Result<Uuid> {
        let original = self
            .entries
            .get(uuid)
            .ok_or_else(|| KeePassExError::EntryNotFound {
                uuid: uuid.to_string(),
            })?
            .clone();

        let mut new_entry = original;
        new_entry.uuid = Uuid::new_v4();
        new_entry
            .title
            .set(format!("{} (copy)", new_entry.title.get()));
        new_entry.history.clear();

        let new_uuid = new_entry.uuid;
        self.entries.insert(new_uuid, new_entry);
        self.dirty = true;
        Ok(new_uuid)
    }

    // ─── All entries ──────────────────────────────────────────────────────────

    pub fn all_entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    pub fn all_groups(&self) -> impl Iterator<Item = &Group> {
        self.groups.values()
    }

    // ─── Recycle bin ─────────────────────────────────────────────────────────

    pub fn ensure_recycle_bin(&mut self) -> Uuid {
        if let Some(uuid) = self.meta.recycle_bin_uuid {
            if self.groups.contains_key(&uuid) {
                return uuid;
            }
        }

        let bin = Group::new("Recycle Bin", Some(self.root_group_uuid));
        let uuid = bin.uuid;
        self.groups.insert(uuid, bin);
        self.meta.recycle_bin_uuid = Some(uuid);
        uuid
    }

    fn move_entry_to_recycle_bin(&mut self, uuid: &Uuid) -> Result<()> {
        if !self.meta.recycle_bin_enabled {
            self.entries.remove(uuid);
            return Ok(());
        }
        let bin_uuid = self.ensure_recycle_bin();
        self.move_entry(uuid, bin_uuid)
    }

    fn move_group_to_recycle_bin(&mut self, uuid: &Uuid) -> Result<()> {
        if !self.meta.recycle_bin_enabled {
            return self.delete_group(uuid, true);
        }
        let bin_uuid = self.ensure_recycle_bin();
        self.move_group(uuid, bin_uuid)
    }

    pub fn empty_recycle_bin(&mut self) -> Result<usize> {
        let bin_uuid = match self.meta.recycle_bin_uuid {
            Some(u) => u,
            None => return Ok(0),
        };

        let entry_uuids: Vec<Uuid> = self
            .entries
            .values()
            .filter(|e| e.group_uuid == bin_uuid)
            .map(|e| e.uuid)
            .collect();

        let count = entry_uuids.len();
        for uuid in entry_uuids {
            self.entries.remove(&uuid);
        }

        // Delete sub-groups permanently
        let child_uuids: Vec<Uuid> = self
            .groups
            .values()
            .filter(|g| g.parent_uuid == Some(bin_uuid))
            .map(|g| g.uuid)
            .collect();

        for uuid in child_uuids {
            self.delete_group(&uuid, true)?;
        }

        self.dirty = true;
        Ok(count)
    }

    // ─── Helpers ──────────────────────────────────────────────────────────────

    fn is_ancestor(&self, ancestor: &Uuid, descendant: &Uuid) -> bool {
        let mut current = *descendant;
        loop {
            if current == *ancestor {
                return true;
            }
            match self.groups.get(&current).and_then(|g| g.parent_uuid) {
                Some(parent) => current = parent,
                None => return false,
            }
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Insert a pre-built group (used by importers)
    pub fn insert_group(&mut self, group: crate::types::Group) {
        self.groups.insert(group.uuid, group);
        self.dirty = true;
    }

    /// Insert a pre-built entry (used by importers)
    pub fn insert_entry(&mut self, entry: crate::types::Entry) {
        self.entries.insert(entry.uuid, entry);
        self.dirty = true;
    }
}
