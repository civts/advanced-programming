use tui::style::{Color, Style};

/// How often the screen is refreshed (in milliseconds)
pub(crate) const REFRESH_RATE_MILLISECONDS: u64 = 100;

pub(crate) const WHITE: Color = Color::Rgb(224, 251, 252);
pub(crate) const RED: Color = Color::Rgb(238, 108, 77);
pub(crate) const BLUE: Color = Color::Rgb(159, 192, 217);
pub(crate) const YELLOW: Color = Color::Rgb(148, 86, 0);
pub(crate) const BLACK: Color = Color::Rgb(33, 39, 45);

pub(crate) const BACKGROUND: Color = BLACK;
pub(crate) const FOREGROUND: Color = WHITE;

pub(crate) fn default_style() -> Style {
    Style::default().bg(BACKGROUND).fg(FOREGROUND)
}
