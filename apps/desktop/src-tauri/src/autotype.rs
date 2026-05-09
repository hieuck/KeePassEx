//! Auto-Type engine for desktop
//!
//! Simulates keyboard input to fill credentials in other applications.
//! Uses `enigo` for cross-platform keyboard simulation (Windows/macOS/Linux).
//!
//! Supports KeePass-compatible placeholders:
//!   {USERNAME}, {PASSWORD}, {URL}, {TITLE}, {TAB}, {ENTER}, {SPACE},
//!   {TOTP}, {DELAY X}, {CLEARFIELD}, {S:FieldName}

use enigo::{Enigo, Key, Keyboard, Settings};

pub struct AutoTypeEngine;

impl AutoTypeEngine {
    /// Parse and execute an Auto-Type sequence
    pub fn execute(sequence: &str, context: &AutoTypeContext) -> Result<(), String> {
        let tokens = Self::parse_sequence(sequence)?;
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to init keyboard: {}", e))?;

        for token in tokens {
            Self::execute_token(&token, context, &mut enigo)?;
        }
        Ok(())
    }

    fn parse_sequence(sequence: &str) -> Result<Vec<AutoTypeToken>, String> {
        let mut tokens = Vec::new();
        let mut chars = sequence.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut placeholder = String::new();
                for inner in chars.by_ref() {
                    if inner == '}' {
                        break;
                    }
                    placeholder.push(inner);
                }
                tokens.push(AutoTypeToken::Placeholder(placeholder));
            } else {
                tokens.push(AutoTypeToken::Char(ch));
            }
        }

        Ok(tokens)
    }

    fn execute_token(
        token: &AutoTypeToken,
        context: &AutoTypeContext,
        enigo: &mut Enigo,
    ) -> Result<(), String> {
        match token {
            AutoTypeToken::Char(ch) => {
                enigo
                    .text(&ch.to_string())
                    .map_err(|e| format!("Type char failed: {}", e))?;
            }
            AutoTypeToken::Placeholder(name) => {
                let upper = name.to_uppercase();
                match upper.as_str() {
                    "TAB" => {
                        enigo
                            .key(Key::Tab, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "ENTER" | "RETURN" => {
                        enigo
                            .key(Key::Return, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "SPACE" => {
                        enigo
                            .key(Key::Space, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "ESCAPE" | "ESC" => {
                        enigo
                            .key(Key::Escape, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "BACKSPACE" | "BS" => {
                        enigo
                            .key(Key::Backspace, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "DELETE" | "DEL" => {
                        enigo
                            .key(Key::Delete, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "HOME" => {
                        enigo
                            .key(Key::Home, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "END" => {
                        enigo
                            .key(Key::End, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "UP" => {
                        enigo
                            .key(Key::UpArrow, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "DOWN" => {
                        enigo
                            .key(Key::DownArrow, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "LEFT" => {
                        enigo
                            .key(Key::LeftArrow, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "RIGHT" => {
                        enigo
                            .key(Key::RightArrow, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "CLEARFIELD" => {
                        // Ctrl+A then Delete
                        enigo
                            .key(Key::Control, enigo::Direction::Press)
                            .map_err(|e| e.to_string())?;
                        enigo
                            .key(Key::Unicode('a'), enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                        enigo
                            .key(Key::Control, enigo::Direction::Release)
                            .map_err(|e| e.to_string())?;
                        enigo
                            .key(Key::Delete, enigo::Direction::Click)
                            .map_err(|e| e.to_string())?;
                    }
                    "USERNAME" => {
                        enigo.text(&context.username).map_err(|e| e.to_string())?;
                    }
                    "PASSWORD" => {
                        enigo.text(&context.password).map_err(|e| e.to_string())?;
                    }
                    "URL" => {
                        enigo.text(&context.url).map_err(|e| e.to_string())?;
                    }
                    "TITLE" => {
                        enigo.text(&context.title).map_err(|e| e.to_string())?;
                    }
                    "TOTP" => {
                        if let Some(ref totp) = context.totp {
                            enigo.text(totp).map_err(|e| e.to_string())?;
                        }
                    }
                    _ if upper.starts_with("DELAY ") => {
                        let ms: u64 = upper[6..].parse().unwrap_or(0);
                        std::thread::sleep(std::time::Duration::from_millis(ms));
                    }
                    _ if upper.starts_with("S:") => {
                        let field_name = &name[2..];
                        if let Some(val) = context.custom_fields.get(field_name) {
                            enigo.text(val).map_err(|e| e.to_string())?;
                        }
                    }
                    _ => {
                        // Unknown placeholder — skip silently
                        tracing::warn!("Unknown auto-type placeholder: {{{}}}", name);
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum AutoTypeToken {
    Char(char),
    Placeholder(String),
}

/// Context for Auto-Type execution
pub struct AutoTypeContext {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub totp: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}
