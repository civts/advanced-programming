use crate::domain::app_theme::AppTheme;
use tui::{backend::Backend, style::Style, widgets::Block};

pub(crate) fn draw_background<B: Backend>(f: &mut tui::Frame<B>, theme: &AppTheme) {
    let size = f.size();
    f.render_widget(
        Block::default().style(Style::default().bg(theme.background)),
        size,
    );
}
