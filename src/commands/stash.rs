use std::io::{self, Write};

use crate::git;

pub fn run(args: &[String]) {
    match args.first().map(String::as_str) {
        None => push(),
        Some("list") => list(),
        Some("pop") => pop(),
        Some("drop") => drop_latest(),
        Some(other) => {
            eprintln!("Unknown option '{other}'. Usage: please stash [list|pop|drop]");
            std::process::exit(1);
        }
    }
}

fn push() {
    if !git::has_pending_changes() {
        println!("Nothing to stash, working tree is clean.");
        return;
    }

    match git::stash_push() {
        Ok(()) => println!("Stashed your changes. Run `please stash pop` to bring them back."),
        Err(err) => {
            eprintln!("Failed to stash: {err}");
            std::process::exit(1);
        }
    }
}

fn list() {
    let stashes = git::stash_list();
    if stashes.is_empty() {
        println!("No stashed changes.");
        return;
    }

    println!("Stashed changes, most recent first:");
    for (i, message) in stashes.iter().enumerate() {
        println!("  {i}: {message}");
    }
}

fn pop() {
    if !git::has_stash() {
        println!("No stashed changes to restore.");
        return;
    }

    match git::stash_pop() {
        Ok(()) => println!("Restored your stashed changes."),
        Err(err) => {
            if git::has_conflicts() {
                eprintln!("Restoring the stash caused conflicts in:");
                for file in git::conflicted_files() {
                    eprintln!("  {file}");
                }
                eprintln!(
                    "Resolve them and run `please commit`, or run `please discard` to cancel \
                     the restore and keep the stash for later."
                );
            } else {
                eprintln!("Failed to restore stash: {err}");
            }
            std::process::exit(1);
        }
    }
}

fn drop_latest() {
    let stashes = git::stash_list();
    let Some(latest) = stashes.first() else {
        println!("No stashed changes to drop.");
        return;
    };

    println!("This will permanently delete the most recent stash:");
    println!("  {latest}");
    print!("Type 'yes' to continue: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() || input.trim() != "yes" {
        println!("Cancelled.");
        return;
    }

    match git::drop_latest_stash() {
        Ok(()) => println!("Dropped."),
        Err(err) => {
            eprintln!("Failed to drop stash: {err}");
            std::process::exit(1);
        }
    }
}
