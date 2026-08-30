use crate::git;

pub fn run(args: &[String]) {
    let purpose = args.join(" ").trim().to_string();
    if purpose.is_empty() {
        eprintln!("usage: please start \"<branch purpose>\"");
        std::process::exit(1);
    }
    let slug = purpose
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .take(6)
        .collect::<Vec<_>>()
        .join("-");
    let name = format!("feature/{slug}");
    match git::create_and_switch_branch(&name) {
        Ok(()) => {
            let _ = std::fs::create_dir_all(".please");
            let _ = std::fs::write(".please/branch-purpose", &purpose);
            println!("Created '{name}'. Purpose: {purpose}");
        }
        Err(err) => {
            eprintln!("Failed to create branch: {err}");
            std::process::exit(1);
        }
    }
}
