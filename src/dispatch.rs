use std::sync::atomic::{AtomicBool, Ordering};

use crate::commands;

pub static FEEDBACK: AtomicBool = AtomicBool::new(false);

pub fn wants_feedback() -> bool {
    FEEDBACK.load(Ordering::Relaxed)
}

/// Shape every literal `please` subcommand is normalized to, so exact and
/// fuzzy matching can treat them uniformly regardless of whether the
/// underlying command actually takes arguments.
type CommandFn = fn(&[String]);

/// Every literal `please` subcommand, normalized to `CommandFn`.
const COMMANDS: &[(&str, CommandFn)] = &[
    ("config", commands::config_cmd::run),
    ("init", |_| commands::init::run()),
    ("clone", commands::clone::run),
    ("commit", |_| commands::commit::run()),
    ("push", |_| commands::push::run()),
    ("setup", |_| commands::setup::run()),
    ("status", |_| commands::status::run()),
    ("branch", commands::branch::run),
    ("switch", commands::switch::run),
    ("sync", commands::sync::run),
    ("undo", |_| commands::undo::run()),
    ("redo", |_| commands::redo::run()),
    ("move-commit", commands::move_commit::run),
    ("discard", |_| commands::discard::run()),
    ("restore", commands::restore::run),
    ("rename", commands::rename::run),
    ("cleanup", |_| commands::cleanup::run()),
    ("log", |_| commands::log::run()),
    ("revert", |_| commands::revert::run()),
    ("stash", commands::stash::run),
    ("squash", commands::squash::run),
    ("purge", commands::purge::run),
    ("chat", |_| commands::chat::run()),
    ("alias", commands::alias::run),
    ("man", commands::man::run),
    ("help", |_| commands::help::run()),
    ("update", commands::update::run),
];

/// Small words that show up in a casual sentence but were never meant as an
/// argument, e.g. the "to" in "switch to master". Stripped only when we're
/// about to run a literal subcommand — the AI agent gets the original
/// sentence untouched, since grammar matters there.
const FILLER_WORDS: &[&str] = &["to", "into", "the", "a", "an"];

/// Subcommands that don't need an existing git repo to make sense — either
/// they set one up (`init`, `clone`), or they never touch git at all
/// (`help`, `man`, `setup`, `config`, `alias`, `update`). Every other
/// subcommand, and agent mode, assumes it's running inside a repo.
const REPO_OPTIONAL: &[&str] = &[
    "init", "clone", "help", "man", "setup", "config", "alias", "update",
];

/// Routes `please`'s own argv (everything after the binary name) to a
/// literal subcommand on an exact or close-enough match, or to the AI agent
/// as a plain-language request if nothing lines up.
pub fn route(words: &[String]) {
    let Some(first) = words.first() else {
        commands::help::run();
        return;
    };

    if matches!(first.as_str(), "--help" | "-h") {
        commands::help::run();
        return;
    }

    if let Some((name, run)) = exact_command(first).or_else(|| fuzzy_command(first)) {
        let args = strip_filler(&words[1..]);
        if args.iter().any(|arg| arg == "-h" || arg == "--help") {
            commands::man::run(std::slice::from_ref(first));
            return;
        }
        if !REPO_OPTIONAL.contains(&name) {
            ensure_repo();
        }
        run(&args);
        return;
    }

    ensure_repo();
    commands::agent::run(&words.join(" "));
}

/// Exits with guidance instead of letting a bare git command leak its own
/// raw "fatal: not a git repository" error onto the screen.
fn ensure_repo() {
    if crate::git::is_repo() {
        return;
    }
    crate::ui::error("not a git repository yet.");
    eprintln!(
        "Run `please init` to start one here, or `please clone <url>` to grab an existing one."
    );
    std::process::exit(1);
}

fn exact_command(word: &str) -> Option<(&'static str, CommandFn)> {
    COMMANDS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(word))
        .map(|&(name, run)| (name, run))
}

/// Matches a typo'd command word (single substitution, insertion, deletion,
/// or adjacent-letter swap) to exactly one known command. Anything looser
/// than that risks hijacking a genuine sentence for the AI agent — "clean
/// up my messy code" is two edits from "cleanup" and must NOT become
/// `please cleanup`, so the threshold stays tight rather than generous.
fn fuzzy_command(word: &str) -> Option<(&'static str, CommandFn)> {
    const MAX_DISTANCE: usize = 1;
    let word = word.to_lowercase();

    let mut best: Option<(usize, &'static str, CommandFn)> = None;
    let mut ambiguous = false;

    for &(name, run) in COMMANDS {
        let distance = edit_distance(&word, name);
        if distance > MAX_DISTANCE {
            continue;
        }
        match best {
            None => best = Some((distance, name, run)),
            Some((best_distance, _, _)) if distance < best_distance => {
                best = Some((distance, name, run));
                ambiguous = false;
            }
            Some((best_distance, _, _)) if distance == best_distance => ambiguous = true,
            _ => {}
        }
    }

    if ambiguous {
        None
    } else {
        best.map(|(_, name, run)| (name, run))
    }
}

fn strip_filler(args: &[String]) -> Vec<String> {
    args.iter()
        .filter(|word| !FILLER_WORDS.contains(&word.to_lowercase().as_str()))
        .cloned()
        .collect()
}

/// Optimal string alignment distance: substitutions, insertions, deletions,
/// and adjacent transpositions each cost one edit. Covers the typo shapes
/// that actually happen when typing fast ("swich", "brnach", "staus").
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());

    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate().take(n + 1) {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }

    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut value = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                value = value.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = value;
        }
    }

    d[n][m]
}
