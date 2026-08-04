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
