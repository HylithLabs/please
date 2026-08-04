use serde::{Deserialize, Serialize};

const ENDPOINT: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-latest:generateContent";

#[derive(Serialize)]
struct GenerateRequest<'a> {
    contents: Vec<Content<'a>>,
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

pub fn generate_commit_message(diff: &str, api_key: &str) -> Result<String, String> {
    let prompt = build_prompt(diff);

    let request = GenerateRequest {
        contents: vec![Content {
            parts: vec![Part { text: &prompt }],
        }],
    };

    let mut response = ureq::post(ENDPOINT)
        .header("Content-Type", "application/json")
        .header("X-goog-api-key", api_key)
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

fn build_prompt(diff: &str) -> String {
    format!(
        "You are generating a git commit message. Write a concise, conventional-commit style \
         message (a short summary line, optionally followed by a brief body) describing the \
         following diff. Output only the commit message itself, with no markdown formatting, \
         no code fences, and no explanation.\n\nDiff:\n{diff}"
    )
}
