mod repository;

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use repository::{add_random_lock_to_db, add_random_trade_to_db, read_locks, read_trades};
use std::io::{self};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

pub(crate) fn viz() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let mut balance: Vec<f32> = vec![10000.0];

    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
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

    let menu_titles = vec!["Trades", "Add", "Quit"];
    let mut trade_list_state = ListState::default();
    trade_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
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

            let copyright = Paragraph::new("SOL Market - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(0)
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            let trades_chunks = Layout::default()
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

            let (left, center, right) = render_trades(balance.clone());

            rect.render_stateful_widget(left, trades_chunks[0], &mut trade_list_state);
            rect.render_widget(center, trades_chunks[1]);
            rect.render_widget(right, trades_chunks[2]);
            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('a') => {
                    let trades = add_random_trade_to_db().expect("can't add new random trade");

                    let random_trade = &trades[trades.len() - 1];
                    let last_balance = balance[balance.len() - 1];

                    if random_trade.operation == "SELL" {
                        let new_balance =
                            last_balance + random_trade.price * random_trade.quantity as f32;
                        balance.push(new_balance)
                    } else {
                        let new_balance =
                            last_balance - random_trade.price * random_trade.quantity as f32;
                        balance.push(new_balance)
                    }
                }
                KeyCode::Char('l') => {
                    add_random_lock_to_db().expect("can't add new random lock");
                }
                KeyCode::Down => {
                    if let Some(selected) = trade_list_state.selected() {
                        let amount_trades = read_trades().expect("can fetch trade list").len();
                        if selected >= amount_trades - 1 {
                            trade_list_state.select(Some(0));
                        } else {
                            trade_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = trade_list_state.selected() {
                        let amount_pets = read_trades().expect("can fetch trade list").len();
                        if selected > 0 {
                            trade_list_state.select(Some(selected - 1));
                        } else {
                            trade_list_state.select(Some(amount_pets - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_trades<'a>(balance: Vec<f32>) -> (List<'a>, Table<'a>, Table<'a>) {
    let balance_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Balance")
        .border_type(BorderType::Plain);

    let trade_vec = read_trades().expect("trades read failed!");
    let lock_vec = read_locks().expect("locks read failed!");

    let balances_items: Vec<_> = balance
        .iter()
        .map(|b| {
            ListItem::new(Spans::from(vec![Span::styled(
                b.to_string(),
                Style::default(),
            )]))
        })
        .collect();

    let list_trades = List::new(balances_items)
        .block(balance_block)
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    let trade_rows: Vec<Row> = trade_vec
        .iter()
        .map(|t| {
            Row::new(vec![
                Cell::from(Span::raw(t.operation.to_string())),
                Cell::from(Span::raw(t.market.to_string())),
                Cell::from(Span::raw(t.good_kind.to_string())),
                Cell::from(Span::raw(t.quantity.to_string())),
                Cell::from(Span::raw(t.timestamp.format("%H:%M:%S").to_string())),
                Cell::from(Span::raw(format!("{:.2}", t.price))),
            ])
        })
        .collect();

    let lock_rows: Vec<Row> = lock_vec
        .iter()
        .map(|l| {
            Row::new(vec![
                Cell::from(Span::raw(l.operation.to_string())),
                Cell::from(Span::raw(l.market.to_string())),
                Cell::from(Span::raw(l.good_kind.to_string())),
                Cell::from(Span::raw(l.quantity.to_string())),
                Cell::from(Span::raw(l.token.to_string())),
                Cell::from(Span::raw(format!("{:.2}", l.price))),
            ])
        })
        .collect();

    let trade_detail = Table::new(trade_rows)
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "Operation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Market",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Good Kind",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Quantity",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Timestamp",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Price",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Trades")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(16),
            Constraint::Percentage(15),
            Constraint::Percentage(23),
            Constraint::Percentage(17),
            Constraint::Percentage(18),
            Constraint::Percentage(16),
        ]);

    let locks_detail = Table::new(lock_rows)
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "Operation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Market",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Good Kind",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Quantity",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Token",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Price",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Locks")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
        ]);

    (list_trades, trade_detail, locks_detail)
}
