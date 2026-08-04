mod gemini;

use crate::config::Config;

pub fn generate_commit_message(diff: &str, config: &Config) -> Result<String, String> {
    match config.provider.as_str() {
        "google" => gemini::generate_commit_message(diff, &config.api_key),
        other => Err(format!(
            "Provider '{other}' is not supported yet. Only 'google' (Gemini) is wired up so far."
        )),
    }
}
