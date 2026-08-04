use std::io::{self, Write};

use crate::git;
use crate::ui;

pub fn run(args: &[String]) {
    let Some(path) = args.first() else {
        eprintln!("usage: please purge <path>");
        eprintln!(
            "Removes a file or folder from your entire git history, not just the working tree."
        );
        std::process::exit(1);
    };

    if git::has_pending_changes() {
        ui::error(
            "you have uncommitted changes. Run `please commit` or `please discard` first, \
             purging history needs a clean working tree.",
        );
        std::process::exit(1);
    }

    if !git::path_ever_tracked(path) {
        println!("'{path}' doesn't show up in any commit. Nothing to purge.");
        return;
    }

    let origin = git::remote_url("origin");

    println!("This rewrites your ENTIRE git history to remove '{path}':");
    println!("  - every commit that ever touched it is rewritten, changing its hash");
    println!("  - every commit that comes after it changes hash too");
    println!("  - it can't be undone once the rewritten history is pushed");
    if let Some(url) = &origin {
        println!("  - it will also be force-pushed to {url}, right after the rewrite");
        println!(
            "  - every collaborator will need to re-clone, or `please sync exactly`, afterward"
        );
    }
    print!("Type 'yes' to continue: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() || input.trim() != "yes" {
        println!("Cancelled.");
        return;
    }

    let use_filter_repo = git::filter_repo_available();
    ui::step(&format!("Rewriting history to remove '{path}'"));

    let rewrite_ok = if use_filter_repo {
        git::purge_path_with_filter_repo(path)
    } else {
        println!(
            "(git-filter-repo isn't installed, falling back to the slower, built-in git \
             filter-branch. Installing git-filter-repo makes this faster and safer next time.)"
        );
        git::purge_path_with_filter_branch(path)
    };

    if !rewrite_ok {
        ui::error(&format!(
            "failed to rewrite history for '{path}'. See git's output above."
        ));
        std::process::exit(1);
    }

    if !use_filter_repo {
        ui::step("Cleaning up unreachable objects");
        if let Err(err) = git::cleanup_after_filter_branch() {
            ui::warn(&format!(
                "history was rewritten, but cleanup failed ({err}). Run `git gc --prune=now` \
                 yourself to fully reclaim the space."
            ));
        }
    }

    // filter-repo drops the origin remote as a safety measure; put it back so
    // the rest of `please` (push, sync, ...) keeps working afterward.
    if let Some(url) = &origin
        && !git::has_remote("origin")
    {
        let _ = git::add_remote("origin", url);
    }

    ui::success(&format!("'{path}' is gone from local history."));

    if origin.is_none() {
        println!("No 'origin' remote is set up, so there's nothing to push.");
        return;
    }

    ui::step("Force-pushing rewritten history to origin");
    if git::force_push_all() {
        ui::success("Pushed the rewritten history to origin.");
        println!(
            "Tell every collaborator to re-clone, or run `please sync exactly`, before doing anything else."
        );
    } else {
        ui::error(
            "force-push failed. See git's output above. The rewrite itself already succeeded locally.",
        );
        std::process::exit(1);
    }
}
