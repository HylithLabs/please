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
mod test_ureq;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--internal-update-check" {
        update_checker::run_internal_check();
        return;
    }
    dispatch::route(&args[1..]);
    update_checker::check_and_notify();
}
