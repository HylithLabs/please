use crate::git;

pub fn run() {
    git::stage_all();
    let diff = git::diff_staged();

    if diff.is_empty() {
        println!("No changes to commit.");
        return;
    }

    print!("{diff}");
}
