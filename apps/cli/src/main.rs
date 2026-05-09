//! KeePassEx CLI — `kpx`
//!
//! Usage:
//!   kpx list [--group <uuid>] [--search <query>]
//!   kpx get <uuid> [--field password|username|url|otp]
//!   kpx add --title <title> [--username <user>] [--url <url>] [--generate]
//!   kpx edit <uuid>
//!   kpx delete <uuid> [--permanent] [--force]
//!   kpx generate [--length 20] [--passphrase] [--count 1]
//!   kpx health
//!   kpx otp <uuid> [--watch]
//!   kpx export --output file.csv --format csv|json
//!   kpx import <file> --format bitwarden|lastpass|chrome|csv [--auto]
//!   kpx sync --provider local --remote /path/to/backup --direction push|pull|merge
//!   kpx stats

mod commands;
mod output;

use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Subcommand)]
enum TagAction {
    /// List all tags with entry counts
    List,
    /// Add a tag to an entry
    Add {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Tag to add
        tag: String,
    },
    /// Remove a tag from an entry
    Remove {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Tag to remove
        tag: String,
    },
    /// List entries with a specific tag
    Entries {
        /// Tag name
        tag: String,
    },
    /// Rename a tag across all entries
    Rename {
        /// Current tag name
        old: String,
        /// New tag name
        new: String,
    },
}

#[derive(Subcommand)]
enum GroupAction {
    /// List all groups in the vault
    List,
    /// Create a new group
    Create {
        /// Group name
        name: String,
        /// Parent group UUID (default: root)
        #[arg(long)]
        parent: Option<String>,
    },
    /// Rename a group
    Rename {
        /// Group UUID (or prefix)
        uuid: String,
        /// New name
        name: String,
    },
    /// Delete a group
    Delete {
        /// Group UUID (or prefix)
        uuid: String,
        /// Permanently delete (skip recycle bin)
        #[arg(long)]
        permanent: bool,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
    /// Move a group to a different parent
    Move {
        /// Group UUID (or prefix)
        uuid: String,
        /// New parent group UUID
        #[arg(long)]
        parent: String,
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// List all available templates
    List,
    /// Show details of a specific template
    Show {
        /// Template ID (e.g. builtin.credit_card)
        id: String,
    },
}

#[derive(Subcommand)]
enum HardwareKeyAction {
    /// List connected hardware keys
    List,
    /// Test the hardware key challenge-response
    Test {
        /// YubiKey slot (1 or 2)
        #[arg(long, default_value = "2")]
        slot: u8,
    },
    /// Interactive setup wizard
    Setup,
}

#[derive(Subcommand)]
enum ServerAction {
    /// Check server health and connection
    Status {
        /// Server URL
        #[arg(long, env = "KPX_SERVER_URL")]
        url: String,
    },
    /// Log in to a KeePassEx server and print the access token
    Login {
        /// Server URL
        #[arg(long, env = "KPX_SERVER_URL")]
        url: String,
        /// Email address
        #[arg(long, env = "KPX_SERVER_EMAIL")]
        email: String,
        /// Password (prefer env var KPX_SERVER_PASSWORD)
        #[arg(long, env = "KPX_SERVER_PASSWORD", hide_env_values = true)]
        password: Option<String>,
    },
    /// Register a new account on a KeePassEx server
    Register {
        /// Server URL
        #[arg(long, env = "KPX_SERVER_URL")]
        url: String,
        /// Email address
        #[arg(long)]
        email: String,
        /// Password (prefer env var KPX_SERVER_PASSWORD)
        #[arg(long, env = "KPX_SERVER_PASSWORD", hide_env_values = true)]
        password: Option<String>,
    },
    /// Show vault version history on the server
    History {
        /// Server URL
        #[arg(long, env = "KPX_SERVER_URL")]
        url: String,
        /// Access token (or set KPX_SERVER_TOKEN)
        #[arg(long, env = "KPX_SERVER_TOKEN", hide_env_values = true)]
        token: String,
    },
}
#[derive(Parser)]
#[command(
    name = "kpx",
    about = "KeePassEx CLI — Manage your KeePass vault from the terminal",
    version,
    author,
    long_about = "KeePassEx CLI (kpx) — A powerful command-line interface for managing KeePass vaults.\n\nAll commands require --vault and --password (or env vars KPX_VAULT / KPX_PASSWORD).\n\nExamples:\n  kpx list\n  kpx get abc123 --field password --copy\n  kpx add --title GitHub --username user@example.com --generate\n  kpx health\n  kpx export --output backup.csv --format csv"
)]
struct Cli {
    /// Path to the vault file (.kdbx)
    #[arg(short, long, env = "KPX_VAULT", global = true)]
    vault: Option<String>,

    /// Master password (prefer env var KPX_PASSWORD for security)
    #[arg(
        short,
        long,
        env = "KPX_PASSWORD",
        global = true,
        hide_env_values = true
    )]
    password: Option<String>,

    /// Key file path
    #[arg(short, long, env = "KPX_KEY_FILE", global = true)]
    key_file: Option<String>,

    /// Output format: table, json, csv
    #[arg(short, long, default_value = "table", global = true)]
    format: String,

    /// Suppress informational output (only print results)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List entries in the vault
    #[command(alias = "ls")]
    List {
        /// Filter by group UUID or name
        #[arg(short, long)]
        group: Option<String>,
        /// Search query (searches title, username, URL)
        #[arg(short, long)]
        search: Option<String>,
        /// Show passwords in output (use with caution)
        #[arg(long)]
        show_passwords: bool,
    },

    /// Get details of a specific entry
    Get {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Field to retrieve: all, password, username, url, otp, notes
        #[arg(short, long, default_value = "all")]
        field: String,
        /// Copy field value to clipboard
        #[arg(short, long)]
        copy: bool,
    },

    /// Add a new entry
    Add {
        /// Entry title (required)
        #[arg(long)]
        title: String,
        /// Username
        #[arg(long)]
        username: Option<String>,
        /// Password (omit to prompt interactively)
        #[arg(long)]
        password: Option<String>,
        /// URL
        #[arg(long)]
        url: Option<String>,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
        /// Target group UUID or name (default: root)
        #[arg(long)]
        group: Option<String>,
        /// Auto-generate a strong password
        #[arg(long)]
        generate: bool,
    },

    /// Edit an existing entry interactively
    Edit {
        /// Entry UUID (or prefix)
        uuid: String,
    },

    /// Delete an entry
    #[command(alias = "rm")]
    Delete {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Permanently delete (skip recycle bin)
        #[arg(long)]
        permanent: bool,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Generate a password or passphrase
    #[command(alias = "gen")]
    Generate {
        /// Password length (random mode)
        #[arg(short, long, default_value = "20")]
        length: usize,
        /// Generate a passphrase instead of random password
        #[arg(long)]
        passphrase: bool,
        /// Number of words (passphrase mode)
        #[arg(long, default_value = "6")]
        words: usize,
        /// Number of passwords to generate
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// Show vault health report
    Health,

    /// Show OTP code for an entry
    Otp {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Watch mode — refresh every second
        #[arg(short, long)]
        watch: bool,
    },

    /// Export vault to CSV or JSON
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
        /// Format: csv, json
        #[arg(short, long, default_value = "csv")]
        format: String,
    },

    /// Import entries from another password manager
    Import {
        /// Input file path
        input: String,
        /// Format: bitwarden, lastpass, chrome, firefox, 1password, csv, auto
        #[arg(short, long, default_value = "auto")]
        format: String,
    },

    /// Sync vault with a remote location
    Sync {
        /// Sync provider: local, webdav
        #[arg(long, default_value = "local")]
        provider: String,
        /// Remote path or URL
        #[arg(long)]
        remote: String,
        /// Direction: push, pull, merge
        #[arg(long, default_value = "merge")]
        direction: String,
    },

    /// Show vault statistics
    Stats,

    /// Check passwords against HaveIBeenPwned breach database
    Breach {
        /// Use online HIBP API (k-anonymity — passwords never sent)
        #[arg(long)]
        online: bool,
    },

    /// List and use entry templates
    #[command(alias = "tmpl")]
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Manage hardware key (YubiKey / FIDO2) configuration
    #[command(name = "hardware-key", alias = "hk")]
    HardwareKey {
        #[command(subcommand)]
        action: HardwareKeyAction,
    },

    /// Compare two vaults and show differences
    #[command(alias = "diff")]
    Compare {
        /// Second vault path
        vault2: String,
        /// Password for second vault (defaults to same as first)
        #[arg(long)]
        password2: Option<String>,
    },

    /// Manage entry tags
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// Advanced entry search (more powerful than list --search)
    #[command(alias = "locate")]
    Find {
        /// Search query (optional — use flags to filter without text)
        query: Option<String>,
        /// Search only in specific field: title, username, url, notes, tags
        #[arg(short, long)]
        field: Option<String>,
        /// Filter by group name
        #[arg(long)]
        group: Option<String>,
        /// Show only expired entries
        #[arg(long)]
        expired: bool,
        /// Show only entries with OTP
        #[arg(long = "has-otp")]
        has_otp: bool,
        /// Show only entries with passkey
        #[arg(long = "has-passkey")]
        has_passkey: bool,
        /// Show only entries with SSH key
        #[arg(long = "has-ssh")]
        has_ssh: bool,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
    },

    /// Change entry password
    #[command(alias = "pw")]
    Passwd {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Auto-generate a strong password
        #[arg(long)]
        generate: bool,
        /// Password length (when --generate)
        #[arg(long, default_value = "20")]
        length: usize,
    },

    /// Show entry details (like keepassxc-cli show)
    Show {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Show password in plaintext
        #[arg(long)]
        show_password: bool,
        /// Show only a specific field: title, username, password, url, notes, or custom field name
        #[arg(short, long)]
        field: Option<String>,
    },

    /// Show vault audit log
    Audit {
        /// Number of recent events to show
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Show password rotation recommendations
    #[command(alias = "rotate")]
    Rotation {
        /// Filter by urgency: aging, soon, overdue, expired
        #[arg(long)]
        urgency: Option<String>,
    },

    /// Copy entry field to clipboard with auto-clear
    #[command(alias = "cp")]
    Clip {
        /// Entry UUID (or prefix)
        uuid: String,
        /// Field to copy: password (default), username, url, otp, notes, or custom field name
        #[arg(short, long, default_value = "password")]
        field: String,
        /// Clear clipboard after N seconds (0 = no auto-clear)
        #[arg(long, default_value = "10")]
        clear: u64,
    },

    /// Manage vault groups (folders)
    #[command(alias = "grp")]
    Group {
        #[command(subcommand)]
        action: GroupAction,
    },

    /// Manage KeePassEx self-hosted sync server
    #[command(alias = "srv")]
    Server {
        #[command(subcommand)]
        action: ServerAction,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if !cli.quiet {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .init();
    }

    // Generate command doesn't need a vault
    if let Commands::Generate {
        length,
        passphrase,
        words,
        count,
    } = &cli.command
    {
        return commands::generate::run(*length, *passphrase, *words, *count);
    }

    // Template command doesn't need a vault
    if let Commands::Template { action } = &cli.command {
        return match action {
            TemplateAction::List => commands::template::run_list(&cli.format),
            TemplateAction::Show { id } => commands::template::run_show(id),
        };
    }

    // Hardware key list/setup don't need a vault
    if let Commands::HardwareKey { action } = &cli.command {
        return match action {
            HardwareKeyAction::List => commands::hardware_key::run_list().await,
            HardwareKeyAction::Setup => commands::hardware_key::run_setup(),
            HardwareKeyAction::Test { slot } => commands::hardware_key::run_test(*slot).await,
        };
    }

    // Server commands don't need a vault
    if let Commands::Server { action } = &cli.command {
        return match action {
            ServerAction::Status { url } => commands::server::run_status(url).await,
            ServerAction::Login {
                url,
                email,
                password,
            } => commands::server::run_login(url, email, password.as_deref()).await,
            ServerAction::Register {
                url,
                email,
                password,
            } => commands::server::run_register(url, email, password.as_deref()).await,
            ServerAction::History { url, token } => commands::server::run_history(url, token).await,
        };
    }

    // All other commands need a vault
    let vault_path_str = cli.vault.ok_or_else(|| {
        anyhow::anyhow!(
            "No vault specified. Use --vault <path> or set KPX_VAULT environment variable."
        )
    })?;

    let password = if let Some(pw) = cli.password {
        pw
    } else {
        rpassword::prompt_password("Master Password: ")?
    };

    // Open vault
    if !cli.quiet {
        eprint!("{}", "Opening vault...".dimmed());
    }

    let vault_path = std::path::Path::new(&vault_path_str);
    let credentials = keepassex_core::vault::operations::VaultCredentials::password_only(&password);

    let mut vault = keepassex_core::vault::operations::open_vault(vault_path, &credentials)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    if !cli.quiet {
        eprintln!(" {}", "✓".green());
    }

    // Dispatch command
    match cli.command {
        Commands::List {
            group,
            search,
            show_passwords,
        } => commands::list::run(&vault, group, search, show_passwords, &cli.format),

        Commands::Get { uuid, field, copy } => commands::get::run(&vault, &uuid, &field, copy),

        Commands::Add {
            title,
            username,
            password: pw,
            url,
            notes,
            group,
            generate,
        } => {
            commands::add::run(
                &mut vault,
                &vault_path_str,
                &password,
                title,
                username,
                pw,
                url,
                notes,
                group,
                generate,
            )
            .await
        }

        Commands::Edit { uuid } => {
            commands::edit::run(&mut vault, &vault_path_str, &password, &uuid).await
        }

        Commands::Delete {
            uuid,
            permanent,
            force,
        } => {
            commands::delete::run(
                &mut vault,
                &vault_path_str,
                &password,
                &uuid,
                permanent,
                force,
            )
            .await
        }

        Commands::Generate {
            length,
            passphrase,
            words,
            count,
        } => commands::generate::run(length, passphrase, words, count),

        Commands::Health => commands::health::run(&vault, &cli.format),

        Commands::Otp { uuid, watch } => commands::otp::run(&vault, &uuid, watch).await,

        Commands::Export { output, format } => commands::export::run(&vault, &output, &format),

        Commands::Import { input, format } => {
            commands::import::run(&mut vault, &vault_path_str, &password, &input, &format).await
        }

        Commands::Sync {
            provider,
            remote,
            direction,
        } => {
            commands::sync::run(
                &vault,
                &vault_path_str,
                &password,
                &provider,
                &remote,
                &direction,
            )
            .await
        }

        Commands::Stats => commands::stats::run(&vault),

        Commands::Breach { online } => commands::breach::run(&vault, online, &cli.format).await,

        Commands::Template { action } => match action {
            TemplateAction::List => commands::template::run_list(&cli.format),
            TemplateAction::Show { id } => commands::template::run_show(&id),
        },

        Commands::HardwareKey { action } => match action {
            HardwareKeyAction::List => commands::hardware_key::run_list().await,
            HardwareKeyAction::Test { slot } => commands::hardware_key::run_test(slot).await,
            HardwareKeyAction::Setup => commands::hardware_key::run_setup(),
        },

        Commands::Compare { vault2, password2 } => {
            commands::compare::run(
                &vault_path_str,
                &vault2,
                &password,
                password2.as_deref(),
                &cli.format,
            )
            .await
        }

        Commands::Audit { limit } => commands::audit::run(&vault, limit, &cli.format),

        Commands::Show {
            uuid,
            show_password,
            field,
        } => commands::show::run(&vault, &uuid, show_password, field.as_deref(), &cli.format),

        Commands::Passwd {
            uuid,
            generate,
            length,
        } => {
            commands::passwd::run(
                &mut vault,
                &vault_path_str,
                &password,
                &uuid,
                generate,
                length,
            )
            .await
        }

        Commands::Rotation { urgency } => {
            commands::rotation::run(&vault, urgency.as_deref(), &cli.format)
        }

        Commands::Find {
            query,
            field,
            group,
            expired,
            has_otp,
            has_passkey,
            has_ssh,
            tag,
        } => commands::find::run(
            &vault,
            query.as_deref(),
            field.as_deref(),
            group.as_deref(),
            expired,
            has_otp,
            has_passkey,
            has_ssh,
            tag.as_deref(),
            &cli.format,
        ),

        Commands::Clip { uuid, field, clear } => commands::clip::run(&vault, &uuid, &field, clear),

        Commands::Tag { action } => match action {
            TagAction::List => commands::tag::run_list(&vault, &cli.format),
            TagAction::Add { uuid, tag } => {
                commands::tag::run_add(&mut vault, &vault_path_str, &password, &uuid, &tag)
            }
            TagAction::Remove { uuid, tag } => {
                commands::tag::run_remove(&mut vault, &vault_path_str, &password, &uuid, &tag)
            }
            TagAction::Entries { tag } => commands::tag::run_entries(&vault, &tag, &cli.format),
            TagAction::Rename { old, new } => {
                commands::tag::run_rename(&mut vault, &vault_path_str, &password, &old, &new)
            }
        },

        Commands::Group { action } => match action {
            GroupAction::List => commands::group::run_list(&vault, &cli.format),
            GroupAction::Create { name, parent } => commands::group::run_create(
                &mut vault,
                &vault_path_str,
                &password,
                &name,
                parent.as_deref(),
            ),
            GroupAction::Rename { uuid, name } => {
                commands::group::run_rename(&mut vault, &vault_path_str, &password, &uuid, &name)
            }
            GroupAction::Delete {
                uuid,
                permanent,
                force,
            } => commands::group::run_delete(
                &mut vault,
                &vault_path_str,
                &password,
                &uuid,
                permanent,
                force,
            ),
            GroupAction::Move { uuid, parent } => {
                commands::group::run_move(&mut vault, &vault_path_str, &password, &uuid, &parent)
            }
        },

        // Server commands are handled before vault open — unreachable here
        Commands::Server { .. } => unreachable!("Server commands handled before vault open"),
    }
}
