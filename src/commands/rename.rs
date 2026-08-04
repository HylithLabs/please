use crate::git;

pub fn run(args: &[String]) {
    let Some(new_name) = args.first() else {
        eprintln!("usage: please rename <new-name>");
        std::process::exit(1);
    };

    let old_name = git::current_branch();

    if old_name == *new_name {
        eprintln!("Already named '{new_name}'.");
        std::process::exit(1);
    }

    if git::branch_exists(new_name) {
        eprintln!("Branch '{new_name}' already exists.");
        std::process::exit(1);
    }

    let had_remote = git::has_remote_branch(&old_name);

    if let Err(err) = git::rename_branch(new_name) {
        eprintln!("Failed to rename branch: {err}");
        std::process::exit(1);
    }
    println!("Renamed '{old_name}' to '{new_name}'.");

    if !had_remote {
        return;
    }

    if !git::push(new_name) {
        eprintln!("Renamed locally, but failed to push '{new_name}' to origin.");
        std::process::exit(1);
    }
    println!("Pushed '{new_name}' to origin.");

    match git::delete_remote_branch(&old_name) {
        Ok(()) => println!("Removed old remote branch 'origin/{old_name}'."),
        Err(err) => eprintln!(
            "Pushed '{new_name}', but failed to remove old remote branch '{old_name}': {err}"
        ),
    }
}
