//! Entry Field References — {REF:F@I:uuid} syntax
//!
//! Allows one entry to reference fields of another entry.
//! Compatible with KeePass/KeePassXC reference syntax.
//!
//! Syntax: {REF:<field>@<search_by>:<value>}
//!
//! Field codes:
//!   T = Title
//!   U = Username
//!   P = Password
//!   A = URL
//!   N = Notes
//!   I = UUID
//!   O = Custom field (use {REF:O:FieldName@I:uuid})
//!
//! Search-by codes:
//!   T = Title
//!   U = Username
//!   P = Password
//!   A = URL
//!   N = Notes
//!   I = UUID (most common — direct reference by UUID)

use crate::error::{KeePassExError, Result};
use crate::types::Entry;
use std::collections::HashMap;
use uuid::Uuid;

/// Maximum recursion depth to prevent circular references
const MAX_DEPTH: usize = 10;

/// A parsed field reference
#[derive(Debug, Clone, PartialEq)]
pub struct FieldRef {
    /// The field to retrieve (T/U/P/A/N/I/O)
    pub field: FieldCode,
    /// How to search for the target entry
    pub search_by: FieldCode,
    /// The search value (UUID string, title, etc.)
    pub value: String,
    /// For custom fields (O), the field name
    pub custom_field_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldCode {
    Title,
    Username,
    Password,
    Url,
    Notes,
    Uuid,
    Custom,
}

impl FieldCode {
    fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'T' => Some(Self::Title),
            'U' => Some(Self::Username),
            'P' => Some(Self::Password),
            'A' => Some(Self::Url),
            'N' => Some(Self::Notes),
            'I' => Some(Self::Uuid),
            'O' => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Parse a single `{REF:...}` token from a string.
/// Returns `None` if the string is not a valid reference.
pub fn parse_ref(token: &str) -> Option<FieldRef> {
    // Strip outer braces: {REF:P@I:550e8400-...}
    let inner = token
        .strip_prefix("{REF:")
        .and_then(|s| s.strip_suffix('}'))?;

    // Split on '@': "P@I:uuid" → ["P", "I:uuid"]
    let at_pos = inner.find('@')?;
    let field_str = &inner[..at_pos];
    let rest = &inner[at_pos + 1..];

    // Split rest on ':': "I:uuid" → ["I", "uuid"]
    let colon_pos = rest.find(':')?;
    let search_by_str = &rest[..colon_pos];
    let value = &rest[colon_pos + 1..];

    // Parse field code — may be "O:FieldName" for custom fields
    let (field, custom_field_name) = if field_str.starts_with("O:") {
        (FieldCode::Custom, Some(field_str[2..].to_string()))
    } else {
        let fc = FieldCode::from_char(field_str.chars().next()?)?;
        (fc, None)
    };

    let search_by = FieldCode::from_char(search_by_str.chars().next()?)?;

    Some(FieldRef {
        field,
        search_by,
        value: value.to_string(),
        custom_field_name,
    })
}

/// Resolve all `{REF:...}` placeholders in a string value.
///
/// `entries` is a flat map of UUID → Entry for the entire vault.
/// Returns the resolved string, or the original if no references found.
pub fn resolve_references(
    value: &str,
    entries: &HashMap<Uuid, &Entry>,
    depth: usize,
) -> Result<String> {
    if depth > MAX_DEPTH {
        return Err(KeePassExError::Other(
            "Circular field reference detected".into(),
        ));
    }

    if !value.contains("{REF:") {
        return Ok(value.to_string());
    }

    let mut result = String::with_capacity(value.len());
    let mut remaining = value;

    while let Some(start) = remaining.find("{REF:") {
        // Append everything before the reference
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        // Find the closing brace
        if let Some(end) = remaining.find('}') {
            let token = &remaining[..=end];
            remaining = &remaining[end + 1..];

            if let Some(field_ref) = parse_ref(token) {
                match resolve_single_ref(&field_ref, entries, depth) {
                    Ok(resolved) => result.push_str(&resolved),
                    Err(_) => {
                        // Leave unresolved reference as-is (KeePass behavior)
                        result.push_str(token);
                    }
                }
            } else {
                // Not a valid reference — keep as-is
                result.push_str(token);
            }
        } else {
            // No closing brace — append rest and stop
            result.push_str(remaining);
            remaining = "";
        }
    }

    result.push_str(remaining);
    Ok(result)
}

/// Resolve a single parsed field reference against the entry map.
fn resolve_single_ref(
    field_ref: &FieldRef,
    entries: &HashMap<Uuid, &Entry>,
    depth: usize,
) -> Result<String> {
    // Find the target entry
    let target = find_entry(field_ref, entries)?;

    // Extract the requested field value
    let raw_value = extract_field(field_ref, target)?;

    // Recursively resolve any nested references in the extracted value
    resolve_references(&raw_value, entries, depth + 1)
}

/// Find an entry matching the search criteria in the field reference.
fn find_entry<'a>(
    field_ref: &FieldRef,
    entries: &'a HashMap<Uuid, &'a Entry>,
) -> Result<&'a Entry> {
    let search_value = &field_ref.value;

    for entry in entries.values() {
        let matches = match &field_ref.search_by {
            FieldCode::Uuid => {
                // Normalize UUID format for comparison
                let normalized = search_value.replace('-', "").to_lowercase();
                let entry_uuid = entry.uuid.to_string().replace('-', "").to_lowercase();
                entry_uuid == normalized
                    || entry.uuid.to_string().to_lowercase() == search_value.to_lowercase()
            }
            FieldCode::Title => entry.title.get().eq_ignore_ascii_case(search_value),
            FieldCode::Username => entry.username.get().eq_ignore_ascii_case(search_value),
            FieldCode::Password => entry.password.get() == search_value,
            FieldCode::Url => entry.url.eq_ignore_ascii_case(search_value),
            FieldCode::Notes => entry.notes.get().eq_ignore_ascii_case(search_value),
            FieldCode::Custom => {
                // Search by custom field value — not standard but useful
                entry
                    .custom_fields
                    .values()
                    .any(|f| f.value.get().eq_ignore_ascii_case(search_value))
            }
        };

        if matches {
            return Ok(entry);
        }
    }

    Err(KeePassExError::EntryNotFound {
        uuid: format!("ref:{}", search_value),
    })
}

/// Extract the requested field value from an entry.
fn extract_field(field_ref: &FieldRef, entry: &Entry) -> Result<String> {
    match &field_ref.field {
        FieldCode::Title => Ok(entry.title.get().to_string()),
        FieldCode::Username => Ok(entry.username.get().to_string()),
        FieldCode::Password => Ok(entry.password.get().to_string()),
        FieldCode::Url => Ok(entry.url.clone()),
        FieldCode::Notes => Ok(entry.notes.get().to_string()),
        FieldCode::Uuid => Ok(entry.uuid.to_string()),
        FieldCode::Custom => {
            let field_name = field_ref
                .custom_field_name
                .as_deref()
                .ok_or_else(|| KeePassExError::Other("Custom field name missing".into()))?;

            entry
                .custom_fields
                .get(field_name)
                .map(|f| f.value.get().to_string())
                .ok_or_else(|| {
                    KeePassExError::Other(format!("Custom field '{}' not found", field_name))
                })
        }
    }
}

/// Resolve all field references in an entry's display fields.
/// Returns a new entry with references resolved (does NOT modify the stored entry).
pub fn resolve_entry_references<'a>(
    entry: &Entry,
    all_entries: &HashMap<Uuid, &'a Entry>,
) -> ResolvedEntry {
    let resolve = |s: &str| resolve_references(s, all_entries, 0).unwrap_or_else(|_| s.to_string());

    ResolvedEntry {
        uuid: entry.uuid,
        title: resolve(entry.title.get()),
        username: resolve(entry.username.get()),
        password: resolve(entry.password.get()),
        url: resolve(&entry.url),
        notes: resolve(entry.notes.get()),
    }
}

/// An entry with all field references resolved to their actual values.
/// Used for display and auto-type — never stored back to the vault.
#[derive(Debug, Clone)]
pub struct ResolvedEntry {
    pub uuid: Uuid,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

/// Check if a string contains any field references.
pub fn has_references(value: &str) -> bool {
    value.contains("{REF:")
}

/// Build a reference token for a given field and target entry UUID.
/// Convenience for the UI when inserting references.
pub fn build_ref(field: char, target_uuid: &Uuid) -> String {
    format!("{{REF:{}@I:{}}}", field.to_ascii_uppercase(), target_uuid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Entry, ProtectedString};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_entry(uuid: Uuid, title: &str, username: &str, password: &str, url: &str) -> Entry {
        let mut e = Entry::new(Uuid::new_v4());
        e.uuid = uuid;
        e.title = ProtectedString::new(title);
        e.username = ProtectedString::new(username);
        e.password = ProtectedString::new(password);
        e.url = url.to_string();
        e
    }

    #[test]
    fn test_parse_ref_by_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let token = format!("{{REF:P@I:{}}}", uuid);
        let parsed = parse_ref(&token).unwrap();
        assert_eq!(parsed.field, FieldCode::Password);
        assert_eq!(parsed.search_by, FieldCode::Uuid);
        assert_eq!(parsed.value, uuid);
    }

    #[test]
    fn test_parse_ref_by_title() {
        let token = "{REF:U@T:MyService}";
        let parsed = parse_ref(token).unwrap();
        assert_eq!(parsed.field, FieldCode::Username);
        assert_eq!(parsed.search_by, FieldCode::Title);
        assert_eq!(parsed.value, "MyService");
    }

    #[test]
    fn test_resolve_password_ref() {
        let target_uuid = Uuid::new_v4();
        let target = make_entry(
            target_uuid,
            "Service A",
            "alice",
            "s3cr3t!",
            "https://a.com",
        );

        let mut entries: HashMap<Uuid, &Entry> = HashMap::new();
        entries.insert(target_uuid, &target);

        let ref_str = format!("{{REF:P@I:{}}}", target_uuid);
        let resolved = resolve_references(&ref_str, &entries, 0).unwrap();
        assert_eq!(resolved, "s3cr3t!");
    }

    #[test]
    fn test_resolve_mixed_string() {
        let target_uuid = Uuid::new_v4();
        let target = make_entry(
            target_uuid,
            "Service A",
            "alice",
            "s3cr3t!",
            "https://a.com",
        );

        let mut entries: HashMap<Uuid, &Entry> = HashMap::new();
        entries.insert(target_uuid, &target);

        let value = format!("user={{REF:U@I:{}}}", target_uuid);
        let resolved = resolve_references(&value, &entries, 0).unwrap();
        assert_eq!(resolved, "user=alice");
    }

    #[test]
    fn test_no_references() {
        let entries: HashMap<Uuid, &Entry> = HashMap::new();
        let value = "plain string without refs";
        let resolved = resolve_references(value, &entries, 0).unwrap();
        assert_eq!(resolved, value);
    }

    #[test]
    fn test_build_ref() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let token = build_ref('P', &uuid);
        assert_eq!(token, "{REF:P@I:550e8400-e29b-41d4-a716-446655440000}");
    }

    #[test]
    fn test_circular_ref_protection() {
        // A references B, B references A — should hit depth limit
        let uuid_a = Uuid::new_v4();
        let uuid_b = Uuid::new_v4();

        let ref_to_b = format!("{{REF:P@I:{}}}", uuid_b);
        let ref_to_a = format!("{{REF:P@I:{}}}", uuid_a);

        let mut entry_a = make_entry(uuid_a, "A", "userA", &ref_to_b, "");
        let mut entry_b = make_entry(uuid_b, "B", "userB", &ref_to_a, "");

        // Manually set password to the reference string
        entry_a.password = crate::types::ProtectedString::new(&ref_to_b);
        entry_b.password = crate::types::ProtectedString::new(&ref_to_a);

        let mut entries: HashMap<Uuid, &Entry> = HashMap::new();
        entries.insert(uuid_a, &entry_a);
        entries.insert(uuid_b, &entry_b);

        let result = resolve_references(&ref_to_a, &entries, 0);
        assert!(result.is_err());
    }
}
