use std::env;

mod commands;
mod config;
mod context;
mod dispatch;
mod git;
mod gitignore;
mod llm;
mod sensitive;
mod ui;
mod update_checker;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--internal-update-check" {
        update_checker::run_internal_check();
        return;
    }

    if crate::config::is_feedback_enabled() || args.iter().any(|a| a == "--feedback" || a == "-f") {
        dispatch::FEEDBACK.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    args.retain(|a| a != "--feedback" && a != "-f");

    let is_update = args.get(1).map(|s| s.as_str()) == Some("update");

    dispatch::route(&args[1..]);

    if !is_update {
        update_checker::check_and_notify();
    }
}
