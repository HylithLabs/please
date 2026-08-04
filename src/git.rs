use std::fs;
use std::path::PathBuf;
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

pub fn staged_files() -> Vec<String> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--name-only"])
        .output()
        .expect("failed to run git diff --staged --name-only");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect()
}

pub fn unstage_file(path: &str) {
    let _ = Command::new("git")
        .args(["restore", "--staged", "--", path])
        .status();
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

pub fn is_tracked(path: &str) -> bool {
    Command::new("git")
        .args(["ls-files", "--", path])
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false)
}

pub fn is_ignored(path: &str) -> bool {
    Command::new("git")
        .args(["check-ignore", "-q", path])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn current_branch() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("failed to run git rev-parse --abbrev-ref HEAD");

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Always sets the upstream (`-u`) so later `please sync` calls have a
/// tracking branch to compare against, even on a branch's first push.
pub fn push(branch: &str) -> bool {
    Command::new("git")
        .args(["push", "-u", "origin", branch])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Fetches + merges from the current branch's upstream, like `git pull`.
/// Returns git's informational stdout on success, or the combined
/// stdout+stderr on failure (which may describe a merge conflict).
pub fn pull() -> Result<String, String> {
    // Explicit --no-rebase: newer git refuses to guess a reconcile strategy
    // when the user's global config doesn't set one, so we pin the classic
    // "git pull" merge behavior instead of failing on every divergent branch.
    let output = Command::new("git")
        .args(["pull", "--no-rebase"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(match (stdout.is_empty(), stderr.is_empty()) {
            (true, _) => stderr,
            (false, true) => stdout,
            (false, false) => format!("{stdout}\n{stderr}"),
        })
    }
}

pub fn fetch() -> Result<(), String> {
    let output = Command::new("git")
        .args(["fetch"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn reset_hard(target: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["reset", "--hard", target])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn conflicted_files() -> Vec<String> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()
        .expect("failed to run git diff --diff-filter=U");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn has_conflicts() -> bool {
    !conflicted_files().is_empty()
}

/// Working-tree changes as (XY status code, path) pairs, straight from
/// `git status --porcelain` — staged/unstaged is collapsed away by the
/// caller since `please` never asks a dev to think in those terms.
pub fn status_entries() -> Vec<(String, String)> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("failed to run git status --porcelain");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.len() > 3)
        .map(|line| (line[..2].to_string(), line[3..].to_string()))
        .collect()
}

/// The upstream tracking branch (e.g. `origin/main`), or `None` if this
/// branch isn't tracking anything yet.
pub fn upstream_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

/// (commits ahead, commits behind) HEAD is relative to `upstream`.
pub fn ahead_behind(upstream: &str) -> Option<(usize, usize)> {
    let output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("HEAD...{upstream}"),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut parts = text.split_whitespace();
    let ahead = parts.next()?.parse().ok()?;
    let behind = parts.next()?.parse().ok()?;
    Some((ahead, behind))
}

/// Local branches as (name, is_current) pairs.
pub fn list_branches() -> Vec<(String, bool)> {
    let output = Command::new("git")
        .args(["branch", "--format=%(HEAD)%(refname:short)"])
        .output()
        .expect("failed to run git branch");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let mut chars = line.chars();
            let marker = chars.next().unwrap_or(' ');
            (chars.as_str().to_string(), marker == '*')
        })
        .collect()
}

pub fn branch_exists(name: &str) -> bool {
    Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{name}"),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn create_and_switch_branch(name: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", "-b", name])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn switch_branch(name: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", name])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn run_ok(args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn has_parent_commit() -> bool {
    // `.output()`, not `.status()`: rev-parse's `-q` only silences the error
    // message — on success it still writes the resolved SHA to stdout, which
    // `.status()` would leak straight into our own output.
    Command::new("git")
        .args(["rev-parse", "--verify", "-q", "HEAD~1"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn last_commit_message() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .output()
        .expect("failed to run git log -1");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn reset_soft_head1() -> Result<(), String> {
    run_ok(&["reset", "--soft", "HEAD~1"])
}

pub fn reset_soft(target: &str) -> Result<(), String> {
    run_ok(&["reset", "--soft", target])
}

pub fn head_sha() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("failed to run git rev-parse HEAD");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn commit_parent(commit: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", &format!("{commit}^")])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn undo_marker_path() -> PathBuf {
    PathBuf::from(".git/PLEASE_UNDO")
}

/// Remembers the commit `please undo` just rewound past, so `please redo`
/// can bring it back — a one-slot undo stack, not a full history.
pub fn record_undo(commit: &str) {
    let _ = fs::write(undo_marker_path(), commit);
}

pub fn take_undo_marker() -> Option<String> {
    let sha = fs::read_to_string(undo_marker_path()).ok()?;
    let sha = sha.trim().to_string();
    (!sha.is_empty()).then_some(sha)
}

pub fn clear_undo_marker() {
    let _ = fs::remove_file(undo_marker_path());
}

pub fn reset_hard_head1() -> Result<(), String> {
    run_ok(&["reset", "--hard", "HEAD~1"])
}

pub fn reset_hard_head() -> Result<(), String> {
    run_ok(&["reset", "--hard", "HEAD"])
}

pub fn create_branch_at_head(name: &str) -> Result<(), String> {
    run_ok(&["branch", name])
}

pub fn clean_untracked() -> Result<(), String> {
    run_ok(&["clean", "-fd"])
}

/// Most recent commit that deleted `path`, or `None` if it was never deleted.
pub fn find_deleting_commit(path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["log", "--diff-filter=D", "--pretty=%H", "-1", "--", path])
        .output()
        .ok()?;

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() { None } else { Some(sha) }
}

pub fn restore_file_from_commit(commit: &str, path: &str) -> Result<(), String> {
    run_ok(&["checkout", &format!("{commit}^"), "--", path])
}

pub fn delete_local_branch(name: &str) -> Result<(), String> {
    run_ok(&["branch", "-d", name])
}

pub fn has_remote_branch(name: &str) -> bool {
    Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/remotes/origin/{name}"),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn delete_remote_branch(name: &str) -> Result<(), String> {
    run_ok(&["push", "origin", "--delete", name])
}

pub fn rename_branch(new_name: &str) -> Result<(), String> {
    run_ok(&["branch", "-m", new_name])
}

/// The repo's primary branch: the remote's default if known (only cached
/// locally after a `clone`), else `main`/`master` if either exists locally,
/// else whatever branch we're currently on.
pub fn default_branch() -> String {
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let full = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Some(name) = full.strip_prefix("refs/remotes/origin/") {
            return name.to_string();
        }
    }

    ["main", "master"]
        .into_iter()
        .find(|name| branch_exists(name))
        .map(str::to_string)
        .unwrap_or_else(current_branch)
}

/// Local branches already merged into `base`, excluding `base` itself.
pub fn merged_branches(base: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)", "--merged", base])
        .output()
        .expect("failed to run git branch --merged");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|name| *name != base)
        .map(str::to_string)
        .collect()
}

/// Recent commits as (short hash, subject) pairs, most recent first.
pub fn recent_commits(count: usize) -> Vec<(String, String)> {
    let output = Command::new("git")
        .args(["log", &format!("-{count}"), "--pretty=%h\t%s"])
        .output()
        .expect("failed to run git log");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(hash, subject)| (hash.to_string(), subject.to_string()))
        .collect()
}

/// Resolves any commit-ish (short hash, full hash, ref) to its full SHA,
/// or `None` if it doesn't name a real commit.
pub fn resolve_commit(reference: &str) -> Option<String> {
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{reference}^{{commit}}"),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!sha.is_empty()).then_some(sha)
}

pub fn revert_commit(commit: &str) -> Result<(), String> {
    run_ok(&["revert", "--no-edit", commit])
}

pub fn log_graph() -> bool {
    Command::new("git")
        .args(["log", "--oneline", "--graph", "--decorate", "-20"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Stashes everything in the working tree, tracked and untracked — a
/// developer stashing before a branch switch usually means "everything I've
/// touched," not just what git already knows about.
pub fn stash_push() -> Result<(), String> {
    run_ok(&["stash", "push", "-u"])
}

/// Restores the most recent stash and removes it from the list. On a
/// conflict, git leaves the stash entry in place rather than dropping it —
/// `please discard` cleanly cancels the attempt without losing it.
pub fn stash_pop() -> Result<(), String> {
    run_ok(&["stash", "pop"])
}

pub fn drop_latest_stash() -> Result<(), String> {
    run_ok(&["stash", "drop"])
}

pub fn has_stash() -> bool {
    !stash_list().is_empty()
}

/// Stash entries, most recent first, as their one-line description (the
/// part after `git stash list`'s `stash@{N}: ` prefix).
pub fn stash_list() -> Vec<String> {
    let output = Command::new("git")
        .args(["stash", "list", "--pretty=%gs"])
        .output()
        .expect("failed to run git stash list");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

/// True if `git filter-repo` (git's own recommended replacement for the
/// deprecated `git filter-branch`) is installed on this machine.
pub fn filter_repo_available() -> bool {
    Command::new("git")
        .args(["filter-repo", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Rewrites every branch and tag to remove `path` from the entire history,
/// not just the working tree, via `git filter-repo`. Runs with inherited
/// stdio so its own progress output shows live, the same as `commit`/`push`.
pub fn purge_path_with_filter_repo(path: &str) -> bool {
    Command::new("git")
        .args(["filter-repo", "--path", path, "--invert-paths", "--force"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Same job as `purge_path_with_filter_repo`, using the slower, built-in
/// `git filter-branch` for machines without `git-filter-repo` installed. The
/// path is passed through an env var rather than interpolated into the
/// index-filter shell script, so spaces or shell metacharacters in it can't
/// break, or inject into, the per-commit filter command.
pub fn purge_path_with_filter_branch(path: &str) -> bool {
    Command::new("git")
        .env("FILTER_BRANCH_SQUELCH_WARNING", "1")
        .env("PLEASE_PURGE_PATH", path)
        .args([
            "filter-branch",
            "--force",
            "--index-filter",
            "git rm -r --cached --ignore-unmatch -- \"$PLEASE_PURGE_PATH\"",
            "--prune-empty",
            "--tag-name-filter",
            "cat",
            "--",
            "--all",
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Reclaims the space `filter-branch` frees up: it leaves the pre-rewrite
/// commits reachable from backup refs under `refs/original/`, specifically
/// so a mistake can be undone, so a plain `gc` alone won't touch them.
pub fn cleanup_after_filter_branch() -> Result<(), String> {
    let _ = fs::remove_dir_all(".git/refs/original");
    run_ok(&["reflog", "expire", "--expire=now", "--all"])?;
    run_ok(&["gc", "--prune=now", "--aggressive"])
}

pub fn remote_url(name: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", name])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!url.is_empty()).then_some(url)
}

pub fn has_remote(name: &str) -> bool {
    remote_url(name).is_some()
}

pub fn add_remote(name: &str, url: &str) -> Result<(), String> {
    run_ok(&["remote", "add", name, url])
}

/// True if any commit, on any branch, ever touched `path` — including ones
/// deleted long ago, unlike a plain working-tree check.
pub fn path_ever_tracked(path: &str) -> bool {
    Command::new("git")
        .args(["log", "--all", "--oneline", "--", path])
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false)
}

/// True if `ancestor` is actually in `descendant`'s history — squashing back
/// to a commit that isn't would silently jump the branch pointer somewhere
/// unrelated instead of just combining commits.
pub fn is_ancestor(ancestor: &str, descendant: &str) -> bool {
    Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Subjects of every commit strictly after `base` up to HEAD, oldest first —
/// the commits a squash is about to combine, given as context to the model
/// writing the combined message.
pub fn commit_subjects_since(base: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--reverse", "--pretty=%s", &format!("{base}..HEAD")])
        .output()
        .expect("failed to run git log");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

/// Force-pushes just `branch`, with `--force-with-lease` rather than a plain
/// `--force` — it fails instead of clobbering if someone else pushed to the
/// branch since we last saw it, unlike `force_push_all`'s full-history reset
/// where a plain force is already the deliberate, full-scope intent.
pub fn force_push_with_lease(branch: &str) -> bool {
    Command::new("git")
        .args(["push", "--force-with-lease", "origin", branch])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Force-pushes every local branch and every tag, matching the scope of what
/// `purge_path_with_filter_repo`/`purge_path_with_filter_branch` just
/// rewrote (`--all` refs), so the remote actually loses the purged history
/// too, not just the current branch.
pub fn force_push_all() -> bool {
    let branches = Command::new("git")
        .args(["push", "origin", "--force", "--all"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    let tags = Command::new("git")
        .args(["push", "origin", "--force", "--tags"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    branches && tags
}
