const SENSITIVE_NAMES: &[&str] = &[
    "credentials.json",
    "secrets.yml",
    "secrets.yaml",
    "secrets.json",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
];

const SENSITIVE_EXTENSIONS: &[&str] = &["pem", "key", "pfx", "p12", "keystore", "jks"];

const SAFE_ENV_SUFFIXES: &[&str] = &[".example", ".sample", ".template"];

/// Whether a file path looks like a secret/credential file that `please`
/// should not silently auto-stage (`.env`, private keys, credential dumps).
/// Files the user has already staged themselves bypass this check entirely.
pub fn is_sensitive(path: &str) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path).to_lowercase();

    if file_name.starts_with(".env") {
        return !SAFE_ENV_SUFFIXES
            .iter()
            .any(|suffix| file_name.ends_with(suffix));
    }

    if SENSITIVE_NAMES.contains(&file_name.as_str()) {
        return true;
    }

    match file_name.rsplit_once('.') {
        Some((_, ext)) => SENSITIVE_EXTENSIONS.contains(&ext),
        None => false,
    }
}
