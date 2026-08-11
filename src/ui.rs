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
    if colors_enabled() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
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

pub fn confirm(message: &str) -> bool {
    use std::io::Write;
    eprint!("{} {} [y/N]: ", cyan_bold("?"), bold(message));
    let _ = std::io::stderr().flush();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        let input = input.trim().to_lowercase();
        input == "y" || input == "yes"
    } else {
        false
    }
}

pub fn print_markdown(text: &str) {
    use termimad::MadSkin;
    use termimad::crossterm::style::Color::Rgb;

    let mut skin = MadSkin::default();

    // primary: foreground: #c9d1d9
    let fg = Rgb {
        r: 201,
        g: 209,
        b: 217,
    };

    // normal & bright colors from Please Dark
    let magenta = Rgb {
        r: 188,
        g: 140,
        b: 255,
    }; // #bc8cff
    let cyan = Rgb {
        r: 57,
        g: 197,
        b: 207,
    }; // #39c5cf
    let bright_white = Rgb {
        r: 255,
        g: 255,
        b: 255,
    }; // #ffffff

    // Slightly lighter backgrounds for code to pop against #0d1117
    let code_bg = Rgb {
        r: 22,
        g: 27,
        b: 34,
    };
    let inline_bg = Rgb {
        r: 72,
        g: 79,
        b: 88,
    }; // #484f58

    skin.paragraph.set_fg(fg);
    skin.bold.set_fg(bright_white);
    skin.italic.set_fg(fg);

    skin.inline_code.set_fg(cyan);
    skin.inline_code.set_bg(inline_bg);

    skin.code_block.set_fg(fg);
    skin.code_block.set_bg(code_bg);

    for h in skin.headers.iter_mut() {
        h.set_fg(magenta);
    }

    skin.print_text(text);
}
