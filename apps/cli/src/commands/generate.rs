//! Password generator command

use keepassex_core::{
    generator::PasswordGenerator,
    types::{PasswordGeneratorConfig, GeneratorMode, WordList},
};
use colored::Colorize;

pub fn run(
    length: usize,
    passphrase: bool,
    words: usize,
    count: usize,
) -> anyhow::Result<()> {
    let config = PasswordGeneratorConfig {
        mode: if passphrase { GeneratorMode::Passphrase } else { GeneratorMode::Random },
        length,
        word_count: words,
        use_uppercase: true,
        use_lowercase: true,
        use_digits: true,
        use_symbols: true,
        ..Default::default()
    };

    for i in 0..count {
        let password = PasswordGenerator::generate(&config)?;
        let entropy = PasswordGenerator::estimate_entropy(&password);
        let strength = PasswordGenerator::score_strength(&password);

        let strength_colored = match strength.score() {
            0 | 1 => strength.label_en().red(),
            2 => strength.label_en().yellow(),
            3 => strength.label_en().green(),
            _ => strength.label_en().bright_green(),
        };

        if count > 1 {
            print!("{}. ", i + 1);
        }

        println!(
            "{}  {} ({:.0} bits)",
            password.bold(),
            strength_colored,
            entropy
        );
    }

    Ok(())
}
