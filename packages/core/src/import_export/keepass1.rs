//! KeePass 1.x XML import

use crate::error::Result;
use crate::import_export::import_types::{ImportEntry, ImportGroup, ImportResult};
use uuid::Uuid;

/// Import a KeePass 1.x XML export.
pub fn import_keepass1_xml(xml_data: &str) -> Result<ImportResult> {
    let mut entries: Vec<ImportEntry> = Vec::new();
    let mut groups: Vec<ImportGroup> = Vec::new();
    let mut warnings = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();
    let root_uuid = Uuid::new_v4().to_string();
    let mut group_map: std::collections::HashMap<String, String> = Default::default();

    let blocks = extract_xml_blocks(xml_data, "pwentry");
    let blocks = if blocks.is_empty() {
        let alt = extract_xml_blocks(xml_data, "entry");
        if alt.is_empty() {
            warnings.push("No entries found. Export as XML from KeePass 1.x first.".into());
            return Ok(ImportResult {
                entries,
                groups,
                warnings,
            });
        }
        alt
    } else {
        blocks
    };

    for block in &blocks {
        let title = xml_val(block, "title")
            .or_else(|| xml_val(block, "Title"))
            .unwrap_or_else(|| "Untitled".into());
        let group_name = xml_val(block, "group")
            .or_else(|| xml_val(block, "Group"))
            .unwrap_or_else(|| "Imported".into());
        let username = xml_val(block, "username")
            .or_else(|| xml_val(block, "UserName"))
            .unwrap_or_default();
        let password = xml_val(block, "password").or_else(|| xml_val(block, "Password"));
        let url = xml_val(block, "url")
            .or_else(|| xml_val(block, "URL"))
            .unwrap_or_default();
        let notes = xml_val(block, "notes")
            .or_else(|| xml_val(block, "Notes"))
            .unwrap_or_default();
        let expire = xml_val(block, "expire").or_else(|| xml_val(block, "ExpiryTime"));

        let expiry = expire.as_deref().and_then(|e| {
            if e == "Never" || e.is_empty() {
                return None;
            }
            chrono::NaiveDateTime::parse_from_str(e, "%d.%m.%Y %H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc().to_rfc3339())
                .or_else(|| {
                    chrono::DateTime::parse_from_rfc3339(e)
                        .ok()
                        .map(|dt| dt.to_rfc3339())
                })
        });

        let is_expired = expiry
            .as_ref()
            .map(|e| {
                chrono::DateTime::parse_from_rfc3339(e)
                    .map(|dt| dt.with_timezone(&chrono::Utc) <= chrono::Utc::now())
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        let group_uuid = group_map
            .entry(group_name.clone())
            .or_insert_with(|| {
                let uuid = Uuid::new_v4().to_string();
                groups.push(ImportGroup {
                    uuid: uuid.clone(),
                    parent_uuid: Some(root_uuid.clone()),
                    name: group_name.clone(),
                    icon_id: 48,
                    is_expanded: true,
                    ..Default::default()
                });
                uuid
            })
            .clone();

        let has_password = password.is_some();
        entries.push(ImportEntry {
            uuid: Uuid::new_v4().to_string(),
            group_uuid,
            title,
            username,
            password,
            url,
            notes,
            icon_id: 0,
            has_password,
            is_expired,
            expiry,
            created_at: now.clone(),
            modified_at: now.clone(),
            auto_type_enabled: true,
            quality_check: true,
            ..Default::default()
        });
    }

    Ok(ImportResult {
        entries,
        groups,
        warnings,
    })
}

fn extract_xml_blocks(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut blocks = Vec::new();
    let mut pos = 0;
    while let Some(start) = xml[pos..].find(&open) {
        let start = pos + start;
        let tag_end = xml[start..]
            .find('>')
            .map(|i| start + i + 1)
            .unwrap_or(xml.len());
        let end = xml[tag_end..]
            .find(&close)
            .map(|i| tag_end + i + close.len())
            .unwrap_or(xml.len());
        blocks.push(xml[start..end].to_string());
        pos = end;
        if pos >= xml.len() {
            break;
        }
    }
    blocks
}

fn xml_val(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    let v = xml[start..end]
        .trim()
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<?xml version="1.0"?><pwlist>
<pwentry><group>Internet</group><title>GitHub</title><username>user@example.com</username><url>https://github.com</url><password>secret123</password><notes>Work account</notes><expire>Never</expire></pwentry>
<pwentry><group>Email</group><title>Gmail</title><username>user@gmail.com</username><url>https://mail.google.com</url><password>gmailpass</password><notes></notes><expire>Never</expire></pwentry>
<pwentry><group>Internet</group><title>Amazon</title><username>shopper</username><url>https://amazon.com</url><password>shoppass</password><notes></notes><expire>Never</expire></pwentry>
</pwlist>"#;

    #[test]
    fn test_import_entries() {
        assert_eq!(import_keepass1_xml(XML).unwrap().entries.len(), 3);
    }

    #[test]
    fn test_import_fields() {
        let r = import_keepass1_xml(XML).unwrap();
        let gh = r.entries.iter().find(|e| e.title == "GitHub").unwrap();
        assert_eq!(gh.username, "user@example.com");
        assert!(gh.has_password);
        assert_eq!(gh.url, "https://github.com");
        assert_eq!(gh.notes, "Work account");
    }

    #[test]
    fn test_group_deduplication() {
        let r = import_keepass1_xml(XML).unwrap();
        assert_eq!(r.groups.iter().filter(|g| g.name == "Internet").count(), 1);
    }

    #[test]
    fn test_empty_xml() {
        assert_eq!(
            import_keepass1_xml("<pwlist></pwlist>")
                .unwrap()
                .entries
                .len(),
            0
        );
    }

    #[test]
    fn test_never_expiry() {
        let r = import_keepass1_xml(XML).unwrap();
        let gh = r.entries.iter().find(|e| e.title == "GitHub").unwrap();
        assert!(gh.expiry.is_none());
        assert!(!gh.is_expired);
    }

    #[test]
    fn test_xml_entities() {
        let xml = r#"<pwlist><pwentry><group>Test</group><title>AT&amp;T</title><username>u</username><url>https://att.com</url><password>p</password><notes></notes><expire>Never</expire></pwentry></pwlist>"#;
        let r = import_keepass1_xml(xml).unwrap();
        assert_eq!(r.entries[0].title, "AT&T");
    }

    #[test]
    fn test_extract_xml_blocks() {
        let xml = "<pwlist><pwentry><title>A</title></pwentry><pwentry><title>B</title></pwentry></pwlist>";
        assert_eq!(extract_xml_blocks(xml, "pwentry").len(), 2);
    }
}
