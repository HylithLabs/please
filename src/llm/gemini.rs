use serde::{Deserialize, Serialize};
use std::time::Duration;

const MODELS_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const FALLBACK_MODEL: &str = "models/gemini-2.5-flash-lite";

// Generation calls can run long (Gemini's own "thinking" pass, out of our
// control) — bounded generously so a real stall fails fast instead of
// hanging forever. Model listing is a plain lookup and should always be quick.
const GENERATION_TIMEOUT: Duration = Duration::from_secs(90);
const LIST_MODELS_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Serialize)]
struct GenerateRequest<'a> {
    contents: Vec<Content<'a>>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Clone)]
struct GenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
    #[serde(rename = "responseSchema")]
    response_schema: serde_json::Value,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct GenerateResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Deserialize)]
struct ModelsListResponse {
    #[serde(default)]
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name: String,
    #[serde(default, rename = "supportedGenerationMethods")]
    supported_generation_methods: Vec<String>,
}

use super::{AgentMessage, AgentTurn, CommitGroup, CommitPlanOutcome, GenerationOutcome, ToolCall, ToolSpec};

pub fn describe_codebase(
    file_list: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<GenerationOutcome, String> {
    let prompt = super::build_description_prompt(file_list);
    generate_with_retry("Analyzing codebase", &prompt, api_key, model, None)
}

/// Lets the model decide how to split the working tree's changes into one or
/// more logically coherent commits, each with its own file list and message.
pub fn plan_commits(
    diff: &str,
    context: Option<&str>,
    api_key: &str,
    model: Option<&str>,
) -> Result<CommitPlanOutcome, String> {
    let prompt = super::build_commit_plan_prompt(diff, context);
    let generation_config = GenerationConfig {
        response_mime_type: "application/json".to_string(),
        response_schema: to_gemini_schema(&super::commit_plan_schema()),
    };

    let outcome = generate_with_retry(
        "Planning commits",
        &prompt,
        api_key,
        model,
        Some(generation_config),
    )?;

    let plan: RawCommitPlan = serde_json::from_str(&outcome.message)
        .map_err(|e| format!("failed to parse commit plan JSON: {e}"))?;

    if plan.commits.is_empty() {
        return Err("model returned an empty commit plan".to_string());
    }

    Ok(CommitPlanOutcome {
        commits: plan.commits,
        model_used: outcome.model_used,
    })
}

#[derive(Deserialize)]
struct RawCommitPlan {
    commits: Vec<CommitGroup>,
}

/// Calls Gemini with the given prompt. If the configured model fails (e.g. it
/// was deprecated/removed by Google), re-runs model discovery in the
/// background and retries once with the fresh pick.
fn generate_with_retry(
    label: &str,
    prompt: &str,
    api_key: &str,
    model: Option<&str>,
    generation_config: Option<GenerationConfig>,
) -> Result<GenerationOutcome, String> {
    let mut current_model = match model {
        Some(model) => model.to_string(),
        None => select_lowest_cost_model(api_key),
    };

    crate::ui::step(&format!("{label} (model: {current_model})"));

    match call_gemini(prompt, api_key, &current_model, generation_config.clone()) {
        Ok(message) => Ok(GenerationOutcome {
            message,
            model_used: current_model,
        }),
        Err(err) => {
            crate::ui::warn(&format!("model '{current_model}' failed ({err}). Re-selecting a model."));
            current_model = select_lowest_cost_model(api_key);
            crate::ui::step(&format!("{label} (model: {current_model})"));
            let message = call_gemini(prompt, api_key, &current_model, generation_config)?;
            Ok(GenerationOutcome {
                message,
                model_used: current_model,
            })
        }
    }
}

fn call_gemini(
    prompt: &str,
    api_key: &str,
    model: &str,
    generation_config: Option<GenerationConfig>,
) -> Result<String, String> {
    let request = GenerateRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
        generation_config,
    };

    let url = format!("https://generativelanguage.googleapis.com/v1beta/{model}:generateContent");

    let mut response = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("X-goog-api-key", api_key)
        .config()
        .timeout_global(Some(GENERATION_TIMEOUT))
        .build()
        .send_json(&request)
        .map_err(|e| format!("Gemini request failed: {e}"))?;

    let parsed: GenerateResponse = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("failed to parse Gemini response: {e}"))?;

    parsed
        .candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text.trim().to_string())
        .ok_or_else(|| "Gemini returned no content".to_string())
}

/// A cheap, side-effect-free way to confirm a key actually authenticates —
/// used by `please setup` to catch a bad key immediately instead of it
/// surfacing confusingly later, on the first real `please commit`.
pub fn validate_api_key(api_key: &str) -> Result<(), String> {
    fetch_models(api_key).map(|_| ())
}

/// Picks the lowest-cost Gemini model available to this API key: prefers
/// `-lite` variants over plain `flash`, skips `pro`/`vision` (higher-latency,
/// higher-cost tiers), and falls back to a hardcoded model if discovery fails.
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
        .header("X-goog-api-key", api_key)
        .config()
        .timeout_global(Some(LIST_MODELS_TIMEOUT))
        .build()
        .call()
        .map_err(|e| format!("failed to list models: {e}"))?;

    let parsed: ModelsListResponse = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("failed to parse model list: {e}"))?;

    Ok(parsed.models)
}

fn pick_lowest_cost_model(models: Vec<ModelInfo>) -> Option<String> {
    let mut lite_candidates = Vec::new();
    let mut flash_candidates = Vec::new();

    for model in models {
        if !model
            .supported_generation_methods
            .iter()
            .any(|method| method == "generateContent")
        {
            continue;
        }

        let name_lower = model.name.to_lowercase();

        // Skip heavy/high-latency tiers entirely.
        if name_lower.contains("pro") || name_lower.contains("vision") {
            continue;
        }

        if name_lower.contains("-lite") {
            lite_candidates.push(model.name);
        } else if name_lower.contains("flash") {
            flash_candidates.push(model.name);
        }
    }

    // Alphabetical sort puts the highest version number last (e.g. 3.5 after 2.5).
    lite_candidates.sort();
    if let Some(best) = lite_candidates.pop() {
        return Some(best);
    }

    flash_candidates.sort();
    flash_candidates.pop()
}

// --- Agent mode (function calling) -----------------------------------------
//
// Separate wire format from the plain-prompt calls above: agent turns need
// `systemInstruction`, `tools`, and round-tripped function call/response
// parts, none of which the simple `generateContent` calls use.

#[derive(Serialize, Deserialize, Clone, Default)]
struct WirePart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    function_call: Option<WireFunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    function_response: Option<WireFunctionResponse>,
    // "Thinking" models attach this to a functionCall part and require it
    // echoed back verbatim on the next turn, or they reject the request.
    #[serde(rename = "thoughtSignature", skip_serializing_if = "Option::is_none")]
    thought_signature: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WireFunctionCall {
    name: String,
    #[serde(default)]
    args: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WireFunctionResponse {
    name: String,
    response: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WireContent {
    role: String,
    parts: Vec<WirePart>,
}

#[derive(Serialize)]
struct AgentRequest {
    contents: Vec<WireContent>,
    #[serde(rename = "systemInstruction")]
    system_instruction: WireContent,
    tools: Vec<WireToolGroup>,
}

#[derive(Serialize)]
struct WireToolGroup {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<WireFunctionDeclaration>,
}

#[derive(Serialize)]
struct WireFunctionDeclaration {
    name: &'static str,
    description: &'static str,
    parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct AgentApiResponse {
    #[serde(default)]
    candidates: Vec<AgentCandidate>,
}

#[derive(Deserialize)]
struct AgentCandidate {
    content: WireContent,
}

/// JSON Schema uses lowercase type names (`"object"`, `"string"`, ...);
/// Gemini's function-declaration schema wants them uppercase. Everything
/// else about the schema passes through unchanged.
fn to_gemini_schema(schema: &serde_json::Value) -> serde_json::Value {
    match schema {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let converted = match (key.as_str(), value) {
                        ("type", serde_json::Value::String(t)) => {
                            serde_json::Value::String(t.to_uppercase())
                        }
                        _ => to_gemini_schema(value),
                    };
                    (key.clone(), converted)
                })
                .collect(),
        ),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(to_gemini_schema).collect())
        }
        other => other.clone(),
    }
}

fn to_wire_history(history: &[AgentMessage]) -> Vec<WireContent> {
    history
        .iter()
        .map(|message| match message {
            AgentMessage::User(text) => WireContent {
                role: "user".to_string(),
                parts: vec![WirePart {
                    text: Some(text.clone()),
                    ..Default::default()
                }],
            },
            AgentMessage::Model { calls, text } => {
                let mut parts: Vec<WirePart> = text
                    .iter()
                    .map(|text| WirePart {
                        text: Some(text.clone()),
                        ..Default::default()
                    })
                    .collect();

                parts.extend(calls.iter().map(|call| WirePart {
                    function_call: Some(WireFunctionCall {
                        name: call.name.clone(),
                        args: call.args.clone(),
                        id: Some(call.id.clone()),
                    }),
                    thought_signature: call.thought_signature.clone(),
                    ..Default::default()
                }));

                WireContent { role: "model".to_string(), parts }
            }
            // Gemini doesn't accept a "function" role for tool results — the
            // response turn is sent back as "user", same as a normal reply.
            AgentMessage::ToolResults(outcomes) => WireContent {
                role: "user".to_string(),
                parts: outcomes
                    .iter()
                    .map(|outcome| WirePart {
                        function_response: Some(WireFunctionResponse {
                            name: outcome.name.clone(),
                            response: serde_json::json!({ "result": outcome.output }),
                            id: Some(outcome.id.clone()),
                        }),
                        ..Default::default()
                    })
                    .collect(),
            },
        })
        .collect()
}

/// Runs one turn of agent function-calling: sends the conversation so far
/// plus the tool catalog, and returns either the tool calls the model wants
/// executed, or its final answer.
pub fn agent_turn(
    system_prompt: &str,
    history: &[AgentMessage],
    tools: &[ToolSpec],
    api_key: &str,
    model: Option<&str>,
) -> Result<AgentTurn, String> {
    let model = model
        .map(str::to_string)
        .unwrap_or_else(|| select_lowest_cost_model(api_key));

    let request = AgentRequest {
        contents: to_wire_history(history),
        system_instruction: WireContent {
            role: "system".to_string(),
            parts: vec![WirePart {
                text: Some(system_prompt.to_string()),
                ..Default::default()
            }],
        },
        tools: vec![WireToolGroup {
            function_declarations: tools
                .iter()
                .map(|tool| WireFunctionDeclaration {
                    name: tool.name,
                    description: tool.description,
                    parameters: to_gemini_schema(&tool.parameters),
                })
                .collect(),
        }],
    };

    let url = format!("https://generativelanguage.googleapis.com/v1beta/{model}:generateContent");

    let mut response = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("X-goog-api-key", api_key)
        .config()
        .timeout_global(Some(GENERATION_TIMEOUT))
        .build()
        .send_json(&request)
        .map_err(|e| format!("Gemini request failed: {e}"))?;

    let parsed: AgentApiResponse = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("failed to parse Gemini response: {e}"))?;

    let content = parsed
        .candidates
        .into_iter()
        .next()
        .map(|c| c.content)
        .ok_or_else(|| "Gemini returned no content".to_string())?;

    let mut tool_calls = Vec::new();
    let mut text_parts = Vec::new();

    for (i, part) in content.parts.into_iter().enumerate() {
        if let Some(call) = part.function_call {
            tool_calls.push(ToolCall {
                id: call.id.unwrap_or_else(|| format!("call_{i}")),
                name: call.name,
                args: call.args,
                thought_signature: part.thought_signature,
            });
        } else if let Some(text) = part.text {
            text_parts.push(text);
        }
    }

    if tool_calls.is_empty() {
        Ok(AgentTurn::Final(text_parts.join("\n").trim().to_string()))
    } else {
        let text = (!text_parts.is_empty()).then(|| text_parts.join("\n").trim().to_string());
        Ok(AgentTurn::ToolCalls { calls: tool_calls, text })
    }
}
