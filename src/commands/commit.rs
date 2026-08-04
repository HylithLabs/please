use crate::config;
use crate::git;
use crate::llm;

pub fn run() {
    git::stage_all();
    let diff = git::diff_staged();

    if diff.is_empty() {
        println!("No changes to commit.");
        return;
    }

    let Some(cfg) = config::load() else {
        eprintln!("No LLM provider configured. Run `please setup` first.");
        std::process::exit(1);
    };

    match llm::generate_commit_message(&diff, &cfg) {
        Ok(message) => println!("{message}"),
        Err(err) => {
            eprintln!("Failed to generate commit message: {err}");
            std::process::exit(1);
        }
    }
}
