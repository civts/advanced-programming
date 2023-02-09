use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

pub(crate) fn render_stats_widget<B: Backend>(
    frame: &mut Frame<B>,
    last_event: &ipc_utils::trading_event::TradingEvent,
    area: Rect,
) {
    frame.render_widget(
        Paragraph::new(last_event.trader_state.cash.len().to_string())
            .style(Style::default().bg(Color::Blue)),
        area,
    );
}
