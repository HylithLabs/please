use std::env;

mod commands;
mod config;
mod git;
mod llm;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("commit") => commands::commit::run(),
        Some("push") => commands::push::run(),
        Some("setup") => commands::setup::run(),
        _ => {
            eprintln!("usage: please <setup|commit|push>");
            std::process::exit(1);
        }
    }
}
