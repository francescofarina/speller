use colored::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use toml;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
enum SpellerError {
    IoError(io::Error),
    TomlError(toml::de::Error),
    ConfigError(String),
}

impl fmt::Display for SpellerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SpellerError::IoError(e) => write!(f, "IO error: {}", e),
            SpellerError::TomlError(e) => write!(f, "TOML parsing error: {}", e),
            SpellerError::ConfigError(s) => write!(f, "Configuration error: {}", s),
        }
    }
}

impl Error for SpellerError {}

impl From<io::Error> for SpellerError {
    fn from(err: io::Error) -> SpellerError {
        SpellerError::IoError(err)
    }
}

impl From<toml::de::Error> for SpellerError {
    fn from(err: toml::de::Error) -> SpellerError {
        SpellerError::TomlError(err)
    }
}

type Result<T> = std::result::Result<T, SpellerError>;

#[derive(Deserialize)]
struct Config {
    #[serde(flatten)]
    languages: HashMap<String, HashMap<String, String>>,
}

fn main() -> Result<()> {
    let config = load_config("alphabets.toml")?;
    let mut current_language = String::from("en");

    println!(
        "{}",
        "Welcome to the Phonetic Alphabet Speller!".green().bold()
    );

    loop {
        print!(
            "{}",
            "Enter a word to spell (or '\\q' to exit, '\\l' to change language): ".cyan()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "\\q" => {
                println!("{}", "Goodbye!".green().bold());
                break;
            }
            "\\l" => {
                current_language = change_language(&config.languages)?;
                println!(
                    "{} {}",
                    "Language changed to:".yellow(),
                    current_language.yellow().bold()
                );
            }
            _ => {
                let language_words = config
                    .languages
                    .get(&current_language)
                    .ok_or_else(|| SpellerError::ConfigError("Language not found".to_string()))?;
                spell_word(input, language_words)?;
            }
        }
        println!();
    }

    Ok(())
}

fn load_config(filename: &str) -> Result<Config> {
    let contents = fs::read_to_string(filename)
        .map_err(|e| SpellerError::ConfigError(format!("Failed to read config file: {}", e)))?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

fn spell_word(word: &str, language_words: &HashMap<String, String>) -> Result<()> {
    println!("{} {}", "Spelling:".blue().bold(), word.white().bold());
    for grapheme in word.graphemes(true) {
        let lowercase_grapheme = grapheme.to_lowercase();
        let phonetic_word = language_words.get(&lowercase_grapheme).unwrap_or(&lowercase_grapheme);
        let first_grapheme = phonetic_word.graphemes(true).next().unwrap_or("");
        let first_grapheme_upper = first_grapheme.to_uppercase();
        let rest_of_word = &phonetic_word[first_grapheme.len()..];
        println!(
            "{}: {}{}",
            grapheme.yellow(),
            first_grapheme_upper.red().bold(),
            rest_of_word.red()
        );
    }
    Ok(())
}

fn change_language(languages: &HashMap<String, HashMap<String, String>>) -> Result<String> {
    let mut language_codes: Vec<&String> = languages.keys().collect();
    language_codes.sort();

    println!("{}", "Available languages:".blue().bold());
    for (i, lang) in language_codes.iter().enumerate() {
        println!("{}. {}", (i + 1).to_string().yellow(), lang.cyan());
    }

    loop {
        print!("{}", "Choose a language (1-".yellow());
        print!("{}", language_codes.len().to_string().yellow().bold());
        print!("{}", "): ".yellow());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice: usize = match input.trim().parse() {
            Ok(num) if num > 0 && num <= language_codes.len() => num,
            Ok(_) => {
                println!("{}", "Invalid choice. Please try again.".red());
                continue;
            }
            Err(_) => {
                println!("{}", "Invalid input. Please enter a number.".red());
                continue;
            }
        };

        return Ok(language_codes[choice - 1].to_string());
    }
}