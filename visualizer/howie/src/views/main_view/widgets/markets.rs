use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::BarChart,
    Frame,
};

pub(crate) fn render_market_chart<B: Backend>(frame: &mut Frame<B>, area: Rect) {
    frame.render_widget(
        BarChart::default()
            .data(&[("a", 2), ("b", 5), ("c", 4)])
            .style(Style::default().bg(Color::Green)),
        area,
    );
}
