use owo_colors::OwoColorize;
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

    let (cfg, base_prompt, tools) = prepare_session();
    let system_prompt = format!("{base_prompt}{}", dynamic_state());

    eprintln!("{}", "Thinking...".color(owo_colors::Rgb(110, 118, 129)));

    match llm::run_agent(prompt, &cfg, &system_prompt, &tools, execute_tool) {
        Ok(text) => crate::ui::print_markdown(&text),
        Err(err) => {
            eprintln!("Agent failed: {err}");
            std::process::exit(1);
        }
    }
}

/// Loads config, generates/loads the cached project description, and builds
/// the *static* half of the system prompt — everything except live repo
/// state — plus the tool catalog. Shared between the one-shot `please "..."`
/// and the long-lived `please chat`.
///
/// Repo state (branch, upstream, pending changes) is deliberately left out
/// here and appended fresh via [`dynamic_state`] right before each request
/// instead of baked in once: a chat session lives across many turns, and
/// every mutating tool call it runs can change that state. A git specialist
/// reasoning from a turn-1 snapshot after five tool calls have moved the
/// branch is reasoning from stale, effectively hallucinated context.
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

    let base_prompt = build_system_prompt(project_context.as_deref());
    (cfg, base_prompt, tool_specs())
}

/// The live repo state to append to the base system prompt right before
/// every request — see [`prepare_session`] for why this stays separate
/// instead of being folded in once.
pub(crate) fn dynamic_state() -> String {
    let mut state = format!("\n\nCurrent branch: {}", git::current_branch());
    match git::upstream_branch() {
        Some(upstream) => state.push_str(&format!(" (tracking {upstream})")),
        None => state.push_str(" (no upstream set)"),
    }
    state.push_str(if git::has_pending_changes() {
        "\nWorking tree has uncommitted changes."
    } else {
        "\nWorking tree is clean."
    });
    state
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
         Every action that changes repo state — a git/gh mutation or a please subcommand — asks \
         the developer to confirm before it runs; the answer comes back as tool output, not as \
         a message to you. A few commands (discard, purge, revert, squash, sync exactly, stash \
         drop, and switching to a branch that doesn't exist yet) can only be confirmed by \
         someone typing at a real keyboard — for those the tool tells you to have the developer \
         run it directly instead of retrying. To create a new branch, use `please branch \
         <name>` (confirmed once, then creates and switches) rather than `please switch \
         <name>`, which the agent can't do at all if the branch doesn't already exist.\n\n\
         Work in as few tool calls as you need, and don't repeat a call you already made. Once \
         the task is done — or you can't do it and need to explain why — respond with plain \
         text and no further tool calls. That ends the conversation, so make it a real answer \
         for the developer, not a status update.",
    );

    if let Some(context) = project_context {
        prompt.push_str("\n\nProject context:\n");
        prompt.push_str(context);
    }

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
            description: "Run a `please` subcommand. `command` is the subcommand name; `args` are any extra arguments it takes (e.g. a branch name, list/pop/drop for stash, a path for purge, or a count/ref for squash).",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "enum": [
                            "status", "branch", "switch", "sync", "undo", "redo",
                            "move-commit", "discard", "restore", "rename", "cleanup", "log",
                            "revert", "stash", "purge", "squash", "commit", "push", "update"
                        ]
                    },
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
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Runs an external `git`/`gh` invocation with captured output, so the model
/// gets real text back to reason about. Anything that isn't read-only is
/// gated behind an interactive confirmation first — the same "explain, then
/// require an explicit yes" pattern every mutating `please` command uses.
fn run_external(bin: &str, args_json: &serde_json::Value) -> String {
    let args = extract_args(args_json);
    if args.is_empty() {
        return format!("No arguments given for {bin}.");
    }

    let command_line = format!("{bin} {}", args.join(" "));
    eprintln!(
        "{}",
        format!("-> {command_line}").color(owo_colors::Rgb(210, 153, 34))
    ); // yellow

    let read_only = if bin == "git" {
        git_read_only(&args)
    } else {
        gh_read_only(&args)
    };
    let high_risk = !read_only
        && if bin == "git" {
            git_high_risk(&args)
        } else {
            gh_high_risk(&args)
        };
    let needs_confirm = crate::dispatch::wants_feedback() || !read_only;
    if needs_confirm && !confirm_command(&command_line, high_risk) {
        return "The developer declined to run this command.".to_string();
    }

    capture_output(Command::new(bin).args(&args))
}

/// Runs a `please` subcommand as a child process rather than in-process:
/// these commands are written to own their whole lifecycle (including
/// exiting the process on failure), so shelling out is what lets one of them
/// fail without taking the agent down with it.
///
/// Every mutating subcommand is confirmed *before* it's spawned — unlike
/// `git`/`gh`, a `please` subcommand can't be trusted to gate itself (most
/// don't ask anything at all; `please stash` and `please switch <existing
/// branch>` used to run instantly with no human in the loop). The ones that
/// only know how to ask via a typed "yes" at a real keyboard
/// (`Risk::InteractiveOnly`) are refused up front instead of spawned with
/// stdin closed — that used to silently read EOF and cancel itself, which
/// wasted a round trip and gave the model a confusing "it just failed"
/// result instead of a clear "go ask the developer" one.
fn run_please_subcommand(args_json: &serde_json::Value) -> String {
    let Some(command) = args_json.get("command").and_then(|v| v.as_str()) else {
        return "Missing 'command' for run_please.".to_string();
    };
    let extra = extract_args(args_json);
    let command_line = if extra.is_empty() {
        format!("please {command}")
    } else {
        format!("please {command} {}", extra.join(" "))
    };

    eprintln!(
        "{}",
        format!("-> {command_line}").color(owo_colors::Rgb(210, 153, 34))
    ); // yellow

    match please_risk(command, &extra) {
        Risk::InteractiveOnly => {
            return format!(
                "`{command_line}` can only be confirmed by someone typing at a real keyboard. \
                 Tell the developer to run `{command_line}` themselves — don't retry it."
            );
        }
        Risk::ReadOnly => {
            if crate::dispatch::wants_feedback() && !confirm_command(&command_line, false) {
                return "The developer declined to run this command.".to_string();
            }
        }
        Risk::Mutating => {
            if !confirm_command(&command_line, false) {
                return "The developer declined to run this command.".to_string();
            }
        }
    }

    let Ok(exe) = std::env::current_exe() else {
        return "Couldn't locate the please binary to run a subcommand.".to_string();
    };

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
        format!(
            "Command failed (exit code {}): {text}",
            output.status.code().unwrap_or(-1)
        )
    };

    if result.len() > MAX_TOOL_OUTPUT {
        result.truncate(MAX_TOOL_OUTPUT);
        result.push_str("\n...(truncated)");
    }
    result
}

/// Git verbs that only ever inspect state. Deliberately an allowlist rather
/// than a blocklist of "dangerous" flags: a blocklist has to guess every
/// risky flag combination up front and silently misses whatever it didn't
/// think of, while an allowlist only has to name the small, stable set of
/// commands that are safe by construction — everything else defaults to
/// asking first, which is the safe direction to be wrong in.
const GIT_READ_VERBS: &[&str] = &[
    "status",
    "log",
    "diff",
    "show",
    "blame",
    "shortlog",
    "describe",
    "rev-parse",
    "ls-files",
    "ls-tree",
    "cat-file",
    "reflog",
    "grep",
];

fn git_read_only(args: &[String]) -> bool {
    let Some(verb) = args.first().map(String::as_str) else {
        return false;
    };
    if GIT_READ_VERBS.contains(&verb) {
        return true;
    }
    // `branch`/`tag`/`remote` are read-only in their bare listing form, but
    // mutate the moment a name or non-listing flag shows up.
    match verb {
        "branch" | "tag" => args[1..]
            .iter()
            .all(|a| matches!(a.as_str(), "-a" | "-r" | "-v" | "--list")),
        "remote" => args[1..].iter().all(|a| a == "-v"),
        _ => false,
    }
}

/// Git invocations that don't just mutate but discard work or history
/// outright — flagged with a stronger warning in the confirmation prompt,
/// though the gate itself already caught them via `git_read_only`.
fn git_high_risk(args: &[String]) -> bool {
    let has = |flag: &str| args.iter().any(|a| a == flag);
    (has("push") && (has("--force") || has("-f") || has("--force-with-lease")))
        || (has("reset") && has("--hard"))
        || (has("clean") && args.iter().any(|a| a.starts_with('-') && a.contains('f')))
        || (has("branch") && has("-D"))
        || (has("push") && has("--delete"))
        || args
            .iter()
            .any(|a| a == "filter-branch" || a == "filter-repo")
}

const GH_READ_NOUNS: &[&str] = &["pr", "issue", "repo", "run", "release", "workflow", "gist"];
const GH_READ_ACTIONS: &[&str] = &["list", "view", "diff", "checks", "status"];

fn gh_read_only(args: &[String]) -> bool {
    let Some(noun) = args.first().map(String::as_str) else {
        return false;
    };
    if noun == "status" {
        return true;
    }
    let Some(action) = args.get(1).map(String::as_str) else {
        return false;
    };
    GH_READ_NOUNS.contains(&noun) && GH_READ_ACTIONS.contains(&action)
}

fn gh_high_risk(args: &[String]) -> bool {
    args.iter().any(|a| a == "delete" || a == "close")
}

/// How risky a `please` subcommand invocation is, decided from the actual
/// command and arguments rather than a prose promise the model might not
/// follow.
enum Risk {
    /// Inspects state only — runs immediately.
    ReadOnly,
    /// Changes repo state but is reversible — confirmed once before running.
    Mutating,
    /// Only confirmable by someone typing "yes" at a real keyboard. Run
    /// through the agent, stdin is closed, so the subcommand would just read
    /// EOF and cancel itself — instead of wasting that round trip, the gate
    /// refuses it up front and points at the developer running it directly.
    InteractiveOnly,
}

const PLEASE_INTERACTIVE_ONLY: &[&str] = &["discard", "purge", "revert", "squash"];

fn please_risk(command: &str, args: &[String]) -> Risk {
    match command {
        "status" | "log" => Risk::ReadOnly,
        "branch" if args.is_empty() => Risk::ReadOnly,
        "stash" => match args.first().map(String::as_str) {
            Some("list") => Risk::ReadOnly,
            Some("drop") => Risk::InteractiveOnly,
            _ => Risk::Mutating, // push (default) or pop
        },
        "sync" if args.first().map(String::as_str) == Some("exactly") => Risk::InteractiveOnly,
        // A `switch` to a branch that already exists just switches; to one
        // that doesn't, switch.rs asks "create it?" and reads stdin itself —
        // checked here dynamically against real repo state, not guessed.
        "switch" => match args.first() {
            Some(name) if git::branch_exists(name) => Risk::Mutating,
            _ => Risk::InteractiveOnly,
        },
        cmd if PLEASE_INTERACTIVE_ONLY.contains(&cmd) => Risk::InteractiveOnly,
        _ => Risk::Mutating,
    }
}

fn confirm_command(command_line: &str, destructive: bool) -> bool {
    let msg = if destructive {
        format!(
            "The agent wants to run `{command_line}`, which looks destructive. Allow it? [y/N]: "
        )
    } else {
        format!("The agent wants to run `{command_line}`. Allow it? [y/N]: ")
    };
    eprint!("{}", msg.color(owo_colors::Rgb(255, 123, 114)));
    let _ = io::stderr().flush();

    let mut input = String::new();
    io::stdin().read_line(&mut input).is_ok()
        && matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
