//! Deploy pipeline (TT1756).
//!
//! Lot A : parser .sodium-deploy.toml, lecteur deploy/state.json,
//! sous-dispatcher `sodium deploy <status|init>`.
//!
//! Module monofichier pour ce lot ; éclatement en dossier src/deploy/ au lot B.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cli;

// ─── Config (.sodium-deploy.toml) ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub deploy: DeploySection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploySection {
    pub env_order: Vec<String>,
    #[serde(default)]
    pub components: BTreeMap<String, ComponentConfig>,
    #[serde(default)]
    pub envs: BTreeMap<String, EnvConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub build: String,
    pub package: String,
    pub test: String,
    pub deploy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    pub host: String,
    pub container: String,
}

// ─── State (deploy/state.json) ────────────────────────────────────────────

/// component name → version (YY.MM.DDII) → state for that version.
pub type DeployState = BTreeMap<String, BTreeMap<String, VersionState>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_at: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub envs: BTreeMap<String, EnvState>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smoke_test: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied_from: Option<String>,
}

// ─── Paths ────────────────────────────────────────────────────────────────

pub fn config_path(repo: &Path) -> PathBuf {
    repo.join(".sodium-deploy.toml")
}

pub fn state_path(repo: &Path) -> PathBuf {
    repo.join("deploy").join("state.json")
}

// ─── Loaders & validation ─────────────────────────────────────────────────

pub fn load_config(repo: &Path) -> Result<DeployConfig, String> {
    let path = config_path(repo);
    if !path.exists() {
        return Err(format!(".sodium-deploy.toml not found at {}", path.display()));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    let cfg: DeployConfig = toml::from_str(&content)
        .map_err(|e| format!("invalid {}: {}", path.display(), e))?;
    validate_config(&cfg)?;
    Ok(cfg)
}

fn validate_config(cfg: &DeployConfig) -> Result<(), String> {
    let s = &cfg.deploy;
    if s.env_order.is_empty() {
        return Err("env_order must not be empty".into());
    }
    if s.components.is_empty() {
        return Err("at least one [deploy.components.<name>] must be declared".into());
    }
    if s.envs.is_empty() {
        return Err("at least one [deploy.envs.<name>] must be declared".into());
    }
    let order_set: BTreeSet<&String> = s.env_order.iter().collect();
    let env_keys: BTreeSet<&String> = s.envs.keys().collect();
    if order_set != env_keys {
        let missing: Vec<&String> = s.env_order.iter().filter(|n| !s.envs.contains_key(n.as_str())).collect();
        let extra: Vec<&String> = s.envs.keys().filter(|n| !s.env_order.contains(n)).collect();
        return Err(format!(
            "env_order must match [deploy.envs] keys (declared in env_order but undefined: {:?}, defined but missing from env_order: {:?})",
            missing, extra
        ));
    }
    Ok(())
}

pub fn load_state(repo: &Path) -> Result<DeployState, String> {
    let path = state_path(repo);
    if !path.exists() {
        return Ok(DeployState::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("invalid {}: {}", path.display(), e))
}

// ─── Dispatcher ───────────────────────────────────────────────────────────

pub fn dispatch(args: &[String]) -> bool {
    let action: &'static str = match args.first().map(|s| s.as_str()) {
        Some("status") => "deploy:status",
        Some("init") => "deploy:init",
        None => {
            cli::emit_err("deploy", "usage: sodium deploy <status|init> [--path <dir>]");
            exit(2);
        }
        Some(other) => {
            cli::emit_err(
                "deploy",
                &format!("unknown subcommand: {} (available: status, init)", other),
            );
            exit(2);
        }
    };
    match action {
        "deploy:status" => run_status(&args[1..]),
        "deploy:init" => run_init(&args[1..]),
        _ => unreachable!(),
    }
    true
}

// ─── Sous-commandes ───────────────────────────────────────────────────────

fn run_status(args: &[String]) {
    let action = "deploy:status";
    let (path, _rest) = cli::parse_path_flag(args);
    let repo = cli::resolve_repo(path, action);

    let config = match load_config(&repo) {
        Ok(c) => c,
        Err(e) => {
            cli::emit_err(action, &e);
            exit(1);
        }
    };
    let state = match load_state(&repo) {
        Ok(s) => s,
        Err(e) => {
            cli::emit_err(action, &e);
            exit(1);
        }
    };

    let data = json!({
        "repo": repo.display().to_string(),
        "config": config,
        "state": state,
    });
    cli::emit_ok(action, "status loaded", Some(data));
}

fn run_init(args: &[String]) {
    let action = "deploy:init";
    let (path, _rest) = cli::parse_path_flag(args);
    let repo = cli::resolve_repo(path, action);
    let cfg_path = config_path(&repo);

    if cfg_path.exists() {
        cli::emit_err(
            action,
            &format!(".sodium-deploy.toml already exists at {}", cfg_path.display()),
        );
        exit(1);
    }

    let project_name = repo
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    let template = format!(
        r#"# Sodium deploy config — {project}
# Format VERSION imposé : YY.MM.DDII (bumpé uniquement en livraison DEV).
# Doc : ~/dev/obsidian/projets/Sodium/pages/deploy-pipeline.md

[deploy]
# Ordre linéaire de promotion. Doit matcher les clés [deploy.envs.<name>].
env_order = ["dev", "prod"]

# ─── Composants livrables ─────────────────────────────────────────────────
# Plusieurs composants possibles (front, api, app…). Chaque composant déclare
# ses scripts ; Sodium les exécute avec SODIUM_ENV, SODIUM_HOST, SODIUM_CONTAINER,
# SODIUM_ARTIFACT_PATH, SODIUM_ARTIFACT_OUT en environnement.

[deploy.components.front]
build   = "deploy/build-front.sh"
package = "deploy/package-front.sh"
test    = "deploy/test-front.sh"
deploy  = "deploy/deploy-front.sh"

# ─── Environnements (host SSH + container LXC) ────────────────────────────
# Sodium invoque : ssh <host> 'incus exec <container> -- …'

[deploy.envs.dev]
host = "TODO"
container = "TODO"

[deploy.envs.prod]
host = "TODO"
container = "TODO"
"#,
        project = project_name
    );

    if let Err(e) = fs::write(&cfg_path, template) {
        cli::emit_err(
            action,
            &format!("failed to write {}: {}", cfg_path.display(), e),
        );
        exit(1);
    }

    cli::emit_ok(
        action,
        format!(".sodium-deploy.toml created at {}", cfg_path.display()),
        Some(json!({
            "path": cfg_path.display().to_string(),
            "next_steps": [
                "edit [deploy.envs.<env>] host and container",
                "create deploy/{build,package,test,deploy}-<component>.sh scripts",
                "run `sodium deploy status` to verify"
            ]
        })),
    );
}
