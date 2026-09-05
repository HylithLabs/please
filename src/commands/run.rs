use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::llm::RunPlan;
use crate::{config, git, llm, ui};

/// The only files the model ever sees for `please run`. An allowlist, not a
/// blocklist: `.env` and friends are simply never candidates, rather than
/// relying on a filter to catch them.
const MANIFEST_FILES: &[&str] = &[
    "README.md",
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "requirements.txt",
    "Dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
    "Makefile",
    "go.mod",
    "Procfile",
    ".tool-versions",
];

const MAX_MANIFEST_LEN: usize = 8000;

/// `.git/please/run.json` when a repo already exists, never the working
/// tree. `please run` works outside a repo too (running a project has
/// nothing to do with git), and must not create a `.git` folder just to
/// have somewhere to write a cache file, so a plain folder gets a global
/// cache instead, keyed by its path, under the same home directory
/// `please`'s own config already lives in.
fn cache_path() -> PathBuf {
    if git::is_repo() {
        return Path::new(".git").join("please").join("run.json");
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    cwd.hash(&mut hasher);
    config::config_dir()
        .join("run-cache")
        .join(format!("{:x}.json", hasher.finish()))
}

pub fn run(args: &[String]) {
    if args.first().map(String::as_str) == Some("reset") {
        reset();
        return;
    }

    if let Some(plan) = load_cached_plan() {
        ui::step(&format!("Running (saved): {}", plan.summary));
        execute(&plan);
        return;
    }

    let manifest = gather_manifest();
    if manifest.is_empty() {
        ui::error(
            "couldn't find anything to run here, no README, package.json, Cargo.toml, or \
             similar manifest file.",
        );
        std::process::exit(1);
    }

    let Some(mut cfg) = config::load() else {
        ui::error("No LLM provider configured. Run `please setup` first.");
        std::process::exit(1);
    };

    let outcome = match llm::plan_run(&manifest, &cfg) {
        Ok(outcome) => outcome,
        Err(err) => {
            ui::error(&format!(
                "couldn't figure out how to run this project: {err}"
            ));
            std::process::exit(1);
        }
    };
    if cfg.model.as_deref() != Some(outcome.model_used.as_str()) {
        cfg.model = Some(outcome.model_used);
        let _ = config::save(&cfg);
    }

    let plan = outcome.plan;
    println!("{}", plan.summary);
    if !plan.precheck.is_empty() {
        ui::detail(&format!("checks first: {}", plan.precheck));
    }
    for cmd in &plan.commands {
        ui::detail(cmd);
    }

    if !ui::confirm("Run this?") {
        ui::warn("Skipped, nothing was saved. Run `please run` again to decide fresh.");
        return;
    }

    if let Err(err) = save_cached_plan(&plan) {
        ui::warn(&format!(
            "running it anyway, but couldn't save it for next time: {err}"
        ));
    }
    execute(&plan);
}

fn reset() {
    match std::fs::remove_file(cache_path()) {
        Ok(()) => println!(
            "Cleared the saved run command. `please run` will figure it out fresh next time."
        ),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!("Nothing saved yet.");
        }
        Err(err) => {
            ui::error(&format!("failed to clear the saved run command: {err}"));
            std::process::exit(1);
        }
    }
}

fn gather_manifest() -> String {
    let mut manifest = String::new();
    for name in MANIFEST_FILES {
        if manifest.len() >= MAX_MANIFEST_LEN {
            break;
        }
        let Ok(contents) = std::fs::read_to_string(name) else {
            continue;
        };
        manifest.push_str("--- ");
        manifest.push_str(name);
        manifest.push_str(" ---\n");
        let remaining = MAX_MANIFEST_LEN.saturating_sub(manifest.len());
        manifest.push_str(truncate_at_char_boundary(&contents, remaining));
        manifest.push_str("\n\n");
    }
    manifest
}

fn truncate_at_char_boundary(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn load_cached_plan() -> Option<RunPlan> {
    let text = std::fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_cached_plan(plan: &RunPlan) -> std::io::Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json =
        serde_json::to_string_pretty(plan).map_err(|err| std::io::Error::other(err.to_string()))?;
    std::fs::write(path, json)
}

fn execute(plan: &RunPlan) {
    if !plan.precheck.is_empty() {
        ui::step(&format!("Checking: {}", plan.precheck));
        if !run_shell(&plan.precheck) {
            if plan.precheck_hint.is_empty() {
                ui::error(&format!("precheck failed: {}", plan.precheck));
            } else {
                ui::error(&plan.precheck_hint);
            }
            std::process::exit(1);
        }
    }

    for cmd in &plan.commands {
        ui::step(cmd);
        if !run_shell(cmd) {
            ui::error(&format!("command failed: {cmd}"));
            std::process::exit(1);
        }
    }
}

fn run_shell(command: &str) -> bool {
    let (program, flag) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };
    Command::new(program)
        .arg(flag)
        .arg(command)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
