//! Small terminal styling helpers so `please`'s own output reads clearly
//! next to the raw git/gh output it's woven around. Colors are skipped
//! automatically when stdout/stderr aren't a real terminal (piped, redirected,
//! or `NO_COLOR` is set), so scripts and log files stay plain text.

use std::io::IsTerminal;

fn colors_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && std::io::stdout().is_terminal()
        && std::io::stderr().is_terminal()
}

fn paint(text: &str, code: &str) -> String {
    if colors_enabled() { format!("\x1b[{code}m{text}\x1b[0m") } else { text.to_string() }
}

fn bold(text: &str) -> String {
    paint(text, "1")
}

fn dim(text: &str) -> String {
    paint(text, "2")
}

fn cyan_bold(text: &str) -> String {
    paint(text, "1;36")
}

fn green_bold(text: &str) -> String {
    paint(text, "1;32")
}

fn yellow(text: &str) -> String {
    paint(text, "33")
}

fn red_bold(text: &str) -> String {
    paint(text, "1;31")
}

/// Announces a step that's about to run (an AI call, a push, etc). Printed
/// to stderr so it never mixes into piped stdout output, with a blank line
/// ahead of it so consecutive steps don't run together.
pub fn step(message: &str) {
    eprintln!("\n{} {}", cyan_bold(">"), bold(message));
}

/// A completed, positive result: a commit made, a push finished.
pub fn success(message: &str) {
    println!("{} {}", green_bold("done"), message);
}

/// A file (or other) line under a success/step message, indented and muted
/// so it reads as detail rather than another headline.
pub fn detail(message: &str) {
    println!("   {}", dim(message));
}

pub fn warn(message: &str) {
    eprintln!("{} {}", yellow("warning:"), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", red_bold("error:"), message);
}
