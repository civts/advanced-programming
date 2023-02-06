/*
  TODOs
   1. Get a raw terminal with crosseterm ✅
   1. Let the user quit with q ✅
   1. Let the user quit with ctrl+c ✅
   1. Loop displaying the data as it comes
   1. Add a graphical help menu
   1. Let the user change views
*/

use crate::{
    cleanup::CleanUp,
    constants::{MARGIN_HORIZONTAL, MARGIN_VERTICAL, REFRESH_RATE_MILLISECONDS},
};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{self, ClearType},
    QueueableCommand,
};
use std::{io::stdout, time::Duration};

mod cleanup;
mod constants;

fn main() {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode().expect("Could not turn on Raw mode");
    let mut stdout = stdout();

    let mut rows: u16 = 40;
    let mut columns: u16 = 80;

    loop {
        update_terminal_size(&mut rows, &mut columns);

        // Check for events in the terminal
        let is_a_new_event_available =
            event::poll(Duration::from_millis(REFRESH_RATE_MILLISECONDS))
                .expect("Could not poll for input");
        if is_a_new_event_available {
            match event::read().expect("Failed to get next event") {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: event::KeyModifiers::NONE,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: event::KeyModifiers::CONTROL,
                    ..
                }) => {
                    println!("Farewell 👋\r");
                    break;
                }
                _ => {}
            }
        }

        // Place the cursor on the last saved position
        // stdout
        //     .queue(cursor::RestorePosition)
        //     .expect("Should have been able to restore the cursor position");
        // Place the cursor back on the top left
        stdout
            .queue(cursor::MoveTo(0, 0))
            .expect("Should have been able to put the cursor on the upper left");
        stdout
            .queue(terminal::Clear(ClearType::All))
            .expect("Could not queue clear command");
        println!("This is the howie trading visualiser 👓\r");
        println!("Terminal is {:?} by {:?}\r", rows, columns);
    }
}

// Would have used crossterm::event::Event::Resize, but that did not work
fn update_terminal_size(rows: &mut u16, columns: &mut u16) {
    let (new_columns, new_rows) = terminal::size().expect("Could not get the terminal size");
    *rows = new_rows - MARGIN_HORIZONTAL;
    *columns = new_columns - MARGIN_VERTICAL;
}
