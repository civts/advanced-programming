use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::domain::app_state::AppState;

pub(crate) fn build_latest_event_widget(state: &AppState) -> impl Widget {
    let inner_text = format!(
        "Event #{}: {:?}",
        state.received_messages,
        state.events.last().expect("There is at least one event")
    );
    let p = Paragraph::new(inner_text).wrap(Wrap { trim: true });

    let block = Block::default().title("Latest event").borders(Borders::ALL);
    let latest_event = p
        .block(block)
        .style(Style::default().bg(Color::Rgb(33, 39, 45)));
    latest_event
}
