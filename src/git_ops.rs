use std::collections::{BTreeSet, HashSet};
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

/// Push `branch` to the optional `github` remote (force-push, since origin is
/// source of truth). Adds the `github` remote on first call if missing.
/// Returns Some(" + GitHub") on success to suffix push messages, None otherwise.
pub fn mirror_to_github(path: &Path, branch: &str, github_url: &str) -> Option<String> {
    let check = Command::new("git")
        .args(["remote", "get-url", "github"])
        .current_dir(path)
        .output();
    let has_remote = check.map(|o| o.status.success()).unwrap_or(false);
    if !has_remote {
        let add = Command::new("git")
            .args(["remote", "add", "github", github_url])
            .current_dir(path)
            .output();
        if !add.map(|o| o.status.success()).unwrap_or(false) {
            return None;
        }
    }

    let push = Command::new("git")
        .args(["push", "--force", "github", branch])
        .current_dir(path)
        .output();

    match push {
        Ok(o) if o.status.success() => Some(" + GitHub".into()),
        _ => None,
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

/// Merge a feature branch into main. Stashes any local changes, checks out
/// main if not already on it, runs `git merge <feature>`, then pops the stash.
/// Leaves the repo on main on success or failure.
pub fn git_merge_into_main(path: &Path, feature: &str) -> Result<String, String> {
    if feature.is_empty() {
        return Err("Empty feature branch name".into());
    }
    if feature == "main" {
        return Err("Cannot merge main into itself".into());
    }

    let head = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;
    let current = String::from_utf8_lossy(&head.stdout).trim().to_string();
    let on_main = current == "main";

    let mut did_stash = false;
    if !on_main {
        if let Ok(stash_out) = Command::new("git")
            .args(["stash", "push", "-m", "sodium-auto-merge"])
            .current_dir(path)
            .output()
        {
            let msg = String::from_utf8_lossy(&stash_out.stdout);
            did_stash = !msg.contains("No local changes");
        }

        let checkout = Command::new("git")
            .args(["checkout", "main"])
            .current_dir(path)
            .output()
            .map_err(|e| format!("{}", e))?;
        if !checkout.status.success() {
            let err = String::from_utf8_lossy(&checkout.stderr).trim().to_string();
            if did_stash {
                let _ = Command::new("git")
                    .args(["stash", "pop"])
                    .current_dir(path)
                    .output();
            }
            return Err(format!("checkout main: {}", err));
        }
    }

    let merge = Command::new("git")
        .args(["merge", feature])
        .current_dir(path)
        .output()
        .map_err(|e| format!("{}", e))?;

    if did_stash {
        let _ = Command::new("git")
            .args(["stash", "pop"])
            .current_dir(path)
            .output();
    }

    if merge.status.success() {
        Ok(format!("'{}' merged into main", feature))
    } else {
        let err = String::from_utf8_lossy(&merge.stderr).trim().to_string();
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

/// Detect known build/artifact directories present locally and not covered
/// by the root .gitignore (regardless of git tracking). Scans root + nested
/// subprojects. Returns directory prefixes (e.g., "joris/target/").
pub fn detect_suspect_unignored(path: &Path) -> Vec<String> {
    const SUSPECT: &[&str] = &[
        "target", "node_modules", ".next", "dist",
        ".svelte-kit", "__pycache__", ".dart_tool",
    ];

    let ignored = read_gitignore_patterns(path);
    let mut found = BTreeSet::new();

    for dir in SUSPECT {
        if path.join(dir).is_dir() {
            let entry = format!("{}/", dir);
            if !ignored.contains(&entry) {
                found.insert(entry);
            }
        }
    }

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
                if ep.join(dir).is_dir() {
                    let pattern = format!("{}/", nested);
                    if !ignored.contains(&pattern) {
                        found.insert(pattern);
                    }
                }
            }
        }
    }

    found.into_iter().collect()
}

/// Detect sensitive files (env, ssh keys, certs) present in the working tree
/// and not covered by .gitignore. Uses `git ls-files --others --cached
/// --exclude-standard` so .gitignore-covered paths are filtered out by git.
/// Returns repo-relative file paths.
pub fn detect_unignored_sensitive(path: &Path) -> Vec<String> {
    let listing = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut found = BTreeSet::new();
    for line in listing.lines() {
        let p = line.trim();
        if p.is_empty() { continue; }
        let basename = match Path::new(p).file_name().and_then(|n| n.to_str()) {
            Some(s) => s,
            None => continue,
        };
        if is_sensitive_basename(basename) {
            found.insert(p.to_string());
        }
    }
    found.into_iter().collect()
}

/// Detect large data dumps (`.sql`, `.csv` > 100 KiB) present and not ignored.
/// Returns (relative path, size in bytes) tuples.
pub fn detect_unignored_large_data(path: &Path) -> Vec<(String, u64)> {
    const THRESHOLD: u64 = 100 * 1024;
    let listing = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut found: Vec<(String, u64)> = Vec::new();
    for line in listing.lines() {
        let p = line.trim();
        if p.is_empty() { continue; }
        let basename = match Path::new(p).file_name().and_then(|n| n.to_str()) {
            Some(s) => s,
            None => continue,
        };
        let ext = basename.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        if !matches!(ext.as_str(), "sql" | "csv") { continue; }
        if let Ok(meta) = std::fs::metadata(path.join(p)) {
            if meta.len() > THRESHOLD {
                found.push((p.to_string(), meta.len()));
            }
        }
    }
    found.sort();
    found
}

/// Detect archive files (`.zip`, `.rar`, `.7z`, `.tar`, `.tgz`, `.tbz2`,
/// `.txz`, `.tzst`, `.iso`, `.dmg`) present and not ignored.
/// Returns (relative path, size in bytes) tuples.
pub fn detect_unignored_archives(path: &Path) -> Vec<(String, u64)> {
    const ARCHIVE_EXTS: &[&str] = &[
        "zip", "rar", "7z", "tar", "tgz", "tbz2", "txz", "tzst", "iso", "dmg",
    ];
    let listing = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut found: Vec<(String, u64)> = Vec::new();
    for line in listing.lines() {
        let p = line.trim();
        if p.is_empty() { continue; }
        let basename = match Path::new(p).file_name().and_then(|n| n.to_str()) {
            Some(s) => s,
            None => continue,
        };
        let ext = basename.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        if !ARCHIVE_EXTS.contains(&ext.as_str()) { continue; }
        if let Ok(meta) = std::fs::metadata(path.join(p)) {
            found.push((p.to_string(), meta.len()));
        }
    }
    found.sort();
    found
}

fn is_sensitive_basename(name: &str) -> bool {
    if name == ".env" { return true; }
    if name.starts_with(".env.") {
        // Skip canonical "template" variants meant to be committed.
        const SAFE_SUFFIX: &[&str] = &[".example", ".sample", ".template", ".dist"];
        if !SAFE_SUFFIX.iter().any(|s| name.ends_with(s)) {
            return true;
        }
    }
    if matches!(name, "id_rsa" | "id_dsa" | "id_ed25519" | "id_ecdsa") {
        return true;
    }
    if let Some(ext) = name.rsplit('.').next() {
        if matches!(ext.to_ascii_lowercase().as_str(), "pem" | "p12" | "pfx") {
            return true;
        }
    }
    false
}

/// Read the root .gitignore patterns into a set of trimmed non-empty,
/// non-comment lines. Returns empty set if file is absent.
fn read_gitignore_patterns(path: &Path) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Ok(content) = std::fs::read_to_string(path.join(".gitignore")) {
        for line in content.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') { continue; }
            set.insert(l.to_string());
        }
    }
    set
}

/// List nested .gitignore files (one level deep, excluding root).
/// Returns relative paths like "subproject/.gitignore".
pub fn find_nested_gitignores(path: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let ep = entry.path();
            if !ep.is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            if ep.join(".gitignore").is_file() {
                out.push(format!("{}/.gitignore", name));
            }
        }
    }
    out.sort();
    out
}

/// Append entries to the root .gitignore under a "Sodium auto-added" section.
/// Skips entries already present. Returns number actually added.
pub fn append_to_root_gitignore(path: &Path, entries: &[String]) -> usize {
    let gi_path = path.join(".gitignore");
    let existing = std::fs::read_to_string(&gi_path).unwrap_or_default();
    let existing_set = read_gitignore_patterns(path);

    let mut seen: HashSet<String> = existing_set.clone();
    let mut to_add: Vec<&String> = Vec::new();
    for e in entries {
        if seen.insert(e.clone()) {
            to_add.push(e);
        }
    }
    if to_add.is_empty() { return 0; }

    let header = "# ── Sodium auto-added ──";
    let mut new_content = existing.clone();
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    if !new_content.contains(header) {
        if !new_content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(header);
        new_content.push('\n');
    }
    for e in &to_add {
        new_content.push_str(e);
        new_content.push('\n');
    }
    let _ = std::fs::write(&gi_path, new_content);
    to_add.len()
}

/// Merge a nested .gitignore into the root one (prefixed with subdir path)
/// and delete the source file. Returns number of patterns merged.
pub fn merge_nested_gitignore(path: &Path, nested_relpath: &str) -> Result<usize, String> {
    let nested_full = path.join(nested_relpath);
    let content = std::fs::read_to_string(&nested_full)
        .map_err(|e| format!("read {}: {}", nested_relpath, e))?;

    let prefix = nested_relpath
        .strip_suffix("/.gitignore")
        .ok_or_else(|| "invalid nested gitignore path".to_string())?;

    let mut prefixed: Vec<String> = Vec::new();
    for line in content.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') { continue; }
        let (neg, pat) = if let Some(rest) = l.strip_prefix('!') {
            ("!", rest)
        } else {
            ("", l)
        };
        let anchored = pat.strip_prefix('/').unwrap_or(pat);
        prefixed.push(format!("{}{}/{}", neg, prefix, anchored));
    }

    let count = prefixed.len();
    append_to_root_gitignore(path, &prefixed);
    std::fs::remove_file(&nested_full)
        .map_err(|e| format!("remove {}: {}", nested_relpath, e))?;
    Ok(count)
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

/// Remove a single file from the git index (git rm --cached) without
/// touching the working tree. Returns true if the rm command succeeded
/// (which includes the case where the file was not tracked).
pub fn git_rm_cached_one(path: &Path, file: &str) -> bool {
    Command::new("git")
        .args(["rm", "--cached", "--quiet", "--ignore-unmatch", "--", file])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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
