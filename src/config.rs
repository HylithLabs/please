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

pub struct Config {
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
}

pub fn save(config: &Config) -> io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;

    let path = config_path();
    let mut contents = format!("provider={}\napi_key={}\n", config.provider, config.api_key);
    if let Some(model) = &config.model {
        contents.push_str(&format!("model={model}\n"));
    }
    fs::write(&path, contents)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn load() -> Option<Config> {
    let contents = fs::read_to_string(config_path()).ok()?;

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

    Some(Config {
        provider: provider?,
        api_key: api_key?,
        model,
    })
}
