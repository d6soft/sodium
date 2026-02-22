use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_remote_host() -> String {
    "git-PM7".into()
}
fn default_remote_path() -> String {
    "repos".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SodiumConfig {
    pub dev_root: String,
    #[serde(default = "default_remote_host")]
    pub remote_host: String,
    #[serde(default = "default_remote_path")]
    pub remote_path: String,
}

impl SodiumConfig {
    /// Resolve dev_root with tilde expansion into an absolute PathBuf.
    pub fn dev_root_path(&self) -> PathBuf {
        expand_tilde(&self.dev_root)
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("sodium").join("sodium.yaml"))
}

/// Load config from ~/.config/sodium/sodium.yaml.
/// Creates a default config if the file doesn't exist.
/// Returns None if config dir can't be determined or file is broken.
pub fn load_config() -> Option<SodiumConfig> {
    let path = config_path()?;

    if !path.exists() {
        // Create default config
        let default = SodiumConfig {
            dev_root: "~/dev".into(),
            remote_host: default_remote_host(),
            remote_path: default_remote_path(),
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(yaml) = serde_yaml::to_string(&default) {
            let _ = fs::write(&path, yaml);
        }
        return Some(default);
    }

    let content = fs::read_to_string(&path).ok()?;
    serde_yaml::from_str(&content).ok()
}
