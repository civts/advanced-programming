use crate::constants::default_style;
use ipc_utils::trader_state::{TraderState, ALL_GOOD_KINDS};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    widgets::Paragraph,
    Frame,
};

pub(crate) fn render_portfolio_widget<B: Backend>(
    frame: &mut Frame<B>,
    state: &TraderState,
    area: Rect,
) {
    let a = Rect { height: 1, ..area };
    let av = Rect {
        height: area.height - 1,
        y: area.y + 1,
        ..area
    };
    let constraints = [Constraint::Min(2)].repeat(ALL_GOOD_KINDS.len());

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .vertical_margin(1)
        .split(av);

    frame.render_widget(
        Paragraph::new("Portfolio").style(default_style().add_modifier(Modifier::BOLD)),
        a,
    );

    let max_len = ALL_GOOD_KINDS
        .iter()
        .fold(0, |acc, &gk| gk.to_string().len().max(acc));

    for i in 0..layout.len() {
        let goodkind = ALL_GOOD_KINDS[i];
        let area = layout[i];
        let quantity = state.cash.get(&goodkind).unwrap();
        let good_str = goodkind.to_string();
        let padding = String::from_iter([' '].repeat(max_len - good_str.len()));
        frame.render_widget(
            Paragraph::new(format!("{}{} {}", good_str, padding, quantity)),
            area,
        );
    }
}
