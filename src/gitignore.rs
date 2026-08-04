use std::fs;
use std::path::Path;

use crate::git;

const JUNK_DIRS: &[&str] = &[
    "node_modules",
    ".next",
    ".nuxt",
    ".output",
    "dist",
    "build",
    "out",
    "__pycache__",
    ".venv",
    "venv",
    "target",
    ".cache",
    "coverage",
    ".pytest_cache",
];

/// Detects common build/dependency directories (node_modules, .next, dist, ...)
/// that exist in the working tree but aren't tracked or already ignored, and
/// adds them to .gitignore (creating it if missing) before staging runs. A
/// directory git already tracks is left untouched — that's explicit prior intent.
pub fn ensure_junk_ignored() {
    let newly_ignored: Vec<&str> = JUNK_DIRS
        .iter()
        .copied()
        .filter(|dir| Path::new(dir).is_dir() && !git::is_tracked(dir) && !git::is_ignored(dir))
        .collect();

    if newly_ignored.is_empty() {
        return;
    }

    append_to_gitignore(&newly_ignored);

    for dir in &newly_ignored {
        println!("Added '{dir}/' to .gitignore (build/dependency directory, not committed).");
    }
}

fn append_to_gitignore(dirs: &[&str]) {
    let mut contents = fs::read_to_string(".gitignore").unwrap_or_default();

    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }

    for dir in dirs {
        contents.push_str(dir);
        contents.push('\n');
    }

    let _ = fs::write(".gitignore", contents);
}
