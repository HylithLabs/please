use crate::commands::{commit, doctor};
use crate::{config, git, ui};

pub fn run() {
    if push_needs_doctor()
        && ui::confirm(
            "You should run `please doctor` first. Would you like to run it instead of pushing?",
        )
    {
        doctor::run(&[]);
        return;
    }

    commit::run();

    let branch = git::current_branch();

    ui::step(&format!("Pushing to origin/{branch}"));

    if crate::dispatch::wants_feedback() {
        let msg = format!("Push to origin/{}?", branch);
        if !ui::confirm(&msg) {
            ui::warn("Skipped push.");
            return;
        }
    }

    if !git::push(&branch) {
        ui::error(&format!("failed to push to origin/{branch}"));
        std::process::exit(1);
    }

    ui::success(&format!("Pushed to origin/{branch}"));
}

/// Detects common conditions that make a push likely to fail or require
/// manual intervention. The user can inspect them with the read-only doctor,
/// or decline and let the normal push flow continue.
fn push_needs_doctor() -> bool {
    git::current_branch() == "HEAD"
        || git::remote_names().is_empty()
        || git::has_conflicts()
        || config::load().is_none()
}
