use crate::{
    constants::REFRESH_RATE_MILLISECONDS,
    domain::app_state::AppState,
    domain::app_view::AppView,
    views::{error_view, wait_view},
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ipc_utils::IPCReceiver;
use std::time::Duration;
use tui::{backend::Backend, Terminal};

pub(crate) struct App {
    receiver: IPCReceiver,

    /// The app state
    state: AppState,
}

impl App {
    pub(crate) fn new() -> Self {
        App {
            receiver: IPCReceiver::new(Duration::from_millis(REFRESH_RATE_MILLISECONDS)),
            state: AppState::default(),
        }
    }

    pub(crate) fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) {
        // Wether on this iteration the terminal should repaint completely
        let mut should_clear: bool;
        let mut received_messages: u64 = 0;
        let mut should_run_again = false;
        self.state.current_view = AppView::WaitingForFirstTrade;

        // Something different from AppState::WaitingForFirstTrade
        let s = AppView::HelpMenu;
        let mut previous_state_variant = std::mem::discriminant(&s);
        loop {
            let current_state_variant = std::mem::discriminant(&self.state.current_view);
            should_clear = previous_state_variant != current_state_variant;

            if should_clear {
                previous_state_variant = current_state_variant;
                terminal.clear().expect("Can clear the terminal");
            }

            match self.state.current_view {
                AppView::WaitingForFirstTrade => {
                    wait_view::draw(&mut terminal);
                }
                AppView::MainTradingView => {
                    let trader_event_res = self.receiver.receive();

                    match trader_event_res {
                        Ok(Some(trader_event)) => {
                            received_messages += 1;

                            todo!("Update state");
                            //MainView::draw(&mut terminal, &self.state);
                        }
                        Ok(None) => {}
                        Err(error) => {
                            error_view::draw(&mut terminal, error);
                        }
                    }
                }
                _ => todo!(),
            }

            // Check for events in the terminal (User input)
            if Self::is_a_new_event_available() {
                // If a key was pressed
                if let Event::Key(key) = Self::get_event() {
                    match key.modifiers {
                        KeyModifiers::NONE => match key.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Char('r') => {
                                let waiting_trader =
                                    std::mem::discriminant(&AppView::WaitingForFirstTrade);
                                if current_state_variant == waiting_trader {
                                    should_run_again = true;
                                    break;
                                }
                            }
                            _ => {}
                        },
                        KeyModifiers::CONTROL => {
                            if let KeyCode::Char('c') = key.code {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        if should_run_again {
            self.receiver.restart();
            self.run(terminal)
        } else {
            Self::prepare_for_exit(terminal)
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