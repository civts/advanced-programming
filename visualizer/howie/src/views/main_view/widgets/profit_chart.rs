use tui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Chart, Dataset},
    Frame,
};
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

use crate::domain::app_theme::AppTheme;

pub(crate) fn chart<'a, B: Backend, T>(
    frame: &mut Frame<B>,
    events: T,
    area: Rect,
    theme: &AppTheme,
) where
    T: Iterator<Item = &'a (u64, f64)>,
{
    let data = Vec::from_iter(events.map(|(k, v)| (*k as f64, *v)));
    let min = &data
        .clone()
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap()
        .1;
    let max = &data
        .clone()
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap()
        .1;
    let len = data.len() as f64;
    let d1 = Dataset::default()
        // .name("data2")
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(theme.c2))
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
        .style(Style::default().bg(theme.background).fg(theme.c2))
        .x_axis(
            Axis::default()
                .title(Span::styled("Days", theme.default_style()))
                .style(Style::default().fg(theme.c1).add_modifier(Modifier::DIM))
                .labels(vec![
                    Span::styled(
                        data.first().unwrap().0.to_string(),
                        theme.default_style().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        data.last().unwrap().0.to_string(),
                        theme.default_style().add_modifier(Modifier::BOLD),
                    ),
                ])
                .bounds([0.0, len]),
        )
        .y_axis(
            Axis::default()
                .title(Spans::from(vec![
                    Span::styled(
                        format!(
                            "Total capital ({})",
                            DEFAULT_GOOD_KIND.to_string().to_uppercase()
                        ),
                        theme.default_style(),
                    ),
                    // Span::styled((" ").repeat(40), theme.default_style().fg(YELLOW)),
                ]))
                .labels_alignment(tui::layout::Alignment::Right)
                .style(Style::default().fg(theme.c1).add_modifier(Modifier::DIM))
                .labels(vec![
                    Span::styled(min_y, theme.default_style().add_modifier(Modifier::BOLD)),
                    Span::styled(max_y, theme.default_style().add_modifier(Modifier::BOLD)),
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
