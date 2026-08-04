use crate::git;

pub fn run() {
    let base = git::default_branch();
    let current = git::current_branch();

    let candidates: Vec<String> = git::merged_branches(&base)
        .into_iter()
        .filter(|name| *name != current)
        .collect();

    if candidates.is_empty() {
        println!("No merged branches to clean up.");
        return;
    }

    println!("Deleting branches already merged into '{base}':");
    for name in &candidates {
        match git::delete_local_branch(name) {
            Ok(()) => println!("  deleted: {name}"),
            Err(err) => eprintln!("  failed to delete '{name}': {err}"),
        }
    }
}
