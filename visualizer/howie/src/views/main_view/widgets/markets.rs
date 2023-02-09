use tui::{backend::Backend, layout::Rect, style::Style, widgets::BarChart, Frame};

use crate::constants::RED;

pub(crate) fn render_market_chart<B: Backend>(frame: &mut Frame<B>, area: Rect) {
    frame.render_widget(
        BarChart::default()
            .data(&[("a", 2), ("b", 5), ("c", 4)])
            .bar_width(5)
            .style(Style::default().fg(RED)),
        area,
    );
}
