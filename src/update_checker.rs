use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

/// The update check runs like the rest of `please`: only when the developer
/// runs a command, never on a timer. Each run shows whatever the last
/// background check already found (instant, from cache) and then kicks off a
/// fresh detached check for next time. There is no scheduled job and no
/// interval, so nothing happens while `please` is idle.
#[derive(Serialize, Deserialize, Default)]
struct UpdateCache {
    latest_version: Option<String>,
    ignored_version: Option<String>,
}

fn cache_path() -> PathBuf {
    crate::config::config_dir().join("update_cache.json")
}

fn read_cache() -> UpdateCache {
    fs::read_to_string(cache_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write_cache(cache: &UpdateCache) {
    let _ = fs::create_dir_all(crate::config::config_dir());
    if let Ok(json) = serde_json::to_string(cache) {
        let _ = fs::write(cache_path(), json);
    }
}

/// Opt out the way every other CLI's update notifier lets you: an env var,
/// plus staying quiet when output isn't a real terminal (CI, pipes, cron).
fn checks_disabled() -> bool {
    std::env::var_os("NO_UPDATE_NOTIFIER").is_some()
        || std::env::var_os("CI").is_some()
        || !std::io::stderr().is_terminal()
}

/// Runs on every command except `please update` itself. Two independent
/// jobs: show the reminder from whatever the last background check already
/// found (instant, no network), and kick off a fully detached background
/// process to refresh that result for next time. The current command never
/// waits on the network, and nothing runs unless the developer ran a
/// command to trigger it.
pub fn check_and_notify() {
    if checks_disabled() {
        return;
    }

    let cache = read_cache();

    if let Some(latest) = &cache.latest_version
        && cache.ignored_version.as_ref() != Some(latest)
        && is_newer(latest, env!("CARGO_PKG_VERSION"))
    {
        eprintln!(
            "\n\x1b[1;33mA new version of please is available:\x1b[0m \
             \x1b[36mv{}\x1b[0m \u{2192} \x1b[36mv{latest}\x1b[0m\n\
             Run \x1b[1mplease update\x1b[0m to install, or \x1b[1mplease update ignore\x1b[0m to hide this.",
            env!("CARGO_PKG_VERSION"),
        );
    }

    spawn_background_refresh();
}

/// Spawns `please --internal-update-check` as a detached child: no shared
/// stdio, its own process group/session, no console window on Windows. The
/// parent returns immediately and the OS reparents and reaps the child, so
/// there is nothing to wait on and no zombie left behind.
fn spawn_background_refresh() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    let mut command = Command::new(exe);
    command
        .arg("--internal-update-check")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: no console window flashes for the child. The
        // parent exiting doesn't take it down, and with stdio nulled there is
        // nothing to keep the parent's console alive for.
        command.creation_flags(0x0800_0000);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Own session, so a Ctrl-C in the parent's shell doesn't reach it.
        unsafe {
            command.pre_exec(|| {
                libc_setsid();
                Ok(())
            });
        }
    }

    let _ = command.spawn();
}

#[cfg(unix)]
fn libc_setsid() {
    // Avoid a libc dependency for one syscall.
    unsafe extern "C" {
        fn setsid() -> i32;
    }
    unsafe {
        setsid();
    }
}

fn is_newer(latest: &str, current: &str) -> bool {
    let parts = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };
    parts(latest) > parts(current)
}

pub fn ignore_latest_version() {
    let mut cache = read_cache();
    match &cache.latest_version {
        Some(latest) => {
            let latest = latest.clone();
            cache.ignored_version = Some(latest.clone());
            write_cache(&cache);
            crate::ui::success(&format!("Hiding the update notice for v{latest}."));
        }
        None => crate::ui::success("No update notice to hide yet."),
    }
}

/// The detached child. Fetches the latest release tag and rewrites the
/// cache, preserving whatever version the developer chose to ignore.
pub fn run_internal_check() {
    let Ok(mut response) =
        ureq::get("https://api.github.com/repos/HylithLabs/please/releases/latest")
            .header("User-Agent", "please-cli")
            .call()
    else {
        return;
    };
    let Ok(body) = response.body_mut().read_to_string() else {
        return;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) else {
        return;
    };
    let Some(tag) = json.get("tag_name").and_then(|s| s.as_str()) else {
        return;
    };

    let mut cache = read_cache();
    cache.latest_version = Some(tag.trim_start_matches('v').to_string());
    write_cache(&cache);
}
