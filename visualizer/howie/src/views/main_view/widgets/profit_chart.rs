use crate::constants::{default_style, BACKGROUND, BLUE, YELLOW};
use ipc_utils::trading_event::TradingEvent;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Chart, Dataset},
    Frame,
};
use unitn_market_2022::good::{consts::DEFAULT_GOOD_KIND, good_kind::GoodKind};

pub(crate) fn chart<B: Backend>(frame: &mut Frame<B>, events: &[TradingEvent], area: Rect) {
    let trades = events.iter().enumerate().map(|(i, e)| {
        let total_capital: f64 = e.trader_state.cash.iter().fold(0.0, |acc, (gk, qt)| {
            let value = GoodKind::get_default_exchange_rate(gk);
            let vf64: f64 = (value * qt).try_into().unwrap();
            acc + vf64
        });
        let i = i as f64;
        (i, total_capital)
    });
    let min = &trades.clone().min_by(|a, b| a.1.total_cmp(&b.1)).unwrap().1;
    let max = &trades.clone().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap().1;
    let data = Vec::from_iter(trades);
    let len = data.len() as f64;
    let d1 = Dataset::default()
        // .name("data2")
        .marker(symbols::Marker::Braille)
        .style(Style::default().bg(BACKGROUND).fg(YELLOW))
        .graph_type(tui::widgets::GraphType::Line)
        .data(&data);
    let datasets = vec![d1];
    let chart = Chart::new(datasets)
        // .block(
        //     Block::default()
        //         .title(Span::styled(
        //             "Chart 1",
        //             Style::default()
        //                 .fg(Color::Cyan)
        //                 .add_modifier(Modifier::BOLD),
        //         ))
        //         .borders(Borders::ALL),
        // )
        .x_axis(
            Axis::default()
                .title("Days")
                .style(Style::default().fg(BLUE).add_modifier(Modifier::DIM))
                .labels(vec![
                    Span::styled(
                        data.first().unwrap().0.to_string(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        data.last().unwrap().0.to_string(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ])
                .bounds([0.0, len]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled(
                    format!(
                        "Total capital ({})",
                        DEFAULT_GOOD_KIND.to_string().to_uppercase()
                    ),
                    default_style(),
                ))
                .labels_alignment(tui::layout::Alignment::Right)
                .style(Style::default().fg(BLUE).add_modifier(Modifier::DIM))
                .labels(vec![
                    Span::styled(
                        format_float(min.to_string()),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format_float(max.to_string()),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ])
                .bounds([*min, *max]),
        );

    frame.render_widget(chart, area);
}

fn format_float(mut s: String) -> String {
    let final_len: usize = match s.find('.') {
        Some(index) => index + 1 + 2,
        None => {
            s.push('.');
            s.len() + 2
        }
    };
    // Pad the string if necessary
    while s.len() < final_len {
        s.push('0');
    }
    // Remove extra characters
    while s.len() != final_len {
        s.pop();
    }
    s
}
