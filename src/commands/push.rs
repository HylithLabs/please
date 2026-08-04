use crate::commands::commit;
use crate::git;

pub fn run() {
    commit::run();

    let branch = git::current_branch();

    if !git::push(&branch) {
        eprintln!("Failed to push to origin/{branch}");
        std::process::exit(1);
    }

    println!("Pushed to origin/{branch}");
}
