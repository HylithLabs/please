use std::process::Command;

pub fn stage_all() {
    let status = Command::new("git")
        .args(["add", "-A"])
        .status()
        .expect("failed to run git add -A");

    if !status.success() {
        eprintln!("failed to stage changes");
        std::process::exit(1);
    }
}

pub fn diff_staged() -> String {
    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .expect("failed to run git diff --staged");

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn list_tracked_files() -> String {
    let output = Command::new("git")
        .args(["ls-files"])
        .output()
        .expect("failed to run git ls-files");

    if !output.status.success() {
        return String::new();
    }

    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn unstage_all() {
    let status = Command::new("git")
        .args(["restore", "--staged", "."])
        .status()
        .expect("failed to run git restore --staged .");

    if !status.success() {
        eprintln!("failed to unstage changes");
        std::process::exit(1);
    }
}

pub fn stage_files(files: &[String]) -> bool {
    if files.is_empty() {
        return false;
    }

    let mut args = vec!["add".to_string(), "--".to_string()];
    args.extend(files.iter().cloned());

    Command::new("git")
        .args(&args)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn commit(message: &str) -> bool {
    Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn has_pending_changes() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false)
}
