//! `kpx template` — list and use entry templates
//!
//! Usage:
//!   kpx template list
//!   kpx template show <id>
//!   kpx template add --title "My Entry" --template builtin.credit_card

use crate::output::{print_table, print_json};
use keepassex_core::templates::{TemplateManager, EntryTemplate};
use colored::Colorize;

pub fn run_list(format: &str) -> anyhow::Result<()> {
    let manager = TemplateManager::new();
    let templates: Vec<&EntryTemplate> = manager.all().iter().collect();

    match format {
        "json" => {
            let data: Vec<serde_json::Value> = templates.iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "name": t.name,
                    "description": t.description,
                    "fields": t.fields.len(),
                    "built_in": t.is_built_in,
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        _ => {
            println!("{}", "Entry Templates".bold());
            println!("{}", "─".repeat(60).dimmed());
            for t in &templates {
                let tag = if t.is_built_in {
                    "built-in".dimmed()
                } else {
                    "custom".cyan()
                };
                println!(
                    "  {} {} {} ({} fields)",
                    t.id.green(),
                    "—".dimmed(),
                    t.name.bold(),
                    t.fields.len().to_string().yellow(),
                );
                println!("    {} [{}]", t.description.dimmed(), tag);
            }
            println!();
            println!("{} templates available", templates.len().to_string().bold());
        }
    }

    Ok(())
}

pub fn run_show(id: &str) -> anyhow::Result<()> {
    let manager = TemplateManager::new();
    let template = manager.get(id)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", id))?;

    println!("{}", template.name.bold());
    println!("{}", template.description.dimmed());
    println!();
    println!("{}", "Fields:".underline());

    for field in &template.fields {
        let protected = if field.protected { " 🔒".dimmed() } else { "".normal() };
        let required = if field.required { " *".red() } else { "".normal() };
        println!(
            "  {} ({:?}){}{}",
            field.label.bold(),
            field.field_type,
            protected,
            required,
        );
        if let Some(ref placeholder) = field.placeholder {
            println!("    {}", format!("e.g. {}", placeholder).dimmed());
        }
    }

    if let Some(ref seq) = template.auto_type_sequence {
        println!();
        println!("{} {}", "Auto-Type:".dimmed(), seq.cyan());
    }

    Ok(())
}
