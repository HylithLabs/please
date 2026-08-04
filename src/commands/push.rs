use crate::commands::commit;
use crate::git;
use crate::ui;

pub fn run() {
    commit::run();

    let branch = git::current_branch();

    ui::step(&format!("Pushing to origin/{branch}"));

    if !git::push(&branch) {
        ui::error(&format!("failed to push to origin/{branch}"));
        std::process::exit(1);
    }

    ui::success(&format!("Pushed to origin/{branch}"));
}
