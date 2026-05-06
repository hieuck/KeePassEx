//! Output formatting utilities

use colored::Colorize;

pub fn success(msg: &str) {
    eprintln!("{} {}", "✓".green(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}

pub fn warn(msg: &str) {
    eprintln!("{} {}", "⚠".yellow(), msg);
}

pub fn info(msg: &str) {
    eprintln!("{} {}", "ℹ".blue(), msg);
}
