use crate::config;
use crate::context;
use crate::git;
use crate::ui;

/// Sets up a fresh repo for `please`: `git init` if one isn't already there,
/// then an eager attempt at `please.md` (the AI-generated project
/// description `please` otherwise only generates lazily, the first time
/// `commit`/agent mode needs it).
pub fn run() {
    if git::is_repo() {
        ui::warn("already a git repository, nothing to initialize there.");
    } else {
        if let Err(err) = git::init() {
            ui::error(&format!("failed to run git init: {err}"));
            std::process::exit(1);
        }
        ui::success("Initialized an empty git repository.");
    }

    let Some(mut cfg) = config::load() else {
        println!(
            "Run `please setup` to configure an AI provider — please.md gets generated \
             automatically the first time an AI command needs it."
        );
        return;
    };

    ui::step("Generating please.md");
    let (project_context, context_model_update) = context::load_or_generate(&cfg);
    if let Some(model) = context_model_update {
        cfg.model = Some(model);
        let _ = config::save(&cfg);
    }

    match project_context {
        Some(_) => ui::success("Generated please.md."),
        None => println!(
            "Nothing to describe yet — add and commit some files, then run `please commit` or \
             `please chat` to generate please.md."
        ),
    }
}
