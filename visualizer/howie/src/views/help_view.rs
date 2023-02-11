use tui::{
    backend::Backend,
    layout::Rect,
    style::Modifier,
    text::{Span, Spans},
    widgets::{Block, List, ListItem},
    Terminal,
};

use crate::constants::default_style;

pub(crate) fn draw_help_view<B: Backend>(terminal: &mut Terminal<B>) {
    let area = terminal.size().expect("Can get terminal size");

    terminal
        .draw(|frame| {
            //Draw the background on the whole screen
            frame.render_widget(Block::default().style(default_style()), area);

            frame.render_widget(
                List::new([
                    ListItem::new(Spans::from(vec![
                        Span::styled("This is the help menu of the ", default_style()),
                        Span::styled(
                            "Howie visualizer 🦀",
                            default_style().add_modifier(Modifier::BOLD),
                        ),
                    ])),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled("Keyboard controls:", default_style())),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "H / ?:                       Toggle this help menu",
                        default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "ESC / Q / CTRL+C:            Quit",
                        default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "P / SPACEBAR:                Pause/resume",
                        default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "+ / -:                       Pause/resume",
                        default_style(),
                    )),
                    ListItem::new(Span::from("")),
                    ListItem::new(Span::styled(
                        "V:                          Toggle trading volume chart",
                        default_style(),
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
