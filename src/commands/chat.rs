use std::io::{self, Write};
use owo_colors::OwoColorize;

use crate::commands::agent;
use crate::llm;

/// An interactive, multi-turn version of `please "<prompt>"`: the same
/// tools and system prompt, but the conversation stays alive across
/// messages instead of starting over each time — so a follow-up like "now
/// undo that" or "why did it fail" actually has something to refer to.
pub fn run() {
    let (cfg, system_prompt, tools) = agent::prepare_session();
    let mut session = llm::AgentSession::new();

    println!("please chat — describe what you want, one message at a time.");
    println!("Type 'exit' or press Ctrl+D to leave.\n");

    loop {
        print!("{}", "> ".color(owo_colors::Rgb(88, 166, 255))); // blue prompt
        let _ = io::stdout().flush();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!();
                break;
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("Failed to read input: {err}");
                break;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if matches!(input.to_lowercase().as_str(), "exit" | "quit") {
            break;
        }

        eprintln!("{}", "Thinking...".color(owo_colors::Rgb(110, 118, 129))); // bright black
        match session.send(input, &cfg, &system_prompt, &tools, agent::execute_tool) {
            Ok(text) => {
                crate::ui::print_markdown(&text);
                println!();
            }
            Err(err) => eprintln!("Agent failed: {err}\n"),
        }
    }
}
