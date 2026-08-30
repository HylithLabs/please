use crate::commands::agent;
pub fn run() {
    agent::run(
        "Analyze commits since the latest tag. Generate release notes and suggest a semantic version. Ask for confirmation before creating a tag or preparing/pushing a release; never push without explicit approval.",
    );
}
