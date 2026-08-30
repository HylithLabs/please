use crate::context;

pub fn run(args: &[String]) {
    if args.iter().any(|a| a == "--refresh") {
        let path = context::context_path();
        let _ = std::fs::remove_file(path);
        if let Some(cfg) = crate::config::load() {
            let (text, _) = context::load_or_generate(&cfg);
            if text.is_some() {
                println!("Project context regenerated.");
            } else {
                println!("Project context cleared; no description could be generated.");
            }
        } else {
            println!("Project context cache cleared; configure an AI provider to regenerate it.");
        }
        return;
    }
    match std::fs::read_to_string(context::context_path()) {
        Ok(text) => println!("{text}"),
        Err(_) => println!(
            "No project context cached yet. Run `please context --refresh` or an AI command."
        ),
    }
    if let Ok(text) = std::fs::read_to_string(".please/instructions.md") {
        println!("\nProject instructions (.please/instructions.md):\n{text}");
    }
}
