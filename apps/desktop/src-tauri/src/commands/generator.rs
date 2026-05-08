//! Password generator Tauri commands

use keepassex_core::{
    generator::PasswordGenerator,
    types::{GeneratorMode, PasswordGeneratorConfig, WordList},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GeneratePasswordArgs {
    pub mode: String,
    pub length: usize,
    pub use_uppercase: bool,
    pub use_lowercase: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
    pub custom_symbols: Option<String>,
    pub exclude_ambiguous: bool,
    pub exclude_chars: String,
    pub min_uppercase: usize,
    pub min_lowercase: usize,
    pub min_digits: usize,
    pub min_symbols: usize,
    pub word_count: usize,
    pub word_separator: String,
    pub capitalize_words: bool,
    pub include_number: bool,
}

#[derive(Debug, Serialize)]
pub struct GeneratePasswordResult {
    pub password: String,
    pub entropy: f64,
    pub strength_score: u8,
    pub strength_label: String,
}

/// Generate a password
#[tauri::command]
pub fn generate_password(args: GeneratePasswordArgs) -> Result<GeneratePasswordResult, String> {
    let config = PasswordGeneratorConfig {
        mode: match args.mode.as_str() {
            "passphrase" => GeneratorMode::Passphrase,
            "pronounceable" => GeneratorMode::Pronounceable,
            _ => GeneratorMode::Random,
        },
        length: args.length,
        use_uppercase: args.use_uppercase,
        use_lowercase: args.use_lowercase,
        use_digits: args.use_digits,
        use_symbols: args.use_symbols,
        custom_symbols: args.custom_symbols,
        exclude_ambiguous: args.exclude_ambiguous,
        exclude_chars: args.exclude_chars,
        min_uppercase: args.min_uppercase,
        min_lowercase: args.min_lowercase,
        min_digits: args.min_digits,
        min_symbols: args.min_symbols,
        word_count: args.word_count,
        word_separator: args.word_separator,
        capitalize_words: args.capitalize_words,
        include_number: args.include_number,
        wordlist: WordList::Eff,
    };

    let password = PasswordGenerator::generate(&config).map_err(|e| e.to_string())?;
    let entropy = PasswordGenerator::estimate_entropy(&password);
    let strength = PasswordGenerator::score_strength(&password);

    Ok(GeneratePasswordResult {
        password,
        entropy,
        strength_score: strength.score(),
        strength_label: strength.label_en().to_string(),
    })
}

/// Estimate entropy of a password
#[tauri::command]
pub fn estimate_entropy(password: String) -> f64 {
    PasswordGenerator::estimate_entropy(&password)
}

/// Score password strength
#[tauri::command]
pub fn score_strength(password: String) -> u8 {
    PasswordGenerator::score_strength(&password).score()
}
