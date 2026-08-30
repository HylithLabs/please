use crate::{git, ui};

/// Changes origin, optionally publishing every local branch and tag to it.
pub fn run(args: &[String]) {
    let (push_history, url) = match args {
        [origin, url] if origin == "origin" => (false, url.as_str()),
        [origin, and, push, url] if origin == "origin" && and == "and" && push == "push" => {
            (true, url.as_str())
        }
        _ => {
            eprintln!(
                "usage: please change origin <url>\n       please change origin and push <url>"
            );
            std::process::exit(1);
        }
    };

    let action = if push_history {
        format!("Change origin to {url} and push all branches and tags?")
    } else {
        format!("Change origin to {url}?")
    };
    if !ui::confirm(&action) {
        ui::warn("Skipped changing origin.");
        return;
    }

    let result = if git::has_remote("origin") {
        git::set_remote_url("origin", url)
    } else {
        git::add_remote("origin", url)
    };
    if let Err(err) = result {
        ui::error(&format!("failed to change origin: {err}"));
        std::process::exit(1);
    }
    ui::success(&format!("Origin is now {url}."));

    if push_history {
        ui::step("Pushing all branches and tags to the new origin");
        if !git::push_all_history() {
            ui::error("origin changed, but some history could not be pushed.");
            std::process::exit(1);
        }
        ui::success("All local branches and tags were pushed to the new origin.");
    }
}
