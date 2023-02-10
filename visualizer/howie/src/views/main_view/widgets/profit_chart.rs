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
        .style(Style::default().fg(YELLOW))
        .graph_type(tui::widgets::GraphType::Line)
        .data(&data);
    let datasets = vec![d1];

    let max_y = format!("{}{}", decimal_format_float(*max), unit_of_measure(*max));
    let min_y = format!("{}{}", decimal_format_float(*min), unit_of_measure(*min));

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
        .style(Style::default().bg(BACKGROUND))
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
                    Span::styled(min_y, Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(max_y, Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([*min, *max]),
        );

    frame.render_widget(chart, area);
}

// pub(crate) fn format_float(f: f64) -> String {
//     let approx = get_approx(f);

//     let mut s = approx.round().to_string();
//     s.push_str(".00");
//     let idx = s.find('.').unwrap();

//     s.truncate(idx);

//     format!("{}{}", s, unit_of_measure(f))
// }

fn get_approx(f: f64) -> f64 {
    let divider = {
        let i = f.abs().log10().floor() as u32;

        let floor = i - i % 3;
        10_i32.pow(floor).max(1)
    };

    f / divider as f64
}

pub(crate) fn decimal_format_float(f: f64) -> String {
    let approx = get_approx(f);

    let mut s = ((approx * 100.0).round() / 100.0).to_string();
    if !s.contains('.') {
        s.push('.');
    }
    s.push_str("00");
    let idx = s.find('.').unwrap();

    s.truncate(idx + 3);

    s
}

pub(crate) fn unit_of_measure(number: f64) -> char {
    let order = number.abs().log10();
    if order < 3.0 {
        ' '
    } else if order < 6.0 {
        'K'
    } else if order < 9.0 {
        'M'
    } else if order < 12.0 {
        'B'
    } else {
        '🤑'
    }
}
