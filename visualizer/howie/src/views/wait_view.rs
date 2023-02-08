use super::utils::draw_background;
use tui::{backend::Backend, layout::Alignment, widgets::Paragraph, Terminal};

pub(crate) fn draw<B: Backend>(terminal: &mut Terminal<B>) {
    terminal
        .draw(|f| {
            draw_background(f);
            f.render_widget(
                Paragraph::new(
                    "Waiting for the first event ⏳\r\n\n \
                                    Start a trader, then press the 'R' key",
                )
                .alignment(Alignment::Center),
                f.size(),
            );
        })
        .expect("Can draw first frame");
}
