use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use serde_json::json;

use crate::api::ApiResponse;
use crate::{audit, config, git_ops};

/// Try to dispatch a Sodium subcommand based on argv.
/// Returns true if a subcommand was recognized and executed (caller should exit).
///
/// All subcommands emit a single JSON line on stdout, regardless of outcome.
/// Exit codes: 0 = ok, 1 = action failed, 2 = usage or repo not found.
pub fn try_dispatch(args: &[String]) -> bool {
    let cmd = match args.get(1).map(|s| s.as_str()) {
        Some(c) => c,
        None => return false,
    };
    match cmd {
        "new-branch" => run_new_branch(&args[2..]),
        "commit" => run_commit(&args[2..]),
        "merge-main" => run_merge_main(&args[2..]),
        "push" => run_push(&args[2..]),
        _ => return false,
    }
    true
}

fn run_new_branch(args: &[String]) {
    let action = "new-branch";
    let (path, rest) = parse_path_flag(args);
    if rest.is_empty() {
        emit_err(action, "usage: sodium new-branch <name> [--path <dir>]");
        exit(2);
    }
    let repo = resolve_repo(path, action);
    let name = &rest[0];
    let result = git_ops::git_new_branch(&repo, name);
    audit::log("cli", &repo, action, name, result.as_deref().map_err(|s| s.as_str()));
    match result {
        Ok(msg) => emit_ok(action, msg, None),
        Err(e) => {
            emit_err(action, &e);
            exit(1);
        }
    }
}

fn run_commit(args: &[String]) {
    let action = "commit";
    let mut path: Option<PathBuf> = None;
    let mut message: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-m" | "--message" => {
                message = args.get(i + 1).cloned();
                i += 2;
            }
            "--path" => {
                path = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            _ => i += 1,
        }
    }
    let message = match message {
        Some(m) if !m.is_empty() => m,
        _ => {
            emit_err(action, "usage: sodium commit -m <message> [--path <dir>]");
            exit(2);
        }
    };
    let repo = resolve_repo(path, action);
    let result = git_ops::git_commit(&repo, &message, &[]);
    audit::log("cli", &repo, action, &message, result.as_deref().map_err(|s| s.as_str()));
    match result {
        Ok(msg) => emit_ok(action, msg, None),
        Err(e) => {
            emit_err(action, &e);
            exit(1);
        }
    }
}

fn run_merge_main(args: &[String]) {
    let action = "merge-main";
    let (path, rest) = parse_path_flag(args);
    if rest.is_empty() {
        emit_err(action, "usage: sodium merge-main <feature> [--path <dir>]");
        exit(2);
    }
    let repo = resolve_repo(path, action);
    let feature = &rest[0];
    let result = git_ops::git_merge_into_main(&repo, feature);
    audit::log("cli", &repo, action, feature, result.as_deref().map_err(|s| s.as_str()));
    match result {
        Ok(msg) => emit_ok(action, msg, None),
        Err(e) => {
            emit_err(action, &e);
            exit(1);
        }
    }
}

fn run_push(args: &[String]) {
    let action = "push";
    let (path, _) = parse_path_flag(args);
    let repo = resolve_repo(path, action);
    let result = git_ops::git_push_main(&repo);
    audit::log(
        "cli",
        &repo,
        action,
        "",
        result.as_ref().map(|_| "").map_err(|s| s.as_str()),
    );
    match result {
        Ok((msg, cleaned)) => {
            let gh = mirror_suffix(&repo);
            let final_msg = format!("{}{}", msg, gh);
            let data = json!({
                "branches_cleaned": cleaned,
                "github_mirrored": !gh.is_empty(),
            });
            emit_ok(action, final_msg, Some(data));
        }
        Err(e) => {
            emit_err(action, &e);
            exit(1);
        }
    }
}

fn mirror_suffix(repo: &Path) -> String {
    let cfg = match config::load_config() {
        Some(c) => c,
        None => return String::new(),
    };
    let project_name = match repo.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return String::new(),
    };
    let github_url = match cfg.github_url(project_name) {
        Some(u) => u.to_string(),
        None => return String::new(),
    };
    git_ops::mirror_to_github(repo, "main", &github_url).unwrap_or_default()
}

fn parse_path_flag(args: &[String]) -> (Option<PathBuf>, Vec<String>) {
    let mut path = None;
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--path" {
            path = args.get(i + 1).map(PathBuf::from);
            i += 2;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    (path, rest)
}

/// Resolve the working repo: explicit --path, or git toplevel of $PWD.
/// On failure emits a JSON error and exits with code 2.
fn resolve_repo(explicit: Option<PathBuf>, action: &'static str) -> PathBuf {
    let candidate = explicit.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });
    let candidate_str = candidate.to_string_lossy().into_owned();
    let out = Command::new("git")
        .args(["-C", &candidate_str, "rev-parse", "--show-toplevel"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
            PathBuf::from(p)
        }
        _ => {
            emit_err(action, &format!("not a git repository: {}", candidate.display()));
            exit(2);
        }
    }
}

fn emit_ok(action: &'static str, message: impl Into<String>, data: Option<serde_json::Value>) {
    let mut response = ApiResponse::ok_msg(message).with_action(action);
    if let Some(d) = data {
        response.data = Some(d);
    }
    println!("{}", serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"ok":true}"#.to_string()
    }));
}

fn emit_err(action: &'static str, error: &str) {
    let response = ApiResponse::err(error).with_action(action);
    println!("{}", serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"ok":false}"#.to_string()
    }));
}
