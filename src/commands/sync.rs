use std::io::{self, Write};

use crate::git;

pub fn run(args: &[String]) {
    match args.first().map(String::as_str) {
        None => sync(),
        Some("exactly") => sync_exactly(),
        Some(other) => {
            eprintln!("Unknown option '{other}'. Usage: please sync [exactly]");
            std::process::exit(1);
        }
    }
}

/// Like `git pull`: fetches and merges the current branch's upstream in.
fn sync() {
    let branch = git::current_branch();
    let upstream = require_upstream(&branch);

    println!("Syncing {branch} with {upstream}...");

    match git::pull() {
        Ok(message) if message.is_empty() => println!("Up to date with {upstream}."),
        Ok(message) => println!("{message}"),
        Err(err) => {
            eprintln!("Sync failed: {err}");
            if git::has_conflicts() {
                eprintln!("Resolve the conflicts listed above, then run `please commit`.");
            }
            std::process::exit(1);
        }
    }
}

/// Makes the local branch match its upstream exactly: any local commits or
/// uncommitted changes not present on the remote are discarded. Untracked
/// files are left alone — this only affects what git already tracks.
fn sync_exactly() {
    let branch = git::current_branch();
    let upstream = require_upstream(&branch);

    println!("Fetching latest from {upstream}...");
    if let Err(err) = git::fetch() {
        eprintln!("Fetch failed: {err}");
        std::process::exit(1);
    }

    println!();
    println!("This will make '{branch}' match {upstream} exactly:");
    println!("  - any local commits not on {upstream} will be discarded");
    println!("  - any uncommitted changes will be discarded");
    println!("  (untracked files are left alone)");
    print!("Type 'yes' to continue: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() || input.trim() != "yes" {
        println!("Cancelled.");
        return;
    }

    match git::reset_hard(&upstream) {
        Ok(()) => println!("'{branch}' now matches {upstream} exactly."),
        Err(err) => {
            eprintln!("Failed to reset: {err}");
            std::process::exit(1);
        }
    }
}

fn require_upstream(branch: &str) -> String {
    git::upstream_branch().unwrap_or_else(|| {
        eprintln!(
            "No remote tracking branch set up for '{branch}'. Push it first with `please push`."
        );
        std::process::exit(1);
    })
}
