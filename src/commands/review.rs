use crate::{config, git, llm};

pub fn run() {
    let diff = git::diff_head();
    if diff.trim().is_empty() {
        println!("No tracked changes to review.");
        return;
    }
    let Some(cfg) = config::load() else {
        crate::ui::error("No LLM provider configured. Run `please setup` first.");
        std::process::exit(1);
    };

    let prompt = format!(
        "Review the following current Git diff as a senior code reviewer. Explain what changed, identify concrete bugs, regressions, security issues, or suspicious modifications, and suggest focused tests. Separate findings by severity (high, medium, low). If there are no findings, say so clearly and still summarize the change and recommended tests. Do not invent files or behavior.\n\nDIFF:\n{diff}"
    );
    eprintln!("Reviewing changes...");
    let system = "You are an expert, pragmatic code reviewer. Base every claim on the supplied diff. Give concise, actionable feedback in Markdown.";
    match llm::run_agent(&prompt, &cfg, system, &[], |_name, _args| String::new()) {
        Ok(text) => crate::ui::print_markdown(&text),
        Err(err) => {
            crate::ui::error(&format!("review failed: {err}"));
            std::process::exit(1);
        }
    }
}
