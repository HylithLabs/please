use crate::git;

pub fn run() {
    if !git::has_parent_commit() {
        eprintln!("Nothing to undo — this is the first commit in the repo.");
        std::process::exit(1);
    }

    let commit = git::head_sha();
    let message = git::last_commit_message();

    match git::reset_soft_head1() {
        Ok(()) => {
            git::record_undo(&commit);
            println!("Undone: {message}");
            println!("Changes are back in your working tree — `please commit` to try again, or `please redo` to bring it back.");
        }
        Err(err) => {
            eprintln!("Failed to undo: {err}");
            std::process::exit(1);
        }
    }
}
