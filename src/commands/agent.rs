use std::io::{self, Write};
use std::process::Command;

use crate::config::{self, Config};
use crate::context;
use crate::git;
use crate::llm::{self, ToolSpec};

const MAX_TOOL_OUTPUT: usize = 4000;

pub fn run(prompt: &str) {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        eprintln!("usage: please \"<what you want to do>\"");
        std::process::exit(1);
    }

    let (cfg, system_prompt, tools) = prepare_session();

    eprintln!("Thinking...");

    match llm::run_agent(prompt, &cfg, &system_prompt, &tools, execute_tool) {
        Ok(text) => println!("{text}"),
        Err(err) => {
            eprintln!("Agent failed: {err}");
            std::process::exit(1);
        }
    }
}

/// Loads config, generates/loads the cached project description, and builds
/// the system prompt and tool catalog every agent invocation needs — shared
/// between the one-shot `please "..."` and the long-lived `please chat`.
pub(crate) fn prepare_session() -> (Config, String, Vec<ToolSpec>) {
    let Some(mut cfg) = config::load() else {
        eprintln!("No LLM provider configured. Run `please setup` first.");
        std::process::exit(1);
    };

    let (project_context, context_model_update) = context::load_or_generate(&cfg);
    if let Some(model) = context_model_update {
        cfg.model = Some(model);
        let _ = config::save(&cfg);
    }

    let system_prompt = build_system_prompt(project_context.as_deref());
    (cfg, system_prompt, tool_specs())
}

fn build_system_prompt(project_context: Option<&str>) -> String {
    let mut prompt = String::from(
        "You are the agent behind `please`, an AI-native git CLI. The developer describes what \
         they want in plain language and you get it done using only the tools you're given: \
         run_git, run_gh, and run_please. You never read, write, or edit files directly — every \
         action goes through git, gh, or a please subcommand.\n\n\
         Prefer run_please for anything it already covers (branching, syncing, undoing, \
         committing, cleanup, restoring deleted files, reverting commits, stashing, and so on). \
         Fall back to run_git or run_gh for things please doesn't wrap: inspecting state, tags, \
         diffs, GitHub PRs/issues, and the like.\n\n\
         A few please subcommands normally ask for interactive confirmation before doing \
         something destructive (please discard, please sync exactly, please revert, please \
         stash drop, please purge, and please switch when the branch doesn't exist yet). Run \
         through you, they can't be confirmed \
         non-interactively, so they'll cancel themselves and say so — that's not a bug, don't \
         retry them. Tell the developer to run that command directly so they can confirm it \
         themselves. To create a new branch, use `please branch <name>` (creates and switches, \
         no confirmation needed) rather than `please switch <name>`.\n\n\
         Work in as few tool calls as you need, and don't repeat a call you already made. Once \
         the task is done — or you can't do it and need to explain why — respond with plain \
         text and no further tool calls. That ends the conversation, so make it a real answer \
         for the developer, not a status update.",
    );

    if let Some(context) = project_context {
        prompt.push_str("\n\nProject context:\n");
        prompt.push_str(context);
    }

    prompt.push_str(&format!("\n\nCurrent branch: {}", git::current_branch()));
    match git::upstream_branch() {
        Some(upstream) => prompt.push_str(&format!(" (tracking {upstream})")),
        None => prompt.push_str(" (no upstream set)"),
    }
    prompt.push_str(if git::has_pending_changes() {
        "\nWorking tree has uncommitted changes."
    } else {
        "\nWorking tree is clean."
    });

    prompt
}

fn tool_specs() -> Vec<ToolSpec> {
    let args_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "args": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["args"]
    });

    vec![
        ToolSpec {
            name: "run_git",
            description: "Run a git command. Give `args` exactly as they'd follow `git` on the command line, e.g. [\"log\", \"--oneline\", \"-5\"].",
            parameters: args_schema.clone(),
        },
        ToolSpec {
            name: "run_gh",
            description: "Run a GitHub CLI (`gh`) command, e.g. for pull requests or issues. Give `args` exactly as they'd follow `gh`.",
            parameters: args_schema,
        },
        ToolSpec {
            name: "run_please",
            description: "Run a `please` subcommand: one of status, branch, switch, sync, undo, redo, move-commit, discard, restore, rename, cleanup, log, revert, stash, purge, commit, push. `command` is the subcommand name; `args` are any extra arguments it takes (e.g. a branch name, list/pop/drop for stash, or a path for purge).",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "args": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["command"]
            }),
        },
    ]
}

pub(crate) fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "run_git" => run_external("git", args),
        "run_gh" => run_external("gh", args),
        "run_please" => run_please_subcommand(args),
        other => format!("Unknown tool '{other}'."),
    }
}

fn extract_args(value: &serde_json::Value) -> Vec<String> {
    value
        .get("args")
        .and_then(|v| v.as_array())
        .map(|items| items.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Runs an external `git`/`gh` invocation with captured output, so the model
/// gets real text back to reason about. Anything that looks destructive is
/// gated behind an interactive confirmation first — the same "explain, then
/// require an explicit yes" pattern every destructive `please` command uses.
fn run_external(bin: &str, args_json: &serde_json::Value) -> String {
    let args = extract_args(args_json);
    if args.is_empty() {
        return format!("No arguments given for {bin}.");
    }

    let command_line = format!("{bin} {}", args.join(" "));
    eprintln!("-> {command_line}");

    if is_destructive(bin, &args) && !confirm_destructive(&command_line) {
        return "The developer declined to run this command.".to_string();
    }

    capture_output(Command::new(bin).args(&args))
}

/// Runs a `please` subcommand as a child process rather than in-process:
/// these commands are written to own their whole lifecycle (including
/// exiting the process on failure), so shelling out is what lets one of them
/// fail without taking the agent down with it.
///
/// Stdin is closed (not inherited), so a subcommand that needs interactive
/// confirmation (e.g. `please discard`, or `please switch` to a branch that
/// doesn't exist yet) safely reads EOF and cancels itself, the same as it
/// would for any non-interactive caller — nothing destructive can happen
/// without a developer physically at the keyboard. The command's own
/// "Cancelled." text comes back to the model so it can tell the developer
/// to run it themselves, instead of assuming a 0 exit code meant success.
fn run_please_subcommand(args_json: &serde_json::Value) -> String {
    let Some(command) = args_json.get("command").and_then(|v| v.as_str()) else {
        return "Missing 'command' for run_please.".to_string();
    };
    let extra = extract_args(args_json);

    let Ok(exe) = std::env::current_exe() else {
        return "Couldn't locate the please binary to run a subcommand.".to_string();
    };

    eprintln!("-> please {command} {}", extra.join(" "));

    capture_output(Command::new(exe).arg(command).args(&extra))
}

fn capture_output(command: &mut Command) -> String {
    let output = match command.output() {
        Ok(output) => output,
        Err(err) => return format!("Failed to run command: {err}"),
    };

    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    let text = text.trim();

    let mut result = if output.status.success() {
        if text.is_empty() {
            "(command succeeded, no output)".to_string()
        } else {
            text.to_string()
        }
    } else {
        format!("Command failed (exit code {}): {text}", output.status.code().unwrap_or(-1))
    };

    if result.len() > MAX_TOOL_OUTPUT {
        result.truncate(MAX_TOOL_OUTPUT);
        result.push_str("\n...(truncated)");
    }
    result
}

/// Best-effort recognition of the git/gh invocations that discard work or
/// history — not exhaustive, but catches the common ones so the agent can't
/// silently run something a developer would want a chance to stop.
fn is_destructive(bin: &str, args: &[String]) -> bool {
    let has = |flag: &str| args.iter().any(|a| a == flag);

    if bin == "git" {
        (has("push") && (has("--force") || has("-f") || has("--force-with-lease")))
            || (has("reset") && has("--hard"))
            || (has("clean") && args.iter().any(|a| a.starts_with('-') && a.contains('f')))
            || (has("branch") && has("-D"))
            || (has("push") && has("--delete"))
            || args.iter().any(|a| a == "filter-branch" || a == "filter-repo")
    } else {
        has("delete") || has("close")
    }
}

fn confirm_destructive(command_line: &str) -> bool {
    eprint!("The agent wants to run `{command_line}`, which looks destructive. Allow it? [y/N]: ");
    let _ = io::stderr().flush();

    let mut input = String::new();
    io::stdin().read_line(&mut input).is_ok() && matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
