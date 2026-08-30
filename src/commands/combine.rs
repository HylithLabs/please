use crate::commands::agent;
pub fn run() {
    agent::run(
        "Inspect recent commits and propose which can be combined. Ask for confirmation before rewriting history, then combine only the approved commits with a clear message.",
    );
}
