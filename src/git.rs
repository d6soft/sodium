use git2::{BranchType, Repository, StatusOptions};
use std::collections::BTreeSet;
use std::path::Path;

use crate::theme::GitconLevel;

// ── Project summary (lightweight, for list screen) ────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProjectSummary {
    pub name: String,
    pub path: std::path::PathBuf,
    pub has_git: bool,
    pub branch: String,
    pub dirty_count: usize,
    pub ahead: usize,
    pub behind: usize,
    pub last_commit_msg: String,
    pub last_commit_age: String,
    pub gitcon: GitconLevel,
}

pub fn gather_project_summary(path: &Path) -> ProjectSummary {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("???")
        .to_string();

    let repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => {
            return ProjectSummary {
                name,
                path: path.to_path_buf(),
                has_git: false,
                branch: String::new(),
                dirty_count: 0,
                ahead: 0,
                behind: 0,
                last_commit_msg: String::new(),
                last_commit_age: String::new(),
                gitcon: GitconLevel::Gitcon5,
            };
        }
    };

    let head = repo.head().ok();
    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD".into());

    let files = calc_file_status(&repo);
    let dirty_count = files.modified + files.staged + files.untracked + files.conflicted;
    let (ahead, behind) = calc_ahead_behind(&repo, &branch);
    let gitcon = calc_gitcon(&files, ahead, behind);

    let (last_commit_msg, last_commit_age) = head
        .as_ref()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| {
            let msg = c
                .message()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            let age = format_commit_age(c.time().seconds());
            (msg, age)
        })
        .unwrap_or_else(|| ("no commits".into(), String::new()));

    ProjectSummary {
        name,
        path: path.to_path_buf(),
        has_git: true,
        branch,
        dirty_count,
        ahead,
        behind,
        last_commit_msg,
        last_commit_age,
        gitcon,
    }
}

fn format_commit_age(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;
    if diff < 0 {
        return "now".into();
    }
    let minutes = diff / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let weeks = days / 7;
    let months = days / 30;

    if minutes < 1 {
        "now".into()
    } else if minutes < 60 {
        format!("{}m ago", minutes)
    } else if hours < 24 {
        format!("{}h ago", hours)
    } else if days < 7 {
        format!("{}d ago", days)
    } else if weeks < 5 {
        format!("{}w ago", weeks)
    } else {
        format!("{}mo ago", months)
    }
}

// ── Data structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_local: bool,
    pub is_remote: bool,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub modified: usize,
    pub staged: usize,
    pub untracked: usize,
    pub conflicted: usize,
}

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub name: String,
    pub current_branch: String,
    pub branches: Vec<BranchInfo>,
    pub last_commit_hash: String,
    pub last_commit_msg: String,
    pub ahead: usize,
    pub behind: usize,
    pub files: FileStatus,
    pub remote_url: Option<String>,
    pub commit_activity: Vec<u64>,
    pub gitcon: GitconLevel,
    pub total_commits: usize,
}

impl Default for RepoInfo {
    fn default() -> Self {
        Self {
            name: String::from("???"),
            current_branch: String::from("???"),
            branches: Vec::new(),
            last_commit_hash: String::from("-------"),
            last_commit_msg: String::from("no commits yet"),
            ahead: 0,
            behind: 0,
            files: FileStatus {
                modified: 0,
                staged: 0,
                untracked: 0,
                conflicted: 0,
            },
            remote_url: None,
            commit_activity: vec![0; 14],
            gitcon: GitconLevel::Gitcon5,
            total_commits: 0,
        }
    }
}

// ── Gather repo info ───────────────────────────────────────────────────────

pub fn gather_repo_info(path: &Path) -> Option<RepoInfo> {
    let repo = Repository::discover(path).ok()?;
    let workdir = repo.workdir()?;
    let repo_name = workdir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("???")
        .to_string();

    let head = repo.head().ok();
    let current_branch = head
        .as_ref()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD".into());

    // ── Branches ───────────────────────────────────────────────────────
    let mut branch_names: BTreeSet<String> = BTreeSet::new();
    let mut local_set: BTreeSet<String> = BTreeSet::new();
    let mut remote_set: BTreeSet<String> = BTreeSet::new();

    if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
        for branch in branches.flatten() {
            if let Some(name) = branch.0.name().ok().flatten() {
                branch_names.insert(name.to_string());
                local_set.insert(name.to_string());
            }
        }
    }

    if let Ok(branches) = repo.branches(Some(BranchType::Remote)) {
        for branch in branches.flatten() {
            if let Some(name) = branch.0.name().ok().flatten() {
                let short = name.strip_prefix("origin/").unwrap_or(name);
                if short != "HEAD" {
                    branch_names.insert(short.to_string());
                    remote_set.insert(short.to_string());
                }
            }
        }
    }

    let branches: Vec<BranchInfo> = branch_names
        .iter()
        .map(|name| BranchInfo {
            name: name.clone(),
            is_local: local_set.contains(name),
            is_remote: remote_set.contains(name),
            is_current: *name == current_branch,
        })
        .collect();

    // ── Last commit ────────────────────────────────────────────────────
    let (last_commit_hash, last_commit_msg) = head
        .as_ref()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| {
            let hash = c.id().to_string()[..7].to_string();
            let msg = c
                .message()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            (hash, msg)
        })
        .unwrap_or_else(|| ("-------".into(), "no commits yet".into()));

    // ── Ahead / Behind ─────────────────────────────────────────────────
    let (ahead, behind) = calc_ahead_behind(&repo, &current_branch);

    // ── File status ────────────────────────────────────────────────────
    let files = calc_file_status(&repo);

    // ── Remote URL ─────────────────────────────────────────────────────
    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(String::from));

    // ── Commit activity (last 14 days) ─────────────────────────────────
    let commit_activity = calc_commit_activity(&repo);

    // ── Total commits ──────────────────────────────────────────────────
    let total_commits = count_commits(&repo);

    // ── GITCON level ───────────────────────────────────────────────────
    let gitcon = calc_gitcon(&files, ahead, behind);

    Some(RepoInfo {
        name: repo_name,
        current_branch,
        branches,
        last_commit_hash,
        last_commit_msg,
        ahead,
        behind,
        files,
        remote_url,
        commit_activity,
        gitcon,
        total_commits,
    })
}

fn calc_ahead_behind(repo: &Repository, branch: &str) -> (usize, usize) {
    let local = match repo.revparse_single(&format!("refs/heads/{branch}")) {
        Ok(obj) => obj.id(),
        Err(_) => return (0, 0),
    };
    let remote = match repo.revparse_single(&format!("refs/remotes/origin/{branch}")) {
        Ok(obj) => obj.id(),
        Err(_) => return (0, 0),
    };
    repo.graph_ahead_behind(local, remote).unwrap_or((0, 0))
}

fn calc_file_status(repo: &Repository) -> FileStatus {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => {
            return FileStatus {
                modified: 0,
                staged: 0,
                untracked: 0,
                conflicted: 0,
            }
        }
    };

    let mut modified = 0;
    let mut staged = 0;
    let mut untracked = 0;
    let mut conflicted = 0;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.is_conflicted() {
            conflicted += 1;
        } else if s.is_wt_new() {
            untracked += 1;
        } else {
            if s.is_wt_modified() || s.is_wt_deleted() || s.is_wt_renamed() || s.is_wt_typechange()
            {
                modified += 1;
            }
            if s.is_index_new()
                || s.is_index_modified()
                || s.is_index_deleted()
                || s.is_index_renamed()
                || s.is_index_typechange()
            {
                staged += 1;
            }
        }
    }

    FileStatus {
        modified,
        staged,
        untracked,
        conflicted,
    }
}

fn calc_commit_activity(repo: &Repository) -> Vec<u64> {
    let mut activity = vec![0u64; 14];
    let now = chrono::Utc::now().timestamp();
    let day_secs: i64 = 86400;

    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return activity,
    };
    let _ = revwalk.push_head();

    for oid in revwalk.flatten() {
        if let Ok(commit) = repo.find_commit(oid) {
            let time = commit.time().seconds();
            let days_ago = (now - time) / day_secs;
            if days_ago >= 0 && days_ago < 14 {
                activity[13 - days_ago as usize] += 1;
            }
            if days_ago >= 14 {
                break;
            }
        }
    }

    activity
}

fn count_commits(repo: &Repository) -> usize {
    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return 0,
    };
    let _ = revwalk.push_head();
    revwalk.count()
}

fn calc_gitcon(files: &FileStatus, ahead: usize, behind: usize) -> GitconLevel {
    if files.conflicted > 0 {
        return GitconLevel::Gitcon4;
    }
    let dirty = files.modified + files.untracked + files.staged;
    if behind > 5 || dirty > 20 {
        return GitconLevel::Gitcon3;
    }
    if ahead > 0 || behind > 0 || dirty > 0 {
        return GitconLevel::Gitcon2;
    }
    GitconLevel::Gitcon1
}
