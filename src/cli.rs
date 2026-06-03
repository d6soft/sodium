use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};

use serde_json::json;

use crate::api::ApiResponse;
use crate::{audit, config, git_ops};

/// Try to dispatch a Sodium subcommand based on argv.
/// Returns true if a subcommand was recognized and executed (caller should exit).
///
/// All subcommands emit a single JSON line on stdout, regardless of outcome.
/// Exit codes: 0 = ok, 1 = action failed, 2 = usage or repo not found.
pub fn try_dispatch(args: &[String]) -> bool {
    let action: &'static str = match args.get(1).map(|s| s.as_str()) {
        Some("new-branch") => "new-branch",
        Some("commit") => "commit",
        Some("merge-main") => "merge-main",
        Some("push") => "push",
        Some("remotes") => "remotes",
        Some("add-github") => "add-github",
        _ => return false,
    };
    if let Err(e) = config::load_config() {
        emit_err(action, &e);
        exit(2);
    }
    match action {
        "new-branch" => run_new_branch(&args[2..]),
        "commit" => run_commit(&args[2..]),
        "merge-main" => run_merge_main(&args[2..]),
        "push" => run_push(&args[2..]),
        "remotes" => run_remotes(&args[2..]),
        "add-github" => run_add_github(&args[2..]),
        _ => unreachable!(),
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
            let (suffix, pushed, failed) = mirror_all(&repo, "main");
            let final_msg = format!("{}{}", msg, suffix);
            let data = json!({
                "branches_cleaned": cleaned,
                "mirrors": {
                    "pushed": pushed,
                    "failed": failed,
                },
            });
            emit_ok(action, final_msg, Some(data));
        }
        Err(e) => {
            emit_err(action, &e);
            exit(1);
        }
    }
}

/// Push `branch` to every configured mirror. Best-effort: failures are
/// collected but do not abort. Returns (suffix string, pushed names, failed
/// entries as "name: error").
fn mirror_all(repo: &Path, branch: &str) -> (String, Vec<String>, Vec<String>) {
    let cfg = config::load_config().expect("config validated at dispatch");
    let project_name = match repo.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return (String::new(), Vec::new(), Vec::new()),
    };
    let mirrors = cfg.mirrors(project_name);
    if mirrors.is_empty() {
        return (String::new(), Vec::new(), Vec::new());
    }

    let mut pushed = Vec::new();
    let mut failed = Vec::new();
    for (name, url) in &mirrors {
        match git_ops::mirror_push(repo, branch, name, url) {
            Ok(()) => pushed.push(name.clone()),
            Err(e) => failed.push(format!("{}: {}", name, e)),
        }
    }

    let suffix = if pushed.is_empty() {
        String::new()
    } else {
        format!(" + {}", pushed.join(", "))
    };
    (suffix, pushed, failed)
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

// ─── remotes (lecture) ────────────────────────────────────────────────────

fn run_remotes(args: &[String]) {
    let action = "remotes";
    let (path, _) = parse_path_flag(args);
    let repo = resolve_repo(path, action);
    let project_name = repo
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let git_remotes = read_git_remotes(&repo);
    let cfg = config::load_config().expect("config validated at dispatch");
    let configured = cfg.mirrors(&project_name);

    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (n, _) in &git_remotes {
        names.insert(n.clone());
    }
    for (n, _) in &configured {
        names.insert(n.clone());
    }
    // origin only exists git-side; include it if found.
    let mut list = Vec::new();
    for n in &names {
        let git_url = git_remotes.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
        let cfg_url = configured.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
        let source = match (git_url.is_some(), cfg_url.is_some()) {
            (true, true) => "both",
            (true, false) => "git",
            (false, true) => "sodium-config",
            _ => unreachable!(),
        };
        let url = git_url.clone().or_else(|| cfg_url.clone()).unwrap_or_default();
        let mut entry = json!({
            "name": n,
            "url": url,
            "source": source,
        });
        if let (Some(g), Some(c)) = (&git_url, &cfg_url) {
            if g != c {
                entry["mismatch"] = json!({"git": g, "sodium_config": c});
            }
        }
        list.push(entry);
    }

    let data = json!({"remotes": list});
    emit_ok(action, format!("{} remote(s)", list.len()), Some(data));
}

/// Parse `git remote -v` into a Vec<(name, fetch_url)>.
fn read_git_remotes(repo: &Path) -> Vec<(String, String)> {
    let out = Command::new("git")
        .args(["remote", "-v"])
        .current_dir(repo)
        .output();
    let stdout = match out {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Vec::new(),
    };
    let mut seen = std::collections::BTreeMap::new();
    for line in String::from_utf8_lossy(&stdout).lines() {
        // format: "name\turl (fetch)" or "name\turl (push)"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 || parts[2] != "(fetch)" {
            continue;
        }
        seen.insert(parts[0].to_string(), parts[1].to_string());
    }
    seen.into_iter().collect()
}

// ─── add-github (création) ───────────────────────────────────────────────

fn run_add_github(args: &[String]) {
    let action = "add-github";
    let mut path: Option<PathBuf> = None;
    let mut owner: String = "d6soft".to_string();
    let mut repo_name_override: Option<String> = None;
    let mut visibility: Option<String> = None;
    let mut yes = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--path" => {
                path = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            "--owner" => {
                if let Some(v) = args.get(i + 1) {
                    owner = v.clone();
                }
                i += 2;
            }
            "--name" => {
                repo_name_override = args.get(i + 1).cloned();
                i += 2;
            }
            "--public" => {
                visibility = Some("public".into());
                i += 1;
            }
            "--private" => {
                visibility = Some("private".into());
                i += 1;
            }
            "--yes" | "-y" => {
                yes = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let repo = resolve_repo(path, action);
    let detected = repo
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let repo_name = repo_name_override.unwrap_or(detected);
    if repo_name.is_empty() {
        emit_err(action, "unable to detect repo name");
        exit(2);
    }
    let url = format!("git@github.com:{}/{}.git", owner, repo_name);

    // gh availability
    let gh_ok = Command::new("gh")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !gh_ok {
        emit_err(action, "gh CLI not found. Install it (https://cli.github.com) and run `gh auth login` before retrying.");
        exit(1);
    }

    // visibility prompt
    let visibility = match visibility {
        Some(v) => v,
        None => match prompt_choice("Visibility for GitHub repo?", &["public", "private"]) {
            Some(v) => v,
            None => {
                emit_err(action, "no visibility chosen (use --public or --private)");
                exit(2);
            }
        },
    };

    eprintln!(
        "About to create GitHub repo {}/{} ({}) and bind remote `github`.",
        owner, repo_name, visibility
    );
    if !yes && !prompt_yes_no("Proceed?", true) {
        emit_err(action, "aborted by user");
        exit(1);
    }

    // gh repo create
    let gh_status = Command::new("gh")
        .args([
            "repo",
            "create",
            &format!("{}/{}", owner, repo_name),
            &format!("--{}", visibility),
            "--source",
            ".",
            "--remote",
            "github",
        ])
        .current_dir(&repo)
        .status();
    match gh_status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            emit_err(action, &format!("gh repo create failed (exit {})", s.code().unwrap_or(-1)));
            exit(1);
        }
        Err(e) => {
            emit_err(action, &format!("gh repo create: {}", e));
            exit(1);
        }
    }

    // Confirm sodium.toml write
    let write_cfg = if yes {
        true
    } else {
        prompt_yes_no(
            &format!(
                "Append [projects.{}.mirrors.github] url=\"{}\" to sodium.toml?",
                repo_name, url
            ),
            true,
        )
    };

    let mut cfg_written = false;
    let mut cfg_warning: Option<String> = None;
    if write_cfg {
        match append_mirror_to_config(&repo_name, "github", &url) {
            Ok(()) => cfg_written = true,
            Err(e) => cfg_warning = Some(e),
        }
    }

    let mut data = json!({
        "owner": owner,
        "repo": repo_name,
        "visibility": visibility,
        "remote": {"name": "github", "url": url},
        "sodium_config_updated": cfg_written,
    });
    if let Some(w) = cfg_warning {
        data["sodium_config_warning"] = json!(w);
    }
    emit_ok(
        action,
        format!("GitHub repo {}/{} created and bound", owner, repo_name),
        Some(data),
    );
}

fn prompt_yes_no(question: &str, default_yes: bool) -> bool {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    eprint!("{} {} ", question, suffix);
    let _ = std::io::stderr().flush();
    let stdin = std::io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return default_yes;
    }
    let answer = line.trim().to_lowercase();
    if answer.is_empty() {
        return default_yes;
    }
    matches!(answer.as_str(), "y" | "yes" | "o" | "oui")
}

fn prompt_choice(question: &str, options: &[&str]) -> Option<String> {
    eprintln!("{}", question);
    for (idx, opt) in options.iter().enumerate() {
        eprintln!("  [{}] {}", idx + 1, opt);
    }
    eprint!("Choice: ");
    let _ = std::io::stderr().flush();
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).ok()?;
    let trimmed = line.trim();
    if let Ok(n) = trimmed.parse::<usize>() {
        if n >= 1 && n <= options.len() {
            return Some(options[n - 1].to_string());
        }
    }
    if options.iter().any(|o| o.eq_ignore_ascii_case(trimmed)) {
        return Some(trimmed.to_lowercase());
    }
    None
}

/// Append `[projects.<project>.mirrors.<mirror>]` with `url = "<url>"` to the
/// sodium config file. Refuses if the exact section already exists.
fn append_mirror_to_config(project: &str, mirror: &str, url: &str) -> Result<(), String> {
    let path = dirs::config_dir()
        .map(|d| d.join("sodium").join("sodium.toml"))
        .ok_or_else(|| "config dir not found".to_string())?;
    let mut content = std::fs::read_to_string(&path)
        .map_err(|e| format!("read {}: {}", path.display(), e))?;
    let header = format!("[projects.{}.mirrors.{}]", project, mirror);
    if content.contains(&header) {
        return Err(format!("section {} already exists in sodium.toml", header));
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    let block = format!("\n{}\nurl = \"{}\"\n", header, url);
    content.push_str(&block);
    std::fs::write(&path, content).map_err(|e| format!("write {}: {}", path.display(), e))?;
    Ok(())
}
