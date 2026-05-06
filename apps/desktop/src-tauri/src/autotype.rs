//! Auto-Type engine for desktop
//! Simulates keyboard input to fill credentials in other applications

/// Auto-Type placeholder engine
/// Supports: {USERNAME}, {PASSWORD}, {URL}, {TITLE}, {TAB}, {ENTER},
///           {TOTP}, {DELAY X}, {CLEARFIELD}, custom fields {S:FieldName}

pub struct AutoTypeEngine;

impl AutoTypeEngine {
    /// Parse and execute an Auto-Type sequence
    pub fn execute(sequence: &str, context: &AutoTypeContext) -> Result<(), String> {
        let tokens = Self::parse_sequence(sequence)?;
        for token in tokens {
            Self::execute_token(&token, context)?;
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

    fn execute_token(token: &AutoTypeToken, context: &AutoTypeContext) -> Result<(), String> {
        match token {
            AutoTypeToken::Char(ch) => {
                // Type the character
                Self::type_char(*ch)?;
            }
            AutoTypeToken::Placeholder(name) => {
                let text = match name.to_uppercase().as_str() {
                    "USERNAME" => context.username.clone(),
                    "PASSWORD" => context.password.clone(),
                    "URL" => context.url.clone(),
                    "TITLE" => context.title.clone(),
                    "TAB" => return Self::press_key("Tab"),
                    "ENTER" => return Self::press_key("Return"),
                    "SPACE" => return Self::type_char(' '),
                    "CLEARFIELD" => return Self::clear_field(),
                    "TOTP" => context.totp.clone().unwrap_or_default(),
                    name if name.starts_with("S:") => {
                        let field_name = &name[2..];
                        context
                            .custom_fields
                            .get(field_name)
                            .cloned()
                            .unwrap_or_default()
                    }
                    name if name.starts_with("DELAY ") => {
                        let ms: u64 = name[6..].parse().unwrap_or(0);
                        std::thread::sleep(std::time::Duration::from_millis(ms));
                        return Ok(());
                    }
                    _ => String::new(),
                };

                for ch in text.chars() {
                    Self::type_char(ch)?;
                }
            }
        }
        Ok(())
    }

    fn type_char(_ch: char) -> Result<(), String> {
        // Platform-specific keyboard simulation
        // Windows: SendInput / keybd_event
        // macOS: CGEventCreateKeyboardEvent
        // Linux: XSendEvent / ydotool
        Ok(())
    }

    fn press_key(_key: &str) -> Result<(), String> {
        Ok(())
    }

    fn clear_field() -> Result<(), String> {
        // Ctrl+A, Delete
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
