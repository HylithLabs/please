use crate::{git, ui};

/// Initializes and publishes a local project to its GitHub origin. A URL is
/// optional when an origin already exists; it is required for a brand-new
/// directory because Git cannot discover a remote that has not been given to
/// it yet.
pub fn run(args: &[String]) {
    if args.len() > 1 {
        eprintln!("usage: please github [github-origin-url]");
        std::process::exit(1);
    }

    let requested_url = args.first().map(String::as_str);
    let existing_url = git::remote_url("origin");
    let Some(url) = requested_url.or(existing_url.as_deref()) else {
        eprintln!("No origin URL found. Run `please github <github-origin-url>` in a new project.");
        std::process::exit(1);
    };

    if !ui::confirm(&format!(
        "Initialize this project on branch `main`, create the first commit, and push to {url}?"
    )) {
        ui::warn("Skipped GitHub setup.");
        return;
    }

    if !git::is_repo() {
        if let Err(err) = git::init() {
            ui::error(&format!("failed to initialize Git: {err}"));
            std::process::exit(1);
        }
        ui::success("Initialized a Git repository.");
    }

    if let Err(err) = git::set_branch("main") {
        ui::error(&format!("failed to set the branch to main: {err}"));
        std::process::exit(1);
    }

    let remote_result = if git::has_remote("origin") {
        if requested_url.is_some() {
            git::set_remote_url("origin", url)
        } else {
            Ok(())
        }
    } else {
        git::add_remote("origin", url)
    };
    if let Err(err) = remote_result {
        ui::error(&format!("failed to configure origin: {err}"));
        std::process::exit(1);
    }

    if !git::has_commits() {
        git::stage_all();
        if git::staged_files().is_empty() || !git::commit("first commit") {
            ui::error("could not create the first commit; make sure the project contains files.");
            std::process::exit(1);
        }
        ui::success("Created first commit.");
    }

    ui::step("Pushing main to origin");
    if !git::push("main") {
        ui::error("failed to push the project to origin.");
        std::process::exit(1);
    }
    ui::success("Project is now published on GitHub.");
}
