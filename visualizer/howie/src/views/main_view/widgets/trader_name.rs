use tui::{backend::Backend, layout::Rect, widgets::Paragraph, Frame};

pub(crate) fn render_trader_name_widget<B: Backend>(
    frame: &mut Frame<B>,
    last_event: &ipc_utils::trading_event::TradingEvent,
    area: Rect,
) {
    frame.render_widget(
        Paragraph::new(last_event.trader_state.name.to_string()),
        area,
    );
}
