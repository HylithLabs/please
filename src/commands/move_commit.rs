use crate::git;

pub fn run(args: &[String]) {
    let Some(name) = args.first() else {
        eprintln!("usage: please move-commit <new-branch>");
        std::process::exit(1);
    };

    if !git::has_parent_commit() {
        eprintln!("Nothing to move — this is the first commit in the repo.");
        std::process::exit(1);
    }

    if git::branch_exists(name) {
        eprintln!("Branch '{name}' already exists. Choose a different name.");
        std::process::exit(1);
    }

    if git::has_pending_changes() {
        eprintln!("You have uncommitted changes — commit or discard them first, then move the commit.");
        std::process::exit(1);
    }

    let branch = git::current_branch();
    let message = git::last_commit_message();

    if let Err(err) = git::create_branch_at_head(name) {
        eprintln!("Failed to create branch '{name}': {err}");
        std::process::exit(1);
    }

    if let Err(err) = git::reset_hard_head1() {
        eprintln!("Failed to remove the commit from '{branch}': {err}");
        std::process::exit(1);
    }

    if let Err(err) = git::switch_branch(name) {
        eprintln!("Commit moved, but failed to switch to '{name}': {err}");
        std::process::exit(1);
    }

    println!("Moved commit \"{message}\" from '{branch}' to '{name}', and switched to '{name}'.");
}
