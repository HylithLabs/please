use crate::git;

pub fn run(args: &[String]) {
    match args.first() {
        None => list(),
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
