use crate::config;
use crate::ui;
use std::io::{self, Write};

fn prompt(label: &str) -> String {
    print!("{label}");
    io::stdout().flush().expect("failed to flush stdout");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) | Err(_) => {
            eprintln!("No input received. Exiting.");
            std::process::exit(1);
        }
        Ok(_) => input.trim().to_string(),
    }
}

fn interactive_config() {
    println!("Please Configuration\n");
    loop {
        let is_on = config::is_feedback_enabled();
        let checkbox = if is_on { "[x]" } else { "[ ]" };
        
        println!("What would you like to configure?");
        println!("  1) Feedback loop {}", checkbox);
        println!("  2) Exit");

        match prompt("Choose (1-2): ").as_str() {
            "1" => {
                let new_state = !is_on;
                if let Err(e) = config::set_feedback_enabled(new_state) {
                    ui::error(&format!("Failed to save config: {e}"));
                } else {
                    let state_str = if new_state { "enabled" } else { "disabled" };
                    ui::success(&format!("Feedback loop is now {}.", state_str));
                }
                println!();
            }
            "2" | "q" | "quit" | "exit" => {
                return;
            }
            _ => println!("Invalid choice.\n"),
        }
    }
}

pub fn run(args: &[String]) {
    if args.is_empty() {
        interactive_config();
        return;
    }

    match args[0].as_str() {
        "feedback" => {
            if args.len() < 2 {
                let state = if config::is_feedback_enabled() { "on" } else { "off" };
                println!("Feedback loop is currently {}.", state);
                return;
            }
            let enable = match args[1].as_str() {
                "on" | "true" | "1" => true,
                "off" | "false" | "0" => false,
                other => {
                    ui::error(&format!("Invalid value '{other}' for feedback. Use 'on' or 'off'."));
                    return;
                }
            };
            if let Err(e) = config::set_feedback_enabled(enable) {
                ui::error(&format!("Failed to save config: {e}"));
            } else {
                let state = if enable { "enabled" } else { "disabled" };
                ui::success(&format!("Feedback loop is now {state}."));
            }
        }
        other => {
            ui::error(&format!("Unknown config key '{other}'."));
        }
    }
}
