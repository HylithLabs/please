mod gemini;

use crate::config::Config;
use serde::Deserialize;

pub struct GenerationOutcome {
    pub message: String,
    pub model_used: String,
}

/// A capability the agent can invoke, described in provider-neutral terms —
/// `parameters` is a standard (lowercase-type) JSON Schema object. Each
/// provider adapter translates this into whatever wire format it needs.
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: serde_json::Value,
}

/// A single tool invocation the model asked for. `thought_signature` is an
/// opaque, provider-specific token some models attach to a tool call and
/// require back verbatim on the next turn (Gemini's "thinking" models do);
/// providers that don't use one just leave it `None`.
struct ToolCall {
    id: String,
    name: String,
    args: serde_json::Value,
    thought_signature: Option<String>,
}

/// What came back after the tool named `name` ran.
struct ToolOutcome {
    id: String,
    name: String,
    output: String,
}

/// One exchange in the agent conversation, kept in a shape every provider
/// adapter can translate to and from its own wire format — this is the only
/// vocabulary the core loop speaks, so it never needs to know which provider
/// is behind it.
enum AgentMessage {
    User(String),
    Model(Vec<ToolCall>),
    ToolResults(Vec<ToolOutcome>),
}

/// What a provider adapter produced for one turn: either more work to do, or
/// a finished answer for the developer.
enum AgentTurn {
    ToolCalls(Vec<ToolCall>),
    Final(String),
}

/// Runs the agent to completion: sends `prompt`, executes whatever tools the
/// model calls via `execute_tool`, and feeds the results back until the model
/// stops calling tools (or a step limit is hit, to bound runaway loops).
///
/// `execute_tool` never fails outright — a tool that can't run just returns a
/// string explaining why, so the model can react to it like any other result.
pub fn run_agent(
    prompt: &str,
    config: &Config,
    system_prompt: &str,
    tools: &[ToolSpec],
    mut execute_tool: impl FnMut(&str, &serde_json::Value) -> String,
) -> Result<String, String> {
    const MAX_TURNS: usize = 8;

    let mut history = vec![AgentMessage::User(prompt.to_string())];

    for turn in 1..=MAX_TURNS {
        let calls = match agent_turn(system_prompt, &history, tools, config)? {
            AgentTurn::Final(text) => {
                return Ok(if text.is_empty() {
                    "Done.".to_string()
                } else {
                    text
                });
            }
            AgentTurn::ToolCalls(calls) => calls,
        };

        if turn == MAX_TURNS {
            return Ok(
                "Stopped after several steps without finishing — try breaking the request into \
                 smaller pieces."
                    .to_string(),
            );
        }

        let mut outcomes = Vec::with_capacity(calls.len());
        for call in &calls {
            let output = execute_tool(&call.name, &call.args);
            outcomes.push(ToolOutcome {
                id: call.id.clone(),
                name: call.name.clone(),
                output,
            });
        }

        history.push(AgentMessage::Model(calls));
        history.push(AgentMessage::ToolResults(outcomes));
    }

    unreachable!("loop always returns by MAX_TURNS")
}

/// Dispatches one model turn to the configured provider's adapter. Adding a
/// new provider means writing its `agent_turn` and adding a match arm here —
/// the loop above and every tool never change.
fn agent_turn(
    system_prompt: &str,
    history: &[AgentMessage],
    tools: &[ToolSpec],
    config: &Config,
) -> Result<AgentTurn, String> {
    match config.provider.as_str() {
        "google" => gemini::agent_turn(system_prompt, history, tools, &config.api_key, config.model.as_deref()),
        other => Err(format!(
            "Provider '{other}' is not supported yet. Only 'google' (Gemini) is wired up so far."
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct CommitGroup {
    pub files: Vec<String>,
    pub message: String,
}

pub struct CommitPlanOutcome {
    pub commits: Vec<CommitGroup>,
    pub model_used: String,
}

pub fn describe_codebase(file_list: &str, config: &Config) -> Result<GenerationOutcome, String> {
    match config.provider.as_str() {
        "google" => gemini::describe_codebase(file_list, &config.api_key, config.model.as_deref()),
        other => Err(format!(
            "Provider '{other}' is not supported yet. Only 'google' (Gemini) is wired up so far."
        )),
    }
}

pub fn plan_commits(
    diff: &str,
    context: Option<&str>,
    config: &Config,
) -> Result<CommitPlanOutcome, String> {
    match config.provider.as_str() {
        "google" => gemini::plan_commits(diff, context, &config.api_key, config.model.as_deref()),
        other => Err(format!(
            "Provider '{other}' is not supported yet. Only 'google' (Gemini) is wired up so far."
        )),
    }
}

/// Picks the lowest-cost model for a provider, once, at `please setup` time.
/// Returns `None` for providers without auto-selection support.
pub fn select_model(provider: &str, api_key: &str) -> Option<String> {
    match provider {
        "google" => Some(gemini::select_lowest_cost_model(api_key)),
        _ => None,
    }
}
