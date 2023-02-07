use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ipc_utils::IpcUtils;
use std::time::Duration;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use crate::constants::REFRESH_RATE_MILLISECONDS;

pub(crate) struct App {}

impl App {
    pub(crate) fn new() -> Self {
        App {}
    }

    pub(crate) fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) {
        terminal
            .draw(|f| {
                draw_background(f);
                f.render_widget(
                    Paragraph::new(
                        "Waiting for the first event ⏳\r\n\n \
                               Start a trader",
                    )
                    .alignment(Alignment::Center),
                    f.size(),
                );
            })
            .expect("Can draw first frame");

        let mut should_clear = true;
        let mut received_messages: i64 = 0;
        loop {
            // Check for events in the terminal
            if Self::is_a_new_event_available() {
                // If a key was pressed
                if let Event::Key(key) = Self::get_event() {
                    match key.modifiers {
                        KeyModifiers::NONE => {
                            if let KeyCode::Char('q') = key.code {
                                Self::prepare_for_exit(terminal);
                                break;
                            }
                        }
                        KeyModifiers::CONTROL => {
                            if let KeyCode::Char('c') = key.code {
                                Self::prepare_for_exit(terminal);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }

            let trader_event_res = IpcUtils::receive();

            match trader_event_res {
                Ok(Some(trader_event)) => {
                    if should_clear {
                        should_clear = false;
                        terminal.clear().expect("Can clear the terminal");
                    }

                    received_messages += 1;

                    terminal
                        .draw(|f| {
                            draw_background(f);

                            let size = f.size();
                            let inner_text =
                                format!("Event #{}: {:?}", received_messages, trader_event);
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
                                .constraints(
                                    [Constraint::Max(30), Constraint::Percentage(89)].as_ref(),
                                )
                                .margin(1)
                                .split(f.size());

                            f.render_widget(p_in_block.clone(), chunks.get(0).unwrap().to_owned());
                            f.render_widget(p_in_block, chunks.get(1).unwrap().to_owned());
                        })
                        .expect("Can draw on the terminal");
                }
                Ok(None) => {}
                Err(error) => {
                    if let std::io::ErrorKind::NotFound = error.kind() {
                        terminal
                            .draw(|f| {
                                draw_background(f);
                                let size = f.size();
                                let p = Paragraph::new(
                                    "Did not find the pipe 🤔\r\nMake sure you started a trader",
                                )
                                .alignment(Alignment::Center)
                                .wrap(Wrap { trim: true })
                                .style(Style {
                                    fg: Some(Color::Red),
                                    ..Style::default()
                                });

                                f.render_widget(p, size);
                            })
                            .expect("Can draw on the terminal");
                    }
                }
            }
        }
    }

    fn prepare_for_exit<B: Backend>(mut terminal: Terminal<B>) {
        terminal.clear().expect("Can clear the terminal");
        println!("\rFarewell 👋\r");
    }

    /// Returns if one or more new events are available on the terminal
    fn is_a_new_event_available() -> bool {
        event::poll(Duration::from_millis(REFRESH_RATE_MILLISECONDS)).expect("Can poll for input")
    }

    /// Returns the next event from the terminal
    fn get_event() -> Event {
        event::read().expect("Can get next event")
    }
}

fn draw_background<B: Backend>(f: &mut tui::Frame<B>) {
    let size = f.size();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(33, 39, 45))),
        size,
    );
}
