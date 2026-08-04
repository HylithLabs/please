use crate::git;

pub fn run(args: &[String]) {
    let Some(name) = args.first() else {
        eprintln!("usage: please switch <branch>");
        std::process::exit(1);
    };

    if !git::branch_exists(name) {
        eprintln!("Branch '{name}' doesn't exist. Run `please branch {name}` to create it.");
        std::process::exit(1);
    }

    match git::switch_branch(name) {
        Ok(()) => println!("Switched to branch '{name}'."),
        Err(err) => {
            eprintln!("Failed to switch to '{name}': {err}");
            std::process::exit(1);
        }
    }
}
