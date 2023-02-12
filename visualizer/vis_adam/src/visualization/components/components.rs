use ipc_utils::trading_event_details::TradeType;
use tui::layout::{Alignment, Constraint};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span};
use tui::widgets::{Block, Borders, BorderType, Cell, Paragraph, Row, Table};
use unitn_market_2022::good::good_kind::GoodKind;
use crate::visualization::repository::repository::{Lock, Trade};

pub fn get_lock_table(locks: Vec<Lock>) -> Table<'static> {
    let locks_detail = Table::new(get_lock_rows(locks))
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
                .title("Locks")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(16),
            Constraint::Percentage(15),
            Constraint::Percentage(18),
            Constraint::Percentage(17),
            Constraint::Percentage(18),
            Constraint::Percentage(21)
        ]);
    locks_detail
}

fn get_lock_rows(lock_vec: Vec<Lock>) -> Vec<Row<'static>> {
    let lock_rows: Vec<Row> = lock_vec
        .iter()
        .map(|l| {
            Row::new(vec![
                Cell::from(Span::raw(get_operation_string(l.operation))),
                Cell::from(Span::raw(l.market.to_string())),
                get_cell_good_kind(l.good_kind),
                Cell::from(Span::raw(l.quantity.to_string())),
                Cell::from(Span::raw(l.timestamp.format("%H:%M:%S").to_string())),
                Cell::from(Span::raw(format!("{:.2}", l.price))),
            ])
        })
        .collect();
    lock_rows
}

pub fn get_balance_table(balance_yen: f32, balance_yuan: f32, balance_usd: f32, balance_eur: f32) -> Table<'static> {
    let balance_detail = Table::new(get_balance_rows(balance_yen, balance_yuan, balance_usd, balance_eur))
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "Good",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Value",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Balances")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(70)
        ]);
    balance_detail
}

fn get_balance_rows(balance_yen: f32, balance_yuan: f32, balance_usd: f32, balance_eur: f32) -> Vec<Row<'static>> {
    let mut balance_rows: Vec<Row> = vec![];

    let cell_yen = Cell::from(Span::raw(format!("{:.2}", balance_yen)))
        .style(Style::default().fg(Color::Yellow));
    let cell_desc_yen = Cell::from(Span::raw("YEN"))
        .style(Style::default().fg(Color::Yellow));

    let cell_yuan = Cell::from(Span::raw(format!("{:.2}", balance_yuan)))
        .style(Style::default().fg(Color::Red));
    let cell_desc_yuan = Cell::from(Span::raw("YUAN"))
        .style(Style::default().fg(Color::Red));

    let cell_eur = Cell::from(Span::raw(format!("{:.2}", balance_eur)))
        .style(Style::default().fg(Color::Blue));
    let cell_desc_eur = Cell::from(Span::raw("EUR"))
        .style(Style::default().fg(Color::Blue));

    let cell_usd = Cell::from(Span::raw(format!("{:.2}", balance_usd)))
        .style(Style::default().fg(Color::Green));
    let cell_desc_usd = Cell::from(Span::raw("USD"))
        .style(Style::default().fg(Color::Green));

    balance_rows.push(Row::new(vec![cell_desc_yen, cell_yen]));
    balance_rows.push(Row::new(vec![cell_desc_yuan, cell_yuan]));
    balance_rows.push(Row::new(vec![cell_desc_eur, cell_eur]));
    balance_rows.push(Row::new(vec![cell_desc_usd, cell_usd]));
    balance_rows
}

pub fn get_trade_table(trades: Vec<Trade>) -> Table<'static> {
    let trade_detail = Table::new(get_trade_rows(trades))
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
            Constraint::Percentage(18),
            Constraint::Percentage(17),
            Constraint::Percentage(18),
            Constraint::Percentage(21)
        ]);
    trade_detail
}

fn get_trade_rows(trade_vec: Vec<Trade>) -> Vec<Row<'static>> {
    let trade_rows: Vec<Row> = trade_vec
        .iter()
        .map(|t| {
            Row::new(vec![
                Cell::from(Span::raw(get_operation_string(t.operation))),
                Cell::from(Span::raw(t.market.to_string())),
                get_cell_good_kind(t.good_kind.clone()),
                Cell::from(Span::raw(t.quantity.to_string())),
                Cell::from(Span::raw(t.timestamp.format("%H:%M:%S").to_string())),
                Cell::from(Span::raw(format!("{:.2}", t.price))),
            ])
        })
        .collect();
    trade_rows
}

fn get_cell_good_kind(gk: GoodKind) -> Cell<'static> {
    let color = match gk {
        GoodKind::EUR => { Color::Blue }
        GoodKind::YEN => { Color::Yellow }
        GoodKind::USD => { Color::Green }
        GoodKind::YUAN => { Color::Red }
    };
    Cell::from(Span::raw(gk.to_string())).style(Style::default().fg(color))
}

fn get_operation_string(trade_type: TradeType) -> String {
    match trade_type {
        TradeType::Buy => String::from("BUY"),
        TradeType::Sell => String::from("SELL")
    }
}

pub fn get_copyright() -> Paragraph<'static> {
    Paragraph::new("SOL Market - all rights reserved")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Copyright")
                .border_type(BorderType::Plain),
        )
}


pub fn get_stats_paragraph(text: &str) -> Paragraph {
    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::White)).title("Stats"))
        .alignment(Alignment::Center)
}
