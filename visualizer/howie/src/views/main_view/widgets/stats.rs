use ipc_utils::trading_event_details::TradeType;
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

    let mut m: Vec<(&str, String)> = Vec::new();
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
        profit_string.push('%');
        m.push(("Profit", profit_string));
    }

    m.push((
        "Buy ops",
        stats
            .total_trades
            .get(&TradeType::Buy)
            .unwrap_or(&0)
            .to_string(),
    ));

    m.push((
        "Sell ops",
        stats
            .total_trades
            .get(&TradeType::Sell)
            .unwrap_or(&0)
            .to_string(),
    ));

    m.push((
        "Lock ops",
        stats
            .total_locks
            .iter()
            .fold(0, |acc, (_, v)| acc + v)
            .to_string(),
    ));

    let max_len = m.iter().fold(0, |max_len, (k, _)| k.len().max(max_len));

    let items = Vec::from_iter(
        m.into_iter()
            .map(|(k, v)| {
                let padding = " ".repeat(max_len - k.len());
                format!("{}{}  {}", k, padding, v)
            })
            .map(|line| ListItem::new(Text::from(format!("{}\n\n", line)))),
    );

    let list = List::new(items);
    frame.render_widget(list, *layout.last().unwrap());
}
