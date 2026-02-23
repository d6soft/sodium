use crate::config::SodiumConfig;
use crate::git::{self, FileEntry, ProjectSummary, RepoInfo};
use std::path::PathBuf;
use std::process::Command;

// ── Screen ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    ProjectList,
    ProjectDetail,
}

// ── Menu items ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum MenuItem {
    Action(ActionKind, String),
    Separator,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionKind {
    NewBranch,
    Commit,
    SwitchBranch,
    Fetch,
    Pull,
    Merge,
    Backup,
    Push,
    History,
    Reinit,
    Quit,
}

// ── Input mode ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CommitReviewState {
    pub files: Vec<FileEntry>,
    pub selected: Vec<bool>,
    pub cursor: usize,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    TextInput { prompt: String, purpose: InputPurpose },
    Confirm { prompt: String, purpose: ConfirmPurpose },
    Select { prompt: String, purpose: SelectPurpose, options: Vec<String>, index: usize },
    CommitReview,
    CommitSelect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputPurpose {
    BranchName,
    CommitMessage,
    RepoName,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmPurpose {
    Reinit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectPurpose {
    SwitchBranch,
    MergeBranch,
}

// ── Notification ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub is_error: bool,
    pub tick: usize,
}

// ── Glitch state ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GlitchState {
    pub active: bool,
    pub frames_left: u8,
}

// ── Application state ──────────────────────────────────────────────────────

pub struct App {
    pub repo_path: PathBuf,
    pub repo_info: RepoInfo,
    pub menu_items: Vec<MenuItem>,
    pub menu_index: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub notification: Option<Notification>,
    pub tick: usize,
    pub glitch: GlitchState,
    pub should_quit: bool,
    pub subtitle_index: usize,
    // Multi-project support
    pub screen: Screen,
    pub config: Option<SodiumConfig>,
    pub projects: Vec<ProjectSummary>,
    pub project_index: usize,
    // Commit review state
    pub commit_review: Option<CommitReviewState>,
}

const SUBTITLES: &[&str] = &[
    "DOUGHCON INTELLIGENCE WATCH",
    "MONITORING GIT THREAT LEVELS",
    "BRANCH INTELLIGENCE NETWORK",
    "COMMIT SURVEILLANCE ACTIVE",
    "TRACKING MERGE ANOMALIES",
    "CLASSIFIED REPO ANALYTICS",
    "OPERATIONAL BRANCH CONTROL",
];

impl App {
    /// Create in multi-project mode (with config).
    pub fn new_multi(config: SodiumConfig) -> Self {
        let subtitle_index = rand::random::<usize>() % SUBTITLES.len();
        let mut app = Self {
            repo_path: PathBuf::new(),
            repo_info: RepoInfo::default(),
            menu_items: Vec::new(),
            menu_index: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            notification: None,
            tick: 0,
            glitch: GlitchState {
                active: true,
                frames_left: 6,
            },
            should_quit: false,
            subtitle_index,
            screen: Screen::ProjectList,
            config: Some(config),
            projects: Vec::new(),
            project_index: 0,
            commit_review: None,
        };
        app.discover_projects();
        app
    }

    /// Create in single-project mode (backward compat, no config).
    pub fn new(repo_path: PathBuf) -> Self {
        let repo_info = git::gather_repo_info(&repo_path).unwrap_or_default();
        let subtitle_index = rand::random::<usize>() % SUBTITLES.len();
        let mut app = Self {
            repo_path,
            repo_info,
            menu_items: Vec::new(),
            menu_index: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            notification: None,
            tick: 0,
            glitch: GlitchState {
                active: true,
                frames_left: 6,
            },
            should_quit: false,
            subtitle_index,
            screen: Screen::ProjectDetail,
            config: None,
            projects: Vec::new(),
            project_index: 0,
            commit_review: None,
        };
        app.rebuild_menu();
        app
    }

    pub fn subtitle(&self) -> &'static str {
        SUBTITLES[self.subtitle_index % SUBTITLES.len()]
    }

    pub fn refresh(&mut self) {
        self.repo_info = git::gather_repo_info(&self.repo_path).unwrap_or_default();
        self.rebuild_menu();
    }

    pub fn tick(&mut self) {
        self.tick += 1;

        // Glitch effect on startup
        if self.glitch.active {
            if self.glitch.frames_left == 0 {
                self.glitch.active = false;
            } else {
                self.glitch.frames_left -= 1;
            }
        }

        // Expire notifications after 40 ticks (~4 seconds)
        if let Some(ref notif) = self.notification {
            if self.tick.saturating_sub(notif.tick) > 40 {
                self.notification = None;
            }
        }
    }

    pub fn notify(&mut self, message: impl Into<String>, is_error: bool) {
        self.notification = Some(Notification {
            message: message.into(),
            is_error,
            tick: self.tick,
        });
    }

    fn rebuild_menu(&mut self) {
        let on_main = self.repo_info.current_branch == "main";
        let branch = &self.repo_info.current_branch;

        let mut items = vec![
            MenuItem::Action(ActionKind::NewBranch, "New branch".into()),
            MenuItem::Action(
                ActionKind::Commit,
                format!("Commit [{}]", branch),
            ),
            MenuItem::Action(ActionKind::SwitchBranch, "Switch branch".into()),
            MenuItem::Action(ActionKind::Fetch, "Fetch (refresh)".into()),
            MenuItem::Action(ActionKind::Pull, "Pull origin".into()),
            MenuItem::Separator,
        ];

        if on_main {
            items.push(MenuItem::Action(
                ActionKind::Merge,
                "Merge into main".into(),
            ));
        } else {
            items.push(MenuItem::Action(
                ActionKind::Merge,
                format!("Merge {} -> main", branch),
            ));
            items.push(MenuItem::Action(
                ActionKind::Backup,
                format!("Backup {} -> origin", branch),
            ));
        }

        items.push(MenuItem::Action(
            ActionKind::Push,
            "Push main -> origin".into(),
        ));
        items.push(MenuItem::Action(ActionKind::History, "Export history".into()));
        items.push(MenuItem::Separator);
        items.push(MenuItem::Action(
            ActionKind::Reinit,
            "Reinitialize repo".into(),
        ));
        items.push(MenuItem::Separator);
        items.push(MenuItem::Action(ActionKind::Quit, "Quit".into()));

        self.menu_items = items;
        // Ensure index is valid and on a selectable item
        if self.menu_index >= self.menu_items.len() {
            self.menu_index = 0;
        }
        self.skip_separators_down();
    }

    pub fn menu_up(&mut self) {
        if self.menu_items.is_empty() {
            return;
        }
        loop {
            if self.menu_index == 0 {
                break;
            }
            self.menu_index -= 1;
            if matches!(self.menu_items[self.menu_index], MenuItem::Action(..)) {
                break;
            }
        }
    }

    pub fn menu_down(&mut self) {
        if self.menu_items.is_empty() {
            return;
        }
        loop {
            if self.menu_index >= self.menu_items.len() - 1 {
                break;
            }
            self.menu_index += 1;
            if matches!(self.menu_items[self.menu_index], MenuItem::Action(..)) {
                break;
            }
        }
    }

    fn skip_separators_down(&mut self) {
        while self.menu_index < self.menu_items.len() {
            if matches!(self.menu_items[self.menu_index], MenuItem::Action(..)) {
                break;
            }
            self.menu_index += 1;
        }
    }

    pub fn selected_action(&self) -> Option<ActionKind> {
        match self.menu_items.get(self.menu_index) {
            Some(MenuItem::Action(kind, _)) => Some(*kind),
            _ => None,
        }
    }

    // ── Select mode navigation ──────────────────────────────────────────

    pub fn select_up(&mut self) {
        if let InputMode::Select { ref mut index, .. } = self.input_mode {
            if *index > 0 {
                *index -= 1;
            }
        }
    }

    pub fn select_down(&mut self) {
        if let InputMode::Select { ref mut index, ref options, .. } = self.input_mode {
            if *index < options.len().saturating_sub(1) {
                *index += 1;
            }
        }
    }

    // ── Action dispatch ─────────────────────────────────────────────────

    pub fn execute_action(&mut self) {
        let action = match self.selected_action() {
            Some(a) => a,
            None => return,
        };

        match action {
            ActionKind::Quit => {
                self.should_quit = true;
            }
            ActionKind::NewBranch => {
                self.input_mode = InputMode::TextInput {
                    prompt: "Branch name".into(),
                    purpose: InputPurpose::BranchName,
                };
                self.input_buffer.clear();
            }
            ActionKind::Commit => {
                if self.repo_info.files.modified + self.repo_info.files.untracked + self.repo_info.files.staged == 0 {
                    self.notify("[SIGINT] Nothing to commit — tree clean", false);
                    return;
                }
                let files = git::gather_file_entries(&self.repo_path);
                if files.is_empty() {
                    self.notify("[SIGINT] Nothing to commit — tree clean", false);
                    return;
                }
                let len = files.len();
                self.commit_review = Some(CommitReviewState {
                    files,
                    selected: vec![false; len],
                    cursor: 0,
                    scroll_offset: 0,
                });
                self.input_mode = InputMode::CommitReview;
            }
            ActionKind::SwitchBranch => {
                let branches: Vec<String> = self.repo_info.branches.iter()
                    .filter(|b| b.is_local && !b.is_current)
                    .map(|b| b.name.clone())
                    .collect();

                if branches.is_empty() {
                    self.notify("[INTEL] No other branches available", false);
                    return;
                }

                self.input_mode = InputMode::Select {
                    prompt: "Switch branch".into(),
                    purpose: SelectPurpose::SwitchBranch,
                    options: branches,
                    index: 0,
                };
            }
            ActionKind::Fetch => {
                self.do_fetch();
            }
            ActionKind::Pull => {
                self.do_pull();
            }
            ActionKind::Merge => {
                if self.repo_info.current_branch == "main" {
                    // On main: pick a branch to merge
                    let branches: Vec<String> = self.repo_info.branches.iter()
                        .filter(|b| b.is_local && b.name != "main")
                        .map(|b| b.name.clone())
                        .collect();

                    if branches.is_empty() {
                        self.notify("[INTEL] No branches to merge", false);
                        return;
                    }

                    self.input_mode = InputMode::Select {
                        prompt: "Merge into main".into(),
                        purpose: SelectPurpose::MergeBranch,
                        options: branches,
                        index: 0,
                    };
                } else {
                    // On feature: merge into main directly
                    let branch = self.repo_info.current_branch.clone();
                    self.do_merge(&branch);
                }
            }
            ActionKind::Push => {
                self.do_push();
            }
            ActionKind::Backup => {
                self.do_backup();
            }
            ActionKind::History => {
                self.do_history();
            }
            ActionKind::Reinit => {
                self.input_mode = InputMode::Confirm {
                    prompt: "Type CONFIRM to reinitialize (destructive)".into(),
                    purpose: ConfirmPurpose::Reinit,
                };
                self.input_buffer.clear();
            }
        }
    }

    pub fn submit_input(&mut self) {
        let mode = self.input_mode.clone();
        let buf = self.input_buffer.clone();
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();

        match mode {
            InputMode::TextInput { purpose, .. } => match purpose {
                InputPurpose::BranchName => self.do_new_branch(&buf),
                InputPurpose::CommitMessage => self.do_commit(&buf),
                InputPurpose::RepoName => self.do_reinit(&buf),
            },
            InputMode::Confirm { purpose, .. } => match purpose {
                ConfirmPurpose::Reinit => {
                    if buf == "CONFIRM" {
                        // Chain to repo name input
                        let default_name = self.repo_info.name.clone();
                        self.input_mode = InputMode::TextInput {
                            prompt: format!("Repo name ({})", default_name),
                            purpose: InputPurpose::RepoName,
                        };
                        self.input_buffer = default_name;
                    } else {
                        self.notify("[ABORT] Operation cancelled", false);
                    }
                }
            },
            InputMode::Select { purpose, options, index, .. } => {
                if let Some(selected) = options.get(index) {
                    let selected = selected.clone();
                    match purpose {
                        SelectPurpose::SwitchBranch => self.do_switch_branch(&selected),
                        SelectPurpose::MergeBranch => self.do_merge(&selected),
                    }
                }
            }
            InputMode::Normal | InputMode::CommitReview | InputMode::CommitSelect => {}
        }
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.commit_review = None;
    }

    // ── Commit review navigation ────────────────────────────────────────

    pub fn commit_review_up(&mut self) {
        if let Some(ref mut state) = self.commit_review {
            if state.cursor > 0 {
                state.cursor -= 1;
                if state.cursor < state.scroll_offset {
                    state.scroll_offset = state.cursor;
                }
            }
        }
    }

    pub fn commit_review_down(&mut self) {
        if let Some(ref mut state) = self.commit_review {
            if state.cursor < state.files.len().saturating_sub(1) {
                state.cursor += 1;
            }
        }
    }

    pub fn commit_toggle_file(&mut self) {
        if let Some(ref mut state) = self.commit_review {
            let i = state.cursor;
            if i < state.selected.len() {
                state.selected[i] = !state.selected[i];
            }
        }
    }

    pub fn commit_select_all(&mut self) {
        if let Some(ref mut state) = self.commit_review {
            for s in state.selected.iter_mut() {
                *s = true;
            }
        }
    }

    pub fn commit_select_none(&mut self) {
        if let Some(ref mut state) = self.commit_review {
            for s in state.selected.iter_mut() {
                *s = false;
            }
        }
    }

    pub fn commit_add_all(&mut self) {
        // Select all files and go straight to commit message
        if let Some(ref mut state) = self.commit_review {
            for s in state.selected.iter_mut() {
                *s = true;
            }
        }
        self.input_mode = InputMode::TextInput {
            prompt: "Commit message".into(),
            purpose: InputPurpose::CommitMessage,
        };
        self.input_buffer.clear();
    }

    pub fn commit_enter_select(&mut self) {
        // Enter multi-select mode — pre-select nothing
        self.input_mode = InputMode::CommitSelect;
    }

    pub fn commit_confirm_selection(&mut self) {
        let has_selection = self
            .commit_review
            .as_ref()
            .map(|s| s.selected.iter().any(|&v| v))
            .unwrap_or(false);

        if !has_selection {
            self.notify("[ABORT] No files selected", true);
            return;
        }
        self.input_mode = InputMode::TextInput {
            prompt: "Commit message".into(),
            purpose: InputPurpose::CommitMessage,
        };
        self.input_buffer.clear();
    }

    // ── Multi-project navigation ───────────────────────────────────────

    pub fn discover_projects(&mut self) {
        let config = match &self.config {
            Some(c) => c.clone(),
            None => return,
        };
        let root = config.dev_root_path();
        let mut projects = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                // Skip hidden directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                } else {
                    continue;
                }
                projects.push(git::gather_project_summary(&path));
            }
        }

        projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.projects = projects;
        if self.project_index >= self.projects.len() {
            self.project_index = 0;
        }
    }

    pub fn project_up(&mut self) {
        if self.project_index > 0 {
            self.project_index -= 1;
        }
    }

    pub fn project_down(&mut self) {
        if !self.projects.is_empty() && self.project_index < self.projects.len() - 1 {
            self.project_index += 1;
        }
    }

    pub fn enter_project(&mut self) {
        if let Some(proj) = self.projects.get(self.project_index) {
            self.repo_path = proj.path.clone();
            self.repo_info = git::gather_repo_info(&self.repo_path).unwrap_or_default();
            self.rebuild_menu();
            self.menu_index = 0;
            self.screen = Screen::ProjectDetail;
            // Trigger glitch effect on transition
            self.glitch = GlitchState {
                active: true,
                frames_left: 4,
            };
        }
    }

    pub fn back_to_list(&mut self) {
        if self.config.is_none() {
            return;
        }
        self.discover_projects();
        self.screen = Screen::ProjectList;
    }

    pub fn refresh_projects(&mut self) {
        self.discover_projects();
    }

    pub fn is_multi_project(&self) -> bool {
        self.config.is_some()
    }

    // ── Git operations ──────────────────────────────────────────────────

    fn do_new_branch(&mut self, name: &str) {
        if name.is_empty() {
            self.notify("[ABORT] Empty branch name", true);
            return;
        }
        let output = Command::new("git")
            .args(["checkout", "-b", name])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                self.notify(format!("[INTEL] Branch '{}' created & active", name), false);
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] {}", err.trim()), true);
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_commit(&mut self, msg: &str) {
        if msg.is_empty() {
            self.notify("[ABORT] Empty commit message", true);
            return;
        }

        // Stage selected files (or all if no commit_review state)
        if let Some(ref state) = self.commit_review {
            let files_to_add: Vec<&str> = state
                .files
                .iter()
                .zip(state.selected.iter())
                .filter(|(_, &sel)| sel)
                .map(|(f, _)| f.path.as_str())
                .collect();

            if files_to_add.is_empty() {
                self.notify("[ABORT] No files selected", true);
                self.commit_review = None;
                return;
            }

            let mut args = vec!["add", "--"];
            args.extend(files_to_add);
            let _ = Command::new("git")
                .args(&args)
                .current_dir(&self.repo_path)
                .output();
        } else {
            let _ = Command::new("git")
                .args(["add", "-A"])
                .current_dir(&self.repo_path)
                .output();
        }

        self.commit_review = None;

        let output = Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                self.notify(format!("[SIGINT] Commit recorded: {}", msg), false);
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] {}", err.trim()), true);
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_switch_branch(&mut self, branch: &str) {
        let output = Command::new("git")
            .args(["checkout", branch])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                self.notify(format!("[INTEL] Switched to '{}'", branch), false);
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] {}", err.trim()), true);
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_fetch(&mut self) {
        let output = Command::new("git")
            .args(["fetch", "--prune", "origin"])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                self.notify("[INTEL] Fetch complete — remote intel updated", false);
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                // git fetch writes to stderr even on success
                if o.status.code() == Some(0) || err.contains("From") {
                    self.notify("[INTEL] Fetch complete", false);
                    self.refresh();
                } else {
                    self.notify(format!("[ERROR] {}", err.trim()), true);
                }
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_pull(&mut self) {
        let branch = self.repo_info.current_branch.clone();
        let rebase = self
            .config
            .as_ref()
            .map(|c| c.pull_rebase)
            .unwrap_or(true);

        let mut args = vec!["pull"];
        if rebase {
            args.push("--rebase");
        }
        args.push("origin");
        args.push(&branch);

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let mode = if rebase { "rebase" } else { "merge" };
                self.notify(
                    format!("[INTEL] Pull complete ({}) — branch synced", mode),
                    false,
                );
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                if err.contains("Already up to date") {
                    self.notify("[INTEL] Already up to date", false);
                } else {
                    self.notify(format!("[ERROR] {}", err.trim()), true);
                }
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_merge(&mut self, branch: &str) {
        let on_main = self.repo_info.current_branch == "main";

        // If not on main, checkout main first
        if !on_main {
            let checkout = Command::new("git")
                .args(["checkout", "main"])
                .current_dir(&self.repo_path)
                .output();
            match checkout {
                Ok(o) if !o.status.success() => {
                    let err = String::from_utf8_lossy(&o.stderr);
                    self.notify(format!("[ERROR] checkout main: {}", err.trim()), true);
                    return;
                }
                Err(e) => {
                    self.notify(format!("[ERROR] {}", e), true);
                    return;
                }
                _ => {}
            }
        }

        let output = Command::new("git")
            .args(["merge", branch])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                self.notify(format!("[SIGINT] '{}' merged into main", branch), false);
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] {}", err.trim()), true);
                self.refresh(); // Refresh to show conflict state
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_push(&mut self) {
        let output = Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let cleaned = self.cleanup_merged_branches();
                if cleaned > 0 {
                    self.notify(
                        format!("[SIGINT] Push complete — {} merged branch(es) cleaned", cleaned),
                        false,
                    );
                } else {
                    self.notify("[SIGINT] Push complete — intel transmitted", false);
                }
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                if err.contains("Everything up-to-date") {
                    self.notify("[INTEL] Already synced — nothing to transmit", false);
                } else {
                    self.notify(format!("[ERROR] {}", err.trim()), true);
                }
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_backup(&mut self) {
        let branch = self.repo_info.current_branch.clone();
        if branch == "main" {
            self.notify("[ABORT] Use Push for main branch", true);
            return;
        }
        let output = Command::new("git")
            .args(["push", "origin", &branch])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                self.notify(format!("[SIGINT] {} backed up to origin", branch), false);
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                if err.contains("Everything up-to-date") {
                    self.notify("[INTEL] Already backed up", false);
                } else {
                    self.notify(format!("[ERROR] {}", err.trim()), true);
                }
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_history(&mut self) {
        let name = &self.repo_info.name;
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let filepath = self.repo_path.join(format!("docs/git-{}-{}.md", name, today));

        let _ = std::fs::create_dir_all(self.repo_path.join("docs"));

        let branch = &self.repo_info.current_branch;
        let remote_url = self.repo_info.remote_url.as_deref().unwrap_or("none");

        // Git log last 10 days
        let log = Command::new("git")
            .args([
                "log", "--all", "--since=10 days ago",
                "--format=### %h — %s\n- **Date** : %ci\n- **Auteur** : %an\n- **Branche** : %D\n",
            ])
            .current_dir(&self.repo_path)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        // All branches
        let branches = Command::new("git")
            .args(["branch", "-a", "--format=- `%(refname:short)`"])
            .current_dir(&self.repo_path)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        // Contributors
        let contributors = Command::new("git")
            .args(["log", "--all", "--format=%an"])
            .current_dir(&self.repo_path)
            .output()
            .map(|o| {
                let text = String::from_utf8_lossy(&o.stdout).to_string();
                let mut names: Vec<String> = text.lines().map(|s| s.to_string()).collect();
                names.sort();
                names.dedup();
                names.iter().map(|n| format!("- {}", n)).collect::<Vec<_>>().join("\n")
            })
            .unwrap_or_default();

        let total = self.repo_info.total_commits;
        let log_section = if log.trim().is_empty() {
            "_Aucune activité sur les 10 derniers jours._".to_string()
        } else {
            log
        };

        let content = format!(
            "# Git — {name}\n\n\
             - **Date** : {today}\n\
             - **Branche** : {branch}\n\
             - **Remote** : {remote_url}\n\n\
             ## Historique (10 derniers jours)\n\n\
             {log_section}\n\
             ---\n\n\
             ## Branches\n\n\
             {branches}\n\
             ## Statistiques\n\n\
             - **Commits total** : {total}\n\
             - **Contributeurs** :\n{contributors}\n"
        );

        match std::fs::write(&filepath, content) {
            Ok(_) => self.notify(
                format!("[INTEL] History exported: {}", filepath.display()),
                false,
            ),
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_reinit(&mut self, repo_name: &str) {
        if repo_name.is_empty() {
            self.notify("[ABORT] Empty repo name", true);
            return;
        }

        let (remote_host, remote_path) = match &self.config {
            Some(cfg) => (cfg.remote_host.clone(), cfg.remote_path.clone()),
            None => ("git-PM7".into(), "repos".into()),
        };

        let bare_path = format!("{}/{}.git", remote_path, repo_name);

        // Check if bare repo exists and delete it
        let check = Command::new("ssh")
            .args([&remote_host, &format!("test -d {}", bare_path)])
            .output();

        if let Ok(o) = check {
            if o.status.success() {
                let del = Command::new("ssh")
                    .args([&remote_host, &format!("rm -rf {}", bare_path)])
                    .output();
                if let Ok(o) = del {
                    if !o.status.success() {
                        self.notify("[ERROR] Failed to delete existing bare repo", true);
                        return;
                    }
                }
            }
        }

        // Create bare repo on remote
        let create = Command::new("ssh")
            .args([&remote_host, &format!("git init --bare {}", bare_path)])
            .output();

        match create {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] Bare init: {}", err.trim()), true);
                return;
            }
            Err(e) => {
                self.notify(format!("[ERROR] SSH: {}", e), true);
                return;
            }
        }

        // Remove local .git
        let git_dir = self.repo_path.join(".git");
        if git_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&git_dir) {
                self.notify(format!("[ERROR] rm .git: {}", e), true);
                return;
            }
        }

        // git init -b main
        let init = Command::new("git")
            .args(["init", "-b", "main", "."])
            .current_dir(&self.repo_path)
            .output();
        if !init.map(|o| o.status.success()).unwrap_or(false) {
            self.notify("[ERROR] git init failed", true);
            return;
        }

        // Add remote
        let remote_url = format!("{}:{}/{}.git", remote_host, remote_path, repo_name);
        let add_remote = Command::new("git")
            .args(["remote", "add", "origin", &remote_url])
            .current_dir(&self.repo_path)
            .output();
        if !add_remote.map(|o| o.status.success()).unwrap_or(false) {
            self.notify("[ERROR] git remote add failed", true);
            return;
        }

        // Generate .gitignore
        let gitignore = self.generate_gitignore();
        let _ = std::fs::write(self.repo_path.join(".gitignore"), gitignore);

        // git add -A && git commit
        let _ = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.repo_path)
            .output();

        let commit = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&self.repo_path)
            .output();
        if !commit.map(|o| o.status.success()).unwrap_or(false) {
            self.notify("[ERROR] Initial commit failed", true);
            return;
        }

        // Push
        let push = Command::new("git")
            .args(["push", "-u", "origin", "main"])
            .current_dir(&self.repo_path)
            .output();

        match push {
            Ok(o) if o.status.success() => {
                self.notify(
                    format!("[SIGINT] Repo reinitialized → {}", remote_url),
                    false,
                );
                self.refresh();
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] Push: {}", err.trim()), true);
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn generate_gitignore(&self) -> String {
        let mut gi = String::new();
        let p = &self.repo_path;

        gi.push_str("# ── OS / Editors ──\n");
        gi.push_str(".DS_Store\nThumbs.db\n");
        gi.push_str(".vscode/\n.idea/\n*.swp\n*.swo\n*~\n");
        gi.push_str("\n# ── Sensitive dotfiles ──\n");
        gi.push_str(".env\n.env.*\n.secret\n.secrets/\n");
        gi.push_str("\n# ── Logs / Temp ──\n");
        gi.push_str("*.log\n*.tmp\n*.bak\n*.pid\n");

        // Node / Svelte
        if p.join("package.json").exists() {
            gi.push_str("\n# ── Node / Svelte ──\n");
            gi.push_str("node_modules/\nbuild/\ndist/\n.svelte-kit/\n");
            gi.push_str(".vercel/\n.netlify/\n.pnpm-store/\n");
            // Capacitor
            if p.join("capacitor.config.ts").exists() || p.join("capacitor.config.json").exists() {
                gi.push_str("\n# ── Capacitor ──\n");
                gi.push_str("android/\nios/\n");
            }
        }

        // Rust
        if p.join("Cargo.toml").exists() {
            gi.push_str("\n# ── Rust ──\n");
            gi.push_str("target/\n*.rs.bk\n");
        }

        // Go
        if p.join("go.mod").exists() {
            gi.push_str("\n# ── Go ──\n");
            gi.push_str("bin/\nvendor/\n");
        }

        // Flutter
        if p.join("pubspec.yaml").exists() {
            gi.push_str("\n# ── Flutter / Dart ──\n");
            gi.push_str(".dart_tool/\n.flutter-plugins\n.flutter-plugins-dependencies\n");
            gi.push_str(".packages\nbuild/\n*.iml\n");
        }

        gi
    }

    /// Delete local + remote branches already merged into main. Returns count cleaned.
    fn cleanup_merged_branches(&mut self) -> usize {
        let output = Command::new("git")
            .args(["branch", "--merged", "main", "--format=%(refname:short)"])
            .current_dir(&self.repo_path)
            .output();

        let mut cleaned = 0;
        if let Ok(o) = output {
            let branches = String::from_utf8_lossy(&o.stdout);
            for branch in branches.lines() {
                let branch = branch.trim();
                if branch.is_empty() || branch == "main" {
                    continue;
                }
                // Delete local
                let _ = Command::new("git")
                    .args(["branch", "-d", branch])
                    .current_dir(&self.repo_path)
                    .output();
                // Delete remote
                let _ = Command::new("git")
                    .args(["push", "origin", "--delete", branch])
                    .current_dir(&self.repo_path)
                    .output();
                cleaned += 1;
            }
        }
        cleaned
    }
}
