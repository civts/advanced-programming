use crate::{
    domain::app_state::AppState,
    views::{
        main_view::widgets::{
            capital::render_capital_widget, markets::render_market_chart,
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

use self::widgets::profit_chart;

mod widgets;
pub(crate) struct MainView {}

impl MainView {
    pub fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &AppState) {
        terminal
            .draw(|f| {
                draw_background(f);

                // let last_event = build_latest_event_widget(state);

                let cell_sizes = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(20), Constraint::Percentage(80)].as_ref())
                    .margin(1)
                    .split(f.size());
                let left = cell_sizes.get(0).unwrap().to_owned();
                let right = cell_sizes.get(1).unwrap().to_owned();

                render_column_widget(state, left, f);

                profit_chart::chart(f, &state.events, right);
            })
            .expect("Can draw on the terminal");
    }
}

fn render_column_widget<B: Backend>(state: &AppState, area: Rect, frame: &mut Frame<B>) {
    let trader_name_height = 2;
    let a = (area.height - trader_name_height) / 3;
    let cell_sizes = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(trader_name_height),
                Constraint::Length(a),
                Constraint::Length(a),
                Constraint::Length(a),
            ]
            .as_ref(),
        )
        .horizontal_margin(1)
        .split(area);

    let last_event = state.events.last().expect("There is at least one event");

    let top = cell_sizes.get(0).unwrap().to_owned();
    let center = cell_sizes.get(1).unwrap().to_owned();
    let center_bottom = cell_sizes.get(2).unwrap().to_owned();
    let bottom = cell_sizes.get(3).unwrap().to_owned();

    render_trader_name_widget(frame, last_event, top);

    render_market_chart(frame, center);

    render_capital_widget(frame, &last_event.trader_state, center_bottom);

    render_stats_widget(frame, last_event, bottom);
}
