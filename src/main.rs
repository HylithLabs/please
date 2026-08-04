use std::env;

mod commands;
mod config;
mod context;
mod dispatch;
mod git;
mod gitignore;
mod llm;
mod sensitive;

fn main() {
    let args: Vec<String> = env::args().collect();
    dispatch::route(&args[1..]);
}
