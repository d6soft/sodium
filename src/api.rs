use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::{config, git, git_ops};

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
}

#[derive(Serialize)]
struct ApiResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ApiResponse {
    fn success(data: Value) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }
    fn err(msg: String) -> Self {
        Self { ok: false, data: None, error: Some(msg) }
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
    match req {
        ApiRequest::Status { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git::gather_repo_info(&p) {
                    Some(info) => ApiResponse::success(serde_json::to_value(&info).unwrap()),
                    None => ApiResponse::err("Not a git repository".into()),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Branches { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git::gather_repo_info(&p) {
                    Some(info) => ApiResponse::success(serde_json::to_value(&info.branches).unwrap()),
                    None => ApiResponse::err("Not a git repository".into()),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Files { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => {
                    let entries = git::gather_file_entries(&p);
                    ApiResponse::success(serde_json::to_value(&entries).unwrap())
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
                        ApiResponse::success(data)
                    }
                    None => ApiResponse::err("Not a git repository".into()),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Projects => {
            match config::load_config() {
                Some(cfg) => {
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
                    ApiResponse::success(serde_json::to_value(&projects).unwrap())
                }
                None => ApiResponse::err("No sodium config found".into()),
            }
        }
        ApiRequest::Fetch { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_fetch(&p) {
                    Ok(msg) => ApiResponse::success(json!({ "message": msg })),
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
                        None => return ApiResponse::err("Not a git repository".into()),
                    };
                    let rb = rebase.unwrap_or(true);
                    match git_ops::git_pull(&p, &info.current_branch, rb) {
                        Ok(msg) => ApiResponse::success(json!({ "message": msg })),
                        Err(e) => ApiResponse::err(e),
                    }
                }
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::Push { path } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_push_main(&p) {
                    Ok((msg, cleaned)) => ApiResponse::success(json!({
                        "message": msg,
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
                        None => return ApiResponse::err("Not a git repository".into()),
                    };
                    match git_ops::git_backup(&p, &info.current_branch) {
                        Ok(msg) => ApiResponse::success(json!({ "message": msg })),
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
                        Ok(msg) => ApiResponse::success(json!({ "message": msg })),
                        Err(e) => ApiResponse::err(e),
                    }
                }
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::NewBranch { path, name } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_new_branch(&p, &name) {
                    Ok(msg) => ApiResponse::success(json!({ "message": msg })),
                    Err(e) => ApiResponse::err(e),
                },
                Err(e) => ApiResponse::err(e),
            }
        }
        ApiRequest::SwitchBranch { path, branch } => {
            match resolve_path(&path, default_path) {
                Ok(p) => match git_ops::git_switch_branch(&p, &branch) {
                    Ok(msg) => ApiResponse::success(json!({ "message": msg })),
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
