mod anthropic;
mod gemini;
mod openai;

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
#[derive(Clone)]
struct ToolCall {
    id: String,
    name: String,
    args: serde_json::Value,
    thought_signature: Option<String>,
}

/// What came back after the tool named `name` ran.
#[derive(Clone)]
struct ToolOutcome {
    id: String,
    name: String,
    output: String,
}

/// One exchange in the agent conversation, kept in a shape every provider
/// adapter can translate to and from its own wire format — this is the only
/// vocabulary the core loop speaks, so it never needs to know which provider
/// is behind it.
#[derive(Clone)]
enum AgentMessage {
    User(String),
    /// A model turn: zero or more tool calls, plus any text it said
    /// alongside them (a plain final answer is `calls: vec![]`).
    Model { calls: Vec<ToolCall>, text: Option<String> },
    ToolResults(Vec<ToolOutcome>),
}

/// What a provider adapter produced for one turn: either more work to do, or
/// a finished answer for the developer.
enum AgentTurn {
    ToolCalls { calls: Vec<ToolCall>, text: Option<String> },
    Final(String),
}

/// A running agent conversation. One-off requests (`please "..."`) use a
/// session that lives for a single `send`; `please chat` keeps one alive
/// across many, so the model actually remembers what was said and done
/// earlier in the conversation instead of starting cold each message.
pub struct AgentSession {
    history: Vec<AgentMessage>,
}

impl AgentSession {
    pub fn new() -> Self {
        Self { history: Vec::new() }
    }

    /// Sends `prompt` as the next user turn, runs however many tool-call
    /// rounds the model needs, and returns its final answer. The exchange is
    /// only committed to this session's history once it succeeds — a failed
    /// call leaves the conversation exactly as it was, so the developer can
    /// just try again.
    ///
    /// `execute_tool` never fails outright — a tool that can't run just
    /// returns a string explaining why, so the model can react to it like
    /// any other result.
    pub fn send(
        &mut self,
        prompt: &str,
        config: &Config,
        system_prompt: &str,
        tools: &[ToolSpec],
        mut execute_tool: impl FnMut(&str, &serde_json::Value) -> String,
    ) -> Result<String, String> {
        const MAX_TURNS: usize = 8;

        // Work on a copy so a failed call leaves `self.history` exactly as
        // it was — only a successful exchange gets committed at the end.
        let mut working = self.history.clone();
        working.push(AgentMessage::User(prompt.to_string()));

        for turn in 1..=MAX_TURNS {
            let (calls, text) = match agent_turn(system_prompt, &working, tools, config)? {
                AgentTurn::Final(text) => {
                    let text = if text.is_empty() { "Done.".to_string() } else { text };
                    working.push(AgentMessage::Model { calls: Vec::new(), text: Some(text.clone()) });
                    self.history = working;
                    return Ok(text);
                }
                AgentTurn::ToolCalls { calls, text } => (calls, text),
            };

            if turn == MAX_TURNS {
                let text = "Stopped after several steps without finishing — try breaking the \
                             request into smaller pieces."
                    .to_string();
                working.push(AgentMessage::Model { calls: Vec::new(), text: Some(text.clone()) });
                self.history = working;
                return Ok(text);
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

            working.push(AgentMessage::Model { calls, text });
            working.push(AgentMessage::ToolResults(outcomes));
        }

        unreachable!("loop always returns by MAX_TURNS")
    }
}

impl Default for AgentSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs a single request-response agent exchange with no memory of anything
/// before or after it — just a session that's used once.
pub fn run_agent(
    prompt: &str,
    config: &Config,
    system_prompt: &str,
    tools: &[ToolSpec],
    execute_tool: impl FnMut(&str, &serde_json::Value) -> String,
) -> Result<String, String> {
    AgentSession::new().send(prompt, config, system_prompt, tools, execute_tool)
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
        "anthropic" => anthropic::agent_turn(system_prompt, history, tools, &config.api_key, config.model.as_deref()),
        "openai" => openai::agent_turn(system_prompt, history, tools, &config.api_key, config.model.as_deref()),
        other => Err(format!(
            "Provider '{other}' is not supported yet. 'google' (Gemini), 'anthropic' (Claude), \
             and 'openai' (ChatGPT) are wired up so far."
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

/// The shape a commit plan response must match, as standard (lowercase-type)
/// JSON Schema — mirrors `CommitGroup`/`CommitPlanOutcome` above. Provider
/// adapters that support structured output translate this into whatever
/// casing/dialect their API expects, the same way they do for `ToolSpec`.
fn commit_plan_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "commits": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "message": { "type": "string" }
                    },
                    "required": ["files", "message"]
                }
            }
        },
        "required": ["commits"]
    })
}

/// Structured-output schemas on Anthropic and OpenAI both require
/// `additionalProperties: false` on every object node (it's what makes the
/// output-shape guarantee possible) — unlike ordinary tool-call schemas,
/// where it's optional. Both adapters run their (already-neutral) schema
/// through this before sending it.
fn require_closed_objects(schema: &serde_json::Value) -> serde_json::Value {
    match schema {
        serde_json::Value::Object(map) => {
            let mut out: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(key, value)| (key.clone(), require_closed_objects(value)))
                .collect();
            if map.get("type").and_then(|t| t.as_str()) == Some("object")
                && !out.contains_key("additionalProperties")
            {
                out.insert("additionalProperties".to_string(), serde_json::Value::Bool(false));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(require_closed_objects).collect())
        }
        other => other.clone(),
    }
}

/// Lets the model decide how to split the working tree's changes into one or
/// more logically coherent commits, worded as a prompt every provider sends
/// verbatim — only the request wrapper (schema, tool declarations) differs
/// per adapter.
fn build_commit_plan_prompt(diff: &str, context: Option<&str>) -> String {
    let mut prompt = String::new();

    if let Some(context) = context {
        prompt.push_str("Project context:\n");
        prompt.push_str(context);
        prompt.push_str("\n\n");
    }

    prompt.push_str(
        "You are an autonomous git agent. Below is the full diff of every changed file in the \
         working tree; each file's section starts with a `diff --git a/<path> b/<path>` \
         header. Decide how to split these changes into one or more logically coherent \
         commits: group files that belong to the same unit of work together, and separate \
         unrelated changes into different commits. If all the changes belong together, return \
         a single commit. For each commit, list the exact file paths (relative to the repo \
         root, exactly as they appear in the diff headers) and write a concise, \
         conventional-commit style message.\n\nDiff:\n",
    );
    prompt.push_str(diff);
    prompt
}

fn build_description_prompt(file_list: &str) -> String {
    format!(
        "You are analyzing a software repository. Based on the list of tracked files below, \
         write a short description (3-6 sentences) of what this codebase is and does: its \
         purpose, its main components, and its language/tech stack. Output only the \
         description itself, with no markdown formatting, no headings, and no explanation.\n\n\
         Tracked files:\n{file_list}"
    )
}

pub fn describe_codebase(file_list: &str, config: &Config) -> Result<GenerationOutcome, String> {
    match config.provider.as_str() {
        "google" => gemini::describe_codebase(file_list, &config.api_key, config.model.as_deref()),
        "anthropic" => anthropic::describe_codebase(file_list, &config.api_key, config.model.as_deref()),
        "openai" => openai::describe_codebase(file_list, &config.api_key, config.model.as_deref()),
        other => Err(format!(
            "Provider '{other}' is not supported yet. 'google' (Gemini), 'anthropic' (Claude), \
             and 'openai' (ChatGPT) are wired up so far."
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
        "anthropic" => anthropic::plan_commits(diff, context, &config.api_key, config.model.as_deref()),
        "openai" => openai::plan_commits(diff, context, &config.api_key, config.model.as_deref()),
        other => Err(format!(
            "Provider '{other}' is not supported yet. 'google' (Gemini), 'anthropic' (Claude), \
             and 'openai' (ChatGPT) are wired up so far."
        )),
    }
}

/// Picks the lowest-cost model for a provider, once, at `please setup` time.
/// Returns `None` for providers without auto-selection support.
pub fn select_model(provider: &str, api_key: &str) -> Option<String> {
    match provider {
        "google" => Some(gemini::select_lowest_cost_model(api_key)),
        "anthropic" => Some(anthropic::select_lowest_cost_model(api_key)),
        "openai" => Some(openai::select_lowest_cost_model(api_key)),
        _ => None,
    }
}

/// Confirms an API key actually authenticates, so `please setup` can tell
/// the developer right away instead of the key failing silently on first
/// real use. `None` means this provider has no cheap way to check — the
/// caller should skip validation rather than treat it as a failure.
pub fn validate_key(provider: &str, api_key: &str) -> Option<Result<(), String>> {
    match provider {
        "google" => Some(gemini::validate_api_key(api_key)),
        "anthropic" => Some(anthropic::validate_api_key(api_key)),
        "openai" => Some(openai::validate_api_key(api_key)),
        _ => None,
    }
}
