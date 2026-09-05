const BANNER: &str = include_str!("../assets/please.txt");

pub fn run() {
    println!("\n");
    println!("{BANNER}");
    println!("\nAn AI-native git CLI. You never type raw `git` commands.\n");

    println!("USAGE");
    println!("  please <command> [args]");
    println!("  please \"<what you want to do>\"    run the AI agent on a plain-language request");
    println!("  please chat                       open an interactive AI session\n");

    println!("REPOSITORY");
    print_cmd("init", "initialize a git repo here, and generate please.md");
    print_cmd(
        "github [url]",
        "initialize, commit, and publish this project to GitHub",
    );
    print_cmd(
        "change origin <url>",
        "change the remote origin without pushing",
    );
    print_cmd(
        "change origin and push <url>",
        "change origin and push all branches and tags",
    );
    print_cmd(
        "clone <url>",
        "clone a remote repository (plain `git clone`, no AI involved)",
    );
    println!();

    println!("AI");
    print_cmd(
        "commit",
        "stage your changes, split into commits, AI-written messages",
    );
    print_cmd(
        "push",
        "commit, then push (sets the upstream on first push)",
    );
    print_cmd(
        "setup",
        "configure AI providers and API keys, switch between saved ones",
    );
    print_cmd(
        "review",
        "AI-review the current changes and suggest focused tests",
    );
    print_cmd(
        "run [reset]",
        "figure out how to run this project, then remember it",
    );
    println!();

    println!("BRANCHING & SYNC");
    print_cmd(
        "status",
        "where you stand: branch, ahead/behind, what's changed",
    );
    print_cmd("doctor", "diagnose Git, repository, and AI setup problems");
    print_cmd(
        "branch [name]",
        "list branches, or create and switch to a new one",
    );
    print_cmd(
        "switch <name>",
        "switch branches (offers to create it if it doesn't exist)",
    );
    print_cmd(
        "sync [exactly]",
        "pull the current branch, or force it to match the remote",
    );
    println!();

    println!("UNDO & RECOVERY");
    print_cmd(
        "undo / redo",
        "preview/undo the last commit, or bring it back",
    );
    print_cmd(
        "recover [changes]",
        "recover a recent commit or discarded changes",
    );
    print_cmd(
        "context [--refresh]",
        "inspect or regenerate project context",
    );
    print_cmd(
        "move-commit <branch>",
        "move your last commit onto a new branch",
    );
    print_cmd("discard", "throw away all uncommitted changes");
    print_cmd(
        "restore <path>",
        "bring back a file deleted in a past commit",
    );
    print_cmd("revert", "interactively revert a past commit");
    print_cmd(
        "squash [n | ref]",
        "combine a run of commits into one, AI-written message",
    );
    println!();

    println!("STASH");
    print_cmd("stash", "save everything in the working tree and clear it");
    print_cmd(
        "stash list / pop / drop",
        "see, restore, or delete what's stashed",
    );
    println!();

    println!("CLEANUP");
    print_cmd(
        "branch delete <name>",
        "delete a branch locally and on origin",
    );
    print_cmd(
        "rename <new-name>",
        "rename the current branch, remote included",
    );
    print_cmd("cleanup", "delete local branches already merged");
    print_cmd("start \"purpose\"", "create a purpose-named feature branch");
    println!();

    println!("HISTORY");
    print_cmd("log", "a readable commit graph");
    print_cmd(
        "purge <path>",
        "permanently remove a file or folder from all git history",
    );
    print_cmd("resolve", "explain and resolve merge conflicts with AI");
    print_cmd(
        "split / reorder / combine",
        "organize commits with AI assistance",
    );
    print_cmd(
        "release",
        "draft release notes and a version from recent commits",
    );
    println!();

    println!("SHORTCUTS");
    print_cmd(
        "alias <name>",
        "add a shorter command name for please, like pls or plz",
    );
    print_cmd(
        "alias / alias remove <name>",
        "list your aliases, or remove one",
    );
    println!();

    println!("MAINTENANCE");
    print_cmd("update", "update please itself to the latest release");
    println!();

    println!("Run `please <command>` for any of these, or `please help` to see this again.");
}

fn print_cmd(name: &str, blurb: &str) {
    println!("  {name:<24} {blurb}");
}
