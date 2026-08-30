use crate::git;

pub fn run() {
    let branch = git::current_branch();
    let entries = git::status_entries();
    let upstream = git::upstream_branch();
    let (ahead, behind) = upstream
        .as_deref()
        .and_then(git::ahead_behind)
        .unwrap_or((0, 0));
    let sync = match (upstream.is_some(), ahead, behind) {
        (false, _, _) => "no upstream configured".to_string(),
        (true, a, 0) if a > 0 => format!("{a} commit(s) ahead, ready to push"),
        (true, _, b) if b > 0 => format!("{b} commit(s) behind remote"),
        _ => "up to date with remote".to_string(),
    };
    let noun = if entries.len() == 1 { "file" } else { "files" };
    println!("Branch {branch}: {} {noun} changed, {sync}.", entries.len());
    println!("Tests have not been run.");
    if !entries.is_empty() {
        println!("\nChanges:");
        for (code, path) in &entries {
            println!("  {:<10} {path}", classify(code));
        }
    }
}

pub(crate) fn classify(code: &str) -> &'static str {
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
