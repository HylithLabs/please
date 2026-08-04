use crate::config;
use crate::context;
use crate::git;
use crate::llm;
use crate::sensitive;

pub fn run() {
    let already_staged = git::staged_files();
    git::stage_all();

    for file in git::staged_files() {
        if !already_staged.contains(&file) && sensitive::is_sensitive(&file) {
            git::unstage_file(&file);
            eprintln!(
                "Skipped staging '{file}' — looks like a secret/credential file. \
                 Run `git add {file}` yourself first if you want it included."
            );
        }
    }

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

    // Unstage — the diff above was just for planning. We re-stage per commit below.
    git::unstage_all();

    match llm::plan_commits(&diff, project_context.as_deref(), &cfg) {
        Ok(outcome) => {
            if cfg.model.as_deref() != Some(outcome.model_used.as_str()) {
                cfg.model = Some(outcome.model_used);
            }
            let _ = config::save(&cfg);

            for group in outcome.commits {
                if !git::stage_files(&group.files) {
                    eprintln!("Skipping commit — failed to stage {:?}", group.files);
                    continue;
                }

                if git::commit(&group.message) {
                    println!("Committed: {}", group.message.lines().next().unwrap_or(""));
                    for file in &group.files {
                        println!("  {file}");
                    }
                } else {
                    eprintln!("Failed to commit {:?}", group.files);
                }
            }

            if git::has_pending_changes() {
                eprintln!(
                    "Note: some changes were left uncommitted (not covered by the AI's commit plan)."
                );
            }
        }
        Err(err) => {
            let _ = config::save(&cfg);
            eprintln!("Failed to plan commits: {err}");
            std::process::exit(1);
        }
    }
}
