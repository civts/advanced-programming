use std::io::{self, Stdout};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ipc_utils::trader_state::ALL_GOOD_KINDS;
use tui::{Frame, Terminal};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Table};
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

use crate::visualization::components::components::{get_balance_table, get_copyright, get_lock_table, get_stats_paragraph, get_trade_table};
use crate::visualization::repository::repository::{clear_all, find_latest_balance, read_locks, read_trades};
use crate::visualization::service::plotlib::{generate_all_balances_plot, plot_for_gk};
use crate::visualization::service::service::Service;

pub mod repository;
pub mod components;
pub mod service;

enum Event<I> {
    Input(I),
    Tick,
}

pub struct Visualization {
    service: Service,
}

impl Visualization {
    pub fn new() -> Visualization {
        Visualization {
            service: Service::new()
        }
    }

    pub(crate) fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut service: Service = Service::new();

        enable_raw_mode().expect("can run in a raw mode");

        let (tx, rx) = mpsc::channel();

        let tick_rate = Duration::from_millis(100);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                service.receive();
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works") {
                    if let CEvent::Key(key) = event::read().expect("can read events") {
                        tx.send(Event::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(Event::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        loop {
            terminal.draw(|rect| {
                let (chunks, tables_chunks) = Self::get_chunks(rect);
                Self::render_ui(self, rect, chunks, tables_chunks);
            })?;

            match rx.recv()? {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        generate_all_balances_plot();
                        ALL_GOOD_KINDS.iter().for_each(|good| { plot_for_gk(good.clone()) });
                        clear_all();
                        break;
                    }
                    _ => {}
                },
                Event::Tick => {}
            }
        }

        Ok(())
    }

    fn get_chunks(rect: &mut Frame<CrosstermBackend<Stdout>>) -> (Vec<Rect>, Vec<Rect>) {
        let size = rect.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ]
                    .as_ref(),
            )
            .split(size);

        let tables_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(45),
                    Constraint::Percentage(45),
                ]
                    .as_ref(),
            )
            .split(chunks[1]);
        (chunks, tables_chunks)
    }

    fn render_ui(&self, rect: &mut Frame<CrosstermBackend<Stdout>>, chunks: Vec<Rect>, tables_chunks: Vec<Rect>) {
        let (left, center, right) = Self::render_tables();

        let yen_prf = self.service.get_profit(YEN);
        let yuan_prf = self.service.get_profit(YUAN);
        let usd_prf = self.service.get_profit(USD);
        let eur_prf = self.service.get_profit(EUR);

        rect.render_widget(get_stats_paragraph(format!("Profit YEN : {:.2}% | Profit YUAN {:.2}% | Profit USD {:.2}% | Profit EUR {:.2}% No. of trades {} | No. of locks {}", yen_prf, yuan_prf, usd_prf, eur_prf, self.service.failed_trades, self.service.failed_locks).as_str()), chunks[0]);
        rect.render_widget(left, tables_chunks[0]);
        rect.render_widget(center, tables_chunks[1]);
        rect.render_widget(right, tables_chunks[2]);
        rect.render_widget(get_copyright(), chunks[2]);
    }

    fn render_tables<'a>() -> (Table<'a>, Table<'a>, Table<'a>) {
        let lock_table = get_lock_table(read_locks().unwrap());
        let trade_table = get_trade_table(read_trades().unwrap());
        let balance_table = get_balance_table(
            find_latest_balance(YEN),
            find_latest_balance(YUAN),
            find_latest_balance(USD),
            find_latest_balance(EUR),
        );

        (balance_table, trade_table, lock_table)
    }
}




