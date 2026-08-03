use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("push") => cmd_push(),
        _ => {
            eprintln!("usage: please push");
            std::process::exit(1);
        }
    }
}

fn cmd_push() {
    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .expect("failed to run git diff --staged");

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    let diff = String::from_utf8_lossy(&output.stdout);

    if diff.is_empty() {
        println!("No staged changes.");
        return;
    }

    print!("{diff}");
}
