use crate::git;

pub fn run(args: &[String]) {
    let Some(path) = args.first() else {
        eprintln!("usage: please restore <path>");
        std::process::exit(1);
    };

    let Some(commit) = git::find_deleting_commit(path) else {
        eprintln!("Couldn't find a commit that deleted '{path}'.");
        std::process::exit(1);
    };

    match git::restore_file_from_commit(&commit, path) {
        Ok(()) => println!("Restored '{path}' (as it was before commit {}).", &commit[..7]),
        Err(err) => {
            eprintln!("Failed to restore '{path}': {err}");
            std::process::exit(1);
        }
    }
}
