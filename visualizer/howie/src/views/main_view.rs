use super::utils::draw_background;
use crate::domain::app_state::AppState;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub(crate) struct MainView {}

impl MainView {
    pub fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &AppState) {
        terminal
            .draw(|f| {
                draw_background(f);

                let size = f.size();
                let inner_text = format!(
                    "Event #{}: {:?}",
                    // self.state.received_messages, trader_event
                    1,
                    2
                );
                let p = Paragraph::new(inner_text).wrap(Wrap { trim: true });

                let block = Block::default()
                    .title(format!("Size is {:?}", size))
                    .borders(Borders::ALL);
                let p_in_block = p
                    .block(block)
                    .style(Style::default().bg(Color::Rgb(33, 39, 45)));

                // let row = Row::new(vec![
                //     // Cow::Borrowed("hello"),
                //     // Cow::Owned("world".to_uppercase()),
                //     Cell::from(Span::from("abcd")),
                //     Cell::from(Span::from("abcde"))
                //         .style(Style::default().bg(Color::Rgb(33, 39, 45))),
                // ]);

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Max(30), Constraint::Percentage(89)].as_ref())
                    .margin(1)
                    .split(f.size());

                f.render_widget(p_in_block.clone(), chunks.get(0).unwrap().to_owned());
                f.render_widget(p_in_block, chunks.get(1).unwrap().to_owned());
            })
            .expect("Can draw on the terminal");
    }
}
