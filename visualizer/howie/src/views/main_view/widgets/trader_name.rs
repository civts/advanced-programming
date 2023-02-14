use tui::{backend::Backend, layout::Rect, widgets::Paragraph, Frame};

use crate::domain::app_state::AppState;

pub(crate) fn render_trader_name_widget<B: Backend>(
    frame: &mut Frame<B>,
    state: &AppState,
    area: Rect,
) {
    if state.trader_finished {
        frame.render_widget(
            Paragraph::new("Trader finished")
                .style(state.theme.default_style().fg(state.theme.accent)),
            area,
        );
    } else {
        let trader_name = state
            .events
            .last()
            .map(|event| event.trader_state.name.to_string())
            .unwrap_or_default();
        frame.render_widget(
            Paragraph::new(trader_name).style(state.theme.default_style()),
            area,
        );
    }
}
