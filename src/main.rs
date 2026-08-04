use std::env;

mod commands;
mod config;
mod context;
mod git;
mod gitignore;
mod llm;
mod sensitive;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("commit") => commands::commit::run(),
        Some("push") => commands::push::run(),
        Some("setup") => commands::setup::run(),
        Some("status") => commands::status::run(),
        Some("branch") => commands::branch::run(&args[2..]),
        Some("switch") => commands::switch::run(&args[2..]),
        Some("sync") => commands::sync::run(&args[2..]),
        Some("undo") => commands::undo::run(),
        Some("redo") => commands::redo::run(),
        Some("move-commit") => commands::move_commit::run(&args[2..]),
        Some("discard") => commands::discard::run(),
        Some("restore") => commands::restore::run(&args[2..]),
        Some("rename") => commands::rename::run(&args[2..]),
        Some("cleanup") => commands::cleanup::run(),
        Some("log") => commands::log::run(),
        Some("revert") => commands::revert::run(),
        Some("stash") => commands::stash::run(&args[2..]),
        Some("chat") => commands::chat::run(),
        Some("alias") => commands::alias::run(&args[2..]),
        Some("help") | Some("--help") | Some("-h") => commands::help::run(),
        Some(_) => commands::agent::run(&args[1..].join(" ")),
        None => commands::help::run(),
    }
}
