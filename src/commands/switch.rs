use std::io::{self, Write};

use crate::git;

pub fn run(args: &[String]) {
    let Some(name) = args.first() else {
        eprintln!("usage: please switch <branch>");
        std::process::exit(1);
    };

    if !git::branch_exists(name) {
        print!("Branch '{name}' doesn't exist. Create it and switch? [y/N]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let confirmed = io::stdin().read_line(&mut input).is_ok()
            && matches!(input.trim().to_lowercase().as_str(), "y" | "yes");

        if !confirmed {
            println!("Cancelled.");
            return;
        }

        match git::create_and_switch_branch(name) {
            Ok(()) => println!("Created and switched to branch '{name}'."),
            Err(err) => {
                eprintln!("Failed to create branch '{name}': {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    match git::switch_branch(name) {
        Ok(()) => println!("Switched to branch '{name}'."),
        Err(err) => {
            eprintln!("Failed to switch to '{name}': {err}");
            std::process::exit(1);
        }
    }
}
