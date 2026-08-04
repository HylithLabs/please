use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("could not determine home directory");
    PathBuf::from(home).join(".please")
}

fn config_path() -> PathBuf {
    config_dir().join("config")
}

/// The active provider's resolved setup — what every AI feature actually
/// uses. Everywhere outside this file, "config" means this.
pub struct Config {
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
}

/// One entry in the saved-providers listing (`config::list`), for `please
/// setup` to display and let the developer pick from.
pub struct ProviderInfo {
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
    pub active: bool,
}

#[derive(Default, Clone)]
struct ProviderEntry {
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Default)]
struct Store {
    active: Option<String>,
    providers: BTreeMap<String, ProviderEntry>,
}

fn read_store() -> Store {
    let Ok(contents) = fs::read_to_string(config_path()) else {
        return Store::default();
    };

    // Pre-multi-provider files have no "active=" line — just bare
    // provider=/api_key=/model= keys for the one provider that existed.
    // Migrate them into the new format transparently, once.
    if !contents.lines().any(|line| line.starts_with("active=")) {
        if let Some(legacy) = parse_legacy(&contents) {
            let mut store = Store::default();
            store.active = Some(legacy.provider.clone());
            store.providers.insert(
                legacy.provider,
                ProviderEntry { api_key: Some(legacy.api_key), model: legacy.model },
            );
            let _ = write_store(&store);
            return store;
        }
    }

    let mut store = Store::default();
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("active=") {
            store.active = Some(value.to_string());
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let Some((provider, field)) = key.split_once('.') else { continue };
        let entry = store.providers.entry(provider.to_string()).or_default();
        match field {
            "api_key" => entry.api_key = Some(value.to_string()),
            "model" => entry.model = Some(value.to_string()),
            _ => {}
        }
    }
    store
}

fn parse_legacy(contents: &str) -> Option<Config> {
    let mut provider = None;
    let mut api_key = None;
    let mut model = None;

    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("provider=") {
            provider = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("api_key=") {
            api_key = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("model=") {
            model = Some(value.to_string());
        }
    }

    Some(Config { provider: provider?, api_key: api_key?, model })
}

fn write_store(store: &Store) -> io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;

    let mut contents = String::new();
    if let Some(active) = &store.active {
        contents.push_str(&format!("active={active}\n"));
    }
    for (provider, entry) in &store.providers {
        let Some(api_key) = &entry.api_key else { continue };
        contents.push_str(&format!("{provider}.api_key={api_key}\n"));
        if let Some(model) = &entry.model {
            contents.push_str(&format!("{provider}.model={model}\n"));
        }
    }

    let path = config_path();
    fs::write(&path, contents)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

/// Loads the currently active provider's config — the one `please` uses
/// for AI features unless the developer switches it. `None` if nothing has
/// been set up yet.
pub fn load() -> Option<Config> {
    let store = read_store();
    let active = store.active?;
    let entry = store.providers.get(&active)?;
    Some(Config { provider: active, api_key: entry.api_key.clone()?, model: entry.model.clone() })
}

/// Saves (or updates) one provider's key and model, and makes it the
/// active provider. Every other provider's saved key is left untouched —
/// this only ever touches `config.provider`'s own entry.
pub fn save(config: &Config) -> io::Result<()> {
    let mut store = read_store();
    store.providers.insert(
        config.provider.clone(),
        ProviderEntry { api_key: Some(config.api_key.clone()), model: config.model.clone() },
    );
    store.active = Some(config.provider.clone());
    write_store(&store)
}

/// Updates just a provider's saved model, leaving its key and the active
/// provider untouched. `None` clears it back to auto-selection.
pub fn update_model(provider: &str, model: Option<String>) -> Result<(), String> {
    let mut store = read_store();
    let entry = store.providers.get_mut(provider).filter(|entry| entry.api_key.is_some());
    let Some(entry) = entry else {
        return Err(format!("No saved key for '{provider}'."));
    };
    entry.model = model;
    write_store(&store).map_err(|e| e.to_string())
}

/// Every provider with a saved key, active one first, for `please setup`
/// to display and choose among.
pub fn list() -> Vec<ProviderInfo> {
    let store = read_store();
    let active = store.active.clone();

    let mut providers: Vec<ProviderInfo> = store
        .providers
        .into_iter()
        .filter_map(|(provider, entry)| {
            let api_key = entry.api_key?;
            let is_active = active.as_deref() == Some(provider.as_str());
            Some(ProviderInfo { provider, api_key, model: entry.model, active: is_active })
        })
        .collect();

    providers.sort_by_key(|p| !p.active);
    providers
}

/// Switches the active provider to one that already has a saved key,
/// without touching any stored key. Errors (unchanged) if it doesn't.
pub fn set_active(provider: &str) -> Result<(), String> {
    let mut store = read_store();
    let has_key = store.providers.get(provider).is_some_and(|entry| entry.api_key.is_some());
    if !has_key {
        return Err(format!("No saved key for '{provider}' yet — set one up first."));
    }
    store.active = Some(provider.to_string());
    write_store(&store).map_err(|e| e.to_string())
}

/// Deletes a saved provider's key. If it was the active provider, clears
/// the active pointer — the caller is responsible for prompting for a
/// replacement so `please` isn't left pointed at a provider with no key.
pub fn remove(provider: &str) -> Result<(), String> {
    let mut store = read_store();
    if store.providers.remove(provider).is_none() {
        return Err(format!("No saved key for '{provider}'."));
    }
    if store.active.as_deref() == Some(provider) {
        store.active = None;
    }
    write_store(&store).map_err(|e| e.to_string())
}
