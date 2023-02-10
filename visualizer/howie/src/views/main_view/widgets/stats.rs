use std::collections::HashMap;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::Text,
    widgets::{List, ListItem, Paragraph},
    Frame,
};

use crate::{constants::default_style, domain::stats::Stats};

pub(crate) fn render_stats_widget<B: Backend>(stats: &Stats, frame: &mut Frame<B>, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(area);

    frame.render_widget(
        Paragraph::new("Stats").style(default_style().add_modifier(Modifier::BOLD)),
        *layout.first().unwrap(),
    );

    let mut m = HashMap::new();
    if stats.starting_capital != 0.0 {
        let profit = stats
            .profit_history
            .back()
            .map(|(_, capital)| *capital)
            .unwrap_or(0.0)
            / (stats.starting_capital as f64)
            - 1.0;
        let mut profit_string = (profit * 100.0).to_string();
        profit_string.push_str(".00");
        profit_string.truncate(profit_string.find('.').unwrap() + 2);
        m.insert("Profit", profit_string);
    }

    let content: Vec<ListItem> = m
        .into_iter()
        .map(|(k, v)| ListItem::new(Text::from(format!("{} {}%", k, v))))
        .collect();
    frame.render_widget(List::new(content), *layout.last().unwrap());
}
