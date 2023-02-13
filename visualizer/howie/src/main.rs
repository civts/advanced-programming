use app::App;
use cleanup::CleanUp;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod cleanup;
mod constants;
mod domain;
mod views;

#[cfg(test)]
mod tests;

fn main() {
    enable_raw_mode().expect("Can enable raw mode");
    let _clean_up = CleanUp;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).expect("Can get a terminal");

    let mut app = App::new();
    app.run(terminal);

    disable_raw_mode().expect("Can disable raw mode");
}
