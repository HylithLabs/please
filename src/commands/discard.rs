use std::io::{self, Write};

use crate::commands::status::classify;
use crate::git;

pub fn run() {
    let entries = git::status_entries();
    if entries.is_empty() {
        println!("Nothing to discard — working tree is already clean.");
        return;
    }

    println!("This will permanently discard all uncommitted changes:");
    for (code, path) in &entries {
        println!("  {} {path}", classify(code));
    }
    print!("Type 'yes' to continue: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() || input.trim() != "yes" {
        println!("Cancelled.");
        return;
    }

    if let Err(err) = git::reset_hard_head() {
        eprintln!("Failed to discard tracked changes: {err}");
        std::process::exit(1);
    }

    match git::clean_untracked() {
        Ok(()) => println!("Discarded all uncommitted changes."),
        Err(err) => {
            eprintln!(
                "Tracked changes were discarded, but failed to remove untracked files: {err}"
            );
            std::process::exit(1);
        }
    }
}
