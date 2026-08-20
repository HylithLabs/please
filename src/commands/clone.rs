use crate::git;
use crate::ui;

/// `please clone` is a thin, literal passthrough to `git clone` — no AI, no
/// guardrails, just the remote URL (and any other args) forwarded as-is.
pub fn run(args: &[String]) {
    let Some(url) = args.first() else {
        eprintln!("usage: please clone <remote-url> [directory]");
        std::process::exit(1);
    };

    ui::step(&format!("Cloning {url}"));

    if !git::clone(args) {
        ui::error(&format!("failed to clone {url}"));
        std::process::exit(1);
    }

    ui::success(&format!("Cloned {url}"));
}
