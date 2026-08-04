use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{AgentMessage, AgentTurn, CommitGroup, CommitPlanOutcome, GenerationOutcome, ToolCall, ToolSpec};

const MESSAGES_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const MODELS_ENDPOINT: &str = "https://api.anthropic.com/v1/models";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const FALLBACK_MODEL: &str = "claude-haiku-4-5";

const GENERATION_TIMEOUT: Duration = Duration::from_secs(90);
const LIST_MODELS_TIMEOUT: Duration = Duration::from_secs(20);

const MAX_TOKENS: u32 = 8192;

// A content block is one of: text, tool_use (in a model turn), or
// tool_result (in a user turn) — flattened into one optional-field struct
// rather than a tagged enum, since Rust enums don't serialize/deserialize
// against this exact "type" + sibling-fields shape without extra plumbing.
#[derive(Serialize, Deserialize, Clone, Default)]
struct WireBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<serde_json::Value>,
    #[serde(rename = "tool_use_id", skip_serializing_if = "Option::is_none")]
    tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

impl WireBlock {
    fn text(text: impl Into<String>) -> Self {
        Self { kind: "text".to_string(), text: Some(text.into()), ..Default::default() }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct WireMessage {
    role: String,
    content: Vec<WireBlock>,
}

#[derive(Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<WireTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig>,
}

#[derive(Serialize)]
struct WireTool {
    name: &'static str,
    description: &'static str,
    input_schema: serde_json::Value,
}

#[derive(Serialize)]
struct OutputConfig {
    format: OutputFormat,
}

#[derive(Serialize)]
struct OutputFormat {
    #[serde(rename = "type")]
    kind: &'static str,
    schema: serde_json::Value,
}

#[derive(Deserialize)]
struct MessagesResponse {
    #[serde(default)]
    content: Vec<WireBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ModelsListResponse {
    #[serde(default)]
    data: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    id: String,
}

pub fn describe_codebase(
    file_list: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<GenerationOutcome, String> {
    let prompt = super::build_description_prompt(file_list);
    generate_with_retry("Analyzing codebase", &prompt, api_key, model, None)
}

pub fn plan_commits(
    diff: &str,
    context: Option<&str>,
    api_key: &str,
    model: Option<&str>,
) -> Result<CommitPlanOutcome, String> {
    let prompt = super::build_commit_plan_prompt(diff, context);
    let format = OutputFormat {
        kind: "json_schema",
        schema: super::require_closed_objects(&super::commit_plan_schema()),
    };

    let outcome = generate_with_retry("Planning commits", &prompt, api_key, model, Some(format))?;

    let plan: RawCommitPlan = serde_json::from_str(&outcome.message)
        .map_err(|e| format!("failed to parse commit plan JSON: {e}"))?;

    if plan.commits.is_empty() {
        return Err("model returned an empty commit plan".to_string());
    }

    Ok(CommitPlanOutcome { commits: plan.commits, model_used: outcome.model_used })
}

#[derive(Deserialize)]
struct RawCommitPlan {
    commits: Vec<CommitGroup>,
}

/// Calls Claude with the given prompt. If the configured model fails (e.g.
/// it was retired), re-runs model discovery and retries once with the fresh
/// pick — mirrors the same self-healing behavior as the Gemini adapter.
fn generate_with_retry(
    label: &str,
    prompt: &str,
    api_key: &str,
    model: Option<&str>,
    format: Option<OutputFormat>,
) -> Result<GenerationOutcome, String> {
    let mut current_model = match model {
        Some(model) => model.to_string(),
        None => select_lowest_cost_model(api_key),
    };

    eprintln!("{label} (model: {current_model})...");

    match call_claude(prompt, api_key, &current_model, &format) {
        Ok(message) => Ok(GenerationOutcome { message, model_used: current_model }),
        Err(err) => {
            eprintln!("Warning: model '{current_model}' failed ({err}). Re-selecting a model...");
            current_model = select_lowest_cost_model(api_key);
            eprintln!("{label} (model: {current_model})...");
            let message = call_claude(prompt, api_key, &current_model, &format)?;
            Ok(GenerationOutcome { message, model_used: current_model })
        }
    }
}

fn call_claude(
    prompt: &str,
    api_key: &str,
    model: &str,
    format: &Option<OutputFormat>,
) -> Result<String, String> {
    let request = MessagesRequest {
        model,
        max_tokens: MAX_TOKENS,
        system: None,
        messages: vec![WireMessage { role: "user".to_string(), content: vec![WireBlock::text(prompt)] }],
        tools: Vec::new(),
        output_config: format.as_ref().map(|format| OutputConfig {
            format: OutputFormat { kind: format.kind, schema: format.schema.clone() },
        }),
    };

    let response = send(api_key, &request)?;

    if response.stop_reason.as_deref() == Some("refusal") {
        return Err("Claude declined this request".to_string());
    }

    response
        .content
        .into_iter()
        .find(|block| block.kind == "text")
        .and_then(|block| block.text)
        .map(|text| text.trim().to_string())
        .ok_or_else(|| "Claude returned no content".to_string())
}

fn send(api_key: &str, request: &MessagesRequest) -> Result<MessagesResponse, String> {
    let mut response = ureq::post(MESSAGES_ENDPOINT)
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .config()
        .timeout_global(Some(GENERATION_TIMEOUT))
        .build()
        .send_json(request)
        .map_err(|e| format!("Claude request failed: {e}"))?;

    response.body_mut().read_json().map_err(|e| format!("failed to parse Claude response: {e}"))
}

/// A cheap, side-effect-free way to confirm a key actually authenticates —
/// used by `please setup` to catch a bad key immediately instead of it
/// surfacing confusingly later, on the first real `please commit`.
pub fn validate_api_key(api_key: &str) -> Result<(), String> {
    fetch_models(api_key).map(|_| ())
}

/// Picks the lowest-cost Claude model available to this API key: prefers
/// the Haiku tier, then Sonnet, and falls back to a hardcoded model if
/// discovery fails.
pub fn select_lowest_cost_model(api_key: &str) -> String {
    match fetch_models(api_key) {
        Ok(models) => pick_lowest_cost_model(models).unwrap_or_else(|| FALLBACK_MODEL.to_string()),
        Err(err) => {
            eprintln!("Warning: model auto-detection failed ({err}). Using fallback model.");
            FALLBACK_MODEL.to_string()
        }
    }
}

fn fetch_models(api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let mut response = ureq::get(MODELS_ENDPOINT)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .config()
        .timeout_global(Some(LIST_MODELS_TIMEOUT))
        .build()
        .call()
        .map_err(|e| format!("failed to list models: {e}"))?;

    let parsed: ModelsListResponse =
        response.body_mut().read_json().map_err(|e| format!("failed to parse model list: {e}"))?;

    Ok(parsed.data)
}

fn pick_lowest_cost_model(models: Vec<ModelInfo>) -> Option<String> {
    let mut haiku: Vec<String> = models.iter().filter(|m| m.id.contains("haiku")).map(|m| m.id.clone()).collect();
    haiku.sort();
    if let Some(best) = haiku.pop() {
        return Some(best);
    }

    let mut sonnet: Vec<String> = models.into_iter().filter(|m| m.id.contains("sonnet")).map(|m| m.id).collect();
    sonnet.sort();
    sonnet.pop()
}

// --- Agent mode (tool use) --------------------------------------------------

fn to_wire_history(history: &[AgentMessage]) -> Vec<WireMessage> {
    history
        .iter()
        .map(|message| match message {
            AgentMessage::User(text) => {
                WireMessage { role: "user".to_string(), content: vec![WireBlock::text(text.clone())] }
            }
            AgentMessage::Model { calls, text } => {
                let mut content: Vec<WireBlock> = text.iter().map(|text| WireBlock::text(text.clone())).collect();
                content.extend(calls.iter().map(|call| WireBlock {
                    kind: "tool_use".to_string(),
                    id: Some(call.id.clone()),
                    name: Some(call.name.clone()),
                    input: Some(call.args.clone()),
                    ..Default::default()
                }));
                WireMessage { role: "assistant".to_string(), content }
            }
            AgentMessage::ToolResults(outcomes) => WireMessage {
                role: "user".to_string(),
                content: outcomes
                    .iter()
                    .map(|outcome| WireBlock {
                        kind: "tool_result".to_string(),
                        tool_use_id: Some(outcome.id.clone()),
                        content: Some(outcome.output.clone()),
                        ..Default::default()
                    })
                    .collect(),
            },
        })
        .collect()
}

/// Runs one turn of agent tool-use: sends the conversation so far plus the
/// tool catalog, and returns either the tool calls the model wants
/// executed, or its final answer.
pub fn agent_turn(
    system_prompt: &str,
    history: &[AgentMessage],
    tools: &[ToolSpec],
    api_key: &str,
    model: Option<&str>,
) -> Result<AgentTurn, String> {
    let model = model.map(str::to_string).unwrap_or_else(|| select_lowest_cost_model(api_key));

    let request = MessagesRequest {
        model: &model,
        max_tokens: MAX_TOKENS,
        system: Some(system_prompt),
        messages: to_wire_history(history),
        tools: tools
            .iter()
            .map(|tool| WireTool { name: tool.name, description: tool.description, input_schema: tool.parameters.clone() })
            .collect(),
        output_config: None,
    };

    let response = send(api_key, &request)?;

    if response.stop_reason.as_deref() == Some("refusal") {
        return Ok(AgentTurn::Final(
            "Claude declined this request for safety reasons.".to_string(),
        ));
    }

    let mut tool_calls = Vec::new();
    let mut text_parts = Vec::new();

    for block in response.content {
        match block.kind.as_str() {
            "tool_use" => tool_calls.push(ToolCall {
                id: block.id.unwrap_or_default(),
                name: block.name.unwrap_or_default(),
                args: block.input.unwrap_or_else(|| serde_json::json!({})),
                thought_signature: None,
            }),
            "text" => {
                if let Some(text) = block.text {
                    text_parts.push(text);
                }
            }
            _ => {}
        }
    }

    if tool_calls.is_empty() {
        Ok(AgentTurn::Final(text_parts.join("\n").trim().to_string()))
    } else {
        let text = (!text_parts.is_empty()).then(|| text_parts.join("\n").trim().to_string());
        Ok(AgentTurn::ToolCalls { calls: tool_calls, text })
    }
}

