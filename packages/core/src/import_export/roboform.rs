//! RoboForm HTML import

use crate::error::Result;
use crate::import_export::import_types::{ImportEntry, ImportGroup, ImportResult};
use uuid::Uuid;

/// Import a RoboForm HTML export file.
pub fn import_roboform(html_data: &str) -> Result<ImportResult> {
    let mut entries: Vec<ImportEntry> = Vec::new();
    let mut groups: Vec<ImportGroup> = Vec::new();
    let mut warnings = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();
    let root_uuid = Uuid::new_v4().to_string();
    let mut folder_groups: std::collections::HashMap<String, String> = Default::default();

    let rows = extract_table_rows(html_data);
    if rows.is_empty() {
        return Ok(ImportResult {
            entries,
            groups,
            warnings,
        });
    }

    let header = &rows[0];
    let col_name = find_col(header, &["name", "title", "site"]);
    let col_url = find_col(header, &["url", "website", "address"]);
    let col_login = find_col(header, &["login", "username", "user", "email"]);
    let col_password = find_col(header, &["password", "passwd", "pass"]);
    let col_note = find_col(header, &["note", "notes", "comment"]);
    let col_folder = find_col(header, &["folder", "group", "category"]);

    let data_rows = if col_name.is_some() || col_url.is_some() {
        &rows[1..]
    } else {
        &rows[..]
    };

    for row in data_rows {
        if row.is_empty() || row.iter().all(|c| c.trim().is_empty()) {
            continue;
        }

        let get = |idx: Option<usize>| -> &str {
            idx.and_then(|i| row.get(i)).map(|s| s.trim()).unwrap_or("")
        };

        let title = get(col_name);
        let url = get(col_url);
        let username = get(col_login);
        let password = get(col_password);
        let note = get(col_note);
        let folder = get(col_folder);

        if title.is_empty() && username.is_empty() && url.is_empty() {
            continue;
        }

        let group_uuid = if !folder.is_empty() {
            folder_groups
                .entry(folder.to_string())
                .or_insert_with(|| {
                    let uuid = Uuid::new_v4().to_string();
                    groups.push(ImportGroup {
                        uuid: uuid.clone(),
                        parent_uuid: Some(root_uuid.clone()),
                        name: folder.to_string(),
                        icon_id: 48,
                        is_expanded: true,
                        ..Default::default()
                    });
                    uuid
                })
                .clone()
        } else {
            root_uuid.clone()
        };

        entries.push(ImportEntry {
            uuid: Uuid::new_v4().to_string(),
            group_uuid,
            title: if title.is_empty() {
                extract_domain(url).unwrap_or_else(|| "Untitled".into())
            } else {
                title.into()
            },
            username: username.into(),
            password: if password.is_empty() {
                None
            } else {
                Some(password.into())
            },
            url: url.into(),
            notes: note.into(),
            icon_id: 1,
            has_password: !password.is_empty(),
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

fn extract_table_rows(html: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(tr_start) = lower[pos..].find("<tr") {
        let tr_start = pos + tr_start;
        let tr_end = lower[tr_start..]
            .find("</tr>")
            .map(|i| tr_start + i + 5)
            .unwrap_or(html.len());
        let cells = extract_cells(&html[tr_start..tr_end]);
        if !cells.is_empty() {
            rows.push(cells);
        }
        pos = tr_end;
        if pos >= html.len() {
            break;
        }
    }
    rows
}

fn extract_cells(row_html: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let lower = row_html.to_lowercase();
    let mut pos = 0;
    while let Some(td_start) = lower[pos..]
        .find("<td")
        .or_else(|| lower[pos..].find("<th"))
    {
        let td_start = pos + td_start;
        let tag_end = lower[td_start..]
            .find('>')
            .map(|i| td_start + i + 1)
            .unwrap_or(row_html.len());
        let close = if lower[td_start..].starts_with("<th") {
            "</th>"
        } else {
            "</td>"
        };
        let td_end = lower[tag_end..]
            .find(close)
            .map(|i| tag_end + i)
            .unwrap_or(row_html.len());
        cells.push(strip_html(&row_html[tag_end..td_end]).trim().to_string());
        pos = td_end + close.len();
        if pos >= row_html.len() {
            break;
        }
    }
    cells
}

fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn find_col(header: &[String], names: &[&str]) -> Option<usize> {
    header.iter().position(|h| {
        let hl = h.to_lowercase();
        names.iter().any(|n| hl.contains(n))
    })
}

fn extract_domain(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    let u = if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    u.split("://")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .map(|h| h.trim_start_matches("www.").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HTML: &str = r#"<table>
<tr><th>Name</th><th>URL</th><th>Login</th><th>Password</th><th>Note</th><th>Folder</th></tr>
<tr><td>GitHub</td><td>https://github.com</td><td>user@example.com</td><td>secret123</td><td>Work account</td><td>Work</td></tr>
<tr><td>Gmail</td><td>https://mail.google.com</td><td>user@gmail.com</td><td>gmailpass</td><td></td><td>Personal</td></tr>
<tr><td>Amazon</td><td>https://amazon.com</td><td>shopper</td><td>shoppass</td><td></td><td>Shopping</td></tr>
</table>"#;

    #[test]
    fn test_import_entries() {
        assert_eq!(import_roboform(HTML).unwrap().entries.len(), 3);
    }

    #[test]
    fn test_import_fields() {
        let r = import_roboform(HTML).unwrap();
        let gh = r.entries.iter().find(|e| e.title == "GitHub").unwrap();
        assert_eq!(gh.username, "user@example.com");
        assert!(gh.has_password);
        assert_eq!(gh.url, "https://github.com");
        assert_eq!(gh.notes, "Work account");
    }

    #[test]
    fn test_import_folders() {
        let r = import_roboform(HTML).unwrap();
        let names: Vec<&str> = r.groups.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"Work"));
        assert!(names.contains(&"Personal"));
        assert!(names.contains(&"Shopping"));
    }

    #[test]
    fn test_empty_html() {
        assert_eq!(
            import_roboform("<html><body></body></html>")
                .unwrap()
                .entries
                .len(),
            0
        );
    }

    #[test]
    fn test_html_entities() {
        let html = r#"<table>
<tr><th>Name</th><th>URL</th><th>Login</th><th>Password</th><th>Note</th><th>Folder</th></tr>
<tr><td>AT&amp;T</td><td>https://att.com</td><td>user</td><td>pass</td><td></td><td></td></tr>
</table>"#;
        let r = import_roboform(html).unwrap();
        assert_eq!(r.entries[0].title, "AT&T");
    }
}
