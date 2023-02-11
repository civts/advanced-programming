use crate::domain::{app_theme::AppTheme, stats::Stats};
use ipc_utils::trading_event_details::{TradeOperation, TradingEventDetails};
use tui::{
    backend::Backend, layout::Rect, style::Style, symbols::bar::THREE_LEVELS, widgets::BarChart,
    Frame,
};

pub(crate) fn render_trading_volume_widget<B: Backend>(
    frame: &mut Frame<B>,
    stats: &Stats,
    area: Rect,
    theme: &AppTheme,
) {
    let area = Rect {
        //since BarChart always draws the labels, I make them go one row below and invisible
        height: area.height + 1,
        ..area
    };
    let daily_volume: Vec<(String, u64)> = stats
        .recent_trades
        .iter()
        .map(|trade| match trade.details {
            TradingEventDetails {
                operation: TradeOperation::TradeFinalized,
                successful: true,
                quantity,
                ..
            } => quantity.round(),
            _ => 0.0,
        })
        .enumerate()
        .map(|(k, v)| (k.to_string(), v as u64))
        .collect();

    let skips = if daily_volume.len() > area.width as usize {
        daily_volume.len() - area.width as usize
    } else {
        0
    };

    frame.render_widget(
        BarChart::default()
            .data(&Vec::from_iter(
                daily_volume
                    .iter()
                    .skip(skips)
                    .map(|(k, v)| (k.as_str(), *v)),
            ))
            .label_style(Style::default().fg(theme.background))
            .bar_width(1)
            .style(Style::default().fg(theme.c1))
            .bar_gap(0)
            .bar_set(THREE_LEVELS)
            .value_style(Style {
                fg: Some(theme.background),
                bg: Some(theme.c1),
                ..Default::default()
            }),
        area,
    )
}
