use crate::{
    domain::app_state::AppState,
    views::{
        main_view::widgets::{
            markets::render_market_chart, portfolio::render_portfolio_widget,
            stats::render_stats_widget, trader_name::render_trader_name_widget,
        },
        utils::draw_background,
    },
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};

use self::widgets::{profit_chart, trading_volume::render_trading_volume_widget};

pub(crate) mod widgets;
pub(crate) struct MainView {}

impl MainView {
    pub fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &AppState) {
        terminal
            .draw(|f| {
                draw_background(f, &state.theme);

                // let last_event = build_latest_event_widget(state);

                let cell_sizes = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Percentage(80)])
                    .margin(1)
                    .split(f.size());
                let left = cell_sizes.get(0).unwrap().to_owned();
                let right = cell_sizes.get(1).unwrap().to_owned();

                render_column_widget(state, left, f);

                render_right(f, state, right);
            })
            .expect("Can draw on the terminal");
    }
}

fn render_right<B: Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    if state.trading_volume_chart_visible {
        // Render both trading volume and chart
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Min(2)])
            .split(area);

        profit_chart::chart(
            frame,
            state.stats.profit_history.iter(),
            *layout.first().unwrap(),
            &state.theme,
        );

        render_trading_volume_widget(frame, &state.stats, *layout.last().unwrap(), &state.theme);
    } else {
        // Render only the chart
        profit_chart::chart(frame, state.stats.profit_history.iter(), area, &state.theme);
    }
}

fn render_column_widget<B: Backend>(state: &AppState, area: Rect, frame: &mut Frame<B>) {
    let trader_name_height = 1;
    let a = (area.height - trader_name_height - 3) / 3;
    let cell_sizes = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(trader_name_height),
                Constraint::Length(1), // Empty space
                Constraint::Length(a),
                Constraint::Length(1), // Empty space
                Constraint::Length(a),
                Constraint::Length(1), // Empty space
                Constraint::Length(a),
            ]
            .as_ref(),
        )
        .horizontal_margin(1)
        .split(area);

    let last_event = state.events.last().expect("There is at least one event");

    let top = cell_sizes.get(0).unwrap().to_owned();
    let center = cell_sizes.get(2).unwrap().to_owned();
    let center_bottom = cell_sizes.get(4).unwrap().to_owned();
    let bottom = cell_sizes.get(6).unwrap().to_owned();

    render_trader_name_widget(frame, last_event, top);

    render_market_chart(&state.stats.trades_with_market, frame, center, &state.theme);

    render_portfolio_widget(frame, &last_event.trader_state, center_bottom, &state.theme);

    render_stats_widget(&state.stats, frame, bottom, &state.theme);
}
