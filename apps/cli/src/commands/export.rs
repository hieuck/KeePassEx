//! CLI `export` command — export vault to various formats

use colored::Colorize;
use keepassex_core::{
    import_export::{export_vault, ExportFormat},
    Vault,
};

pub fn run(vault: &Vault, output: &str, format: &str) -> anyhow::Result<()> {
    let export_format = match format.to_lowercase().as_str() {
        "csv" => {
            eprintln!(
                "{} CSV export contains unencrypted passwords. Keep the file secure!",
                "⚠".yellow()
            );
            ExportFormat::CsvUnencrypted
        }
        "json" => {
            eprintln!(
                "{} JSON export contains unencrypted passwords. Keep the file secure!",
                "⚠".yellow()
            );
            ExportFormat::JsonUnencrypted
        }
        "html" => {
            eprintln!(
                "{} HTML export contains unencrypted passwords. Keep the file secure!",
                "⚠".yellow()
            );
            return export_html(vault, output);
        }
        "kdbx" => {
            eprintln!(
                "{}",
                "Use the original vault file for KDBX export.".dimmed()
            );
            return Ok(());
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown format '{}'. Supported: csv, json, html, kdbx",
                format
            ));
        }
    };

    let data = export_vault(vault, export_format)?;
    std::fs::write(output, &data)?;

    let size = data.len();
    eprintln!(
        "{} Exported {} entries to {} ({} bytes)",
        "✓".green(),
        vault.entry_count().to_string().bold(),
        output.bold(),
        size
    );

    Ok(())
}

/// Export vault to a readable HTML file (like KeePassXC's HTML export)
fn export_html(vault: &Vault, output: &str) -> anyhow::Result<()> {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>KeePassEx Vault Export</title>
<style>
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 900px; margin: 40px auto; padding: 0 20px; color: #111; }
  h1 { font-size: 24px; border-bottom: 2px solid #2563eb; padding-bottom: 8px; }
  h2 { font-size: 16px; color: #6b7280; margin: 24px 0 8px; }
  table { width: 100%; border-collapse: collapse; margin-bottom: 24px; }
  th { background: #f3f4f6; text-align: left; padding: 8px 12px; font-size: 12px; text-transform: uppercase; letter-spacing: .05em; color: #6b7280; }
  td { padding: 8px 12px; border-bottom: 1px solid #e5e7eb; font-size: 14px; word-break: break-all; }
  tr:hover td { background: #f9fafb; }
  .pw { font-family: monospace; color: #dc2626; }
  .tag { background: #eff6ff; color: #2563eb; padding: 1px 6px; border-radius: 4px; font-size: 11px; }
  .warning { background: #fef3c7; border: 1px solid #fcd34d; padding: 12px 16px; border-radius: 6px; margin-bottom: 24px; font-size: 13px; color: #92400e; }
  .meta { font-size: 12px; color: #9ca3af; margin-bottom: 24px; }
</style>
</head>
<body>
<h1>🔐 KeePassEx Vault Export</h1>
<div class="warning">⚠️ This file contains unencrypted passwords. Store it securely and delete after use.</div>
<div class="meta">Exported: "#,
    );

    html.push_str(&chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());
    html.push_str(&format!(" · {} entries</div>\n", vault.entry_count()));

    // Group entries by group
    let groups: Vec<_> = vault.all_groups().collect();
    for group in &groups {
        let entries: Vec<_> = vault.get_group_entries(&group.uuid);
        if entries.is_empty() {
            continue;
        }

        html.push_str(&format!("<h2>📁 {}</h2>\n", html_escape(&group.name)));
        html.push_str("<table>\n<tr><th>Title</th><th>Username</th><th>Password</th><th>URL</th><th>Notes</th><th>Tags</th></tr>\n");

        for entry in entries {
            let tags_html = entry
                .tags
                .iter()
                .map(|t| format!("<span class=\"tag\">{}</span>", html_escape(t)))
                .collect::<Vec<_>>()
                .join(" ");

            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td class=\"pw\">{}</td><td><a href=\"{}\">{}</a></td><td>{}</td><td>{}</td></tr>\n",
                html_escape(entry.title.get()),
                html_escape(entry.username.get()),
                html_escape(entry.password.get()),
                html_escape(&entry.url),
                html_escape(&entry.url),
                html_escape(entry.notes.get()),
                tags_html,
            ));
        }
        html.push_str("</table>\n");
    }

    html.push_str("</body></html>");

    std::fs::write(output, html.as_bytes())?;

    eprintln!(
        "{} Exported {} entries to {} (HTML)",
        "✓".green(),
        vault.entry_count().to_string().bold(),
        output.bold()
    );

    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
