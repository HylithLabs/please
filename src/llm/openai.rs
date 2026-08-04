use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{AgentMessage, AgentTurn, CommitGroup, CommitPlanOutcome, GenerationOutcome, ToolCall, ToolSpec};

const CHAT_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const MODELS_ENDPOINT: &str = "https://api.openai.com/v1/models";
const FALLBACK_MODEL: &str = "gpt-4o-mini";

const GENERATION_TIMEOUT: Duration = Duration::from_secs(90);
const LIST_MODELS_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Serialize, Deserialize, Clone)]
struct WireMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<WireToolCall>>,
    #[serde(rename = "tool_call_id", skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

impl WireMessage {
    fn user(text: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: Some(text.into()), tool_calls: None, tool_call_id: None }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct WireToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: WireFunctionCall,
}

#[derive(Serialize, Deserialize, Clone)]
struct WireFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<WireTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct WireTool {
    #[serde(rename = "type")]
    kind: &'static str,
    function: WireFunctionDeclaration,
}

#[derive(Serialize)]
struct WireFunctionDeclaration {
    name: &'static str,
    description: &'static str,
    parameters: serde_json::Value,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
    json_schema: JsonSchemaFormat,
}

#[derive(Serialize)]
struct JsonSchemaFormat {
    name: &'static str,
    schema: serde_json::Value,
    strict: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<WireToolCall>>,
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
    let format = ResponseFormat {
        kind: "json_schema",
        json_schema: JsonSchemaFormat {
            name: "commit_plan",
            schema: super::require_closed_objects(&super::commit_plan_schema()),
            strict: true,
        },
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

/// Calls ChatGPT with the given prompt. If the configured model fails (e.g.
/// it was retired), re-runs model discovery and retries once with the fresh
/// pick — mirrors the same self-healing behavior as the other adapters.
fn generate_with_retry(
    label: &str,
    prompt: &str,
    api_key: &str,
    model: Option<&str>,
    format: Option<ResponseFormat>,
) -> Result<GenerationOutcome, String> {
    let mut current_model = match model {
        Some(model) => model.to_string(),
        None => select_lowest_cost_model(api_key),
    };

    crate::ui::step(&format!("{label} (model: {current_model})"));

    match call_chatgpt(prompt, api_key, &current_model, &format) {
        Ok(message) => Ok(GenerationOutcome { message, model_used: current_model }),
        Err(err) => {
            crate::ui::warn(&format!("model '{current_model}' failed ({err}). Re-selecting a model."));
            current_model = select_lowest_cost_model(api_key);
            crate::ui::step(&format!("{label} (model: {current_model})"));
            let message = call_chatgpt(prompt, api_key, &current_model, &format)?;
            Ok(GenerationOutcome { message, model_used: current_model })
        }
    }
}

fn call_chatgpt(
    prompt: &str,
    api_key: &str,
    model: &str,
    format: &Option<ResponseFormat>,
) -> Result<String, String> {
    let request = ChatRequest {
        model,
        messages: vec![WireMessage::user(prompt)],
        tools: Vec::new(),
        response_format: format.as_ref().map(|format| ResponseFormat {
            kind: format.kind,
            json_schema: JsonSchemaFormat {
                name: format.json_schema.name,
                schema: format.json_schema.schema.clone(),
                strict: format.json_schema.strict,
            },
        }),
    };

    let response = send(api_key, &request)?;

    response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .map(|text| text.trim().to_string())
        .ok_or_else(|| "ChatGPT returned no content".to_string())
}

fn send(api_key: &str, request: &ChatRequest) -> Result<ChatResponse, String> {
    let mut response = ureq::post(CHAT_ENDPOINT)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {api_key}"))
        .config()
        .timeout_global(Some(GENERATION_TIMEOUT))
        .build()
        .send_json(request)
        .map_err(|e| format!("ChatGPT request failed: {e}"))?;

    response.body_mut().read_json().map_err(|e| format!("failed to parse ChatGPT response: {e}"))
}

/// A cheap, side-effect-free way to confirm a key actually authenticates —
/// used by `please setup` to catch a bad key immediately instead of it
/// surfacing confusingly later, on the first real `please commit`.
pub fn validate_api_key(api_key: &str) -> Result<(), String> {
    fetch_models(api_key).map(|_| ())
}

/// Picks the lowest-cost OpenAI chat model available to this API key:
/// prefers `nano`/`mini` tiers, skips non-chat models (embeddings, audio,
/// image, moderation), and falls back to a hardcoded model if discovery
/// fails.
pub fn select_lowest_cost_model(api_key: &str) -> String {
    match fetch_models(api_key) {
        Ok(models) => pick_lowest_cost_model(models).unwrap_or_else(|| FALLBACK_MODEL.to_string()),
        Err(err) => {
            crate::ui::warn(&format!("model auto-detection failed ({err}). Using fallback model."));
            FALLBACK_MODEL.to_string()
        }
    }
}

fn fetch_models(api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let mut response = ureq::get(MODELS_ENDPOINT)
        .header("Authorization", &format!("Bearer {api_key}"))
        .config()
        .timeout_global(Some(LIST_MODELS_TIMEOUT))
        .build()
        .call()
        .map_err(|e| format!("failed to list models: {e}"))?;

    let parsed: ModelsListResponse =
        response.body_mut().read_json().map_err(|e| format!("failed to parse model list: {e}"))?;

    Ok(parsed.data)
}

const NON_CHAT_MARKERS: [&str; 9] =
    ["embedding", "whisper", "tts", "dall-e", "moderation", "davinci", "babbage", "audio", "realtime"];

fn pick_lowest_cost_model(models: Vec<ModelInfo>) -> Option<String> {
    let chat_capable = |id: &str| !NON_CHAT_MARKERS.iter().any(|marker| id.contains(marker));

    let mut nano: Vec<String> =
        models.iter().filter(|m| chat_capable(&m.id) && m.id.contains("nano")).map(|m| m.id.clone()).collect();
    nano.sort();
    if let Some(best) = nano.pop() {
        return Some(best);
    }

    let mut mini: Vec<String> =
        models.into_iter().filter(|m| chat_capable(&m.id) && m.id.contains("mini")).map(|m| m.id).collect();
    mini.sort();
    mini.pop()
}

// --- Agent mode (tool use) --------------------------------------------------

fn to_wire_history(history: &[AgentMessage]) -> Vec<WireMessage> {
    history
        .iter()
        .flat_map(|message| -> Vec<WireMessage> {
            match message {
                AgentMessage::User(text) => vec![WireMessage::user(text.clone())],
                AgentMessage::Model { calls, text } => vec![WireMessage {
                    role: "assistant".to_string(),
                    content: text.clone(),
                    tool_calls: (!calls.is_empty()).then(|| {
                        calls
                            .iter()
                            .map(|call| WireToolCall {
                                id: call.id.clone(),
                                kind: "function".to_string(),
                                function: WireFunctionCall {
                                    name: call.name.clone(),
                                    arguments: call.args.to_string(),
                                },
                            })
                            .collect()
                    }),
                    tool_call_id: None,
                }],
                // Each tool result is its own "tool" message in this API,
                // unlike Gemini/Anthropic where they share one turn.
                AgentMessage::ToolResults(outcomes) => outcomes
                    .iter()
                    .map(|outcome| WireMessage {
                        role: "tool".to_string(),
                        content: Some(outcome.output.clone()),
                        tool_calls: None,
                        tool_call_id: Some(outcome.id.clone()),
                    })
                    .collect(),
            }
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

    let mut messages = vec![WireMessage {
        role: "system".to_string(),
        content: Some(system_prompt.to_string()),
        tool_calls: None,
        tool_call_id: None,
    }];
    messages.extend(to_wire_history(history));

    let request = ChatRequest {
        model: &model,
        messages,
        tools: tools
            .iter()
            .map(|tool| WireTool {
                kind: "function",
                function: WireFunctionDeclaration {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.parameters.clone(),
                },
            })
            .collect(),
        response_format: None,
    };

    let response = send(api_key, &request)?;

    let Choice { message, .. } = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| "ChatGPT returned no choices".to_string())?;

    let tool_calls: Vec<ToolCall> = message
        .tool_calls
        .unwrap_or_default()
        .into_iter()
        .map(|call| ToolCall {
            id: call.id,
            name: call.function.name,
            args: serde_json::from_str(&call.function.arguments).unwrap_or_else(|_| serde_json::json!({})),
            thought_signature: None,
        })
        .collect();

    if tool_calls.is_empty() {
        Ok(AgentTurn::Final(message.content.unwrap_or_default().trim().to_string()))
    } else {
        let text = message.content.map(|text| text.trim().to_string()).filter(|text| !text.is_empty());
        Ok(AgentTurn::ToolCalls { calls: tool_calls, text })
    }
}

