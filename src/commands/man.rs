pub fn run(args: &[String]) {
    let cmd_name = args.first().map(|s| s.as_str()).unwrap_or("");

    if cmd_name.is_empty() {
        println!("What manual page do you want?");
        println!("For example, try 'please man commit' or 'please man sync'.");
        return;
    }

    let manual = get_manual(cmd_name);
    if let Some(text) = manual {
        println!("\n{}\n", text);
    } else {
        println!("No manual entry for '{}'", cmd_name);
        println!("Run 'please help' for a list of commands.");
    }
}

fn get_manual(cmd: &str) -> Option<&'static str> {
    match cmd {
        "commit" => Some("NAME\n    please commit - stage your changes, split into commits, and use AI-written messages\n\nDESCRIPTION\n    Stages your changes, sends the diff to the AI, and lets it split the work into one or more logically coherent commits — each with its own files and message. You never run `git add` or write a commit message yourself.\n\n    Guardrails run automatically before staging:\n    - Sensitive files (.env, credentials, private keys, etc.) are skipped unless you've already staged them yourself.\n    - Build/dependency directories (node_modules, dist, target, .venv, etc.) that aren't already tracked or ignored are added to .gitignore automatically."),
        "push" => Some("NAME\n    please push - commit and push to remote\n\nDESCRIPTION\n    Runs `please commit`, then pushes the current branch to `origin` (setting the upstream on first push)."),
        "status" => Some("NAME\n    please status - show plain-language working tree status\n\nDESCRIPTION\n    A plain-language view of where you stand: current branch, how far ahead/behind the remote you are, and what's changed — no staged/unstaged jargon."),
        "branch" => Some("NAME\n    please branch - list, create, or switch branches\n\nSYNOPSIS\n    please branch\n    please branch <name>\n\nDESCRIPTION\n    No name: lists local branches. With a name: creates and switches to a new branch."),
        "switch" => Some("NAME\n    please switch - switch to an existing or new branch\n\nSYNOPSIS\n    please switch <name>\n\nDESCRIPTION\n    Switches to an existing branch. If the branch doesn't exist, offers to create and switch to it."),
        "sync" => Some("NAME\n    please sync - fetch and merge from upstream\n\nSYNOPSIS\n    please sync\n    please sync exactly\n\nDESCRIPTION\n    Fetches and merges the current branch's upstream in (like `git pull`). Reports \"up to date\" or the merge result; on a real conflict, shows git's conflict output and points you to `please commit` once you've resolved it.\n\n    `please sync exactly`:\n    Makes the local branch match its remote exactly — discards local commits and uncommitted changes not on the remote. Destructive, so it requires typing `yes` to proceed."),
        "undo" => Some("NAME\n    please undo - undo the last commit\n\nDESCRIPTION\n    Undoes the last commit — replaces `git reset --soft HEAD~1` — leaving its changes back in your working tree so you can fix them and try again."),
        "redo" => Some("NAME\n    please redo - redo the undone commit\n\nDESCRIPTION\n    Brings back the undone commit, as long as history hasn't moved on since (a new commit invalidates it)."),
        "move-commit" => Some("NAME\n    please move-commit - move your last commit onto a new branch\n\nSYNOPSIS\n    please move-commit <branch>\n\nDESCRIPTION\n    Fixes \"committed to the wrong branch\": moves your last commit onto a new branch and switches you to it. Refuses if you have uncommitted changes."),
        "discard" => Some("NAME\n    please discard - throw away all uncommitted changes\n\nDESCRIPTION\n    Throws away all uncommitted changes, tracked and untracked. Shows exactly what will be lost and requires typing `yes` to proceed."),
        "restore" => Some("NAME\n    please restore - bring back a deleted file\n\nSYNOPSIS\n    please restore <path>\n\nDESCRIPTION\n    Brings back a file that was deleted in a past commit."),
        "rename" => Some("NAME\n    please rename - rename the current branch\n\nSYNOPSIS\n    please rename <new-name>\n\nDESCRIPTION\n    Renames the current branch, including on the remote if it's been pushed."),
        "cleanup" => Some("NAME\n    please cleanup - delete merged branches\n\nDESCRIPTION\n    Deletes local branches already merged into the repo's main branch."),
        "log" => Some("NAME\n    please log - show a readable commit graph\n\nDESCRIPTION\n    A readable commit graph — replaces remembering `git log --oneline --graph --decorate`."),
        "revert" => Some("NAME\n    please revert - interactively revert a past commit\n\nDESCRIPTION\n    Interactive, no AI involved. Lists your recent commits; you pick one, and it's reverted. Won't run into a wall of uncommitted changes either: if your tree is dirty it tells you up front to `please commit` or `please discard` first."),
        "stash" => Some("NAME\n    please stash - save or restore uncommitted changes\n\nSYNOPSIS\n    please stash\n    please stash list\n    please stash pop\n    please stash drop\n\nDESCRIPTION\n    `please stash` saves everything in the working tree, tracked and untracked, and clears it. `please stash list` shows what's saved. `please stash pop` restores the most recent one. `please stash drop` deletes the most recent one."),
        "squash" => Some("NAME\n    please squash - combine commits and rewrite history\n\nSYNOPSIS\n    please squash\n    please squash <n>\n    please squash <ref>\n\nDESCRIPTION\n    Combines a run of commits into one, AI-written message and all. No argument squashes everything ahead of the branch's upstream. A number squashes the last <n> commits; a branch or commit name squashes back to it directly. Requires typing `yes` first."),
        "purge" => Some("NAME\n    please purge - permanently remove a file or folder from git history\n\nSYNOPSIS\n    please purge <path>\n\nDESCRIPTION\n    Permanently removes a file or folder from your entire git history, not just the working tree. Uses `git filter-repo` if it's installed, falling back to the built-in `git filter-branch`. Rewrites commit hashes for the file's entire history. Requires typing `yes` first."),
        "chat" => Some("NAME\n    please chat - open an interactive AI session\n\nDESCRIPTION\n    The same agent, kept alive across multiple messages instead of one and done. Use when you want to go back and forth — ask a follow-up, correct something, or build up a multi-step task piece by piece."),
        "alias" => Some("NAME\n    please alias - manage aliases for the please CLI\n\nSYNOPSIS\n    please alias <name>\n    please alias\n    please alias remove <name>\n\nDESCRIPTION\n    Give `please` a shorter name, like `pls` or `plz`. Creates a real symlink next to the `please` binary itself."),
        "setup" => Some("NAME\n    please setup - configure AI providers and API keys\n\nDESCRIPTION\n    Configure AI providers and API keys, switch between saved ones, and change the model used."),
        "update" => Some("NAME\n    please update - update please itself\n\nDESCRIPTION\n    Updates `please` itself to the latest release in place, no reinstalling by hand."),
        _ => None,
    }
}
