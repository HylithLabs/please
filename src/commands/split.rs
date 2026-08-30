use crate::commands::agent;
pub fn run() {
    agent::run(
        "Split the current working tree into logical commits. Inspect the complete diff, propose the groups and messages first, then apply only approved staging and commits. Preserve unrelated work and report validation results.",
    );
}
