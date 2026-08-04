use crate::git;

pub fn run(args: &[String]) {
    match args.first().map(String::as_str) {
        None => list(),
        Some("delete") => delete(&args[1..]),
        Some(name) => create(name),
    }
}

fn list() {
    let branches = git::list_branches();
    println!("Branches:");
    for (name, is_current) in branches {
        if is_current {
            println!("  * {name} (current)");
        } else {
            println!("    {name}");
        }
    }
}

fn create(name: &str) {
    if git::branch_exists(name) {
        eprintln!("Branch '{name}' already exists. Run `please switch {name}` to switch to it.");
        std::process::exit(1);
    }

    match git::create_and_switch_branch(name) {
        Ok(()) => println!("Created and switched to branch '{name}'."),
        Err(err) => {
            eprintln!("Failed to create branch '{name}': {err}");
            std::process::exit(1);
        }
    }
}

fn delete(args: &[String]) {
    let Some(name) = args.first() else {
        eprintln!("usage: please branch delete <name>");
        std::process::exit(1);
    };

    if !git::branch_exists(name) {
        eprintln!("Branch '{name}' doesn't exist locally.");
        std::process::exit(1);
    }

    if git::current_branch() == *name {
        eprintln!("Can't delete '{name}' — it's the branch you're on. Switch to another branch first.");
        std::process::exit(1);
    }

    if let Err(err) = git::delete_local_branch(name) {
        eprintln!("Failed to delete local branch '{name}': {err}");
        std::process::exit(1);
    }
    println!("Deleted local branch '{name}'.");

    if !git::has_remote_branch(name) {
        return;
    }

    match git::delete_remote_branch(name) {
        Ok(()) => println!("Deleted remote branch 'origin/{name}'."),
        Err(err) => eprintln!("Local branch deleted, but failed to delete remote branch: {err}"),
    }
}
