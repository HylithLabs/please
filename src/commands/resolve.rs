use crate::commands::agent;

pub fn run() {
    agent::run(
        "Resolve the current Git merge conflicts. First inspect all conflicted files and explain both sides. Propose a resolution for each file, ask for confirmation before any edits, apply only the approved resolutions, then run the repository's relevant validation or tests and report the result. Do not discard unrelated work.",
    );
}
