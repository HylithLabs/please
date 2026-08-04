use std::io::{self, Write};

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
        print!("> ");
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

        eprintln!("Thinking...");
        match session.send(input, &cfg, &system_prompt, &tools, agent::execute_tool) {
            Ok(text) => println!("{text}\n"),
            Err(err) => eprintln!("Agent failed: {err}\n"),
        }
    }
}
