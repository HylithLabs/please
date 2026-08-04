use crate::git;

pub fn run() {
    let Some(commit) = git::take_undo_marker() else {
        eprintln!("Nothing to redo.");
        std::process::exit(1);
    };

    let parent_matches = git::commit_parent(&commit).as_deref() == Some(git::head_sha().as_str());

    if !parent_matches {
        git::clear_undo_marker();
        eprintln!("Can't redo — history has moved on since the last undo.");
        std::process::exit(1);
    }

    match git::reset_soft(&commit) {
        Ok(()) => {
            git::clear_undo_marker();
            println!("Redone: {}", git::last_commit_message());
        }
        Err(err) => {
            eprintln!("Failed to redo: {err}");
            std::process::exit(1);
        }
    }
}
