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

use super::{CommitGroup, CommitPlanOutcome, GenerationOutcome};

pub fn describe_codebase(
    file_list: &str,
    api_key: &str,
    model: Option<&str>,
) -> Result<GenerationOutcome, String> {
    let prompt = build_description_prompt(file_list);
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
    let prompt = build_commit_plan_prompt(diff, context);
    let generation_config = GenerationConfig {
        response_mime_type: "application/json".to_string(),
        response_schema: commit_plan_schema(),
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

fn commit_plan_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "OBJECT",
        "properties": {
            "commits": {
                "type": "ARRAY",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "files": {
                            "type": "ARRAY",
                            "items": { "type": "STRING" }
                        },
                        "message": { "type": "STRING" }
                    },
                    "required": ["files", "message"]
                }
            }
        },
        "required": ["commits"]
    })
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

    eprintln!("{label} (model: {current_model})...");

    match call_gemini(prompt, api_key, &current_model, generation_config.clone()) {
        Ok(message) => Ok(GenerationOutcome {
            message,
            model_used: current_model,
        }),
        Err(err) => {
            eprintln!(
                "Warning: model '{current_model}' failed ({err}). Re-selecting a model..."
            );
            current_model = select_lowest_cost_model(api_key);
            eprintln!("{label} (model: {current_model})...");
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

/// Picks the lowest-cost Gemini model available to this API key: prefers
/// `-lite` variants over plain `flash`, skips `pro`/`vision` (higher-latency,
/// higher-cost tiers), and falls back to a hardcoded model if discovery fails.
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
