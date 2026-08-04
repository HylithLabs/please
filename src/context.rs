use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::git;
use crate::llm;

fn context_path() -> PathBuf {
    PathBuf::from(".git").join("PLEASE.MD")
}

/// Loads the cached codebase description from `.git/PLEASE.MD`, generating it
/// via the LLM on first use. Returns the description text and, if generating
/// it required re-selecting the model, the model that should be persisted.
pub fn load_or_generate(config: &Config) -> (Option<String>, Option<String>) {
    let path = context_path();

    if let Ok(existing) = fs::read_to_string(&path) {
        return (Some(existing), None);
    }

    let file_list = git::list_tracked_files();
    if file_list.trim().is_empty() {
        return (None, None);
    }

    match llm::describe_codebase(&file_list, config) {
        Ok(outcome) => {
            let _ = fs::write(&path, &outcome.message);

            let updated_model = if config.model.as_deref() != Some(outcome.model_used.as_str()) {
                Some(outcome.model_used)
            } else {
                None
            };

            (Some(outcome.message), updated_model)
        }
        Err(err) => {
            eprintln!("Warning: failed to generate project description ({err}).");
            (None, None)
        }
    }
}
