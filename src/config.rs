use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub github: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SodiumConfig {
    pub dev_root: String,
    pub remote_host: String,
    pub remote_path: String,
    pub pull_rebase: bool,
    pub activity_show: bool,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub projects: Option<HashMap<String, ProjectConfig>>,
}

impl SodiumConfig {
    pub fn dev_root_path(&self) -> PathBuf {
        expand_tilde(&self.dev_root)
    }

    pub fn github_url(&self, project_name: &str) -> Option<&str> {
        self.projects
            .as_ref()?
            .get(project_name)?
            .github
            .as_deref()
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

fn config_path() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|d| d.join("sodium").join("sodium.toml"))
        .ok_or_else(|| "unable to determine config directory".into())
}

/// Load config from ~/.config/sodium/sodium.toml.
/// Strict : fails if the file is missing, unreadable, or has missing fields.
pub fn load_config() -> Result<SodiumConfig, String> {
    let path = config_path()?;

    if !path.exists() {
        return Err(format!(
            "config file not found: {}\n\
             Create it with all required fields: dev_root, remote_host, remote_path, pull_rebase, activity_show.",
            path.display()
        ));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    toml::from_str(&content)
        .map_err(|e| format!("invalid config {}: {}", path.display(), e))
}
