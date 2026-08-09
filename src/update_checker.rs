use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Default)]
struct UpdateCache {
    last_checked_timestamp: u64,
    latest_version: Option<String>,
}

fn cache_path() -> PathBuf {
    crate::config::config_dir().join("update_cache.json")
}

pub fn check_and_notify() {
    let path = cache_path();
    let cache_data = fs::read_to_string(&path).unwrap_or_default();
    let cache: UpdateCache = serde_json::from_str(&cache_data).unwrap_or_default();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check if we should notify
    if let Some(latest) = &cache.latest_version {
        let current = env!("CARGO_PKG_VERSION");
        if is_newer(latest, current) {
            println!("\n✨ A new version of please is available (v{})! Run 'please update' to install.", latest);
        }
    }

    // Update the cache silently in background if older than 24h
    if now > cache.last_checked_timestamp + 24 * 60 * 60 {
        if let Ok(exe) = std::env::current_exe() {
            let _ = Command::new(exe).arg("--internal-update-check").spawn();
        }
    }
}

fn is_newer(latest: &str, current: &str) -> bool {
    let l_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
    let c_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    l_parts > c_parts
}

pub fn run_internal_check() {
    if let Ok(response) =
        ureq::get("https://api.github.com/repos/HylithLabs/please/releases/latest")
            .set("User-Agent", "please-cli")
            .call()
    {
        if let Ok(json) = response.into_json::<serde_json::Value>() {
            if let Some(tag) = json.get("tag_name").and_then(|s| s.as_str()) {
                let version = tag.trim_start_matches('v');
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let cache = UpdateCache {
                    last_checked_timestamp: now,
                    latest_version: Some(version.to_string()),
                };
                let _ = fs::create_dir_all(crate::config::config_dir());
                let _ = fs::write(
                    cache_path(),
                    serde_json::to_string(&cache).unwrap_or_default(),
                );
            }
        }
    }
}
