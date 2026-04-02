use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

pub fn git_fetch(path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["fetch", "--prune", "origin"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if output.status.success() {
        return Ok("Fetch complete — remote intel updated".into());
    }
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    // git fetch writes to stderr even on success
    if output.status.code() == Some(0) || err.contains("From") {
        Ok("Fetch complete".into())
    } else {
        Err(err)
    }
}

pub fn git_pull(path: &Path, branch: &str, rebase: bool) -> Result<String, String> {
    let mut args = vec!["pull"];
    if rebase {
        args.push("--rebase");
    }
    args.push("origin");
    args.push(branch);

    let output = Command::new("git")
        .args(&args)
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if output.status.success() {
        let mode = if rebase { "rebase" } else { "merge" };
        return Ok(format!("Pull complete ({}) — branch synced", mode));
    }
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if err.contains("Already up to date") {
        Ok("Already up to date".into())
    } else {
        Err(err)
    }
}

pub fn git_push_main(path: &Path) -> Result<(String, usize), String> {
    let output = Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if output.status.success() {
        let cleaned = cleanup_merged_branches(path);
        return Ok(("Push complete — intel transmitted".into(), cleaned));
    }
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if err.contains("Everything up-to-date") {
        Ok(("Already synced — nothing to transmit".into(), 0))
    } else {
        Err(err)
    }
}

pub fn git_backup(path: &Path, branch: &str) -> Result<String, String> {
    if branch == "main" {
        return Err("Use Push for main branch".into());
    }
    let output = Command::new("git")
        .args(["push", "origin", branch])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if output.status.success() {
        return Ok(format!("{} backed up to origin", branch));
    }
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if err.contains("Everything up-to-date") {
        Ok("Already backed up".into())
    } else {
        Err(err)
    }
}

pub fn git_new_branch(path: &Path, name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Empty branch name".into());
    }
    let output = Command::new("git")
        .args(["checkout", "-b", name])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if output.status.success() {
        Ok(format!("Branch '{}' created & active", name))
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(err)
    }
}

pub fn git_switch_branch(path: &Path, branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["checkout", branch])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if output.status.success() {
        Ok(format!("Switched to '{}'", branch))
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(err)
    }
}

pub fn git_commit(path: &Path, message: &str, files: &[String]) -> Result<String, String> {
    if message.is_empty() {
        return Err("Empty commit message".into());
    }

    // TT705: Auto-clean tracked files matching .gitignore before staging
    let cleaned = git_clean_tracked_ignored(path);

    // Stage files
    if files.is_empty() {
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(path)
            .output()
            .map_err(|e| format!("{}", e))?;
    } else {
        let mut args: Vec<&str> = vec!["add", "--"];
        for f in files {
            args.push(f.as_str());
        }
        Command::new("git")
            .args(&args)
            .current_dir(path)
            .output()
            .map_err(|e| format!("{}", e))?;
    }

    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if output.status.success() {
        let suffix = if cleaned > 0 {
            format!(" ({} cached files cleaned)", cleaned)
        } else {
            String::new()
        };
        Ok(format!("Commit recorded: {}{}", message, suffix))
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(err)
    }
}

/// Detect known build/artifact directories tracked in git index.
/// Checks root level and nested subprojects (Cargo.toml, package.json, etc.).
/// Returns list of directory prefixes found (e.g., ["target/", "dubnium/target/"]).
pub fn detect_suspect_tracked(path: &Path) -> Vec<String> {
    const SUSPECT: &[&str] = &[
        "target", "node_modules", ".next", "dist",
        ".svelte-kit", "__pycache__", ".dart_tool",
    ];

    let mut found = BTreeSet::new();

    // Check root-level suspects
    for dir in SUSPECT {
        if path.join(dir).is_dir() && is_dir_in_index(path, dir) {
            found.insert(format!("{}/", dir));
        }
    }

    // Check nested subprojects (one level deep)
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let ep = entry.path();
            if !ep.is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }

            let mut sub_suspects: Vec<&str> = Vec::new();
            if ep.join("Cargo.toml").exists() { sub_suspects.push("target"); }
            if ep.join("package.json").exists() {
                sub_suspects.extend_from_slice(&["node_modules", ".next", ".svelte-kit", "dist"]);
            }
            if ep.join("go.mod").exists() { sub_suspects.push("vendor"); }
            if ep.join("pubspec.yaml").exists() { sub_suspects.extend_from_slice(&[".dart_tool", "build"]); }

            for dir in sub_suspects {
                let nested = format!("{}/{}", name, dir);
                if ep.join(dir).is_dir() && is_dir_in_index(path, &nested) {
                    found.insert(format!("{}/", nested));
                }
            }
        }
    }

    found.into_iter().collect()
}

fn is_dir_in_index(repo_path: &Path, dir: &str) -> bool {
    let pattern = format!("{}/", dir);
    Command::new("git")
        .args(["ls-files", "--cached", &pattern])
        .current_dir(repo_path)
        .output()
        .ok()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

/// List tracked files that match current .gitignore patterns.
pub fn git_tracked_ignored(path: &Path) -> Vec<String> {
    Command::new("git")
        .args(["ls-files", "--cached", "-i", "--exclude-standard"])
        .current_dir(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Remove tracked files that match .gitignore from the index (git rm --cached).
/// Returns count of files cleaned.
pub fn git_clean_tracked_ignored(path: &Path) -> usize {
    let files = git_tracked_ignored(path);
    if files.is_empty() {
        return 0;
    }
    let count = files.len();
    let mut args: Vec<&str> = vec!["rm", "--cached", "--quiet", "--"];
    for f in &files {
        args.push(f.as_str());
    }
    let _ = Command::new("git")
        .args(&args)
        .current_dir(path)
        .output();
    count
}

fn cleanup_merged_branches(path: &Path) -> usize {
    let output = Command::new("git")
        .args(["branch", "--merged", "main", "--format=%(refname:short)"])
        .current_dir(path)
        .output();

    let mut cleaned = 0;
    if let Ok(o) = output {
        let branches = String::from_utf8_lossy(&o.stdout);
        for branch in branches.lines() {
            let branch = branch.trim();
            if branch.is_empty() || branch == "main" {
                continue;
            }
            let _ = Command::new("git")
                .args(["branch", "-d", branch])
                .current_dir(path)
                .output();
            let _ = Command::new("git")
                .args(["push", "origin", "--delete", branch])
                .current_dir(path)
                .output();
            cleaned += 1;
        }
    }
    cleaned
}
