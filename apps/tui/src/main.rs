//! KeePassEx TUI — Terminal User Interface
//!
//! Full-featured terminal UI using Ratatui with vim-style keybindings.
//! No competitor has a TUI this complete.
//!
//! # Keybindings
//! - j/k or ↑/↓  — Navigate entries
//! - h/l or ←/→  — Switch panels
//! - /            — Search (natural language supported)
//! - Enter        — View entry detail
//! - e            — Edit entry
//! - d            — Delete entry (with confirmation)
//! - y            — Copy password to clipboard
//! - u            — Copy username
//! - o            — Open URL
//! - n            — New entry
//! - g            — Go to group
//! - ?            — Help
//! - q / Esc      — Quit / back
//! - :            — Command mode (vim-style)

mod app;
mod commands;
mod events;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use app::{App, AppMode};

#[derive(Parser)]
#[command(
    name = "kpx-tui",
    about = "KeePassEx TUI — Full-featured terminal interface for KeePass vaults",
    version,
    long_about = "KeePassEx TUI (kpx-tui) — A vim-inspired terminal UI for managing KeePass vaults.\n\nKeybindings:\n  j/k     Navigate\n  /       Search\n  Enter   View entry\n  y       Copy password\n  n       New entry\n  ?       Help\n  q       Quit"
)]
struct Cli {
    /// Path to the vault file (.kdbx)
    #[arg(short, long, env = "KPX_VAULT")]
    vault: Option<String>,

    /// Master password (prefer env var KPX_PASSWORD)
    #[arg(short, long, env = "KPX_PASSWORD", hide_env_values = true)]
    password: Option<String>,

    /// Key file path
    #[arg(short, long, env = "KPX_KEY_FILE")]
    key_file: Option<String>,

    /// Color theme: dark, light, solarized, nord, gruvbox
    #[arg(long, default_value = "dark")]
    theme: String,

    /// Enable mouse support
    #[arg(long)]
    mouse: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get vault path
    let vault_path = cli.vault.unwrap_or_else(|| {
        eprintln!("No vault specified. Use --vault <path> or set KPX_VAULT.");
        std::process::exit(1);
    });

    // Get password
    let password = if let Some(pw) = cli.password {
        pw
    } else {
        rpassword::prompt_password("Master Password: ")?
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(&vault_path, &password, cli.key_file.as_deref(), &cli.theme).await?;

    // Main event loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events with timeout
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Global quit
                if key.code == KeyCode::Char('q') && app.mode == AppMode::Normal {
                    break;
                }
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }

                // Dispatch key to app
                if app.handle_key(key).await? {
                    break; // App requested exit
                }
            }
        }

        // Check if app wants to exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
