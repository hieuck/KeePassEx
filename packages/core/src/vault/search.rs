//! Full-text search across vault entries

use crate::types::{Entry, SearchQuery};
use crate::vault::Vault;

impl Vault {
    /// Search entries matching the query
    pub fn search(&self, query: &SearchQuery) -> Vec<&Entry> {
        if query.text.is_empty() {
            return self.all_entries().collect();
        }

        let needle = if query.case_sensitive {
            query.text.clone()
        } else {
            query.text.to_lowercase()
        };

        self.all_entries()
            .filter(|entry| {
                // Group filter
                if let Some(group_uuid) = query.group_uuid {
                    if query.recursive {
                        if !self.is_in_group_recursive(entry, &group_uuid) {
                            return false;
                        }
                    } else if entry.group_uuid != group_uuid {
                        return false;
                    }
                }

                // Exclude expired
                if query.exclude_expired && entry.is_expired {
                    return false;
                }

                self.entry_matches(entry, &needle, query)
            })
            .collect()
    }

    fn entry_matches(&self, entry: &Entry, needle: &str, query: &SearchQuery) -> bool {
        let matches_field = |value: &str| -> bool {
            let haystack = if query.case_sensitive {
                value.to_string()
            } else {
                value.to_lowercase()
            };

            if query.regex {
                // Simple contains for now; production uses regex crate
                haystack.contains(needle)
            } else {
                haystack.contains(needle)
            }
        };

        if query.search_title && matches_field(entry.title.get()) {
            return true;
        }
        if query.search_username && matches_field(entry.username.get()) {
            return true;
        }
        if query.search_password && matches_field(entry.password.get()) {
            return true;
        }
        if query.search_url && matches_field(&entry.url) {
            return true;
        }
        if query.search_notes && matches_field(entry.notes.get()) {
            return true;
        }
        if query.search_tags {
            for tag in &entry.tags {
                if matches_field(tag) {
                    return true;
                }
            }
        }
        if query.search_custom_fields {
            for field in entry.custom_fields.values() {
                if matches_field(field.value.get()) {
                    return true;
                }
            }
        }

        false
    }

    fn is_in_group_recursive(&self, entry: &Entry, group_uuid: &uuid::Uuid) -> bool {
        if entry.group_uuid == *group_uuid {
            return true;
        }
        // Walk up the group tree
        let mut current = entry.group_uuid;
        loop {
            match self.get_group(&current).and_then(|g| g.parent_uuid) {
                Some(parent) => {
                    if parent == *group_uuid {
                        return true;
                    }
                    current = parent;
                }
                None => return false,
            }
        }
    }
}
