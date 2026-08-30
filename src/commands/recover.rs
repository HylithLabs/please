use crate::git;

pub fn run(args: &[String]) {
    if args.first().map(String::as_str) == Some("changes") {
        match git::stash_pop() {
            Ok(()) => println!("Recovered the most recently discarded changes."),
            Err(err) => {
                eprintln!("Could not recover changes: {err}");
                std::process::exit(1);
            }
        }
        return;
    }
    let entries = git::reflog();
    if entries.is_empty() {
        println!("No recoverable reflog entries found.");
        return;
    }
    println!("Recent recoverable history:");
    for (i, (ref_name, message)) in entries.iter().take(15).enumerate() {
        println!("  {}. {ref_name} — {message}", i + 1);
    }
    let choice = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    let Some((ref_name, _)) = entries.get(choice.saturating_sub(1)) else {
        eprintln!("Choose a number from the list.");
        std::process::exit(1);
    };
    let sha = ref_name.split_whitespace().next().unwrap_or("");
    let branch = format!("please-recovered-{sha}");
    if let Err(err) = git::create_branch_at(&branch, sha) {
        eprintln!("Could not create a recovery branch '{branch}': {err}");
        std::process::exit(1);
    }
    println!("Recovered {sha} on branch '{branch}'. Your current branch was not changed.");
}
