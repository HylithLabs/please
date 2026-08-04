use std::io::{self, Write};

use crate::git;

const HISTORY_SIZE: usize = 20;

pub fn run() {
    if git::has_pending_changes() {
        eprintln!(
            "You have uncommitted changes. Run `please commit` to save them, or `please discard` to throw them away, then try `please revert` again."
        );
        std::process::exit(1);
    }

    let commits = git::recent_commits(HISTORY_SIZE);
    if commits.is_empty() {
        println!("No commits to revert.");
        return;
    }

    println!("Choose a commit to revert:");
    for (i, (hash, subject)) in commits.iter().enumerate() {
        println!("  {}. {hash}  {subject}", i + 1);
    }
    print!("Enter a number or commit hash: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Failed to read input.");
        std::process::exit(1);
    }
    let input = input.trim();

    let reference = match input.parse::<usize>() {
        Ok(n) if n >= 1 && n <= commits.len() => commits[n - 1].0.clone(),
        _ => input.to_string(),
    };

    let Some(commit) = git::resolve_commit(&reference) else {
        eprintln!("'{input}' isn't a valid commit number or hash.");
        std::process::exit(1);
    };

    let label = commits
        .iter()
        .find(|(hash, _)| commit.starts_with(hash.as_str()))
        .map(|(_, subject)| subject.clone())
        .unwrap_or_else(|| commit[..7].to_string());

    match git::revert_commit(&commit) {
        Ok(()) => println!("Reverted \"{label}\" — created a new commit undoing those changes."),
        Err(err) => {
            if git::has_conflicts() {
                eprintln!("Reverting \"{label}\" caused conflicts in:");
                for file in git::conflicted_files() {
                    eprintln!("  {file}");
                }
                eprintln!(
                    "Resolve them and run `please commit`, or run `please discard` to cancel the revert."
                );
            } else {
                eprintln!("Failed to revert: {err}");
            }
            std::process::exit(1);
        }
    }
}
