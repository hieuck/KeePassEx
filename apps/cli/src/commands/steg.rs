//! CLI — Steganography commands
//!
//! kpx steg embed --carrier photo.png --vault vault.kdbx --output photo_out.png
//! kpx steg extract --carrier photo_out.png --output extracted.kdbx
//! kpx steg detect --carrier photo.png

use crate::output::print_success;
use colored::Colorize;
use std::path::PathBuf;

#[derive(clap::Subcommand)]
pub enum StegAction {
    /// Embed an encrypted vault into an image or video file
    Embed {
        /// Carrier file (PNG, JPEG, MP4, AVI)
        #[arg(short, long)]
        carrier: PathBuf,
        /// Vault file to embed (.kdbx)
        #[arg(short = 'V', long)]
        vault: PathBuf,
        /// Output file (same format as carrier)
        #[arg(short, long)]
        output: PathBuf,
        /// Steganography password (separate from vault password)
        #[arg(short = 'P', long, env = "KPX_STEG_PASSWORD", hide_env_values = true)]
        steg_password: Option<String>,
    },
    /// Extract a vault from a carrier file
    Extract {
        /// Carrier file with embedded vault
        #[arg(short, long)]
        carrier: PathBuf,
        /// Output vault file (.kdbx)
        #[arg(short, long)]
        output: PathBuf,
        /// Steganography password
        #[arg(short = 'P', long, env = "KPX_STEG_PASSWORD", hide_env_values = true)]
        steg_password: Option<String>,
    },
    /// Detect if a file contains an embedded vault
    Detect {
        /// File to check
        #[arg(short, long)]
        carrier: PathBuf,
    },
    /// Show capacity of a carrier file
    Capacity {
        /// Carrier file
        #[arg(short, long)]
        carrier: PathBuf,
    },
}

pub fn run(action: &StegAction, quiet: bool) -> anyhow::Result<()> {
    match action {
        StegAction::Embed {
            carrier,
            vault,
            output,
            steg_password,
        } => {
            let password = get_steg_password(steg_password.as_deref())?;

            if !quiet {
                println!(
                    "{}",
                    format!(
                        "Embedding vault '{}' into '{}'...",
                        vault.display(),
                        carrier.display()
                    )
                    .cyan()
                );
            }

            let carrier_data = std::fs::read(carrier)?;
            let vault_data = std::fs::read(vault)?;

            let modified = keepassex_core::steg::embed(&carrier_data, &vault_data, &password)
                .map_err(|e| anyhow::anyhow!("Embedding failed: {}", e))?;

            std::fs::write(output, &modified)?;

            print_success(&format!(
                "Vault embedded into '{}' ({} bytes)",
                output.display(),
                modified.len()
            ));
            Ok(())
        }

        StegAction::Extract {
            carrier,
            output,
            steg_password,
        } => {
            let password = get_steg_password(steg_password.as_deref())?;

            if !quiet {
                println!(
                    "{}",
                    format!("Extracting vault from '{}'...", carrier.display()).cyan()
                );
            }

            let carrier_data = std::fs::read(carrier)?;
            let vault_data = keepassex_core::steg::extract(&carrier_data, &password)
                .map_err(|e| anyhow::anyhow!("Extraction failed: {}", e))?;

            std::fs::write(output, &vault_data)?;

            print_success(&format!(
                "Vault extracted to '{}' ({} bytes)",
                output.display(),
                vault_data.len()
            ));
            Ok(())
        }

        StegAction::Detect { carrier } => {
            let carrier_data = std::fs::read(carrier)?;
            let has_vault = keepassex_core::steg::has_embedded_vault(&carrier_data);

            if has_vault {
                println!(
                    "{} {}",
                    "✓".green().bold(),
                    format!(
                        "'{}' contains an embedded KeePassEx vault",
                        carrier.display()
                    )
                    .green()
                );
            } else {
                println!(
                    "{} {}",
                    "✗".yellow(),
                    format!("'{}' does not contain an embedded vault", carrier.display()).yellow()
                );
            }
            Ok(())
        }

        StegAction::Capacity { carrier } => {
            let carrier_data = std::fs::read(carrier)?;
            let capacity = keepassex_core::steg::max_capacity(&carrier_data);

            match capacity {
                Some(cap) => {
                    let cap_str = if cap == usize::MAX / 2 {
                        "Unlimited".to_string()
                    } else {
                        format_bytes(cap)
                    };
                    println!(
                        "{} Maximum capacity: {}",
                        "📦".cyan(),
                        cap_str.green().bold()
                    );
                }
                None => {
                    eprintln!("{}", "Unsupported carrier format".red());
                }
            }
            Ok(())
        }
    }
}

fn get_steg_password(provided: Option<&str>) -> anyhow::Result<String> {
    if let Some(p) = provided {
        return Ok(p.to_string());
    }
    // Prompt interactively
    let password = rpassword::prompt_password("Steganography password: ")?;
    if password.is_empty() {
        return Err(anyhow::anyhow!("Steganography password cannot be empty"));
    }
    Ok(password)
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} bytes", bytes)
    }
}
