use crate::{git, ui};

/// Prints one useful follow-up action after a command completes. Suggestions
/// are intentionally deterministic: they are based on the repository state,
/// not another AI request, so they are instant and free.
pub fn show(args: &[String]) {
    let command = args.first().map(String::as_str);

    // No arguments already displays the full help screen.
    if args.is_empty() {
        return;
    }

    // `please chat` is an interactive experience and already has its own
    // conversational flow. Do not append a tip after the user exits it.
    if command == Some("chat") || command == Some("help") || command == Some("man") {
        return;
    }

    if !git::is_repo() {
        // A natural-language request can work without Git, but a Git-specific
        // follow-up cannot. Keep the hint useful without interrupting setup,
        // configuration, or other repository-independent commands.
        if command.is_none() || !matches!(command, Some("setup" | "config" | "alias" | "update")) {
            ui::tip("run `please init` here when you’re ready to use Git features.");
        }
        return;
    }

    let pending = git::has_pending_changes();
    let upstream = git::upstream_branch();
    let ahead = upstream
        .as_deref()
        .and_then(git::ahead_behind)
        .map(|(ahead, _)| ahead)
        .unwrap_or(0);

    match command {
        Some("init") => ui::tip("run `please status` to see your new repository."),
        Some("status") if pending => ui::tip("run `please commit` to save these changes."),
        Some("commit") if upstream.is_some() => ui::tip("run `please push` to share your commit."),
        Some("push") => ui::tip("run `please status` to confirm everything is up to date."),
        Some("branch") | Some("switch") | Some("sync") => {
            ui::tip("run `please status` to inspect the current branch.")
        }
        Some("status") if ahead > 0 => ui::tip("run `please push` to share your local commits."),
        Some("status") => ui::tip("your working tree is clean — try `please chat` for help."),
        _ if pending => ui::tip("run `please status` or `please commit` for the next step."),
        _ => {}
    }
}
