use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn default_remote_host() -> String {
    "git-PM7".into()
}
fn default_remote_path() -> String {
    "repos".into()
}
fn default_pull_rebase() -> bool {
    true
}
fn default_activity_show() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub github: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SodiumConfig {
    pub dev_root: String,
    #[serde(default = "default_remote_host")]
    pub remote_host: String,
    #[serde(default = "default_remote_path")]
    pub remote_path: String,
    #[serde(default = "default_pull_rebase")]
    pub pull_rebase: bool,
    #[serde(default = "default_activity_show")]
    pub activity_show: bool,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub projects: Option<HashMap<String, ProjectConfig>>,
}

impl SodiumConfig {
    /// Resolve dev_root with tilde expansion into an absolute PathBuf.
    pub fn dev_root_path(&self) -> PathBuf {
        expand_tilde(&self.dev_root)
    }

    /// Return the GitHub mirror URL for a project, if configured.
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

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("sodium").join("sodium.toml"))
}

/// Load config from ~/.config/sodium/sodium.toml.
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
            pull_rebase: default_pull_rebase(),
            activity_show: default_activity_show(),
            exclude: Vec::new(),
            projects: None,
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(toml_str) = toml::to_string_pretty(&default) {
            let _ = fs::write(&path, toml_str);
        }
        return Some(default);
    }

    let content = fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}
