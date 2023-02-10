use std::collections::HashMap;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{BarChart, Paragraph},
    Frame,
};

use crate::constants::{default_style, BACKGROUND, RED};

pub(crate) fn render_market_chart<B: Backend>(
    stats: &HashMap<String, u64>,
    frame: &mut Frame<B>,
    area: Rect,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(area);

    render_title(*layout.first().unwrap(), frame);

    render_chart(stats, *layout.last().unwrap(), frame);
}

fn render_title<B: Backend>(area: Rect, frame: &mut Frame<B>) {
    frame.render_widget(
        Paragraph::new("Most Active Markets").style(default_style().add_modifier(Modifier::BOLD)),
        area,
    );
}

fn render_chart<B: Backend>(stats: &HashMap<String, u64>, area: Rect, frame: &mut Frame<B>) {
    let markets = Vec::from_iter(stats.iter().map(|(k, v)| (k.as_str(), *v)));
    let number_of_markets: u16 = markets
        .len()
        .try_into()
        .unwrap_or_else(|_| panic!("Should have less than {} markets", u16::MAX));
    let margin = 1;
    let bar_width = if number_of_markets != 0 {
        area.width / number_of_markets - margin
    } else {
        0
    };
    frame.render_widget(
        BarChart::default()
            .data(&markets)
            .bar_width(bar_width)
            .style(Style::default().fg(RED))
            .value_style(Style {
                fg: Some(BACKGROUND),
                bg: Some(RED),
                ..Default::default()
            }),
        area,
    );
}