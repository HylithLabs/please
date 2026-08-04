use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn run(args: &[String]) {
    match args.first().map(String::as_str) {
        None => list(),
        Some("remove") => remove(&args[1..]),
        Some(name) => create(name),
    }
}

fn create(name: &str) {
    if let Err(err) = validate_name(name) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    let exe = current_exe();
    let link = exe.with_file_name(name);

    if let Some(existing_alias) = alias_target(&link, &exe) {
        if existing_alias {
            println!("`{name}` is already aliased to please.");
            return;
        }
        eprintln!(
            "'{name}' already exists at {} and isn't a please alias. Remove it yourself first \
             if you want to reuse that name.",
            link.display()
        );
        std::process::exit(1);
    }

    if let Some(shadowed) = find_on_path(name, exe.parent()) {
        eprintln!(
            "'{name}' is already a separate program at {}.",
            shadowed.display()
        );
        if !confirm("Alias it anyway? Whichever comes first on your PATH will win. [y/N]: ", false) {
            println!("Cancelled.");
            return;
        }
    }

    match create_symlink(&exe, &link) {
        Ok(()) => println!(
            "Done. `{name}` now runs please, exactly like typing `please` yourself. It's a \
             real command, not a shell alias, so it works in every shell and in scripts too."
        ),
        Err(err) => {
            eprintln!("Couldn't create '{name}': {err}");
            std::process::exit(1);
        }
    }
}

fn remove(args: &[String]) {
    let Some(name) = args.first() else {
        eprintln!("usage: please alias remove <name>");
        std::process::exit(1);
    };

    let exe = current_exe();
    let link = exe.with_file_name(name);

    match alias_target(&link, &exe) {
        Some(true) => {
            if let Err(err) = fs::remove_file(&link) {
                eprintln!("Couldn't remove '{name}': {err}");
                std::process::exit(1);
            }
            println!("Removed `{name}`.");
        }
        Some(false) => {
            eprintln!("'{name}' exists but isn't a please alias, so it was left alone.");
            std::process::exit(1);
        }
        None => {
            eprintln!("`{name}` isn't a please alias.");
            std::process::exit(1);
        }
    }
}

fn list() {
    let exe = current_exe();
    let Some(dir) = exe.parent() else {
        println!("No aliases set up yet.");
        return;
    };

    let Ok(entries) = fs::read_dir(dir) else {
        println!("No aliases set up yet.");
        return;
    };

    let mut aliases: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| alias_target(&entry.path(), &exe) == Some(true))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    if aliases.is_empty() {
        println!("No aliases set up yet. Run `please alias <name>` to add one, like `please alias pls`.");
        return;
    }

    aliases.sort();
    println!("Aliases for please:");
    for name in aliases {
        println!("  {name}");
    }
}

/// `Some(true)` if `path` is a please alias (a symlink pointing at `exe`),
/// `Some(false)` if something else entirely lives there (a plain file, a
/// symlink elsewhere, anything not ours), `None` if there's nothing there
/// at all.
fn alias_target(path: &Path, exe: &Path) -> Option<bool> {
    // `symlink_metadata` doesn't follow the link, so it succeeds for a
    // symlink, a regular file, or a broken symlink alike — exactly "is
    // there anything at all here" — and fails only when there truly isn't.
    fs::symlink_metadata(path).ok()?;

    let points_at_exe = fs::read_link(path).ok().and_then(|target| {
        let resolved = if target.is_absolute() { target } else { path.parent()?.join(target) };
        resolved.canonicalize().ok()
    }) == Some(exe.to_path_buf());

    Some(points_at_exe)
}

/// Checks every directory on `PATH` (other than the one please itself lives
/// in) for an existing program named `name` — so aliasing to something
/// generic doesn't silently shadow a real tool without a warning.
fn find_on_path(name: &str, own_dir: Option<&Path>) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .filter(|dir| Some(dir.as_path()) != own_dir)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Alias name can't be empty.".to_string());
    }
    if name == "please" {
        return Err("please is already called please.".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Alias name has to be a plain command name, not a path.".to_string());
    }
    Ok(())
}

fn current_exe() -> PathBuf {
    std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .expect("failed to resolve the please binary's own path")
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

fn confirm(label: &str, default_yes: bool) -> bool {
    print!("{label}");
    let _ = io::stdout().flush();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) | Err(_) => false,
        Ok(_) => match input.trim().to_lowercase().as_str() {
            "" => default_yes,
            answer => matches!(answer, "y" | "yes"),
        },
    }
}
