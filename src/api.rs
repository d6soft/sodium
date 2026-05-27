use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::{audit, config, git, git_ops};

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ApiRequest {
    Status { path: Option<String> },
    Branches { path: Option<String> },
    Files { path: Option<String> },
    Gitcon { path: Option<String> },
    Projects,
    Fetch { path: Option<String> },
    Pull { path: Option<String>, rebase: Option<bool> },
    Push { path: Option<String> },
    Backup { path: Option<String> },
    Commit { path: Option<String>, message: String, files: Option<Vec<String>> },
    NewBranch { path: Option<String>, name: String },
    SwitchBranch { path: Option<String>, branch: String },
    MergeIntoMain { path: Option<String>, branch: String },
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ApiResponse {
    pub fn ok_msg(msg: impl Into<String>) -> Self {
        Self { ok: true, action: None, message: Some(msg.into()), data: None, error: None }
    }
    pub fn ok_with(msg: impl Into<String>, data: Value) -> Self {
        Self { ok: true, action: None, message: Some(msg.into()), data: Some(data), error: None }
    }
    pub fn ok_data(data: Value) -> Self {
        Self { ok: true, action: None, message: None, data: Some(data), error: None }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, action: None, message: None, data: None, error: Some(msg.into()) }
    }
    pub fn with_action(mut self, action: &'static str) -> Self {
        self.action = Some(action);
        self
    }
}

fn resolve_path(req_path: &Option<String>, default: &Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = req_path {
        let expanded = if p.starts_with("~/") {
            dirs::home_dir()
                .map(|h| h.join(&p[2..]))
                .unwrap_or_else(|| PathBuf::from(p))
        } else {
            PathBuf::from(p)
        };
        if expanded.is_dir() {
            Ok(expanded)
        } else {
            Err(format!("Path not found: {}", p))
        }
    } else if let Some(d) = default {
        Ok(d.clone())
    } else {
        Err("No path specified and no default path".into())
    }
}

fn handle_request(req: ApiRequest, default_path: &Option<PathBuf>) -> ApiResponse {
    let (action, req_path_str, args_repr) = describe_request(&req);
    let audit_repo = req_path_str
        .as_ref()
        .and_then(|p| resolve_path(&Some(p.clone()), default_path).ok())
        .or_else(|| default_path.clone())
        .unwrap_or_else(|| PathBuf::from("-"));

    let response = dispatch(req, default_path).with_action(action);

    let result: Result<&str, &str> = if response.ok {
        Ok("")
    } else {
        Err(response.error.as_deref().unwrap_or(""))
    };
    audit::log("api", &audit_repo, action, &args_repr, result);

    response
}

fn describe_request(req: &ApiRequest) -> (&'static str, Option<String>, String) {
    match req {
        ApiRequest::Status { path } => ("status", path.clone(), String::new()),
        ApiRequest::Branches { path } => ("branches", path.clone(), String::new()),
        ApiRequest::Files { path } => ("files", path.clone(), String::new()),
        ApiRequest::Gitcon { path } => ("gitcon", path.clone(), String::new()),
        ApiRequest::Projects => ("projects", None, String::new()),
        ApiRequest::Fetch { path } => ("fetch", path.clone(), String::new()),
        ApiRequest::Pull { path, rebase } => (
            "pull",
            path.clone(),
            format!("rebase={}", rebase.unwrap_or(true)),
        ),
        ApiRequest::Push { path } => ("push", path.clone(), String::new()),
        ApiRequest::Backup { path } => ("backup", path.clone(), String::new()),
        ApiRequest::Commit { path, message, .. } => ("commit", path.clone(), message.clone()),
        ApiRequest::NewBranch { path, name } => ("new-branch", path.clone(), name.clone()),
        ApiRequest::SwitchBranch { path, branch } => {
            ("switch-branch", path.clone(), branch.clone())
        }
        ApiRequest::MergeIntoMain { path, branch } => {
            ("merge-main", path.clone(), branch.clone())
        }
    }
}

fn dispatch(req: ApiRequest, default_path: &Option<PathBuf>) -> ApiResponse {
    match req {
        ApiRequest::Status { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git::gather_repo_info(&p) {
                    Some(info) => ApiResponse::ok_data(serde_json::to_value(&info).unwrap()),
                    None => ApiResponse::err("Not a git repository"),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Branches { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git::gather_repo_info(&p) {
                    Some(info) => ApiResponse::ok_data(serde_json::to_value(&info.branches).unwrap()),
                    None => ApiResponse::err("Not a git repository"),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Files { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => {
                    let entries = git::gather_file_entries(&p);
                    ApiResponse::ok_data(serde_json::to_value(&entries).unwrap())
                }
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Gitcon { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git::gather_repo_info(&p) {
                    Some(info) => {
                        let data = json!({
                            "level": info.gitcon,
                            "label": info.gitcon.label(),
                            "subtitle": info.gitcon.subtitle(),
                        });
                        ApiResponse::ok_data(data)
                    }
                    None => ApiResponse::err("Not a git repository"),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Projects => {
            match config::load_config() {
                Ok(cfg) => {
                    let root = cfg.dev_root_path();
                    let mut projects = Vec::new();
                    if let Ok(entries) = std::fs::read_dir(&root) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if !path.is_dir() { continue; }
                            let name = match path.file_name().and_then(|n| n.to_str()) {
                                Some(n) => n,
                                None => continue,
                            };
                            if name.starts_with('.') { continue; }
                            if cfg.exclude.iter().any(|e| e == name) { continue; }
                            projects.push(git::gather_project_summary(&path));
                        }
                    }
                    projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    ApiResponse::ok_data(serde_json::to_value(&projects).unwrap())
                }
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Fetch { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_fetch(&p) {
                    Ok(msg) => ApiResponse::ok_msg(msg),
                    Err(e) => ApiResponse::err(e),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Pull { path, rebase } => {
            match resolve_path(&path, default_path) {
                Ok(p) => {
                    let info = match git::gather_repo_info(&p) {
                        Some(i) => i,
                        None => return ApiResponse::err("Not a git repository"),
                    };
                    let rb = rebase.unwrap_or(true);
                    match git_ops::git_pull(&p, &info.current_branch, rb) {
                        Ok(msg) => ApiResponse::ok_msg(msg),
                        Err(e) => ApiResponse::err(e),
                    }
                }
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Push { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_push_main(&p) {
                    Ok((msg, cleaned)) => ApiResponse::ok_with(msg, json!({
                        "branches_cleaned": cleaned,
                    })),
                    Err(e) => ApiResponse::err(e),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Backup { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => {
                    let info = match git::gather_repo_info(&p) {
                        Some(i) => i,
                        None => return ApiResponse::err("Not a git repository"),
                    };
                    match git_ops::git_backup(&p, &info.current_branch) {
                        Ok(msg) => ApiResponse::ok_msg(msg),
                        Err(e) => ApiResponse::err(e),
                    }
                }
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Commit { path, message, files } => {
            match resolve_path(&path, default_path) {
                Ok(p) => {
                    let f = files.unwrap_or_default();
                    match git_ops::git_commit(&p, &message, &f) {
                        Ok(msg) => ApiResponse::ok_msg(msg),
                        Err(e) => ApiResponse::err(e),
                    }
                }
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::NewBranch { path, name } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_new_branch(&p, &name) {
                    Ok(msg) => ApiResponse::ok_msg(msg),
                    Err(e) => ApiResponse::err(e),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::SwitchBranch { path, branch } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_switch_branch(&p, &branch) {
                    Ok(msg) => ApiResponse::ok_msg(msg),
                    Err(e) => ApiResponse::err(e),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::MergeIntoMain { path, branch } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_merge_into_main(&p, &branch) {
                    Ok(msg) => ApiResponse::ok_msg(msg),
                    Err(e) => ApiResponse::err(e),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
    }
}

pub fn run_api_server(socket_path: &str, default_path: Option<PathBuf>) {
    // Remove stale socket
    let _ = std::fs::remove_file(socket_path);

    let listener = match UnixListener::bind(socket_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind socket {}: {}", socket_path, e);
            std::process::exit(1);
        }
    };

    eprintln!("Sodium API listening on {}", socket_path);
    if let Some(ref p) = default_path {
        eprintln!("Default repo path: {}", p.display());
    }

    let running = Arc::new(AtomicBool::new(true));

    // Use non-blocking accept with a check on `running`
    listener.set_nonblocking(true).ok();

    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                // Set stream back to blocking for read/write
                stream.set_nonblocking(false).ok();

                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let response = match serde_json::from_str::<ApiRequest>(line) {
                    Ok(req) => handle_request(req, &default_path),
                    Err(e) => ApiResponse::err(format!("Invalid JSON: {}", e)),
                };

                let mut writer = stream;
                let json = serde_json::to_string(&response).unwrap_or_else(|_| {
                    r#"{"ok":false,"error":"serialization error"}"#.to_string()
                });
                let _ = writer.write_all(json.as_bytes());
                let _ = writer.write_all(b"\n");
                let _ = writer.flush();
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    let _ = std::fs::remove_file(socket_path);
}
