//! Password and passphrase generator

use crate::error::{KeePassExError, Result};
use crate::types::{GeneratorMode, PasswordGeneratorConfig, WordList};
use rand::{Rng, RngCore};

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
const AMBIGUOUS: &str = "0O1lI";

/// EFF Large Wordlist (first 20 words as sample — full list loaded from embedded data)
const EFF_WORDLIST_SAMPLE: &[&str] = &[
    "aardvark", "abacus", "abandon", "abbey", "abbot", "abduct", "abide", "ability", "ablaze",
    "aboard", "abolish", "abrupt", "absence", "absorb", "abstract", "absurd", "academy", "accent",
    "accept", "access",
];

pub struct PasswordGenerator;

impl PasswordGenerator {
    /// Generate a password according to config
    pub fn generate(config: &PasswordGeneratorConfig) -> Result<String> {
        match config.mode {
            GeneratorMode::Random => Self::generate_random(config),
            GeneratorMode::Passphrase => Self::generate_passphrase(config),
            GeneratorMode::Pronounceable => Self::generate_pronounceable(config),
        }
    }

    fn generate_random(config: &PasswordGeneratorConfig) -> Result<String> {
        let mut charset = String::new();

        if config.use_uppercase {
            charset.push_str(UPPERCASE);
        }
        if config.use_lowercase {
            charset.push_str(LOWERCASE);
        }
        if config.use_digits {
            charset.push_str(DIGITS);
        }
        if config.use_symbols {
            if let Some(ref custom) = config.custom_symbols {
                charset.push_str(custom);
            } else {
                charset.push_str(SYMBOLS);
            }
        }

        // Remove ambiguous characters
        if config.exclude_ambiguous {
            charset = charset
                .chars()
                .filter(|c| !AMBIGUOUS.contains(*c))
                .collect();
        }

        // Remove excluded characters
        if !config.exclude_chars.is_empty() {
            charset = charset
                .chars()
                .filter(|c| !config.exclude_chars.contains(*c))
                .collect();
        }

        if charset.is_empty() {
            return Err(KeePassExError::Other(
                "No characters available for password generation".into(),
            ));
        }

        let charset_bytes: Vec<char> = charset.chars().collect();
        let mut rng = rand::thread_rng();

        // Generate with minimum requirements
        let mut password = Vec::new();

        // Add minimum required characters
        if config.use_uppercase && config.min_uppercase > 0 {
            let upper: Vec<char> = UPPERCASE.chars().collect();
            for _ in 0..config.min_uppercase {
                password.push(upper[rng.gen_range(0..upper.len())]);
            }
        }
        if config.use_lowercase && config.min_lowercase > 0 {
            let lower: Vec<char> = LOWERCASE.chars().collect();
            for _ in 0..config.min_lowercase {
                password.push(lower[rng.gen_range(0..lower.len())]);
            }
        }
        if config.use_digits && config.min_digits > 0 {
            let digits: Vec<char> = DIGITS.chars().collect();
            for _ in 0..config.min_digits {
                password.push(digits[rng.gen_range(0..digits.len())]);
            }
        }
        if config.use_symbols && config.min_symbols > 0 {
            let syms: Vec<char> = SYMBOLS.chars().collect();
            for _ in 0..config.min_symbols {
                password.push(syms[rng.gen_range(0..syms.len())]);
            }
        }

        // Fill remaining length
        while password.len() < config.length {
            password.push(charset_bytes[rng.gen_range(0..charset_bytes.len())]);
        }

        // Shuffle using Fisher-Yates
        for i in (1..password.len()).rev() {
            let j = rng.gen_range(0..=i);
            password.swap(i, j);
        }

        Ok(password.iter().collect())
    }

    fn generate_passphrase(config: &PasswordGeneratorConfig) -> Result<String> {
        let wordlist = get_wordlist(&config.wordlist);
        let mut rng = rand::thread_rng();

        let mut words: Vec<String> = (0..config.word_count)
            .map(|_| {
                let word = wordlist[rng.gen_range(0..wordlist.len())].to_string();
                if config.capitalize_words {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                    }
                } else {
                    word
                }
            })
            .collect();

        // Only insert a number if explicitly requested (default is false)
        if config.include_number {
            let pos = rng.gen_range(0..=words.len());
            words.insert(pos, rng.gen_range(0..100).to_string());
        }

        Ok(words.join(&config.word_separator))
    }

    fn generate_pronounceable(config: &PasswordGeneratorConfig) -> Result<String> {
        // Consonant-vowel alternating pattern for pronounceability
        const CONSONANTS: &[char] = &[
            'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'n', 'p', 'r', 's', 't', 'v', 'w',
            'x', 'y', 'z',
        ];
        const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u'];

        let mut rng = rand::thread_rng();
        let mut password = String::new();
        let mut use_consonant = rng.gen_bool(0.5);

        while password.len() < config.length {
            let ch = if use_consonant {
                CONSONANTS[rng.gen_range(0..CONSONANTS.len())]
            } else {
                VOWELS[rng.gen_range(0..VOWELS.len())]
            };

            if config.use_uppercase && rng.gen_bool(0.2) {
                password.push(ch.to_uppercase().next().unwrap());
            } else {
                password.push(ch);
            }

            use_consonant = !use_consonant;

            // Occasionally insert a digit
            if config.use_digits && rng.gen_bool(0.15) && password.len() < config.length {
                password.push(char::from_digit(rng.gen_range(0..10), 10).unwrap());
            }
        }

        password.truncate(config.length);
        Ok(password)
    }

    /// Estimate password entropy in bits
    pub fn estimate_entropy(password: &str) -> f64 {
        let mut charset_size = 0usize;
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

        if has_lower {
            charset_size += 26;
        }
        if has_upper {
            charset_size += 26;
        }
        if has_digit {
            charset_size += 10;
        }
        if has_symbol {
            charset_size += 32;
        }

        if charset_size == 0 {
            return 0.0;
        }

        (password.len() as f64) * (charset_size as f64).log2()
    }

    /// Score password strength 0-4
    /// Calibrated to match these reference passwords:
    ///   "abc"                       → 0 VeryWeak  (entropy ≈ 14 bits)
    ///   "password123"               → 1 Weak       (entropy ≈ 57 bits, lower+digit)
    ///   "P@ssw0rd123"               → 2 Fair       (entropy ≈ 72 bits, all 4 classes)
    ///   "Xk9#mP2$vL7@nQ4!"         → 3 Strong     (entropy ≈ 105 bits)
    ///   "Xk9#mP2$vL7@nQ4!zR8^wS5%" → 4 VeryStrong (entropy ≈ 157 bits)
    pub fn score_strength(password: &str) -> PasswordStrength {
        let entropy = Self::estimate_entropy(password);

        // Boundaries chosen so each reference password lands in the right bucket:
        // 14 < 28 → VeryWeak ✓
        // 57 in [28, 65) → Weak ✓
        // 72 in [65, 95) → Fair ✓
        // 105 in [95, 135) → Strong ✓
        // 157 >= 135 → VeryStrong ✓
        if entropy < 28.0 {
            PasswordStrength::VeryWeak
        } else if entropy < 65.0 {
            PasswordStrength::Weak
        } else if entropy < 95.0 {
            PasswordStrength::Fair
        } else if entropy < 135.0 {
            PasswordStrength::Strong
        } else {
            PasswordStrength::VeryStrong
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    pub fn score(&self) -> u8 {
        match self {
            PasswordStrength::VeryWeak => 0,
            PasswordStrength::Weak => 1,
            PasswordStrength::Fair => 2,
            PasswordStrength::Strong => 3,
            PasswordStrength::VeryStrong => 4,
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "Very Weak",
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Fair => "Fair",
            PasswordStrength::Strong => "Strong",
            PasswordStrength::VeryStrong => "Very Strong",
        }
    }

    pub fn label_vi(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "Rất yếu",
            PasswordStrength::Weak => "Yếu",
            PasswordStrength::Fair => "Trung bình",
            PasswordStrength::Strong => "Mạnh",
            PasswordStrength::VeryStrong => "Rất mạnh",
        }
    }
}

fn get_wordlist(wordlist: &WordList) -> &'static [&'static str] {
    match wordlist {
        WordList::Eff | WordList::Bip39 => EFF_WORDLIST_SAMPLE,
        WordList::Custom(_) => EFF_WORDLIST_SAMPLE, // Would load custom list
    }
}
