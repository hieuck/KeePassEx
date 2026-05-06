//! KDBX XML payload parser and serializer
//!
//! Handles the XML inner payload of KDBX 4.x files.
//! Format: <KeePassFile><Meta>...</Meta><Root><Group>...</Group></Root></KeePassFile>

use crate::error::{KeePassExError, Result};
use crate::types::*;
use crate::vault::Vault;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

// ─── XML Serializer ───────────────────────────────────────────────────────────

pub struct XmlSerializer {
    inner_stream_key: Vec<u8>,
}

impl XmlSerializer {
    pub fn new(inner_stream_key: Vec<u8>) -> Self {
        Self { inner_stream_key }
    }

    pub fn serialize(&self, vault: &Vault) -> Result<Vec<u8>> {
        let mut xml = String::with_capacity(64 * 1024);

        xml.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
        xml.push('\n');
        xml.push_str("<KeePassFile>\n");

        // Meta section
        self.write_meta(&mut xml, vault);

        // Root group
        xml.push_str("\t<Root>\n");
        self.write_group(&mut xml, vault, &vault.root_group_uuid, 2)?;
        xml.push_str("\t</Root>\n");

        xml.push_str("</KeePassFile>\n");

        Ok(xml.into_bytes())
    }

    fn write_meta(&self, xml: &mut String, vault: &Vault) {
        xml.push_str("\t<Meta>\n");
        xml.push_str(&format!("\t\t<Generator>KeePassEx</Generator>\n"));
        xml.push_str(&format!(
            "\t\t<DatabaseName>{}</DatabaseName>\n",
            escape_xml(&vault.meta.name)
        ));
        xml.push_str(&format!(
            "\t\t<DatabaseDescription>{}</DatabaseDescription>\n",
            escape_xml(&vault.meta.description)
        ));
        xml.push_str(&format!(
            "\t\t<DefaultUserName>{}</DefaultUserName>\n",
            escape_xml(&vault.meta.default_username)
        ));
        xml.push_str(&format!(
            "\t\t<MaintenanceHistoryDays>{}</MaintenanceHistoryDays>\n",
            vault.meta.maintenance_history_days
        ));
        xml.push_str(&format!(
            "\t\t<RecycleBinEnabled>{}</RecycleBinEnabled>\n",
            vault.meta.recycle_bin_enabled
        ));
        if let Some(bin_uuid) = vault.meta.recycle_bin_uuid {
            xml.push_str(&format!(
                "\t\t<RecycleBinUUID>{}</RecycleBinUUID>\n",
                uuid_to_base64(&bin_uuid)
            ));
        }
        xml.push_str(&format!(
            "\t\t<HistoryMaxItems>{}</HistoryMaxItems>\n",
            vault.meta.history_max_items
        ));
        xml.push_str(&format!(
            "\t\t<HistoryMaxSize>{}</HistoryMaxSize>\n",
            vault.meta.history_max_size
        ));
        xml.push_str("\t</Meta>\n");
    }

    fn write_group(
        &self,
        xml: &mut String,
        vault: &Vault,
        group_uuid: &Uuid,
        depth: usize,
    ) -> Result<()> {
        let group = vault
            .get_group(group_uuid)
            .ok_or_else(|| KeePassExError::GroupNotFound {
                uuid: group_uuid.to_string(),
            })?;

        let indent = "\t".repeat(depth);
        xml.push_str(&format!("{}<Group>\n", indent));
        xml.push_str(&format!(
            "{}\t<UUID>{}</UUID>\n",
            indent,
            uuid_to_base64(group_uuid)
        ));
        xml.push_str(&format!(
            "{}\t<Name>{}</Name>\n",
            indent,
            escape_xml(&group.name)
        ));
        xml.push_str(&format!(
            "{}\t<Notes>{}</Notes>\n",
            indent,
            escape_xml(&group.notes)
        ));
        xml.push_str(&format!("{}\t<IconID>{}</IconID>\n", indent, group.icon_id));
        xml.push_str(&format!(
            "{}\t<IsExpanded>{}</IsExpanded>\n",
            indent, group.is_expanded
        ));

        // Times
        xml.push_str(&format!("{}\t<Times>\n", indent));
        xml.push_str(&format!(
            "{}\t\t<CreationTime>{}</CreationTime>\n",
            indent,
            format_time(&group.created_at)
        ));
        xml.push_str(&format!(
            "{}\t\t<LastModificationTime>{}</LastModificationTime>\n",
            indent,
            format_time(&group.modified_at)
        ));
        xml.push_str(&format!(
            "{}\t\t<LastAccessTime>{}</LastAccessTime>\n",
            indent,
            format_time(&group.accessed_at)
        ));
        xml.push_str(&format!("{}\t</Times>\n", indent));

        // Entries in this group
        for entry in vault.get_group_entries(group_uuid) {
            self.write_entry(xml, entry, depth + 1);
        }

        // Child groups
        for child in vault.get_child_groups(group_uuid) {
            self.write_group(xml, vault, &child.uuid, depth + 1)?;
        }

        xml.push_str(&format!("{}</Group>\n", indent));
        Ok(())
    }

    fn write_entry(&self, xml: &mut String, entry: &Entry, depth: usize) {
        let indent = "\t".repeat(depth);
        xml.push_str(&format!("{}<Entry>\n", indent));
        xml.push_str(&format!(
            "{}\t<UUID>{}</UUID>\n",
            indent,
            uuid_to_base64(&entry.uuid)
        ));
        xml.push_str(&format!("{}\t<IconID>{}</IconID>\n", indent, entry.icon_id));

        // Standard string fields
        self.write_string(xml, &indent, "Title", entry.title.get(), false);
        self.write_string(xml, &indent, "UserName", entry.username.get(), false);
        self.write_string(xml, &indent, "Password", entry.password.get(), true);
        self.write_string(xml, &indent, "URL", &entry.url, false);
        self.write_string(xml, &indent, "Notes", entry.notes.get(), false);

        // Custom fields
        for (key, field) in &entry.custom_fields {
            self.write_string(xml, &indent, key, field.value.get(), field.value.protected);
        }

        // Tags
        if !entry.tags.is_empty() {
            xml.push_str(&format!(
                "{}\t<Tags>{}</Tags>\n",
                indent,
                escape_xml(&entry.tags.join(";"))
            ));
        }

        // Times
        xml.push_str(&format!("{}\t<Times>\n", indent));
        xml.push_str(&format!(
            "{}\t\t<CreationTime>{}</CreationTime>\n",
            indent,
            format_time(&entry.created_at)
        ));
        xml.push_str(&format!(
            "{}\t\t<LastModificationTime>{}</LastModificationTime>\n",
            indent,
            format_time(&entry.modified_at)
        ));
        xml.push_str(&format!(
            "{}\t\t<LastAccessTime>{}</LastAccessTime>\n",
            indent,
            format_time(&entry.accessed_at)
        ));
        if let Some(expiry) = entry.expiry {
            xml.push_str(&format!(
                "{}\t\t<ExpiryTime>{}</ExpiryTime>\n",
                indent,
                format_time(&expiry)
            ));
            xml.push_str(&format!("{}\t\t<Expires>True</Expires>\n", indent));
        } else {
            xml.push_str(&format!("{}\t\t<Expires>False</Expires>\n", indent));
        }
        xml.push_str(&format!("{}\t</Times>\n", indent));

        // Auto-type
        xml.push_str(&format!("{}\t<AutoType>\n", indent));
        xml.push_str(&format!(
            "{}\t\t<Enabled>{}</Enabled>\n",
            indent, entry.auto_type.enabled
        ));
        if let Some(ref seq) = entry.auto_type.default_sequence {
            xml.push_str(&format!(
                "{}\t\t<DefaultSequence>{}</DefaultSequence>\n",
                indent,
                escape_xml(seq)
            ));
        }
        xml.push_str(&format!("{}\t</AutoType>\n", indent));

        // History
        if !entry.history.is_empty() {
            xml.push_str(&format!("{}\t<History>\n", indent));
            for hist in &entry.history {
                self.write_entry(xml, &hist.entry_snapshot, depth + 2);
            }
            xml.push_str(&format!("{}\t</History>\n", indent));
        }

        xml.push_str(&format!("{}</Entry>\n", indent));
    }

    fn write_string(
        &self,
        xml: &mut String,
        indent: &str,
        key: &str,
        value: &str,
        protected: bool,
    ) {
        xml.push_str(&format!("{}\t<String>\n", indent));
        xml.push_str(&format!("{}\t\t<Key>{}</Key>\n", indent, escape_xml(key)));
        if protected {
            // In production: XOR with inner stream key
            // For now: write as-is with Protected attribute
            xml.push_str(&format!(
                "{}\t\t<Value Protected=\"True\">{}</Value>\n",
                indent,
                escape_xml(value)
            ));
        } else {
            xml.push_str(&format!(
                "{}\t\t<Value>{}</Value>\n",
                indent,
                escape_xml(value)
            ));
        }
        xml.push_str(&format!("{}\t</String>\n", indent));
    }
}

// ─── XML Parser ───────────────────────────────────────────────────────────────

pub struct XmlParser {
    inner_stream_key: Vec<u8>,
}

impl XmlParser {
    pub fn new(inner_stream_key: Vec<u8>) -> Self {
        Self { inner_stream_key }
    }

    pub fn parse(&self, xml: &[u8], _binaries: Vec<(String, Vec<u8>)>) -> Result<Vault> {
        let content = std::str::from_utf8(xml).map_err(|_| KeePassExError::CorruptedVault {
            reason: "Invalid UTF-8 in XML payload".into(),
        })?;

        // Parse using quick-xml
        let mut vault = Vault::new("Imported Vault");
        self.parse_xml(content, &mut vault)?;
        Ok(vault)
    }

    fn parse_xml(&self, xml: &str, vault: &mut Vault) -> Result<()> {
        // Extract database name from Meta
        if let Some(name) = extract_xml_text(xml, "DatabaseName") {
            vault.meta.name = unescape_xml(&name);
        }
        if let Some(desc) = extract_xml_text(xml, "DatabaseDescription") {
            vault.meta.description = unescape_xml(&desc);
        }

        // Find Root group
        if let Some(root_xml) = extract_xml_element(xml, "Root") {
            if let Some(group_xml) = extract_xml_element(&root_xml, "Group") {
                self.parse_group(&group_xml, vault, None)?;
            }
        }

        Ok(())
    }

    fn parse_group(&self, xml: &str, vault: &mut Vault, parent_uuid: Option<Uuid>) -> Result<Uuid> {
        let uuid = if let Some(uuid_b64) = extract_xml_text(xml, "UUID") {
            base64_to_uuid(&uuid_b64).unwrap_or_else(Uuid::new_v4)
        } else {
            Uuid::new_v4()
        };

        let name = extract_xml_text(xml, "Name")
            .map(|s| unescape_xml(&s))
            .unwrap_or_else(|| "Group".to_string());

        let notes = extract_xml_text(xml, "Notes")
            .map(|s| unescape_xml(&s))
            .unwrap_or_default();

        let icon_id = extract_xml_text(xml, "IconID")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(48);

        // Create or update group
        let group_uuid = if parent_uuid.is_none() {
            // This is the root group — update existing
            vault.meta.name = name.clone();
            vault.root_group_uuid
        } else {
            let parent = parent_uuid.unwrap_or(vault.root_group_uuid);
            vault
                .create_group(&name, parent)
                .unwrap_or(vault.root_group_uuid)
        };

        // Update group properties
        if let Some(group) = vault.get_group_mut(&group_uuid) {
            group.name = name;
            group.notes = notes;
            group.icon_id = icon_id;
        }

        // Parse entries in this group
        let entries_xml = extract_all_xml_elements(xml, "Entry");
        for entry_xml in entries_xml {
            self.parse_entry(&entry_xml, vault, group_uuid)?;
        }

        // Parse child groups
        let child_groups_xml = extract_all_xml_elements(xml, "Group");
        for child_xml in child_groups_xml {
            self.parse_group(&child_xml, vault, Some(group_uuid))?;
        }

        Ok(group_uuid)
    }

    fn parse_entry(&self, xml: &str, vault: &mut Vault, group_uuid: Uuid) -> Result<Uuid> {
        let entry_uuid = vault.create_entry(group_uuid)?;

        if let Some(entry) = vault.get_entry_mut(&entry_uuid) {
            // Parse string fields
            let strings = extract_all_xml_elements(xml, "String");
            for string_xml in strings {
                let key = extract_xml_text(&string_xml, "Key")
                    .map(|s| unescape_xml(&s))
                    .unwrap_or_default();
                let value = extract_xml_text(&string_xml, "Value")
                    .map(|s| unescape_xml(&s))
                    .unwrap_or_default();

                match key.as_str() {
                    "Title" => entry.title.set(&value),
                    "UserName" => entry.username.set(&value),
                    "Password" => entry.password.set(&value),
                    "URL" => entry.url = value,
                    "Notes" => entry.notes.set(&value),
                    _ => {
                        let protected = string_xml.contains("Protected=\"True\"");
                        let mut ps = ProtectedString::new(&value);
                        ps.protected = protected;
                        entry.custom_fields.insert(
                            key.clone(),
                            CustomField {
                                key,
                                value: ps,
                                protected,
                            },
                        );
                    }
                }
            }

            // Parse tags
            if let Some(tags_str) = extract_xml_text(xml, "Tags") {
                entry.tags = tags_str
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| unescape_xml(s))
                    .collect();
            }

            // Parse times
            if let Some(created) = extract_xml_text(xml, "CreationTime") {
                if let Ok(dt) = parse_kdbx_time(&created) {
                    entry.created_at = dt;
                }
            }
            if let Some(modified) = extract_xml_text(xml, "LastModificationTime") {
                if let Ok(dt) = parse_kdbx_time(&modified) {
                    entry.modified_at = dt;
                }
            }
            if let Some(expiry) = extract_xml_text(xml, "ExpiryTime") {
                let expires = extract_xml_text(xml, "Expires")
                    .map(|s| s.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                if expires {
                    if let Ok(dt) = parse_kdbx_time(&expiry) {
                        entry.expiry = Some(dt);
                    }
                }
            }

            // Parse icon
            if let Some(icon_str) = extract_xml_text(xml, "IconID") {
                if let Ok(icon_id) = icon_str.parse::<u32>() {
                    entry.icon_id = icon_id;
                }
            }
        }

        Ok(entry_uuid)
    }
}

// ─── XML Helpers ──────────────────────────────────────────────────────────────

/// Escape special XML characters
pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Unescape XML entities
pub fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Extract text content of a single XML element
fn extract_xml_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;

    Some(xml[start..end].to_string())
}

/// Extract the content of a single XML element (including nested tags)
fn extract_xml_element(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;

    Some(xml[start..end].to_string())
}

/// Extract all occurrences of an XML element
fn extract_all_xml_elements(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(start_pos) = xml[search_from..].find(&open) {
        let abs_start = search_from + start_pos + open.len();
        if let Some(end_pos) = xml[abs_start..].find(&close) {
            let abs_end = abs_start + end_pos;
            results.push(xml[abs_start..abs_end].to_string());
            search_from = abs_end + close.len();
        } else {
            break;
        }
    }

    results
}

/// Convert UUID to base64 (KDBX format)
pub fn uuid_to_base64(uuid: &Uuid) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, uuid.as_bytes())
}

/// Convert base64 to UUID
fn base64_to_uuid(b64: &str) -> Option<Uuid> {
    let bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64.trim()).ok()?;

    if bytes.len() != 16 {
        return None;
    }

    let mut arr = [0u8; 16];
    arr.copy_from_slice(&bytes);
    Some(Uuid::from_bytes(arr))
}

/// Format DateTime for KDBX XML
pub fn format_time(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Parse KDBX time format
fn parse_kdbx_time(s: &str) -> std::result::Result<DateTime<Utc>, chrono::ParseError> {
    // Try ISO 8601 format first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    // Try KDBX format
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
        .map(|ndt| DateTime::from_naive_utc_and_offset(ndt, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_ampersand() {
        assert_eq!(escape_xml("AT&T"), "AT&amp;T");
    }

    #[test]
    fn test_escape_xml_brackets() {
        assert_eq!(escape_xml("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_escape_xml_quotes() {
        assert_eq!(escape_xml("\"hello\""), "&quot;hello&quot;");
    }

    #[test]
    fn test_unescape_xml() {
        assert_eq!(unescape_xml("AT&amp;T"), "AT&T");
        assert_eq!(unescape_xml("&lt;tag&gt;"), "<tag>");
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let original = "Hello <World> & \"Friends\" 'here'";
        let escaped = escape_xml(original);
        let unescaped = unescape_xml(&escaped);
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_uuid_base64_roundtrip() {
        let uuid = Uuid::new_v4();
        let b64 = uuid_to_base64(&uuid);
        let recovered = base64_to_uuid(&b64).unwrap();
        assert_eq!(uuid, recovered);
    }

    #[test]
    fn test_format_time() {
        let dt = DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(format_time(&dt), "2025-01-15T10:30:00Z");
    }

    #[test]
    fn test_extract_xml_text() {
        let xml = "<Name>My Group</Name>";
        assert_eq!(extract_xml_text(xml, "Name"), Some("My Group".to_string()));
    }

    #[test]
    fn test_extract_xml_text_missing() {
        let xml = "<Name>My Group</Name>";
        assert_eq!(extract_xml_text(xml, "Notes"), None);
    }

    #[test]
    fn test_extract_all_xml_elements() {
        let xml = "<Entry><UUID>1</UUID></Entry><Entry><UUID>2</UUID></Entry>";
        let entries = extract_all_xml_elements(xml, "Entry");
        assert_eq!(entries.len(), 2);
        assert!(entries[0].contains("1"));
        assert!(entries[1].contains("2"));
    }

    #[test]
    fn test_serializer_produces_valid_xml() {
        let mut vault = Vault::new("Test Vault");
        let root = vault.root_group_uuid;

        let uuid = vault.create_entry(root).unwrap();
        if let Some(entry) = vault.get_entry_mut(&uuid) {
            entry.title.set("GitHub");
            entry.username.set("user@example.com");
            entry.password.set("SecureP@ss123!");
            entry.url = "https://github.com".to_string();
        }

        let serializer = XmlSerializer::new(vec![0u8; 64]);
        let xml_bytes = serializer.serialize(&vault).unwrap();
        let xml = String::from_utf8(xml_bytes).unwrap();

        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<KeePassFile>"));
        assert!(xml.contains("</KeePassFile>"));
        assert!(xml.contains("<Generator>KeePassEx</Generator>"));
        assert!(xml.contains("GitHub"));
        assert!(xml.contains("user@example.com"));
        assert!(xml.contains("https://github.com"));
    }

    #[test]
    fn test_serializer_escapes_special_chars() {
        let mut vault = Vault::new("Test & Vault");
        let root = vault.root_group_uuid;

        let uuid = vault.create_entry(root).unwrap();
        if let Some(entry) = vault.get_entry_mut(&uuid) {
            entry.title.set("Entry <with> special & chars");
        }

        let serializer = XmlSerializer::new(vec![0u8; 64]);
        let xml_bytes = serializer.serialize(&vault).unwrap();
        let xml = String::from_utf8(xml_bytes).unwrap();

        // Special chars should be escaped
        assert!(!xml.contains("<with>"));
        assert!(xml.contains("&lt;with&gt;"));
        assert!(xml.contains("&amp;"));
    }
}
