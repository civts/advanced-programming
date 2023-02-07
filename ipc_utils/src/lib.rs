use nix::{sys::stat, unistd};
use std::{fs, path::PathBuf};

pub mod domain;
pub mod receive;
pub mod send;

pub(crate) const PIPE_PATH: &str = "/tmp/sol_fifo_pipe";

/// Allows to send and receive info amongst processes.
/// As of now, the operations are **blocking**, meaning that the sender thread
/// will pause when calling `send` until a process consumes the message and,
/// simmetrically, the consumer thread will pause upon calling `read` until a
/// new message becomes available.
///
/// # Examples
/// ## Sender
/// 1. Instanciate IpcUtils
/// 1. Call `send` on the instance each time you want to notify about an event
/// ## Receiver
/// 1. Call `IpcUtils::receive()` to get the next event.
pub struct IpcUtils {
    pub pipe_path: PathBuf,
}

impl IpcUtils {
    /// Create a new sender (and the relative FIFO)
    pub fn new() -> Self {
        let pb = PathBuf::from(PIPE_PATH);
        // If the "file" already exists, remove it
        if fs::metadata(&pb).is_ok() {
            println!("Found a preexisting IPC pipe, removing it.\r");
            let _ = fs::remove_file(&pb);
        }

        // Create the named pipe
        unistd::mkfifo(&pb, stat::Mode::S_IWUSR | stat::Mode::S_IRUSR)
            .expect("Should have been able to create the pipe.");
        IpcUtils { pipe_path: pb }
    }
}

impl Default for IpcUtils {
    fn default() -> Self {
        Self::new()
    }
}
