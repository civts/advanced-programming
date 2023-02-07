use std::{fmt::format, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ipc_utils::IpcUtils;
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::constants::REFRESH_RATE_MILLISECONDS;

pub(crate) struct App {}

impl App {
    pub(crate) fn new() -> Self {
        App {}
    }

    pub(crate) fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) {
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

            let event = IpcUtils::receive();

            terminal
                .draw(|f| {
                    let inner_text = format!("New event: {:?}", event);
                    let p = Paragraph::new(inner_text);

                    let size = f.size();
                    let block = Block::default()
                        .title(format!("Size is {:?}", size))
                        .borders(Borders::ALL);

                    let p_in_block = p.block(block);

                    f.render_widget(p_in_block, size);
                })
                .expect("Can draw on the terminal");
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
