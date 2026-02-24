use ratatui::style::{Color, Modifier, Style};

// ── Dark-ops palette (pizzint.watch inspired) ──────────────────────────────

/// Deep space background
pub const BG: Color = Color::Rgb(8, 11, 22);
/// Slightly lighter for card backgrounds
pub const BG_CARD: Color = Color::Rgb(12, 17, 32);
/// Card/block borders
pub const BORDER: Color = Color::Rgb(30, 58, 95);
/// Border when focused/highlighted
pub const BORDER_HI: Color = Color::Rgb(56, 120, 200);

/// Default text
pub const FG: Color = Color::Rgb(160, 174, 192);
/// Dimmed text
pub const FG_DIM: Color = Color::Rgb(75, 85, 99);
/// Bright text (titles)
pub const FG_BRIGHT: Color = Color::Rgb(220, 230, 245);

/// Neon cyan — primary accent
pub const CYAN: Color = Color::Rgb(34, 211, 238);
/// Neon green — success / synced
pub const GREEN: Color = Color::Rgb(34, 197, 94);
/// Orange — warnings
pub const ORANGE: Color = Color::Rgb(234, 179, 8);
/// Red — danger / behind / conflicts
pub const RED: Color = Color::Rgb(239, 68, 68);
/// Magenta — special accents
pub const MAGENTA: Color = Color::Rgb(168, 85, 247);
/// Blue — subtle accent
#[allow(dead_code)]
pub const BLUE: Color = Color::Rgb(59, 130, 246);

// ── GITCON levels ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum GitconLevel {
    /// Everything synced, clean tree
    Gitcon1,
    /// Minor changes (untracked files, small ahead)
    Gitcon2,
    /// Significant divergence or many modifications
    Gitcon3,
    /// Conflicts or major issues
    Gitcon4,
    /// No repo / broken state
    Gitcon5,
}

impl GitconLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Gitcon1 => "GITCON 1",
            Self::Gitcon2 => "GITCON 2",
            Self::Gitcon3 => "GITCON 3",
            Self::Gitcon4 => "GITCON 4",
            Self::Gitcon5 => "GITCON 5",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            Self::Gitcon1 => "ALL CLEAR — REPOS SYNCED",
            Self::Gitcon2 => "LOW ACTIVITY — MINOR CHANGES DETECTED",
            Self::Gitcon3 => "ELEVATED — SIGNIFICANT DIVERGENCE",
            Self::Gitcon4 => "HIGH ALERT — CONFLICTS DETECTED",
            Self::Gitcon5 => "CRITICAL — NO REPO / BROKEN STATE",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Gitcon1 => GREEN,
            Self::Gitcon2 => Color::Rgb(132, 204, 22),
            Self::Gitcon3 => ORANGE,
            Self::Gitcon4 => Color::Rgb(249, 115, 22),
            Self::Gitcon5 => RED,
        }
    }
}

// ── Style helpers ──────────────────────────────────────────────────────────

pub fn title_style() -> Style {
    Style::default().fg(FG_BRIGHT).add_modifier(Modifier::BOLD)
}

#[allow(dead_code)]
pub fn label_style() -> Style {
    Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD)
}

#[allow(dead_code)]
pub fn value_style() -> Style {
    Style::default().fg(FG)
}

#[allow(dead_code)]
pub fn accent_style() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}

#[allow(dead_code)]
pub fn highlight_style() -> Style {
    Style::default()
        .fg(BG)
        .bg(CYAN)
        .add_modifier(Modifier::BOLD)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER)
}

pub fn border_hi_style() -> Style {
    Style::default().fg(BORDER_HI)
}
