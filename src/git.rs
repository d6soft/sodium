use git2::{BranchType, Repository, StatusOptions};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;

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

#[derive(Debug, Clone, Default)]
pub struct DayActivity {
    pub commits: u8,
    pub merges: u8,
    pub branches: u8,
    pub pulls: u8,
}

impl DayActivity {
    pub fn total(&self) -> u16 {
        self.commits as u16 + self.merges as u16 + self.branches as u16 + self.pulls as u16
    }
}

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
pub struct FileEntry {
    pub path: String,
    pub status_char: char, // M, A, D, ?, C, R
    pub insertions: usize,
    pub deletions: usize,
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
    pub github_url: Option<String>,
    pub activity_grid: Vec<DayActivity>,
    pub gitcon: GitconLevel,
    pub total_commits: usize,
    pub ahead_of_main: usize,
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
            github_url: None,
            activity_grid: vec![DayActivity::default(); 91],
            gitcon: GitconLevel::Gitcon5,
            total_commits: 0,
            ahead_of_main: 0,
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

    // ── Ahead of main (feature branch commits) ─────────────────────────
    let ahead_of_main = if current_branch != "main" {
        calc_ahead_of_main(&repo, &current_branch)
    } else {
        0
    };

    // ── File status ────────────────────────────────────────────────────
    let files = calc_file_status(&repo);

    // ── Remote URL ─────────────────────────────────────────────────────
    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(String::from));

    // ── GitHub remote URL ────────────────────────────────────────────
    let github_url = repo
        .find_remote("github")
        .ok()
        .and_then(|r| r.url().map(String::from));

    // ── Activity grid (last 14 days, from reflog) ─────────────────────
    let activity_grid = calc_activity_grid(&repo);

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
        github_url,
        activity_grid,
        gitcon,
        total_commits,
        ahead_of_main,
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

fn calc_ahead_of_main(repo: &Repository, branch: &str) -> usize {
    let local = match repo.revparse_single(&format!("refs/heads/{branch}")) {
        Ok(obj) => obj.id(),
        Err(_) => return 0,
    };
    let main = match repo.revparse_single("refs/heads/main") {
        Ok(obj) => obj.id(),
        Err(_) => return 0,
    };
    repo.graph_ahead_behind(local, main)
        .map(|(ahead, _)| ahead)
        .unwrap_or(0)
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

fn calc_activity_grid(repo: &Repository) -> Vec<DayActivity> {
    let mut grid = vec![DayActivity::default(); 91];
    let now = chrono::Utc::now().timestamp();
    let day_secs: i64 = 86400;

    let reflog = match repo.reflog("HEAD") {
        Ok(r) => r,
        Err(_) => return grid,
    };

    for i in 0..reflog.len() {
        let entry = match reflog.get(i) {
            Some(e) => e,
            None => continue,
        };

        let time = entry.committer().when().seconds();
        let days_ago = (now - time) / day_secs;
        if days_ago >= 91 {
            break; // reflog is newest-first
        }
        if days_ago < 0 {
            continue;
        }
        let idx = 90 - days_ago as usize;
        let msg = entry.message().unwrap_or("");

        if msg.starts_with("commit (merge)") || msg.starts_with("merge ") {
            grid[idx].merges = grid[idx].merges.saturating_add(1);
        } else if msg.starts_with("commit") {
            grid[idx].commits = grid[idx].commits.saturating_add(1);
        } else if msg.starts_with("checkout:") {
            grid[idx].branches = grid[idx].branches.saturating_add(1);
        } else if msg.starts_with("pull") {
            grid[idx].pulls = grid[idx].pulls.saturating_add(1);
        }
    }

    grid
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

// ── Server info (remote disk + repo sizes) ────────────────────────────────

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub host: String,
    pub disk_total: String,
    pub disk_used: String,
    pub disk_available: String,
    pub disk_use_percent: u8,
    pub repos: Vec<(String, String)>, // (name, size) sorted by size desc
    pub error: Option<String>,
}

pub fn gather_server_info(host: &str, path: &str) -> ServerInfo {
    let cmd = format!(
        "df -h / | tail -1; echo '---'; du -sh ~/{path}/*.git 2>/dev/null | sort -rh"
    );
    let output = Command::new("ssh")
        .args(["-o", "ConnectTimeout=3", host, &cmd])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            return ServerInfo {
                host: host.to_string(),
                disk_total: String::new(),
                disk_used: String::new(),
                disk_available: String::new(),
                disk_use_percent: 0,
                repos: Vec::new(),
                error: Some(if err.is_empty() { "SSH failed".into() } else { err }),
            };
        }
        Err(e) => {
            return ServerInfo {
                host: host.to_string(),
                disk_total: String::new(),
                disk_used: String::new(),
                disk_available: String::new(),
                disk_use_percent: 0,
                repos: Vec::new(),
                error: Some(format!("SSH error: {e}")),
            };
        }
    };

    let mut parts = output.splitn(2, "---\n");
    let df_line = parts.next().unwrap_or("").trim();
    let du_part = parts.next().unwrap_or("").trim();

    // Parse df line: filesystem  size  used  avail  use%  mount
    let df_cols: Vec<&str> = df_line.split_whitespace().collect();
    let (disk_total, disk_used, disk_available, disk_use_percent) = if df_cols.len() >= 5 {
        let pct = df_cols[4].trim_end_matches('%').parse::<u8>().unwrap_or(0);
        (
            df_cols[1].to_string(),
            df_cols[2].to_string(),
            df_cols[3].to_string(),
            pct,
        )
    } else {
        (String::new(), String::new(), String::new(), 0)
    };

    // Parse du lines: size\tpath
    let repos: Vec<(String, String)> = du_part
        .lines()
        .filter_map(|line| {
            let mut cols = line.split('\t');
            let size = cols.next()?.trim().to_string();
            let repo_path = cols.next()?.trim();
            let name = repo_path
                .rsplit('/')
                .next()?
                .strip_suffix(".git")
                .unwrap_or(repo_path.rsplit('/').next()?)
                .to_string();
            Some((name, size))
        })
        .collect();

    ServerInfo {
        host: host.to_string(),
        disk_total,
        disk_used,
        disk_available,
        disk_use_percent,
        repos,
        error: None,
    }
}

// ── File entries for commit review ─────────────────────────────────────────

fn parse_numstat(output: &str) -> HashMap<String, (usize, usize)> {
    let mut map = HashMap::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() == 3 {
            let ins = parts[0].parse::<usize>().unwrap_or(0); // binary = "-"
            let del = parts[1].parse::<usize>().unwrap_or(0);
            map.insert(parts[2].to_string(), (ins, del));
        }
    }
    map
}

fn count_file_lines(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

pub fn gather_file_entries(path: &Path) -> Vec<FileEntry> {
    let repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let workdir = match repo.workdir() {
        Some(w) => w.to_path_buf(),
        None => return Vec::new(),
    };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    // Gather diff stats via git diff --numstat (unstaged) + --cached (staged)
    let unstaged_stats = Command::new("git")
        .args(["diff", "--numstat"])
        .current_dir(path)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let cached_stats = Command::new("git")
        .args(["diff", "--cached", "--numstat"])
        .current_dir(path)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let unstaged_map = parse_numstat(&unstaged_stats);
    let cached_map = parse_numstat(&cached_stats);

    let mut entries = Vec::new();

    for entry in statuses.iter() {
        let s = entry.status();
        let file_path = match entry.path() {
            Some(p) => p.to_string(),
            None => continue,
        };

        let status_char = if s.is_conflicted() {
            'C'
        } else if s.is_wt_new() {
            '?'
        } else if s.is_index_new() {
            'A'
        } else if s.is_wt_deleted() || s.is_index_deleted() {
            'D'
        } else if s.is_wt_renamed() || s.is_index_renamed() {
            'R'
        } else {
            'M'
        };

        // Compute insertions/deletions
        let (mut ins, mut del) = (0usize, 0usize);
        if let Some(&(i, d)) = unstaged_map.get(&file_path) {
            ins += i;
            del += d;
        }
        if let Some(&(i, d)) = cached_map.get(&file_path) {
            ins += i;
            del += d;
        }
        // For untracked files, count lines as insertions
        if status_char == '?' {
            ins = count_file_lines(&workdir.join(&file_path));
        }

        entries.push(FileEntry {
            path: file_path,
            status_char,
            insertions: ins,
            deletions: del,
        });
    }

    entries
}
