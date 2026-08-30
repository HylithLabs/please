use owo_colors::OwoColorize;
use std::io::{self, Write};

use crate::commands::agent;
use crate::llm;

/// An interactive, multi-turn version of `please "<prompt>"`: the same
/// tools and system prompt, but the conversation stays alive across
/// messages instead of starting over each time — so a follow-up like "now
/// undo that" or "why did it fail" actually has something to refer to.
pub fn run(args: &[String]) {
    if args.iter().any(|a| a == "--last") {
        let mut files: Vec<_> = std::fs::read_dir(".please/history")
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .collect();
        files.sort_by_key(|e| e.file_name());
        if let Some(file) = files.last() {
            if let Ok(text) = std::fs::read_to_string(file.path()) {
                println!("{text}");
            }
        } else {
            println!("No saved conversations yet.");
        }
        return;
    }
    let (cfg, base_prompt, tools) = agent::prepare_session();
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
        // Rebuilt fresh every turn — earlier turns in this same session may
        // have branched, committed, or stashed, so a snapshot taken once at
        // startup would go stale the moment the first tool call runs.
        let system_prompt = format!("{base_prompt}{}", agent::dynamic_state());
        match session.send(input, &cfg, &system_prompt, &tools, agent::execute_tool) {
            Ok(text) => {
                crate::ui::print_markdown(&text);
                if let Err(err) = std::fs::create_dir_all(".please/history") {
                    eprintln!("Could not save conversation history: {err}");
                    println!();
                    continue;
                }
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let safe = input.replace('\n', " ");
                if let Err(err) = std::fs::write(
                    format!(".please/history/{stamp}.md"),
                    format!("# {safe}\n\n{text}\n"),
                ) {
                    eprintln!("Could not save conversation history: {err}");
                }
                println!();
            }
            Err(err) => eprintln!("Agent failed: {err}\n"),
        }
    }
}
