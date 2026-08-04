const BANNER: &str = include_str!("../assets/please.txt");

pub fn run() {
    println!("\n");
    println!("{BANNER}");
    println!("\nAn AI-native git CLI. You never type raw `git` commands.\n");

    println!("USAGE");
    println!("  please <command> [args]");
    println!("  please \"<what you want to do>\"    run the AI agent on a plain-language request");
    println!("  please chat                       open an interactive AI session\n");

    println!("AI");
    print_cmd("commit", "stage your changes, split into commits, AI-written messages");
    print_cmd("push", "commit, then push (sets the upstream on first push)");
    print_cmd("setup", "configure AI providers and API keys, switch between saved ones");
    println!();

    println!("BRANCHING & SYNC");
    print_cmd("status", "where you stand: branch, ahead/behind, what's changed");
    print_cmd("branch [name]", "list branches, or create and switch to a new one");
    print_cmd("switch <name>", "switch branches (offers to create it if it doesn't exist)");
    print_cmd("sync [exactly]", "pull the current branch, or force it to match the remote");
    println!();

    println!("UNDO & RECOVERY");
    print_cmd("undo / redo", "undo the last commit, or bring it back");
    print_cmd("move-commit <branch>", "move your last commit onto a new branch");
    print_cmd("discard", "throw away all uncommitted changes");
    print_cmd("restore <path>", "bring back a file deleted in a past commit");
    print_cmd("revert", "interactively revert a past commit");
    println!();

    println!("STASH");
    print_cmd("stash", "save everything in the working tree and clear it");
    print_cmd("stash list / pop / drop", "see, restore, or delete what's stashed");
    println!();

    println!("CLEANUP");
    print_cmd("branch delete <name>", "delete a branch locally and on origin");
    print_cmd("rename <new-name>", "rename the current branch, remote included");
    print_cmd("cleanup", "delete local branches already merged");
    println!();

    println!("HISTORY");
    print_cmd("log", "a readable commit graph");
    println!();

    println!("SHORTCUTS");
    print_cmd("alias <name>", "add a shorter command name for please, like pls or plz");
    print_cmd("alias / alias remove <name>", "list your aliases, or remove one");
    println!();

    println!("Run `please <command>` for any of these, or `please help` to see this again.");
}

fn print_cmd(name: &str, blurb: &str) {
    println!("  {name:<24} {blurb}");
}
