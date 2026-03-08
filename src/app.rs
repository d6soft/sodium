use crate::config::SodiumConfig;
use crate::git::{self, FileEntry, ProjectSummary, RepoInfo, ServerInfo};
use crate::git_ops;
use std::collections::HashSet;
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
    SectionHeader(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionKind {
    NewBranch,
    Commit,
    SwitchBranch,
    Fetch,
    Pull,
    CheckoutRemote,
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
    CloneTarget(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmPurpose {
    Reinit,
    DeleteBareRepo(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectPurpose {
    SwitchBranch,
    MergeBranch,
    CheckoutRemoteBranch,
    ServerRepos,
}

// ── Pending operation (for pre-render before blocking) ────────────────

#[derive(Debug, Clone)]
pub enum PendingOp {
    Fetch,
    Pull,
    Push,
    Backup,
    Merge(String),
    Reinit(String),
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
    pub input_cursor: usize,
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
    // Flow hint
    pub flow_hint: Option<(ActionKind, String)>,
    // Actions completed in current session (for ✓ indicators)
    pub done_actions: HashSet<ActionKind>,
    // Running action indicator (shown before blocking op)
    pub running_action: Option<(ActionKind, String)>,
    pub pending_op: Option<PendingOp>,
    // Persistent message log (shown in Messages card)
    pub messages: Vec<Notification>,
    // Server disk info
    pub server_info: Option<ServerInfo>,
    // Focus: true = SERVER card, false = PROJECTS card
    pub server_focused: bool,
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
            input_cursor: 0,
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
            flow_hint: None,
            done_actions: HashSet::new(),
            running_action: None,
            pending_op: None,
            messages: Vec::new(),
            server_info: None,
            server_focused: false,
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
            input_cursor: 0,
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
            flow_hint: None,
            done_actions: HashSet::new(),
            running_action: None,
            pending_op: None,
            messages: Vec::new(),
            server_info: None,
            server_focused: false,
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
        let msg = message.into();
        self.notification = Some(Notification {
            message: msg.clone(),
            is_error,
            tick: self.tick,
        });
        self.messages.push(Notification {
            message: msg,
            is_error,
            tick: self.tick,
        });
    }

    fn rebuild_menu(&mut self) {
        let on_main = self.repo_info.current_branch == "main";
        let branch = self.repo_info.current_branch.clone();

        // ── Flow : numbered steps ──
        let mut items = vec![
            MenuItem::SectionHeader("FLOW".into()),
            MenuItem::Action(ActionKind::NewBranch, "1. New branch".into()),
            MenuItem::Action(
                ActionKind::Commit,
                format!("2. Commit [{}]", branch),
            ),
        ];

        if !on_main {
            items.push(MenuItem::Action(
                ActionKind::Backup,
                format!("3. Backup {} -> origin", branch),
            ));
            items.push(MenuItem::Action(
                ActionKind::Merge,
                format!("4. Merge {} -> main", branch),
            ));
        } else {
            items.push(MenuItem::Action(
                ActionKind::Merge,
                "3. Merge into main".into(),
            ));
        }

        items.push(MenuItem::Action(
            ActionKind::Push,
            format!("{}. Push main -> origin", if on_main { "4" } else { "5" }),
        ));

        // ── Extras ──
        items.push(MenuItem::SectionHeader("+".into()));
        items.push(MenuItem::Action(ActionKind::SwitchBranch, "Switch branch".into()));
        items.push(MenuItem::Action(ActionKind::Fetch, "Fetch origin (refresh)".into()));
        items.push(MenuItem::Action(ActionKind::Pull, "Pull origin".into()));
        items.push(MenuItem::Action(ActionKind::CheckoutRemote, "Checkout remote branch".into()));
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

        // Compute flow hint
        let dirty = self.repo_info.files.modified + self.repo_info.files.untracked + self.repo_info.files.staged;
        self.flow_hint = if on_main && self.repo_info.ahead > 0 {
            Some((ActionKind::Push, "Push main to origin".into()))
        } else if on_main {
            Some((ActionKind::NewBranch, "Create a new feature branch".into()))
        } else if dirty > 0 {
            Some((ActionKind::Commit, "Commit your changes".into()))
        } else if self.done_actions.contains(&ActionKind::NewBranch)
            && !self.done_actions.contains(&ActionKind::Commit)
        {
            Some((ActionKind::Commit, "Commit your changes".into()))
        } else {
            Some((ActionKind::Merge, format!("Merge {} into main", branch)))
        };
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
            if Self::is_selectable(&self.menu_items[self.menu_index]) {
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
            if Self::is_selectable(&self.menu_items[self.menu_index]) {
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

    fn is_selectable(item: &MenuItem) -> bool {
        matches!(item, MenuItem::Action(..))
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
                self.input_cursor = 0;
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
                self.running_action = Some((ActionKind::Fetch, "Fetch origin en cours...".into()));
                self.pending_op = Some(PendingOp::Fetch);
            }
            ActionKind::Pull => {
                self.running_action = Some((ActionKind::Pull, "Pull origin en cours...".into()));
                self.pending_op = Some(PendingOp::Pull);
            }
            ActionKind::CheckoutRemote => {
                let branches: Vec<String> = self.repo_info.branches.iter()
                    .filter(|b| b.is_remote && !b.is_local && b.name != "main")
                    .map(|b| b.name.clone())
                    .collect();

                if branches.is_empty() {
                    self.notify("[INTEL] No remote-only branches available", false);
                    return;
                }

                self.input_mode = InputMode::Select {
                    prompt: "Checkout remote branch".into(),
                    purpose: SelectPurpose::CheckoutRemoteBranch,
                    options: branches,
                    index: 0,
                };
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
                    self.running_action = Some((ActionKind::Merge, format!("Merge {} -> main en cours...", branch)));
                    self.pending_op = Some(PendingOp::Merge(branch));
                }
            }
            ActionKind::Push => {
                self.running_action = Some((ActionKind::Push, "Push main -> origin en cours...".into()));
                self.pending_op = Some(PendingOp::Push);
            }
            ActionKind::Backup => {
                let branch = self.repo_info.current_branch.clone();
                self.running_action = Some((ActionKind::Backup, format!("Backup {} -> origin en cours...", branch)));
                self.pending_op = Some(PendingOp::Backup);
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
                self.input_cursor = 0;
            }
        }
    }

    pub fn submit_input(&mut self) {
        let mode = self.input_mode.clone();
        let buf = self.input_buffer.clone();
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.input_cursor = 0;

        match mode {
            InputMode::TextInput { purpose, .. } => match purpose {
                InputPurpose::BranchName => self.do_new_branch(&buf),
                InputPurpose::CommitMessage => self.do_commit(&buf),
                InputPurpose::RepoName => {
                    self.running_action = Some((ActionKind::Reinit, "Reinitialize en cours...".into()));
                    self.pending_op = Some(PendingOp::Reinit(buf));
                }
                InputPurpose::CloneTarget(name) => self.do_clone_bare_repo(&name, &buf),
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
                        self.input_cursor = self.input_buffer.chars().count();
                    } else {
                        self.notify("[ABORT] Operation cancelled", false);
                    }
                }
                ConfirmPurpose::DeleteBareRepo(name) => {
                    if buf == "CONFIRM" {
                        self.do_delete_bare_repo(&name);
                    } else {
                        self.notify("[ABORT] Deletion cancelled", false);
                    }
                }
            },
            InputMode::Select { purpose, options, index, .. } => {
                if let Some(selected) = options.get(index) {
                    let selected = selected.clone();
                    match purpose {
                        SelectPurpose::SwitchBranch => self.do_switch_branch(&selected),
                        SelectPurpose::MergeBranch => {
                            self.running_action = Some((ActionKind::Merge, format!("Merge {} -> main en cours...", selected)));
                            self.pending_op = Some(PendingOp::Merge(selected));
                        }
                        SelectPurpose::CheckoutRemoteBranch => self.do_checkout_remote(&selected),
                        SelectPurpose::ServerRepos => {
                            // Enter alone does nothing; use [c] clone or [d] delete
                        }
                    }
                }
            }
            InputMode::Normal | InputMode::CommitReview | InputMode::CommitSelect => {}
        }
    }

    pub fn run_pending_op(&mut self) {
        if let Some(op) = self.pending_op.take() {
            match op {
                PendingOp::Fetch => self.do_fetch(),
                PendingOp::Pull => self.do_pull(),
                PendingOp::Push => self.do_push(),
                PendingOp::Backup => self.do_backup(),
                PendingOp::Merge(branch) => self.do_merge(&branch),
                PendingOp::Reinit(name) => self.do_reinit(&name),
            }
            self.running_action = None;
        }
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.input_cursor = 0;
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
        self.input_cursor = 0;
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
        self.input_cursor = 0;
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
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => continue,
                };
                // Skip hidden directories
                if name.starts_with('.') {
                    continue;
                }
                // Skip excluded directories
                if config.exclude.iter().any(|e| e == name) {
                    continue;
                }
                projects.push(git::gather_project_summary(&path));
            }
        }

        // Sort: repos first (alphabetical), then no-repo (alphabetical)
        projects.sort_by(|a, b| {
            match (a.has_git, b.has_git) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        self.projects = projects;
        if self.project_index >= self.projects.len() {
            self.project_index = 0;
        }
        // Refresh server disk info
        self.server_info = Some(git::gather_server_info(
            &config.remote_host,
            &config.remote_path,
        ));
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
            let is_no_repo = !proj.has_git;
            self.repo_path = proj.path.clone();
            self.repo_info = git::gather_repo_info(&self.repo_path).unwrap_or_default();
            self.done_actions.clear();
            self.messages.clear();
            let on_main = self.repo_info.current_branch == "main";
            if !on_main {
                self.done_actions.insert(ActionKind::NewBranch);
                if self.repo_info.ahead_of_main > 0 {
                    self.done_actions.insert(ActionKind::Commit);
                }
            }
            self.rebuild_menu();
            self.menu_index = if is_no_repo {
                self.menu_items.iter().position(|item| matches!(item, MenuItem::Action(ActionKind::Reinit, _))).unwrap_or(0)
            } else {
                0
            };
            self.skip_separators_down();
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
        self.done_actions.clear();
        self.messages.clear();
        self.discover_projects();
        self.screen = Screen::ProjectList;
    }

    pub fn refresh_projects(&mut self) {
        self.discover_projects();
    }

    pub fn toggle_server_focus(&mut self) {
        self.server_focused = !self.server_focused;
    }

    pub fn open_server_repos(&mut self) {
        let options: Vec<String> = match &self.server_info {
            Some(info) if info.error.is_none() => {
                info.repos.iter().map(|(name, size)| format!("{}  ({})", name, size)).collect()
            }
            _ => {
                self.notify("[ERROR] Server info not available", true);
                return;
            }
        };
        if options.is_empty() {
            self.notify("[INFO] No bare repos on server", false);
            return;
        }
        self.input_mode = InputMode::Select {
            prompt: "Server repos  [c] clone  [d] delete".into(),
            purpose: SelectPurpose::ServerRepos,
            options,
            index: 0,
        };
    }

    pub fn is_multi_project(&self) -> bool {
        self.config.is_some()
    }

    pub fn server_repo_clone(&mut self) {
        if let InputMode::Select { purpose: SelectPurpose::ServerRepos, options, index, .. } = &self.input_mode {
            if let Some(selected) = options.get(*index) {
                let name = selected.split("  (").next().unwrap_or(selected).to_string();
                self.input_mode = InputMode::TextInput {
                    prompt: format!("Clone target directory for {}", name),
                    purpose: InputPurpose::CloneTarget(name),
                };
                self.input_buffer.clear();
                self.input_cursor = 0;
            }
        }
    }

    pub fn server_repo_delete(&mut self) {
        if let InputMode::Select { purpose: SelectPurpose::ServerRepos, options, index, .. } = &self.input_mode {
            if let Some(selected) = options.get(*index) {
                let name = selected.split("  (").next().unwrap_or(selected).to_string();
                self.input_mode = InputMode::Confirm {
                    prompt: format!("Type CONFIRM to delete {}.git", name),
                    purpose: ConfirmPurpose::DeleteBareRepo(name),
                };
                self.input_buffer.clear();
                self.input_cursor = 0;
            }
        }
    }

    fn do_clone_bare_repo(&mut self, name: &str, target_dir: &str) {
        if target_dir.is_empty() {
            self.notify("[ABORT] Empty target directory", true);
            return;
        }

        let (remote_host, remote_path) = match &self.config {
            Some(cfg) => (cfg.remote_host.clone(), cfg.remote_path.clone()),
            None => return,
        };

        let url = format!("{}:{}/{}.git", remote_host, remote_path, name);
        let expanded_target = if target_dir.starts_with('~') {
            target_dir.replacen('~', &std::env::var("HOME").unwrap_or_default(), 1)
        } else {
            target_dir.to_string()
        };
        let dest = format!("{}/{}", expanded_target, name);

        let result = Command::new("git")
            .args(["clone", &url, &dest])
            .output();

        match result {
            Ok(o) if o.status.success() => {
                self.notify(format!("[OK] Cloned {} into {}", name, dest), false);
                // Refresh projects if target is within dev_root
                if let Some(cfg) = &self.config {
                    let dev_root = cfg.dev_root_path().to_string_lossy().to_string();
                    if expanded_target.starts_with(&dev_root) {
                        self.discover_projects();
                    }
                }
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] {}", err.trim()), true);
            }
            Err(e) => {
                self.notify(format!("[ERROR] git clone: {}", e), true);
            }
        }
    }

    // ── Git operations ──────────────────────────────────────────────────

    fn do_new_branch(&mut self, name: &str) {
        match git_ops::git_new_branch(&self.repo_path, name) {
            Ok(msg) => {
                self.done_actions.insert(ActionKind::NewBranch);
                self.notify(format!("[INTEL] {}", msg), false);
                self.refresh();
                if let Some(idx) = self.menu_items.iter().position(|item| {
                    matches!(item, MenuItem::Action(ActionKind::Commit, _))
                }) {
                    self.menu_index = idx;
                }
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_commit(&mut self, msg: &str) {
        // Build file list from commit review state
        let files: Vec<String> = if let Some(ref state) = self.commit_review {
            let selected: Vec<String> = state
                .files
                .iter()
                .zip(state.selected.iter())
                .filter(|(_, &sel)| sel)
                .map(|(f, _)| f.path.clone())
                .collect();
            if selected.is_empty() {
                self.notify("[ABORT] No files selected", true);
                self.commit_review = None;
                return;
            }
            selected
        } else {
            Vec::new() // empty = add all
        };

        self.commit_review = None;

        match git_ops::git_commit(&self.repo_path, msg, &files) {
            Ok(m) => {
                self.done_actions.insert(ActionKind::Commit);
                self.notify(format!("[SIGINT] {}", m), false);
                self.refresh();
                if let Some(idx) = self.menu_items.iter().position(|item| {
                    matches!(item, MenuItem::Action(ActionKind::Backup, _))
                }) {
                    self.menu_index = idx;
                }
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_switch_branch(&mut self, branch: &str) {
        match git_ops::git_switch_branch(&self.repo_path, branch) {
            Ok(msg) => {
                self.notify(format!("[INTEL] {}", msg), false);
                self.refresh();
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_checkout_remote(&mut self, branch: &str) {
        let remote_ref = format!("origin/{}", branch);
        let output = Command::new("git")
            .args(["checkout", "-b", branch, &remote_ref])
            .current_dir(&self.repo_path)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                self.notify(format!("[INTEL] Checked out '{}' from remote", branch), false);
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
        match git_ops::git_fetch(&self.repo_path) {
            Ok(msg) => {
                self.notify(format!("[INTEL] {}", msg), false);
                self.refresh();
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_pull(&mut self) {
        let branch = self.repo_info.current_branch.clone();

        let has_remote = self.repo_info.branches.iter()
            .any(|b| b.name == branch && b.is_remote);
        if !has_remote {
            self.notify(format!("[INTEL] '{}' has no remote — nothing to pull", branch), false);
            return;
        }

        let rebase = self
            .config
            .as_ref()
            .map(|c| c.pull_rebase)
            .unwrap_or(true);

        match git_ops::git_pull(&self.repo_path, &branch, rebase) {
            Ok(msg) => {
                self.notify(format!("[INTEL] {}", msg), false);
                self.refresh();
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
                self.done_actions.insert(ActionKind::Merge);
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
        match git_ops::git_push_main(&self.repo_path) {
            Ok((msg, cleaned)) => {
                self.done_actions.insert(ActionKind::Push);
                let gh_suffix = self.mirror_to_github("main").unwrap_or_default();
                if cleaned > 0 {
                    self.notify(
                        format!("[SIGINT] Push complete — {} merged branch(es) cleaned{}", cleaned, gh_suffix),
                        false,
                    );
                } else {
                    self.notify(format!("[SIGINT] {}{}", msg, gh_suffix), false);
                }
                self.refresh();
            }
            Err(e) => self.notify(format!("[ERROR] {}", e), true),
        }
    }

    fn do_backup(&mut self) {
        let branch = self.repo_info.current_branch.clone();
        match git_ops::git_backup(&self.repo_path, &branch) {
            Ok(msg) => {
                self.done_actions.insert(ActionKind::Backup);
                let gh_suffix = self.mirror_to_github(&branch).unwrap_or_default();
                self.notify(format!("[SIGINT] {}{}", msg, gh_suffix), false);
                self.refresh();
            }
            Err(e) => {
                if e == "Use Push for main branch" {
                    self.notify(format!("[ABORT] {}", e), true);
                } else {
                    self.notify(format!("[ERROR] {}", e), true);
                }
            }
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

    fn do_delete_bare_repo(&mut self, name: &str) {
        let (remote_host, remote_path) = match &self.config {
            Some(cfg) => (cfg.remote_host.clone(), cfg.remote_path.clone()),
            None => return,
        };

        let bare_path = format!("{}/{}.git", remote_path, name);
        let result = Command::new("ssh")
            .args([&remote_host, &format!("rm -rf {}", bare_path)])
            .output();

        match result {
            Ok(o) if o.status.success() => {
                self.notify(format!("[OK] Deleted {}.git", name), false);
                // Refresh server info
                self.server_info = Some(git::gather_server_info(&remote_host, &remote_path));
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.notify(format!("[ERROR] {}", err.trim()), true);
            }
            Err(e) => {
                self.notify(format!("[ERROR] SSH: {}", e), true);
            }
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

        // Add github remote if configured
        if let Some(gh_url) = self
            .config
            .as_ref()
            .and_then(|c| c.github_url(repo_name).map(String::from))
        {
            let _ = Command::new("git")
                .args(["remote", "add", "github", &gh_url])
                .current_dir(&self.repo_path)
                .output();
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
                self.menu_index = 0;
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

    /// Mirror push a branch to the `github` remote if configured.
    /// Returns a suffix string (e.g. " + GitHub") on success, or None.
    fn mirror_to_github(&self, branch: &str) -> Option<String> {
        let project_name = &self.repo_info.name;
        let github_url = self
            .config
            .as_ref()
            .and_then(|c| c.github_url(project_name).map(String::from))?;

        // Ensure the github remote exists
        let check = Command::new("git")
            .args(["remote", "get-url", "github"])
            .current_dir(&self.repo_path)
            .output();

        let has_remote = check.map(|o| o.status.success()).unwrap_or(false);
        if !has_remote {
            let add = Command::new("git")
                .args(["remote", "add", "github", &github_url])
                .current_dir(&self.repo_path)
                .output();
            if !add.map(|o| o.status.success()).unwrap_or(false) {
                return None;
            }
        }

        // Force push to github (mirror — origin is source of truth)
        let push = Command::new("git")
            .args(["push", "--force", "github", branch])
            .current_dir(&self.repo_path)
            .output();

        match push {
            Ok(o) if o.status.success() => Some(" + GitHub".into()),
            _ => None,
        }
    }

}
