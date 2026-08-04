use crate::config;
use crate::context;
use crate::git;
use crate::llm;

pub fn run() {
    git::stage_all();
    let diff = git::diff_staged();

    if diff.is_empty() {
        println!("No changes to commit.");
        return;
    }

    let Some(mut cfg) = config::load() else {
        eprintln!("No LLM provider configured. Run `please setup` first.");
        std::process::exit(1);
    };

    let (project_context, context_model_update) = context::load_or_generate(&cfg);
    if let Some(model) = context_model_update {
        cfg.model = Some(model);
    }

    match llm::generate_commit_message(&diff, project_context.as_deref(), &cfg) {
        Ok(outcome) => {
            if cfg.model.as_deref() != Some(outcome.model_used.as_str()) {
                cfg.model = Some(outcome.model_used);
            }
            let _ = config::save(&cfg);
            println!("{}", outcome.message);
        }
        Err(err) => {
            let _ = config::save(&cfg);
            eprintln!("Failed to generate commit message: {err}");
            std::process::exit(1);
        }
    }
}
