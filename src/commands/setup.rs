use crate::config::{self, Config, ProviderInfo};
use crate::llm;
use std::io::{self, Write};

/// (internal provider id, display name, one-line description)
const PROVIDERS: [(&str, &str, &str); 4] = [
    ("anthropic", "Anthropic", "Claude"),
    ("google", "Google", "Gemini"),
    ("openai", "ChatGPT", "OpenAI's GPT models"),
    ("other", "Other provider", "not wired up yet — saves, but won't run anything"),
];

pub fn run() {
    let saved = config::list();

    if saved.is_empty() {
        println!("No providers set up yet — let's add one.\n");
        add_or_update_provider();
        return;
    }

    print_saved(&saved);

    println!("\nWhat would you like to do?");
    println!("  1) Add or update a provider");
    println!("  2) Switch the active provider");
    println!("  3) Remove a saved provider");
    println!("  4) Nothing — just checking");

    match prompt("Choose (1-4): ").as_str() {
        "1" => add_or_update_provider(),
        "2" => switch_active(&saved),
        "3" => remove_provider(&saved),
        _ => println!("Nothing changed."),
    }
}

fn print_saved(saved: &[ProviderInfo]) {
    println!("Providers you've set up:");
    for info in saved {
        let marker = if info.active { "*" } else { " " };
        println!(
            "  {marker} {} (model: {}, key ending in {})",
            display_name(&info.provider),
            info.model.as_deref().unwrap_or("auto-selected"),
            mask_key(&info.api_key),
        );
    }
    println!("  (* = active — this is what `please` uses right now)");
}

fn add_or_update_provider() {
    println!("Who is your model provider?");
    for (i, (_, name, blurb)) in PROVIDERS.iter().enumerate() {
        println!("  {}) {name} — {blurb}", i + 1);
    }

    let provider = select_provider();
    let api_key = collect_and_validate_key(&provider);

    println!("Selecting the lowest-cost model for {}...", display_name(&provider));
    let model = llm::select_model(&provider, &api_key);
    if let Some(model) = &model {
        println!("Selected model: {model}");
    }

    config::save(&Config { provider: provider.clone(), api_key, model }).expect("failed to save config");

    println!(
        "\nSetup complete — please will use {} for AI features. Run `please commit` to try it out.",
        display_name(&provider)
    );
}

fn switch_active(saved: &[ProviderInfo]) {
    let inactive: Vec<&ProviderInfo> = saved.iter().filter(|info| !info.active).collect();
    if inactive.is_empty() {
        println!("Only one provider is set up — add another first (option 1) before switching.");
        return;
    }

    println!("\nSwitch to which provider?");
    for (i, info) in inactive.iter().enumerate() {
        println!("  {}) {}", i + 1, display_name(&info.provider));
    }

    let Some(choice) = pick(&prompt("Choose a number: "), inactive.len()) else {
        eprintln!("Not a valid choice — nothing changed.");
        return;
    };

    let provider = &inactive[choice].provider;
    match config::set_active(provider) {
        Ok(()) => println!("Switched — please now uses {} for AI features.", display_name(provider)),
        Err(err) => eprintln!("Couldn't switch: {err}"),
    }
}

fn remove_provider(saved: &[ProviderInfo]) {
    println!("\nRemove which provider's saved key?");
    for (i, info) in saved.iter().enumerate() {
        println!("  {}) {}", i + 1, display_name(&info.provider));
    }

    let Some(choice) = pick(&prompt("Choose a number: "), saved.len()) else {
        eprintln!("Not a valid choice — nothing changed.");
        return;
    };

    let target = &saved[choice];
    if !confirm(&format!("Remove the saved {} key? [y/N]: ", display_name(&target.provider)), false) {
        println!("Cancelled.");
        return;
    }

    if let Err(err) = config::remove(&target.provider) {
        eprintln!("Couldn't remove it: {err}");
        return;
    }
    println!("Removed {}.", display_name(&target.provider));

    if !target.active {
        return;
    }

    let remaining = config::list();
    if remaining.is_empty() {
        println!("No providers left — run `please setup` to add one.");
    } else {
        println!("That was your active provider — pick a new one:");
        switch_active(&remaining);
    }
}

/// Parses a 1-based menu choice against `len` options; `None` on anything
/// that doesn't land in range.
fn pick(input: &str, len: usize) -> Option<usize> {
    input.parse::<usize>().ok().filter(|n| (1..=len).contains(n)).map(|n| n - 1)
}

fn select_provider() -> String {
    loop {
        let choice = prompt("Select a provider (1-4): ");
        let index: usize = match choice.parse() {
            Ok(n) if (1..=PROVIDERS.len()).contains(&n) => n - 1,
            _ => {
                eprintln!("Please enter a number from 1 to {}.", PROVIDERS.len());
                continue;
            }
        };

        let (id, _, _) = PROVIDERS[index];
        if id != "other" {
            return id.to_string();
        }

        let name = prompt("Enter the provider name: ");
        if name.is_empty() {
            eprintln!("Provider name can't be empty.");
            continue;
        }
        println!(
            "Note: only Anthropic, Google, and OpenAI are wired up right now — '{name}' will \
             save, but AI commands will error until it's supported."
        );
        return name;
    }
}

fn collect_and_validate_key(provider: &str) -> String {
    loop {
        let api_key = prompt("Paste your API key: ");
        if api_key.is_empty() {
            eprintln!("API key can't be empty.");
            continue;
        }

        match llm::validate_key(provider, &api_key) {
            None => return api_key, // no cheap way to validate this provider — trust it
            Some(Ok(())) => {
                println!("Key looks good.");
                return api_key;
            }
            Some(Err(err)) => {
                eprintln!("That key didn't work: {err}");
                if !confirm("Try a different key? [Y/n]: ", true) {
                    eprintln!("Keeping it anyway — you can re-run `please setup` any time.");
                    return api_key;
                }
            }
        }
    }
}

fn display_name(provider: &str) -> String {
    PROVIDERS
        .iter()
        .find(|(id, _, _)| *id == provider)
        .map(|(_, name, _)| name.to_string())
        .unwrap_or_else(|| provider.to_string())
}

/// Shows just enough of a stored key to confirm "yes, that's the one I set"
/// without ever displaying the whole secret.
fn mask_key(key: &str) -> String {
    let tail: String = key.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    if key.len() <= 4 { "****".to_string() } else { format!("...{tail}") }
}

/// Prompts a yes/no question. `default_yes` decides what an empty answer
/// (just pressing enter) means.
fn confirm(label: &str, default_yes: bool) -> bool {
    match read_line(label) {
        Some(answer) => match answer.to_lowercase().as_str() {
            "" => default_yes,
            answer => matches!(answer, "y" | "yes"),
        },
        None => false,
    }
}

fn prompt(label: &str) -> String {
    read_line(label).unwrap_or_else(|| {
        eprintln!("No input received. Exiting.");
        std::process::exit(1);
    })
}

/// Reads one line of input, or `None` on EOF (closed stdin, or the
/// terminal's own EOF key) — read_line's `Ok(0)` is EOF, not an error, so a
/// bare `.expect(...)` on the Result alone would loop forever asking for
/// input that will never come.
fn read_line(label: &str) -> Option<String> {
    print!("{label}");
    io::stdout().flush().expect("failed to flush stdout");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(input.trim().to_string()),
    }
}
