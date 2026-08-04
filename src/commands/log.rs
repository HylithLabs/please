use crate::git;

pub fn run() {
    if !git::log_graph() {
        eprintln!("Failed to read git log.");
        std::process::exit(1);
    }
}
