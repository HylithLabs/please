use std::io::{self, Write};

use crate::config;
use crate::context;
use crate::git;
use crate::llm;
use crate::ui;

pub fn run(args: &[String]) {
    if git::has_pending_changes() {
        ui::error(
            "you have uncommitted changes. Run `please commit` or `please discard` first, \
             squashing needs a clean working tree.",
        );
        std::process::exit(1);
    }

    let base_ref = match resolve_base(args) {
        Ok(base) => base,
        Err(err) => {
            ui::error(&err);
            std::process::exit(1);
        }
    };

    let Some(base_sha) = git::resolve_commit(&base_ref) else {
        ui::error(&format!("'{base_ref}' doesn't name a commit I can find."));
        std::process::exit(1);
    };

    if base_sha == git::head_sha() {
        println!("Already at '{base_ref}', nothing to squash.");
        return;
    }

    if !git::is_ancestor(&base_sha, "HEAD") {
        ui::error(&format!(
            "'{base_ref}' isn't an ancestor of HEAD, so it's not safe to squash back to it. \
             Pick a commit that's actually part of this branch's history."
        ));
        std::process::exit(1);
    }

    let subjects = git::commit_subjects_since(&base_sha);
    if subjects.len() < 2 {
        println!(
            "Only {} commit ahead of '{base_ref}', nothing to squash.",
            subjects.len()
        );
        return;
    }

    let branch = git::current_branch();
    let will_push = git::upstream_branch().is_some();

    println!(
        "This will squash {} commits into one, replacing their hashes:",
        subjects.len()
    );
    for subject in &subjects {
        println!("  - {subject}");
    }
    if will_push {
        println!(
            "'{branch}' has already been pushed, so the squashed result will be force-pushed \
             (with a safety check) right after."
        );
    }
    print!("Type 'yes' to continue: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() || input.trim() != "yes" {
        println!("Cancelled.");
        return;
    }

    let Some(mut cfg) = config::load() else {
        ui::error("No LLM provider configured. Run `please setup` first.");
        std::process::exit(1);
    };

    let (project_context, context_model_update) = context::load_or_generate(&cfg);
    if let Some(model) = context_model_update {
        cfg.model = Some(model);
    }

    let commit_summaries: String = subjects
        .iter()
        .map(|subject| format!("- {subject}\n"))
        .collect();

    if let Err(err) = git::reset_soft(&base_sha) {
        ui::error(&format!("failed to combine the commits: {err}"));
        std::process::exit(1);
    }

    let diff = git::diff_staged();

    ui::step("Writing squash message");
    match llm::squash_message(&commit_summaries, &diff, project_context.as_deref(), &cfg) {
        Ok(outcome) => {
            if cfg.model.as_deref() != Some(outcome.model_used.as_str()) {
                cfg.model = Some(outcome.model_used);
            }
            let _ = config::save(&cfg);

            if !git::commit(&outcome.message) {
                ui::error(
                    "failed to create the squashed commit. Your original commits are still \
                     safe, find them with `git reflog`.",
                );
                std::process::exit(1);
            }
        }
        Err(err) => {
            let _ = config::save(&cfg);
            ui::error(&format!(
                "failed to write a squash message ({err}). Your original commits are still \
                 safe, find them with `git reflog`."
            ));
            std::process::exit(1);
        }
    }

    ui::success(&format!(
        "Squashed {} commits into one: {}",
        subjects.len(),
        git::last_commit_message()
    ));

    if !will_push {
        return;
    }

    ui::step(&format!("Force-pushing '{branch}' to origin"));
    if git::force_push_with_lease(&branch) {
        ui::success(&format!("Pushed the squashed commit to origin/{branch}."));
    } else {
        ui::error(
            "force-push failed, likely because someone else pushed to this branch in the \
             meantime. Run `please sync` to see what changed, then try again.",
        );
        std::process::exit(1);
    }
}

/// No argument: squashes everything ahead of the upstream (or the repo's
/// default branch, if there's no upstream yet). A number `n`: squashes the
/// last `n` commits. Anything else: treated as a branch or commit to squash
/// back to directly.
fn resolve_base(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        None => Ok(git::upstream_branch().unwrap_or_else(git::default_branch)),
        Some(arg) => match arg.parse::<u32>() {
            Ok(0) => Err("usage: please squash [<n> | <branch-or-commit>]".to_string()),
            Ok(n) => Ok(format!("HEAD~{n}")),
            Err(_) => Ok(arg.to_string()),
        },
    }
}
