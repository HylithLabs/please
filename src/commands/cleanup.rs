use crate::git;

pub fn run(args: &[String]) {
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

    let preview = args.iter().any(|a| a == "--preview");
    println!(
        "{} branches already merged into '{base}':",
        if preview { "Candidates" } else { "Deleting" }
    );
    for name in &candidates {
        if preview {
            println!("  {name}");
            continue;
        }
        if !crate::ui::confirm(&format!("Delete branch '{name}'?")) {
            println!("  skipped: {name}");
            continue;
        }
        match git::delete_local_branch(name) {
            Ok(()) => println!("  deleted: {name}"),
            Err(err) => eprintln!("  failed to delete '{name}': {err}"),
        }
    }
}
