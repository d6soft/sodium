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
        Ok(format!("Commit recorded: {}", message))
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(err)
    }
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
