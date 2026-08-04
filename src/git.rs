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

pub fn has_conflicts() -> bool {
    Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false)
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
        .args(["rev-list", "--left-right", "--count", &format!("HEAD...{upstream}")])
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
        .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{name}")])
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
