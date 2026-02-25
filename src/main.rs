mod app;
mod config;
mod git;
mod theme;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, InputMode, Screen};

const TICK_RATE: Duration = Duration::from_millis(100);

fn main() -> Result<()> {
    color_eyre::install()?;

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Try to load config for multi-project mode
    let mut app = match config::load_config() {
        Some(cfg) => {
            let dev_root = cfg.dev_root_path();
            if dev_root.is_dir() {
                App::new_multi(cfg)
            } else {
                // Config exists but dev_root doesn't → fallback to single-project
                App::new(cwd)
            }
        }
        None => App::new(cwd),
    };

    // If multi-project but no projects found, fallback to single-project on cwd
    if app.screen == Screen::ProjectList && app.projects.is_empty() {
        let cwd = std::env::current_dir()?;
        app = App::new(cwd);
    }

    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match &app.input_mode {
                    InputMode::Normal => match app.screen {
                        Screen::ProjectList => match key.code {
                            KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.should_quit = true;
                            }
                            KeyCode::Up | KeyCode::Char('k') => app.project_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.project_down(),
                            KeyCode::Enter => app.enter_project(),
                            KeyCode::Char('r') => app.refresh_projects(),
                            _ => {}
                        },
                        Screen::ProjectDetail => match key.code {
                            KeyCode::Esc | KeyCode::Backspace => {
                                if app.is_multi_project() {
                                    app.back_to_list();
                                } else {
                                    app.should_quit = true;
                                }
                            }
                            KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.should_quit = true;
                            }
                            KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
                            KeyCode::Enter => app.execute_action(),
                            KeyCode::Char('r') => app.refresh(),
                            _ => {}
                        },
                    },
                    InputMode::TextInput { .. } | InputMode::Confirm { .. } => match key.code {
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Enter => app.submit_input(),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        _ => {}
                    },
                    InputMode::Select { .. } => match key.code {
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Enter => app.submit_input(),
                        KeyCode::Up | KeyCode::Char('k') => app.select_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.select_down(),
                        _ => {}
                    },
                    InputMode::CommitReview => match key.code {
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Char('a') => app.commit_add_all(),
                        KeyCode::Enter => app.commit_enter_select(),
                        KeyCode::Up | KeyCode::Char('k') => app.commit_review_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.commit_review_down(),
                        _ => {}
                    },
                    InputMode::CommitSelect => match key.code {
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Enter => app.commit_confirm_selection(),
                        KeyCode::Char(' ') => app.commit_toggle_file(),
                        KeyCode::Char('a') => app.commit_select_all(),
                        KeyCode::Char('n') => app.commit_select_none(),
                        KeyCode::Up | KeyCode::Char('k') => app.commit_review_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.commit_review_down(),
                        _ => {}
                    },
                }
            }
        }

        // If a pending op was queued, re-render to show running indicator, then execute
        if app.pending_op.is_some() {
            terminal.draw(|f| ui::render(f, &app))?;
            app.run_pending_op();
        }

        if last_tick.elapsed() >= TICK_RATE {
            app.tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
