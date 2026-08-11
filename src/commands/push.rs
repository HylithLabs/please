use crate::commands::commit;
use crate::git;
use crate::ui;

pub fn run() {
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
