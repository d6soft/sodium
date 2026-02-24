use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
    Frame,
};

use chrono::Datelike;

use crate::app::{App, CommitReviewState, InputMode, MenuItem, Screen};
use crate::theme;

// ── Helpers ────────────────────────────────────────────────────────────────

fn truncate_str(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

// ── ASCII art header ───────────────────────────────────────────────────────

const SODIUM_LOGO: &[&str] = &[
    "◉━━━ ░▒▓    ███████  ██████  ██████  ██ ██    ██ ███    ███    ▓▒░ ━━━◉",
    "┃    ░▒▓    ██      ██    ██ ██   ██ ██ ██    ██ ████  ████    ▓▒░    ┃",
    "◎━▸▸ ░▒▓    ███████ ██    ██ ██   ██ ██ ██    ██ ██ ████ ██    ▓▒░ ◂◂━◎",
    "┃    ░▒▓         ██ ██    ██ ██   ██ ██ ██    ██ ██  ██  ██    ▓▒░    ┃",
    "◉━━━ ░▒▓    ███████  ██████  ██████  ██  ██████  ██      ██    ▓▒░ ━━━◉",
];

const GLITCH_CHARS: &[char] = &[
    '█', '▓', '░', '▒', '╳', '◈', '◇', '▲', '●', '╬', '┼', '╪', '∎', '⊗', '⌬',
];

// ── Main render ────────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // Background fill
    let bg_block = Block::default().style(Style::default().bg(theme::BG));
    f.render_widget(bg_block, area);

    match app.screen {
        Screen::ProjectList => render_project_list(f, app, area),
        Screen::ProjectDetail => {
            // Main layout: header | body | footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(9),  // Header + GITCON
                    Constraint::Min(10),   // Body
                    Constraint::Length(3), // Footer
                ])
                .split(area);

            render_header(f, app, chunks[0]);
            render_body(f, app, chunks[1]);
            render_footer(f, app, chunks[2]);

            // Overlay: input mode
            if app.input_mode != InputMode::Normal {
                render_input_overlay(f, app, area);
            }
        }
    }
}

// ── Project list screen ────────────────────────────────────────────────────

fn render_project_list(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),  // Header (logo + summary bar)
            Constraint::Min(4),    // Project cards
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_project_list_header(f, app, chunks[0]);
    render_project_cards(f, app, chunks[1]);
    render_project_list_footer(f, app, chunks[2]);
}

fn render_project_list_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Logo
            Constraint::Length(3), // Summary bar
        ])
        .split(area);

    // Logo (reuse same rendering with glitch)
    let logo_lines: Vec<Line> = if app.glitch.active {
        SODIUM_LOGO
            .iter()
            .map(|line| {
                let chars: Vec<Span> = line
                    .chars()
                    .map(|c| {
                        if c != ' ' && rand::random::<f32>() < 0.3 {
                            let gc = GLITCH_CHARS[rand::random::<usize>() % GLITCH_CHARS.len()];
                            Span::styled(
                                gc.to_string(),
                                Style::default()
                                    .fg(if rand::random::<bool>() {
                                        theme::MAGENTA
                                    } else {
                                        theme::RED
                                    })
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::styled(
                                c.to_string(),
                                Style::default()
                                    .fg(theme::CYAN)
                                    .add_modifier(Modifier::BOLD),
                            )
                        }
                    })
                    .collect();
                Line::from(chars)
            })
            .collect()
    } else {
        SODIUM_LOGO
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    *line,
                    Style::default()
                        .fg(theme::CYAN)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect()
    };

    let mut all_lines = logo_lines;
    all_lines.push(Line::from(Span::styled(
        app.subtitle(),
        Style::default().fg(theme::FG_DIM),
    )));

    let logo_widget = Paragraph::new(all_lines).alignment(Alignment::Center);
    f.render_widget(logo_widget, chunks[0]);

    // Summary bar
    let total = app.projects.len();
    let clean = app
        .projects
        .iter()
        .filter(|p| p.has_git && p.dirty_count == 0 && p.ahead == 0 && p.behind == 0)
        .count();
    let dirty = app
        .projects
        .iter()
        .filter(|p| p.has_git && p.dirty_count > 0)
        .count();
    let no_repo = app.projects.iter().filter(|p| !p.has_git).count();

    let summary_text = format!(
        "{} PROJECTS — {} clean │ {} dirty │ {} no repo",
        total, clean, dirty, no_repo
    );

    let summary_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER))
        .style(Style::default().bg(theme::BG_CARD))
        .padding(Padding::horizontal(1));

    let summary_line = Line::from(vec![
        Span::styled(
            "██",
            Style::default().fg(theme::CYAN),
        ),
        Span::raw(" "),
        Span::styled(
            summary_text,
            Style::default()
                .fg(theme::FG_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let summary_widget = Paragraph::new(summary_line).block(summary_block);
    f.render_widget(summary_widget, chunks[1]);
}

fn render_project_cards(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(theme::CYAN)),
            Span::styled("PROJECTS", theme::title_style()),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_style(theme::border_hi_style())
        .style(Style::default().bg(theme::BG_CARD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.projects.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No projects found in dev_root",
            Style::default().fg(theme::FG_DIM),
        ));
        f.render_widget(empty, inner);
        return;
    }

    // Calculate visible window for scrolling
    let visible_height = inner.height as usize;
    let total = app.projects.len();
    let scroll_offset = if app.project_index >= visible_height {
        app.project_index - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .projects
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, proj)| {
            let is_selected = i == app.project_index;
            let arrow = if is_selected { "▸" } else { " " };

            if !proj.has_git {
                // No git repo
                let line = Line::from(vec![
                    Span::styled(
                        format!("  {} ", arrow),
                        Style::default().fg(if is_selected {
                            theme::CYAN
                        } else {
                            theme::FG_DIM
                        }),
                    ),
                    Span::styled(
                        format!("{:<16}", proj.name),
                        Style::default()
                            .fg(if is_selected {
                                theme::FG_BRIGHT
                            } else {
                                theme::FG
                            })
                            .add_modifier(if is_selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                    Span::styled(
                        "—       ",
                        Style::default().fg(theme::FG_DIM),
                    ),
                    Span::styled(
                        "NO REPO",
                        Style::default()
                            .fg(theme::RED)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);
                ListItem::new(line)
            } else {
                // Git repo with details
                let status_span = if proj.dirty_count == 0
                    && proj.ahead == 0
                    && proj.behind == 0
                {
                    Span::styled(
                        "● CLEAN     ",
                        Style::default()
                            .fg(theme::GREEN)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    let mut status = String::new();
                    if proj.ahead > 0 {
                        status.push_str(&format!("▲{} ", proj.ahead));
                    }
                    if proj.behind > 0 {
                        status.push_str(&format!("▼{} ", proj.behind));
                    }
                    if proj.dirty_count > 0 {
                        status.push_str(&format!("▪{} dirty", proj.dirty_count));
                    }
                    Span::styled(
                        format!("{:<12}", status),
                        Style::default()
                            .fg(theme::ORANGE)
                            .add_modifier(Modifier::BOLD),
                    )
                };

                let commit_msg = truncate_str(&proj.last_commit_msg, 30);

                let line = Line::from(vec![
                    Span::styled(
                        format!("  {} ", arrow),
                        Style::default().fg(if is_selected {
                            theme::CYAN
                        } else {
                            theme::FG_DIM
                        }),
                    ),
                    Span::styled(
                        format!("{:<16}", proj.name),
                        Style::default()
                            .fg(if is_selected {
                                theme::FG_BRIGHT
                            } else {
                                theme::FG
                            })
                            .add_modifier(if is_selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                    Span::styled(
                        format!("{:<10}", proj.branch),
                        Style::default()
                            .fg(theme::MAGENTA)
                            .add_modifier(Modifier::BOLD),
                    ),
                    status_span,
                    Span::styled(
                        format!("{:<8}", proj.last_commit_age),
                        Style::default().fg(theme::FG_DIM),
                    ),
                    Span::styled(
                        format!("\"{}\"", commit_msg),
                        Style::default().fg(theme::FG_DIM),
                    ),
                ]);
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);

    // Scroll indicator
    if total > visible_height {
        let indicator = format!(" {}/{} ", app.project_index + 1, total);
        let indicator_span = Span::styled(
            indicator,
            Style::default().fg(theme::FG_DIM),
        );
        let indicator_widget = Paragraph::new(Line::from(indicator_span))
            .alignment(Alignment::Right);
        let indicator_area = Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(1),
            inner.width,
            1,
        );
        f.render_widget(indicator_widget, indicator_area);
    }
}

fn render_project_list_footer(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let branding = format!(
        "{:>width$}",
        "⚛ sodium v26.02.24",
        width = area.width.saturating_sub(55) as usize
    );

    let line = Line::from(vec![
        Span::styled(
            "  [q]",
            Style::default()
                .fg(theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit  ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" open  ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            "[↑↓]",
            Style::default()
                .fg(theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" navigate  ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            "[r]",
            Style::default()
                .fg(theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" refresh", Style::default().fg(theme::FG_DIM)),
        Span::styled(branding, Style::default().fg(theme::FG_DIM)),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}

// ── Header ─────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Logo
            Constraint::Length(3), // GITCON bar
        ])
        .split(area);

    // Logo
    let logo_lines: Vec<Line> = if app.glitch.active {
        SODIUM_LOGO
            .iter()
            .map(|line| {
                let chars: Vec<Span> = line
                    .chars()
                    .map(|c| {
                        if c != ' ' && rand::random::<f32>() < 0.3 {
                            let gc = GLITCH_CHARS[rand::random::<usize>() % GLITCH_CHARS.len()];
                            Span::styled(
                                gc.to_string(),
                                Style::default()
                                    .fg(if rand::random::<bool>() {
                                        theme::MAGENTA
                                    } else {
                                        theme::RED
                                    })
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::styled(
                                c.to_string(),
                                Style::default()
                                    .fg(theme::CYAN)
                                    .add_modifier(Modifier::BOLD),
                            )
                        }
                    })
                    .collect();
                Line::from(chars)
            })
            .collect()
    } else {
        SODIUM_LOGO
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    *line,
                    Style::default()
                        .fg(theme::CYAN)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect()
    };

    let mut all_lines = logo_lines;
    all_lines.push(Line::from(Span::styled(
        app.subtitle(),
        Style::default().fg(theme::FG_DIM),
    )));

    let logo_widget = Paragraph::new(all_lines).alignment(Alignment::Center);
    f.render_widget(logo_widget, chunks[0]);

    // GITCON bar
    let gitcon = &app.repo_info.gitcon;
    let gitcon_color = gitcon.color();
    let gitcon_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(gitcon_color))
        .style(Style::default().bg(theme::BG_CARD))
        .padding(Padding::horizontal(1));

    let blink = app.tick % 10 < 7;
    let indicator = if blink { "██" } else { "  " };

    let gitcon_line = Line::from(vec![
        Span::styled(
            indicator,
            Style::default().fg(gitcon_color),
        ),
        Span::raw(" "),
        Span::styled(
            gitcon.label(),
            Style::default()
                .fg(gitcon_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" — ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            gitcon.subtitle(),
            Style::default().fg(gitcon_color),
        ),
    ]);

    let gitcon_widget = Paragraph::new(gitcon_line).block(gitcon_block);
    f.render_widget(gitcon_widget, chunks[1]);
}

// ── Body ───────────────────────────────────────────────────────────────────

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Repo name + branch + remote
            Constraint::Length(12), // Middle row (branches + activity + files)
            Constraint::Length(1), // Spacer
            Constraint::Min(8),    // Actions menu
        ])
        .margin(1)
        .split(area);

    render_repo_bar(f, app, body[0]);
    render_middle_row(f, app, body[1]);
    render_actions(f, app, body[3]);
}

fn render_repo_bar(f: &mut Frame, app: &App, area: Rect) {
    let info = &app.repo_info;

    let remote_display = info
        .remote_url
        .as_deref()
        .unwrap_or("no remote");

    let line1 = Line::from(vec![
        Span::styled("  ◆ ", Style::default().fg(theme::CYAN)),
        Span::styled(
            &info.name,
            Style::default()
                .fg(theme::FG_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  on  ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            &info.current_branch,
            Style::default()
                .fg(theme::MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(theme::BORDER)),
        Span::styled(
            format!("{} {}", &info.last_commit_hash, &info.last_commit_msg),
            Style::default().fg(theme::FG_DIM),
        ),
    ]);

    let mut line2_spans = vec![
        Span::styled("    ╰ ", Style::default().fg(theme::BORDER)),
        Span::styled(remote_display, Style::default().fg(theme::FG_DIM)),
    ];

    if info.github_url.is_some() {
        line2_spans.push(Span::styled("  ◆ GitHub", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)));
    }

    let line2 = Line::from(line2_spans);

    f.render_widget(Paragraph::new(vec![line1, line2]), area);
}

fn render_middle_row(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Branches
            Constraint::Percentage(35), // Activity
            Constraint::Percentage(25), // Status
        ])
        .split(area);

    render_branches(f, app, cols[0]);
    render_activity(f, app, cols[1]);
    render_status(f, app, cols[2]);
}

fn render_branches(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ⊙ ", Style::default().fg(theme::CYAN)),
            Span::styled("BRANCHS", theme::title_style()),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BG_CARD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.repo_info.branches.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No branches detected",
            Style::default().fg(theme::FG_DIM),
        ));
        f.render_widget(empty, inner);
        return;
    }

    // Header
    let header_line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{:<20}", "Local"),
            Style::default()
                .fg(theme::FG_DIM)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Remote",
            Style::default()
                .fg(theme::FG_DIM)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut lines = vec![header_line];

    for branch in &app.repo_info.branches {
        let local_col = if branch.is_local {
            if branch.is_current {
                Span::styled(
                    format!("▸ {:<18}", branch.name),
                    Style::default()
                        .fg(theme::RED)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("  {:<18}", branch.name),
                    Style::default().fg(theme::FG),
                )
            }
        } else {
            Span::styled(format!("{:20}", ""), Style::default())
        };

        let remote_col = if branch.is_remote {
            if !branch.is_local {
                Span::styled(
                    format!("○ {}", branch.name),
                    Style::default().fg(theme::CYAN),
                )
            } else {
                Span::styled(
                    format!("● {}", branch.name),
                    Style::default().fg(theme::FG_DIM),
                )
            }
        } else {
            Span::styled("  —", Style::default().fg(theme::FG_DIM))
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            local_col,
            remote_col,
        ]));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}

fn render_activity(f: &mut Frame, app: &App, area: Rect) {
    let grid = &app.repo_info.activity_grid;
    let recent_ops: u16 = grid.iter().map(|d| d.total()).sum();

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ⚡ ", Style::default().fg(theme::ORANGE)),
            Span::styled("ACTIVITY ", theme::title_style()),
        ]))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BG_CARD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Heatmap (7 rows = days of week)
            Constraint::Length(1), // Legend
            Constraint::Length(1), // Stats
            Constraint::Length(1), // Sync
        ])
        .split(inner);

    // ── GitHub-style heatmap ──────────────────────────────────────────
    const HEAT: [Color; 5] = [
        Color::Rgb(25, 32, 45),   // 0: no activity (dim)
        Color::Rgb(14, 68, 41),   // 1: low
        Color::Rgb(0, 109, 50),   // 2: medium-low
        Color::Rgb(38, 166, 65),  // 3: medium-high
        Color::Rgb(57, 211, 83),  // 4: high
    ];

    let now = chrono::Utc::now();
    let today_dow = now.weekday().num_days_from_monday() as usize; // 0=Mon, 6=Sun
    let max_ops: u16 = grid.iter().map(|d| d.total()).max().unwrap_or(0);

    // Build 7×13 grid (rows=day-of-week, cols=weeks)
    let mut heatmap: [[Option<u16>; 13]; 7] = [[None; 13]; 7];

    for (idx, day) in grid.iter().enumerate() {
        let days_ago = (grid.len() - 1 - idx) as i64;
        let dow = ((today_dow as i64 + 7 - (days_ago % 7)) % 7) as usize;
        let weeks_ago = (days_ago + dow as i64 - today_dow as i64) / 7;
        if weeks_ago >= 0 && (weeks_ago as usize) < 13 {
            let col = 12 - weeks_ago as usize;
            heatmap[dow][col] = Some(day.total());
        }
    }

    let day_labels = ["L", "M", "M", "J", "V", "S", "D"];
    let mut lines = Vec::new();

    for (row, label) in day_labels.iter().enumerate() {
        let mut spans = vec![
            Span::styled(
                format!("{} ", label),
                Style::default().fg(theme::FG_DIM),
            ),
        ];

        for col in 0..13 {
            match heatmap[row][col] {
                None => spans.push(Span::raw("  ")),
                Some(val) => {
                    let level = if val == 0 {
                        0
                    } else if max_ops <= 4 {
                        (val as usize).min(4)
                    } else {
                        let pct = val as f32 / max_ops as f32;
                        if pct <= 0.25 { 1 }
                        else if pct <= 0.50 { 2 }
                        else if pct <= 0.75 { 3 }
                        else { 4 }
                    };
                    spans.push(Span::styled(
                        "█ ",
                        Style::default().fg(HEAT[level]),
                    ));
                }
            }
        }

        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), inner_chunks[0]);

    // ── Legend ─────────────────────────────────────────────────────────
    let legend = Line::from(vec![
        Span::styled(" Less ", Style::default().fg(theme::FG_DIM)),
        Span::styled("█ ", Style::default().fg(HEAT[0])),
        Span::styled("█ ", Style::default().fg(HEAT[1])),
        Span::styled("█ ", Style::default().fg(HEAT[2])),
        Span::styled("█ ", Style::default().fg(HEAT[3])),
        Span::styled("█ ", Style::default().fg(HEAT[4])),
        Span::styled("More", Style::default().fg(theme::FG_DIM)),
    ]);
    f.render_widget(Paragraph::new(legend), inner_chunks[1]);

    // ── Stats ─────────────────────────────────────────────────────────
    let total = app.repo_info.total_commits;
    let stats_line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            format!("{}", total),
            Style::default()
                .fg(theme::FG_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" commits  ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            format!("{}", recent_ops),
            Style::default()
                .fg(theme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" actions récentes", Style::default().fg(theme::FG_DIM)),
    ]);
    f.render_widget(Paragraph::new(stats_line), inner_chunks[2]);

    // ── Sync status ───────────────────────────────────────────────────
    let (ahead, behind) = (app.repo_info.ahead, app.repo_info.behind);
    let sync_spans = if ahead == 0 && behind == 0 {
        vec![
            Span::styled(" ● ", Style::default().fg(theme::GREEN)),
            Span::styled("SYNCED", Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD)),
        ]
    } else {
        let mut v = vec![Span::styled(" ", Style::default())];
        if ahead > 0 {
            v.push(Span::styled(
                format!("▲{} ", ahead),
                Style::default()
                    .fg(theme::ORANGE)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        if behind > 0 {
            v.push(Span::styled(
                format!("▼{} ", behind),
                Style::default()
                    .fg(theme::RED)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        v
    };
    f.render_widget(Paragraph::new(Line::from(sync_spans)), inner_chunks[3]);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let files = &app.repo_info.files;
    let total = files.modified + files.staged + files.untracked + files.conflicted;

    let status_color = if files.conflicted > 0 {
        theme::RED
    } else if total == 0 {
        theme::GREEN
    } else {
        theme::ORANGE
    };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◎ ", Style::default().fg(status_color)),
            Span::styled("FILES", theme::title_style()),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BG_CARD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if total == 0 {
        let clean = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  ● CLEAN",
                Style::default()
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  All clear",
                Style::default().fg(theme::FG_DIM),
            )),
        ]);
        f.render_widget(clean, inner);
        return;
    }

    let mut lines = vec![Line::from("")];

    if files.modified > 0 {
        lines.push(Line::from(vec![
            Span::styled("  ▪ ", Style::default().fg(theme::ORANGE)),
            Span::styled(
                format!("{}", files.modified),
                Style::default()
                    .fg(theme::ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" modified", Style::default().fg(theme::FG_DIM)),
        ]));
    }
    if files.staged > 0 {
        lines.push(Line::from(vec![
            Span::styled("  ▪ ", Style::default().fg(theme::GREEN)),
            Span::styled(
                format!("{}", files.staged),
                Style::default()
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" staged", Style::default().fg(theme::FG_DIM)),
        ]));
    }
    if files.untracked > 0 {
        lines.push(Line::from(vec![
            Span::styled("  ▪ ", Style::default().fg(theme::FG_DIM)),
            Span::styled(
                format!("{}", files.untracked),
                Style::default().fg(theme::FG_DIM).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" untracked", Style::default().fg(theme::FG_DIM)),
        ]));
    }
    if files.conflicted > 0 {
        lines.push(Line::from(vec![
            Span::styled("  ▪ ", Style::default().fg(theme::RED)),
            Span::styled(
                format!("{}", files.conflicted),
                Style::default()
                    .fg(theme::RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" CONFLICT", Style::default().fg(theme::RED)),
        ]));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

// ── Actions menu ───────────────────────────────────────────────────────────

fn render_actions(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ▶ ", Style::default().fg(theme::CYAN)),
            Span::styled("ACTIONS", theme::title_style()),
            Span::styled(
                " (↑↓ navigate, Enter select) ",
                Style::default().fg(theme::FG_DIM),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(theme::border_hi_style())
        .style(Style::default().bg(theme::BG_CARD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible = inner.height as usize;
    let total = app.menu_items.len();

    // Compute scroll offset to keep selected item visible
    let scroll_offset = if total <= visible {
        0
    } else {
        let max_offset = total.saturating_sub(visible);
        // Keep selected item roughly centered, clamped to valid range
        app.menu_index.saturating_sub(visible / 2).min(max_offset)
    };

    let end = (scroll_offset + visible).min(total);

    let items: Vec<ListItem> = app
        .menu_items[scroll_offset..end]
        .iter()
        .enumerate()
        .map(|(vi, item)| {
            let i = scroll_offset + vi;
            match item {
                MenuItem::Separator => {
                    ListItem::new(Line::from(Span::styled(
                        "    ─────────────────────────────────",
                        Style::default().fg(theme::BORDER),
                    )))
                }
                MenuItem::Action(kind, label) => {
                    let is_selected = i == app.menu_index;
                    if is_selected {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                "  ▸ ",
                                Style::default()
                                    .fg(theme::CYAN)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                label.as_str(),
                                Style::default()
                                    .fg(theme::FG_BRIGHT)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]))
                    } else {
                        let color = match kind {
                            crate::app::ActionKind::Quit => theme::FG_DIM,
                            crate::app::ActionKind::Reinit => theme::RED,
                            _ => theme::FG,
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled("    ", Style::default()),
                            Span::styled(label.as_str(), Style::default().fg(color)),
                        ]))
                    }
                }
            }
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);

    // Scroll indicators
    if scroll_offset > 0 {
        let arrow = Span::styled(" ▲ ", Style::default().fg(theme::FG_DIM));
        f.render_widget(
            Paragraph::new(Line::from(arrow)).alignment(Alignment::Right),
            Rect { x: inner.x, y: inner.y, width: inner.width, height: 1 },
        );
    }
    if end < total {
        let arrow = Span::styled(" ▼ ", Style::default().fg(theme::FG_DIM));
        f.render_widget(
            Paragraph::new(Line::from(arrow)).alignment(Alignment::Right),
            Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(1),
                width: inner.width,
                height: 1,
            },
        );
    }
}

// ── Footer ─────────────────────────────────────────────────────────────────

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::border_style())
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Notification or default footer
    let line = if let Some(ref notif) = app.notification {
        let blink = app.tick % 6 < 4;
        let color = if notif.is_error { theme::RED } else { theme::GREEN };
        if blink {
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    &notif.message,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(Span::styled("  ", Style::default()))
        }
    } else if app.is_multi_project() {
        let branding = format!(
            "{:>width$}",
            "⚛ sodium v26.02.24",
            width = area.width.saturating_sub(65) as usize
        );
        Line::from(vec![
            Span::styled("  [Esc]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" back  ", Style::default().fg(theme::FG_DIM)),
            Span::styled("[Enter]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" select  ", Style::default().fg(theme::FG_DIM)),
            Span::styled("[↑↓]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" navigate  ", Style::default().fg(theme::FG_DIM)),
            Span::styled("[r]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" refresh", Style::default().fg(theme::FG_DIM)),
            Span::styled(branding, Style::default().fg(theme::FG_DIM)),
        ])
    } else {
        let branding = format!(
            "{:>width$}",
            "⚛ sodium v26.02.24",
            width = area.width.saturating_sub(45) as usize
        );
        Line::from(vec![
            Span::styled("  [q]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" quit  ", Style::default().fg(theme::FG_DIM)),
            Span::styled("[Enter]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" select  ", Style::default().fg(theme::FG_DIM)),
            Span::styled("[↑↓]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" navigate", Style::default().fg(theme::FG_DIM)),
            Span::styled(branding, Style::default().fg(theme::FG_DIM)),
        ])
    };

    f.render_widget(Paragraph::new(line), inner);
}

// ── Input overlay ──────────────────────────────────────────────────────────

fn render_input_overlay(f: &mut Frame, app: &App, area: Rect) {
    match &app.input_mode {
        InputMode::Normal => return,
        InputMode::TextInput { prompt, .. } | InputMode::Confirm { prompt, .. } => {
            let width = 60u16.min(area.width.saturating_sub(4));
            let height = 5u16;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup = Rect::new(x, y, width, height);

            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Line::from(vec![
                    Span::styled(" ✎ ", Style::default().fg(theme::ORANGE)),
                    Span::styled(prompt.as_str(), theme::title_style()),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::ORANGE))
                .style(Style::default().bg(theme::BG_CARD));

            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let cursor_char = if app.tick % 6 < 4 { "█" } else { " " };
            let input_line = Line::from(vec![
                Span::styled("  > ", Style::default().fg(theme::CYAN)),
                Span::styled(
                    &app.input_buffer,
                    Style::default()
                        .fg(theme::FG_BRIGHT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(cursor_char, Style::default().fg(theme::CYAN)),
            ]);

            let help_line = Line::from(Span::styled(
                "  [Enter] confirm  [Esc] cancel",
                Style::default().fg(theme::FG_DIM),
            ));

            let paragraph = Paragraph::new(vec![input_line, help_line]);
            f.render_widget(paragraph, inner);
        }
        InputMode::CommitReview => {
            if let Some(ref state) = app.commit_review {
                render_commit_review_overlay(f, app, state, area);
            }
            return;
        }
        InputMode::CommitSelect => {
            if let Some(ref state) = app.commit_review {
                render_commit_select_overlay(f, app, state, area);
            }
            return;
        }
        InputMode::Select { prompt, options, index, .. } => {
            let item_count = options.len() as u16;
            let height = (item_count + 4).min(area.height.saturating_sub(4));
            let width = 50u16.min(area.width.saturating_sub(4));
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup = Rect::new(x, y, width, height);

            f.render_widget(Clear, popup);

            let block = Block::default()
                .title(Line::from(vec![
                    Span::styled(" ⊙ ", Style::default().fg(theme::CYAN)),
                    Span::styled(prompt.as_str(), theme::title_style()),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::CYAN))
                .style(Style::default().bg(theme::BG_CARD));

            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // Options list
                    Constraint::Length(1), // Help
                ])
                .split(inner);

            let items: Vec<ListItem> = options
                .iter()
                .enumerate()
                .map(|(i, opt)| {
                    if i == *index {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                "  ▸ ",
                                Style::default()
                                    .fg(theme::CYAN)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                opt.as_str(),
                                Style::default()
                                    .fg(theme::FG_BRIGHT)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]))
                    } else {
                        ListItem::new(Line::from(vec![
                            Span::styled("    ", Style::default()),
                            Span::styled(opt.as_str(), Style::default().fg(theme::FG)),
                        ]))
                    }
                })
                .collect();

            let list = List::new(items);
            f.render_widget(list, inner_chunks[0]);

            let help = Paragraph::new(Line::from(Span::styled(
                "  [Enter] select  [Esc] cancel  [↑↓] navigate",
                Style::default().fg(theme::FG_DIM),
            )));
            f.render_widget(help, inner_chunks[1]);
        }
    }
}

// ── Commit review / select overlays ────────────────────────────────────────

fn status_color(ch: char) -> ratatui::style::Color {
    match ch {
        'M' => theme::ORANGE,
        'A' => theme::GREEN,
        'D' => theme::RED,
        '?' => theme::FG_DIM,
        'C' => theme::RED,
        'R' => theme::MAGENTA,
        _ => theme::FG,
    }
}

fn status_modifier(ch: char) -> Modifier {
    if ch == 'C' {
        Modifier::BOLD
    } else {
        Modifier::empty()
    }
}

fn render_commit_review_overlay(f: &mut Frame, app: &App, state: &CommitReviewState, area: Rect) {
    let file_count = state.files.len() as u16;
    let max_visible = area.height.saturating_sub(8).min(20);
    let height = (file_count + 6).min(max_visible + 6).max(8);
    let width = 60u16.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◎ ", Style::default().fg(theme::CYAN)),
            Span::styled("COMMIT REVIEW", theme::title_style()),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::CYAN))
        .style(Style::default().bg(theme::BG_CARD));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // File list
            Constraint::Length(1), // Summary
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // File list
    let visible_height = inner_chunks[0].height as usize;
    let scroll = if state.cursor >= state.scroll_offset + visible_height {
        state.cursor.saturating_sub(visible_height - 1)
    } else {
        state.scroll_offset
    };

    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, file)| {
            let is_cursor = i == state.cursor;
            let arrow = if is_cursor { "▸" } else { " " };
            let sc = file.status_char;
            let path_display = truncate_str(&file.path, (width as usize).saturating_sub(22));

            // Right-align stats
            let ins_str = format!("+{}", file.insertions);
            let del_str = format!("-{}", file.deletions);

            Line::from(vec![
                Span::styled(
                    format!(" {} ", arrow),
                    Style::default().fg(if is_cursor { theme::CYAN } else { theme::FG_DIM }),
                ),
                Span::styled(
                    format!("{}  ", sc),
                    Style::default()
                        .fg(status_color(sc))
                        .add_modifier(status_modifier(sc) | Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<width$}", path_display, width = (width as usize).saturating_sub(22)),
                    Style::default().fg(if is_cursor { theme::FG_BRIGHT } else { theme::FG }),
                ),
                Span::styled(
                    format!("{:>5}", ins_str),
                    Style::default().fg(theme::GREEN),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{:>5}", del_str),
                    Style::default().fg(theme::RED),
                ),
            ])
        })
        .map(ListItem::new)
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner_chunks[0]);

    // Summary line
    let total_ins: usize = state.files.iter().map(|f| f.insertions).sum();
    let total_del: usize = state.files.iter().map(|f| f.deletions).sum();
    let summary = Line::from(vec![
        Span::styled(
            format!("  {} files", state.files.len()),
            Style::default().fg(theme::FG_DIM),
        ),
        Span::styled(" — ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            format!("{}(+)", total_ins),
            Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            format!("{}(-)", total_del),
            Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(summary), inner_chunks[1]);

    // Help line
    let blink = app.tick % 8 < 6;
    let help = if blink {
        Line::from(vec![
            Span::styled("  [a]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" Add All  ", Style::default().fg(theme::FG_DIM)),
            Span::styled("[Enter]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" Select files  ", Style::default().fg(theme::FG_DIM)),
            Span::styled("[Esc]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" Cancel", Style::default().fg(theme::FG_DIM)),
        ])
    } else {
        Line::from(Span::styled(
            "  [a] Add All  [Enter] Select files  [Esc] Cancel",
            Style::default().fg(theme::FG_DIM),
        ))
    };
    f.render_widget(Paragraph::new(help), inner_chunks[2]);
}

fn render_commit_select_overlay(f: &mut Frame, _app: &App, state: &CommitReviewState, area: Rect) {
    let file_count = state.files.len() as u16;
    let max_visible = area.height.saturating_sub(8).min(20);
    let height = (file_count + 6).min(max_visible + 6).max(8);
    let width = 60u16.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◎ ", Style::default().fg(theme::CYAN)),
            Span::styled("SELECT FILES", theme::title_style()),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::CYAN))
        .style(Style::default().bg(theme::BG_CARD));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // File list
            Constraint::Length(1), // Summary
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // File list with checkboxes
    let visible_height = inner_chunks[0].height as usize;
    let scroll = if state.cursor >= state.scroll_offset + visible_height {
        state.cursor.saturating_sub(visible_height - 1)
    } else {
        state.scroll_offset
    };

    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, file)| {
            let is_cursor = i == state.cursor;
            let is_selected = *state.selected.get(i).unwrap_or(&false);
            let arrow = if is_cursor { "▸" } else { " " };
            let checkbox = if is_selected { "[x]" } else { "[ ]" };
            let sc = file.status_char;
            let path_display = truncate_str(&file.path, (width as usize).saturating_sub(26));

            let ins_str = format!("+{}", file.insertions);
            let del_str = format!("-{}", file.deletions);

            Line::from(vec![
                Span::styled(
                    format!(" {} ", arrow),
                    Style::default().fg(if is_cursor { theme::CYAN } else { theme::FG_DIM }),
                ),
                Span::styled(
                    format!("{} ", checkbox),
                    Style::default().fg(if is_selected { theme::GREEN } else { theme::FG_DIM })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}  ", sc),
                    Style::default()
                        .fg(status_color(sc))
                        .add_modifier(status_modifier(sc) | Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<width$}", path_display, width = (width as usize).saturating_sub(26)),
                    Style::default().fg(if is_cursor { theme::FG_BRIGHT } else { theme::FG }),
                ),
                Span::styled(
                    format!("{:>5}", ins_str),
                    Style::default().fg(theme::GREEN),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{:>5}", del_str),
                    Style::default().fg(theme::RED),
                ),
            ])
        })
        .map(ListItem::new)
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner_chunks[0]);

    // Summary line with selection count
    let selected_count = state.selected.iter().filter(|&&s| s).count();
    let total = state.files.len();
    let sel_ins: usize = state.files.iter().zip(state.selected.iter())
        .filter(|(_, &s)| s)
        .map(|(f, _)| f.insertions)
        .sum();
    let sel_del: usize = state.files.iter().zip(state.selected.iter())
        .filter(|(_, &s)| s)
        .map(|(f, _)| f.deletions)
        .sum();

    let summary = Line::from(vec![
        Span::styled(
            format!("  {}/{} selected", selected_count, total),
            Style::default().fg(theme::FG_DIM),
        ),
        Span::styled(" — ", Style::default().fg(theme::FG_DIM)),
        Span::styled(
            format!("{}(+)", sel_ins),
            Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            format!("{}(-)", sel_del),
            Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(summary), inner_chunks[1]);

    // Help line
    let help = Line::from(vec![
        Span::styled("  [Space]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
        Span::styled(" toggle ", Style::default().fg(theme::FG_DIM)),
        Span::styled("[a]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("ll ", Style::default().fg(theme::FG_DIM)),
        Span::styled("[n]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("one ", Style::default().fg(theme::FG_DIM)),
        Span::styled("[Enter]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
        Span::styled(" confirm ", Style::default().fg(theme::FG_DIM)),
        Span::styled("[Esc]", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(help), inner_chunks[2]);
}
