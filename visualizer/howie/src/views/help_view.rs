use tui::{
    backend::Backend,
    layout::Rect,
    style::Modifier,
    text::{Span, Spans},
    widgets::{Block, List, ListItem},
    Terminal,
};

use crate::domain::app_theme::AppTheme;

pub(crate) fn draw_help_view<B: Backend>(terminal: &mut Terminal<B>, theme: &AppTheme) {
    let area = terminal.size().expect("Can get terminal size");

    terminal
        .draw(|frame| {
            //Draw the background on the whole screen
            frame.render_widget(Block::default().style(theme.default_style()), area);

            frame.render_widget(
                List::new([
                    ListItem::new(Spans::from(vec![
                        Span::styled("This is the help menu of the ", theme.default_style()),
                        Span::styled(
                            "Howie visualizer 🦀",
                            theme.default_style().add_modifier(Modifier::BOLD),
                        ),
                    ])),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled("Keyboard controls:", theme.default_style())),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "H / ?:                       Toggle this help menu",
                        theme.default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "ESC / Q / CTRL+C:            Quit",
                        theme.default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "P / SPACEBAR:                Pause/resume",
                        theme.default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "+ / -:                       Adjust screen refresh speed",
                        theme.default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "V:                           Toggle trading volume chart",
                        theme.default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "T:                           Change UI theme",
                        theme.default_style(),
                    )),
                    ListItem::new(Span::from("")),
                ]),
                Rect {
                    x: area.x + 2,
                    y: area.y + 2,
                    width: area.width - 2,
                    height: area.height - 2,
                },
            );
        })
        .expect("Can draw help screen");
}
