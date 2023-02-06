use crossterm::terminal;

/* This struct has a destructor that puts the terminal back in normal mode.
   This way, even if the main program panics, the terminal should be set back
   into normal mode.
*/
pub(crate) struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}
