use crate::PIPE_PATH;
use nix::{sys::stat, unistd};
use std::{fs, path::PathBuf};

mod send;

/// Allows to send info amongst processes.
/// The operations is **blocking**, meaning that the sender thread
/// will pause when calling `send` until a process consumes the message.
///
/// # Examples
/// ```rust,no_run
/// let event: TradingEvent = [...];
/// let sender = IPCSender::new();
/// sender.send(event).unwrap();
/// ```
///
pub struct IPCSender {
    pub(crate) pipe_path: PathBuf,
}

impl IPCSender {
    /// Creates a new sender (and the relative named FIFO pipe for IPC)
    pub fn new() -> Self {
        let pb = PathBuf::from(PIPE_PATH);
        try_remove_pipe(&pb);

        // Create the named pipe
        unistd::mkfifo(&pb, stat::Mode::S_IWUSR | stat::Mode::S_IRUSR)
            .expect("Can create the pipe.");

        IPCSender { pipe_path: pb }
    }
}

/// If a named pipe with the same name already exists, it tries to remove it
///
/// # Panics
/// If a named pipe with the same name exists and it can not be removed
fn try_remove_pipe(pb: &PathBuf) {
    // If the "file" already exists, remove it
    if fs::metadata(pb).is_ok() {
        fs::remove_file(pb).expect("Can remove the old pipe");
    }
}

impl Default for IPCSender {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IPCSender {
    fn drop(&mut self) {
        let pb = PathBuf::from(PIPE_PATH);
        try_remove_pipe(&pb);
    }
}
