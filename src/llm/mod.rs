mod gemini;

use crate::config::Config;

pub struct GenerationOutcome {
    pub message: String,
    pub model_used: String,
}

pub fn generate_commit_message(
    diff: &str,
    context: Option<&str>,
    config: &Config,
) -> Result<GenerationOutcome, String> {
    match config.provider.as_str() {
        "google" => gemini::generate_commit_message(
            diff,
            context,
            &config.api_key,
            config.model.as_deref(),
        ),
        other => Err(format!(
            "Provider '{other}' is not supported yet. Only 'google' (Gemini) is wired up so far."
        )),
    }
}

pub fn describe_codebase(file_list: &str, config: &Config) -> Result<GenerationOutcome, String> {
    match config.provider.as_str() {
        "google" => gemini::describe_codebase(file_list, &config.api_key, config.model.as_deref()),
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
