use super::utils::draw_background;
use std::io;
use tui::{
    backend::Backend,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
    Terminal,
};

pub(crate) fn draw<B: Backend>(terminal: &mut Terminal<B>, error: io::Error) {
    let message: String;
    if let std::io::ErrorKind::NotFound = error.kind() {
        message = "Did not find the pipe 🤔\r\nMake sure you started a trader".to_string();
    } else {
        message = "Unknown error: ".to_string() + &error.to_string();
    }
    terminal
        .draw(|f| {
            draw_background(f);
            let size = f.size();
            let p = Paragraph::new(message)
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
