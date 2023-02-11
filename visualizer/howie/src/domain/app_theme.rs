use tui::style::{Color, Style};

use crate::constants::{BACKGROUND, BLUE, FOREGROUND, RED, YELLOW};

#[derive(Clone, Copy)]
pub(crate) struct AppTheme {
    pub(crate) background: Color,
    pub(crate) foreground: Color,
    pub(crate) c1: Color,
    pub(crate) c2: Color,
    pub(crate) accent: Color,
}

impl AppTheme {
    pub(crate) fn default_style(&self) -> Style {
        Style::default().bg(self.background).fg(self.foreground)
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            background: BACKGROUND,
            foreground: FOREGROUND,
            c1: BLUE,
            c2: YELLOW,
            accent: RED,
        }
    }
}

pub(crate) fn app_themes() -> Vec<AppTheme> {
    vec![
        AppTheme::default(),
        AppTheme {
            background: Color::Rgb(2, 52, 54),
            foreground: Color::Rgb(3, 181, 170),
            c1: Color::Rgb(3, 121, 113),
            c2: Color::Rgb(190, 178, 200),
            accent: Color::Rgb(247, 235, 236),
        },
        AppTheme {
            background: Color::Rgb(13, 14, 18),
            foreground: Color::Rgb(235, 252, 239),
            c1: Color::Rgb(204, 207, 205),
            c2: Color::Rgb(104, 158, 100),
            accent: Color::Rgb(41, 97, 42),
        },
    ]
}
