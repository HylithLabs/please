use crate::config;
use crate::context;
use crate::git;
use crate::gitignore;
use crate::llm;
use crate::sensitive;

pub fn run() {
    gitignore::ensure_junk_ignored();

    let already_staged = git::staged_files();
    git::stage_all();

    for file in git::staged_files() {
        if !already_staged.contains(&file) && sensitive::is_sensitive(&file) {
            git::unstage_file(&file);
            crate::ui::warn(&format!(
                "skipped staging '{file}', looks like a secret/credential file. \
                 Run `git add {file}` yourself first if you want it included."
            ));
        }
    }

    let diff = git::diff_staged();

    if diff.is_empty() {
        println!("No changes to commit.");
        return;
    }

    let Some(mut cfg) = config::load() else {
        crate::ui::error("No LLM provider configured. Run `please setup` first.");
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

            let total = outcome.commits.len();
            for (index, group) in outcome.commits.into_iter().enumerate() {
                if !git::stage_files(&group.files) {
                    crate::ui::error(&format!(
                        "skipping commit, failed to stage {:?}",
                        group.files
                    ));
                    continue;
                }

                if total > 1 {
                    crate::ui::step(&format!("Commit {}/{total}", index + 1));
                }

                if crate::dispatch::wants_feedback() {
                    let msg = format!("Commit with message: '{}'?", group.message.trim());
                    if !crate::ui::confirm(&msg) {
                        crate::ui::warn("Skipped commit.");
                        continue;
                    }
                }

                if git::commit(&group.message) {
                    crate::ui::success(&format!(
                        "Committed: {}",
                        group.message.lines().next().unwrap_or("")
                    ));
                    for file in &group.files {
                        crate::ui::detail(file);
                    }
                } else {
                    crate::ui::error(&format!("failed to commit {:?}", group.files));
                }
            }

            if git::has_pending_changes() {
                crate::ui::warn(
                    "some changes were left uncommitted (not covered by the AI's commit plan).",
                );
            }
        }
        Err(err) => {
            let _ = config::save(&cfg);
            crate::ui::error(&format!("failed to plan commits: {err}"));
            std::process::exit(1);
        }
    }
}
