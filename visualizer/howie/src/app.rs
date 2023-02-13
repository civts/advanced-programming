use crate::{
    constants::REFRESH_RATE_MILLISECONDS,
    domain::app_state::AppState,
    domain::{app_theme::app_themes, app_view::AppView},
    views::{error_view, help_view::draw_help_view, main_view::MainView, wait_view},
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ipc_utils::IPCReceiver;
use std::time::Duration;
use tui::{backend::Backend, Terminal};

pub(crate) struct App {
    receiver: IPCReceiver,

    /// The app state
    state: AppState,

    /// How often we refresh the screen
    refresh_speed: Duration,
}

impl App {
    pub(crate) fn new() -> Self {
        let duration = Duration::from_millis(REFRESH_RATE_MILLISECONDS);
        App {
            receiver: IPCReceiver::new(duration),
            state: AppState::default(),
            refresh_speed: duration,
        }
    }

    pub(crate) fn run<B: Backend>(&mut self, mut terminal: Terminal<B>) {
        // Wether on this iteration the terminal should repaint completely
        let mut should_clear: bool;
        let mut should_run_again = false;
        self.state.current_view = AppView::WaitingForFirstTrade;
        self.state.paused = false;
        let minimum_refresh_duration = Duration::from_millis(REFRESH_RATE_MILLISECONDS / 2);
        let mut theme_index: usize = 0;
        let themes = app_themes();

        // Something different from AppState::WaitingForFirstTrade
        let s = AppView::HelpMenu;
        let mut previous_state_variant = std::mem::discriminant(&s);
        loop {
            let current_state_variant = std::mem::discriminant(&self.state.current_view);
            should_clear = previous_state_variant != current_state_variant;

            if !self.state.paused {
                match self.state.current_view {
                    AppView::HelpMenu => {
                        //do nothing
                    }
                    _ => {
                        // Receive and process next event (if any)
                        let trader_event_res = self.receiver.receive();
                        self.state.pipe_closed = false;
                        match trader_event_res {
                            Ok(event_opt) => match event_opt {
                                Some(trader_event) => {
                                    self.state.update(&trader_event);

                                    self.state.current_view = AppView::MainTradingView;
                                }
                                None => self.state.current_view = AppView::WaitingForFirstTrade,
                            },
                            Err(error) => {
                                if self.state.events.is_empty() {
                                    self.state.current_view = AppView::ErrorView(error);
                                } else {
                                    self.state.pipe_closed = true;
                                }
                            }
                        }
                    }
                }

                // Redraw the screen
                if should_clear {
                    previous_state_variant = current_state_variant;
                    terminal.clear().expect("Can clear the terminal");
                }
                match &self.state.current_view {
                    AppView::WaitingForFirstTrade => {
                        wait_view::draw(&mut terminal, &self.state.theme)
                    }
                    AppView::MainTradingView => MainView::draw(&mut terminal, &self.state),
                    AppView::HelpMenu => draw_help_view(&mut terminal, &self.state.theme),
                    AppView::ErrorView(e) => error_view::draw(&mut terminal, e, &self.state.theme),
                }
            }

            // Check for events in the terminal (User input)
            if self.is_a_new_event_available() {
                // If a key was pressed
                if let Event::Key(key) = Self::get_event() {
                    match key.modifiers {
                        KeyModifiers::NONE => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                break;
                            }
                            KeyCode::Char('p') | KeyCode::Char(' ') => {
                                self.state.paused = !self.state.paused;
                            }
                            KeyCode::Char('v') => {
                                self.state.trading_volume_chart_visible =
                                    !self.state.trading_volume_chart_visible;
                            }
                            KeyCode::Char('+') => {
                                self.refresh_speed = self.refresh_speed.mul_f32(1.1);
                            }
                            KeyCode::Char('-') => {
                                self.refresh_speed = self
                                    .refresh_speed
                                    .div_f32(1.1)
                                    .max(minimum_refresh_duration);
                            }
                            KeyCode::Char('t') => {
                                theme_index += 1;
                                theme_index %= themes.len();
                                self.state.theme = *themes.get(theme_index).unwrap();
                            }
                            KeyCode::Char('r') => {
                                let waiting_trader =
                                    std::mem::discriminant(&AppView::WaitingForFirstTrade);
                                if current_state_variant == waiting_trader {
                                    should_run_again = true;
                                    break;
                                }
                            }
                            KeyCode::Char('h') | KeyCode::Char('?') => {
                                match self.state.current_view {
                                    AppView::HelpMenu => {
                                        self.state.current_view = AppView::MainTradingView
                                    }
                                    _ => {
                                        self.state.current_view = AppView::HelpMenu;
                                    }
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
        } // loop end

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
    fn is_a_new_event_available(&self) -> bool {
        event::poll(self.refresh_speed).expect("Can poll for input")
    }

    /// Returns the next event from the terminal
    fn get_event() -> Event {
        event::read().expect("Can get next event")
    }
}
