use crate::commands::agent;
pub fn run() {
    agent::run(
        "Inspect the recent commits and help reorder them safely. Explain the proposed order and risks, ask for confirmation, then perform the smallest safe interactive rebase and report the result.",
    );
}
