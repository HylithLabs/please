use crate::{config, git, llm};

/// Read-only repository and configuration health check. Every suggested
/// action is safe to run manually; the doctor never rewrites history or files.
pub fn run(args: &[String]) {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!(
            "usage: please doctor\n\nDiagnose Git, repository, and AI provider setup problems."
        );
        return;
    }

    println!("Please doctor\n");
    let mut problems = 0;

    match git::git_version() {
        Some((major, minor, patch, text)) => {
            println!("✓ Git: {text}");
            if (major, minor, patch) < (2, 20, 0) {
                println!("  ! Git is outdated; upgrade to Git 2.20 or newer.");
                problems += 1;
            }
        }
        None => {
            println!("✗ Git: not available");
            problems += 1;
        }
    }

    if !git::is_repo() {
        println!("✗ Repository: not inside a Git working tree");
        println!("  Fix: run `please init` or move into a repository.");
        problems += 1;
    } else {
        println!("✓ Repository: Git working tree found");
        let branch = git::current_branch();
        if branch == "HEAD" {
            println!("! HEAD: detached");
            println!("  Fix: create a branch with `please branch <name>` before making commits.");
            problems += 1;
        } else {
            println!("✓ Branch: {branch}");
        }

        let remotes = git::remote_names();
        if remotes.is_empty() {
            println!("! Remotes: none configured");
            println!("  Fix: add one with `git remote add origin <url>`.");
            problems += 1;
        } else {
            println!("✓ Remotes: {}", remotes.join(", "));
        }

        let conflicts = git::conflicted_files();
        if conflicts.is_empty() {
            println!("✓ Conflicts: none");
        } else {
            println!("✗ Conflicts: {} file(s)", conflicts.len());
            println!(
                "  Fix: resolve {}, then commit or abort the merge.",
                conflicts.join(", ")
            );
            problems += 1;
        }

        let changes = git::status_entries();
        if changes.is_empty() {
            println!("✓ Working tree: clean");
        } else {
            println!("! Working tree: {} uncommitted change(s)", changes.len());
            println!("  Fix: run `please commit` or `please stash` when ready.");
            problems += 1;
        }
    }

    match config::load() {
        None => {
            println!("✗ AI provider: not configured");
            println!("  Fix: run `please setup`.");
            problems += 1;
        }
        Some(cfg) => match cfg.provider.as_str() {
            "google" | "anthropic" | "openai" => {
                match llm::validate_key(&cfg.provider, &cfg.api_key) {
                    Some(Ok(())) => {
                        println!("✓ AI provider: {} configured and reachable", cfg.provider)
                    }
                    Some(Err(err)) => {
                        println!("✗ AI provider: {err}");
                        println!("  Fix: run `please setup` to update the key.");
                        problems += 1;
                    }
                    None => println!(
                        "! AI provider: {} configured (key validation unavailable)",
                        cfg.provider
                    ),
                }
            }
            other => {
                println!("✗ AI provider: '{other}' is not supported");
                println!("  Fix: run `please setup` and choose Google, Anthropic, or OpenAI.");
                problems += 1;
            }
        },
    }

    println!();
    if problems == 0 {
        println!("All checks passed.");
    } else {
        println!("Found {problems} item(s) to review. No changes were made.");
    }
}
