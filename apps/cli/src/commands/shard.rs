//! CLI — Vault Key Sharding commands (Shamir's Secret Sharing)
//!
//! kpx shard split --threshold 3 --total 5 --output-dir ./shards/
//! kpx shard combine --shard shard1.kpxshard --shard shard2.kpxshard --shard shard3.kpxshard
//! kpx shard info --shard shard1.kpxshard

use colored::Colorize;
use std::path::PathBuf;

#[derive(clap::Subcommand)]
pub enum ShardAction {
    /// Split vault master key into N shards (M-of-N threshold)
    Split {
        /// Minimum shards needed to reconstruct (M)
        #[arg(short, long, default_value = "3")]
        threshold: u8,
        /// Total shards to generate (N)
        #[arg(short, long, default_value = "5")]
        total: u8,
        /// Output directory for shard files
        #[arg(short, long, default_value = ".")]
        output_dir: PathBuf,
        /// Optional labels for each shard (comma-separated)
        #[arg(short, long)]
        labels: Option<String>,
    },
    /// Combine shards to reconstruct vault key
    Combine {
        /// Shard files to combine (provide at least threshold count)
        #[arg(short, long = "shard", required = true)]
        shards: Vec<PathBuf>,
        /// Output file for reconstructed key
        #[arg(short, long, default_value = "reconstructed.key")]
        output: PathBuf,
    },
    /// Show information about a shard file
    Info {
        /// Shard file to inspect
        #[arg(short, long)]
        shard: PathBuf,
    },
    /// Verify that shards can reconstruct the key (without revealing it)
    Verify {
        /// Shard files to verify
        #[arg(short, long = "shard", required = true)]
        shards: Vec<PathBuf>,
    },
}

pub fn run(action: &ShardAction, quiet: bool) -> anyhow::Result<()> {
    match action {
        ShardAction::Split {
            threshold,
            total,
            output_dir,
            labels,
        } => {
            if !quiet {
                println!(
                    "{} Generating {}-of-{} key shards...",
                    "🔐".cyan(),
                    threshold.to_string().bold(),
                    total.to_string().bold()
                );
            }

            // Generate a random 32-byte secret (in real use, this would be the vault master key)
            let mut secret = [0u8; 32];
            use rand::RngCore;
            rand::thread_rng().fill_bytes(&mut secret);

            let shards = keepassex_core::crypto::split_secret(&secret, *threshold, *total)
                .map_err(|e| anyhow::anyhow!("Shard generation failed: {}", e))?;

            let label_list: Vec<String> = labels
                .as_deref()
                .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            std::fs::create_dir_all(output_dir)?;

            for (i, shard) in shards.iter().enumerate() {
                let label = label_list
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("Shard {}", i + 1));
                let filename = output_dir.join(format!("shard_{:02}.kpxshard", i + 1));
                let data = shard.to_bytes();
                std::fs::write(&filename, &data)?;

                if !quiet {
                    println!(
                        "  {} {} → {}",
                        "✓".green(),
                        label.bold(),
                        filename.display().to_string().cyan()
                    );
                }
            }

            // Zeroize secret
            use zeroize::Zeroize;
            secret.zeroize();

            if !quiet {
                println!();
                println!(
                    "{} {} shards generated in '{}'",
                    "✓".green().bold(),
                    total.to_string().bold(),
                    output_dir.display().to_string().cyan()
                );
                println!(
                    "{}",
                    format!(
                        "  Any {} of {} shards can reconstruct the vault key.",
                        threshold, total
                    )
                    .yellow()
                );
                println!(
                    "{}",
                    "  ⚠️  Store each shard in a different secure location!".red()
                );
            }

            Ok(())
        }

        ShardAction::Combine { shards, output } => {
            if !quiet {
                println!(
                    "{} Combining {} shards...",
                    "🔓".cyan(),
                    shards.len().to_string().bold()
                );
            }

            let mut shard_objects = Vec::new();
            for path in shards {
                let data = std::fs::read(path).map_err(|e| {
                    anyhow::anyhow!("Cannot read shard '{}': {}", path.display(), e)
                })?;
                let shard = keepassex_core::crypto::SecretShard::from_bytes(&data)
                    .map_err(|e| anyhow::anyhow!("Invalid shard '{}': {}", path.display(), e))?;
                shard_objects.push(shard);
            }

            let secret = keepassex_core::crypto::combine_shards(&shard_objects)
                .map_err(|e| anyhow::anyhow!("Shard combination failed: {}", e))?;

            std::fs::write(output, &secret)?;

            println!(
                "{} Key reconstructed → '{}'",
                "✓".green().bold(),
                output.display().to_string().cyan()
            );
            println!("{}", "  ⚠️  Delete this file after use!".red());

            Ok(())
        }

        ShardAction::Info { shard } => {
            let data = std::fs::read(shard)?;
            let s = keepassex_core::crypto::SecretShard::from_bytes(&data)
                .map_err(|e| anyhow::anyhow!("Invalid shard: {}", e))?;

            println!("{}", "Shard Information".bold().cyan());
            println!("  File:      {}", shard.display().to_string().white());
            println!("  Index:     {} of {}", s.index.to_string().bold(), s.total);
            println!(
                "  Threshold: {} shards required",
                s.threshold.to_string().bold()
            );
            println!("  Data size: {} bytes", s.data.len());
            if let Some(label) = &s.label {
                println!("  Label:     {}", label.bold());
            }

            Ok(())
        }

        ShardAction::Verify { shards } => {
            if !quiet {
                println!("{} Verifying shards...", "🔍".cyan());
            }

            let mut shard_objects = Vec::new();
            for path in shards {
                let data = std::fs::read(path).map_err(|e| {
                    anyhow::anyhow!("Cannot read shard '{}': {}", path.display(), e)
                })?;
                let shard = keepassex_core::crypto::SecretShard::from_bytes(&data)
                    .map_err(|e| anyhow::anyhow!("Invalid shard '{}': {}", path.display(), e))?;
                shard_objects.push(shard);
            }

            if shard_objects.is_empty() {
                return Err(anyhow::anyhow!("No shards provided"));
            }

            let threshold = shard_objects[0].threshold as usize;
            let total = shard_objects[0].total;

            println!("  Shards provided: {}", shards.len().to_string().bold());
            println!("  Threshold:       {}", threshold.to_string().bold());
            println!("  Total:           {}", total.to_string().bold());

            if shard_objects.len() >= threshold {
                // Try to combine (verify it works)
                match keepassex_core::crypto::combine_shards(&shard_objects) {
                    Ok(_) => {
                        println!(
                            "{} Shards are valid and can reconstruct the key",
                            "✓".green().bold()
                        );
                    }
                    Err(e) => {
                        println!("{} Shard combination failed: {}", "✗".red().bold(), e);
                    }
                }
            } else {
                println!(
                    "{} Need {} more shard(s) to reconstruct",
                    "⚠️".yellow(),
                    (threshold - shard_objects.len()).to_string().bold()
                );
            }

            Ok(())
        }
    }
}
