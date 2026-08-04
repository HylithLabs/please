use crate::git;

pub fn run() {
    let branch = git::current_branch();
    println!("On branch {branch}");

    match git::upstream_branch() {
        Some(upstream) => match git::ahead_behind(&upstream) {
            Some((0, 0)) => println!("Up to date with {upstream}."),
            Some((ahead, 0)) => println!("{ahead} commit(s) ahead of {upstream}."),
            Some((0, behind)) => println!("{behind} commit(s) behind {upstream}."),
            Some((ahead, behind)) => {
                println!("{ahead} ahead, {behind} behind {upstream}.")
            }
            None => {}
        },
        None => println!("No remote tracking branch set up."),
    }

    println!();

    let entries = git::status_entries();
    if entries.is_empty() {
        println!("Nothing to commit — working tree clean.");
        return;
    }

    println!("Changes:");
    for (code, path) in &entries {
        println!("  {:<10} {path}", classify(code));
    }

    println!();
    println!("Run `please commit` to save these changes.");
}

fn classify(code: &str) -> &'static str {
    if code.starts_with("??") {
        "new file:"
    } else if code.contains('D') {
        "deleted:"
    } else if code.contains('R') {
        "renamed:"
    } else if code.contains('A') {
        "new file:"
    } else if code.contains('M') {
        "modified:"
    } else {
        "changed:"
    }
}
