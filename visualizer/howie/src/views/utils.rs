use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::Block,
};

pub fn draw_background<B: Backend>(f: &mut tui::Frame<B>) {
    let size = f.size();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(33, 39, 45))),
        size,
    );
}
