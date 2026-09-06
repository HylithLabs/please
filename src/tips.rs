use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

use crate::ui;

/// Catchy one-liners in the spirit of "did you know", pointing at a feature
/// the developer might not have reached for yet. Kept short, one command per
/// tip, phrased to make trying it sound worth it.
const TIPS: &[&str] = &[
    "`please review` gives you an AI read on your changes, and the bugs in them, before you commit.",
    "`please run` figures out how to start any project and remembers it. Tell `please chat` how it really runs to override.",
    "Skip the command names entirely: `please \"undo my last two commits but keep the changes\"`.",
    "`please doctor` checks your Git, your repo, and your AI setup in one pass.",
    "`please squash` folds a messy branch into one clean, AI-written commit.",
    "`please chat` keeps the agent alive across turns for multi-step work.",
    "`please undo` takes back your last commit but leaves the work in your tree.",
    "`please alias plz` (or any name) gives you less to type every day.",
    "`please stash` puts everything aside, tracked and untracked, and hands you a clean tree.",
    "`please start \"add dark mode\"` opens a tidy `feature/add-dark-mode` branch.",
    "`please purge <path>` scrubs a file out of your whole history, not just the latest commit.",
    "`please recover` brings back a commit you thought you lost.",
    "Write your team's conventions once in `.please/instructions.md` and every AI command follows them.",
    "`please resolve` walks you through a merge conflict and explains both sides.",
    "`please context --refresh` regenerates what the AI knows about your project.",
];

/// Roughly one run in this many prints a tip. Small enough to notice, rare
/// enough not to nag.
const ODDS: u64 = 5;

/// Commands with their own flow or output where a trailing tip would just be
/// noise.
const SKIP: &[&str] = &["chat", "help", "man", "update", "setup", "config"];

pub fn show(args: &[String]) {
    let Some(command) = args.first().map(String::as_str) else {
        return;
    };
    if SKIP.contains(&command) {
        return;
    }

    // `RandomState::new()` is seeded from system entropy per call, so an empty
    // hasher's `finish()` is a fresh random number without pulling in an RNG
    // crate just for a dice roll.
    let roll = RandomState::new().build_hasher().finish();
    if !roll.is_multiple_of(ODDS) {
        return;
    }

    ui::tip(TIPS[(roll / ODDS) as usize % TIPS.len()]);
}
