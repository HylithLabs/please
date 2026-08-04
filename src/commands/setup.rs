use crate::config;
use std::io::{self, Write};

const PROVIDERS: [&str; 4] = ["Anthropic", "Google", "ChatGPT", "Other Provider"];

pub fn run() {
    println!("Who is your model provider?");
    for (i, name) in PROVIDERS.iter().enumerate() {
        println!("  {}) {}", i + 1, name);
    }

    let provider = match prompt("Select a provider (1-4): ").trim() {
        "1" => "anthropic".to_string(),
        "2" => "google".to_string(),
        "3" => "openai".to_string(),
        "4" => prompt("Enter provider name: ").trim().to_string(),
        _ => {
            eprintln!("Invalid selection.");
            std::process::exit(1);
        }
    };

    let api_key = prompt("Paste your API key: ").trim().to_string();

    if api_key.is_empty() {
        eprintln!("API key cannot be empty.");
        std::process::exit(1);
    }

    config::save(&provider, &api_key).expect("failed to save config");

    println!("Setup complete. Provider: {provider}");
}

fn prompt(label: &str) -> String {
    print!("{label}");
    io::stdout().flush().expect("failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");
    input
}
