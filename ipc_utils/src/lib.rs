use domain::trading_event::TradingEvent;
use ipc_receiver::ThreadResult;
use nix::{sys::stat, unistd};
use std::{
    fs,
    io::Result,
    path::PathBuf,
    sync::mpsc::{self, channel},
    thread::JoinHandle,
    time::Duration,
};

pub mod domain;
pub mod ipc_receiver;
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
pub struct IPCSender {
    pub(crate) pipe_path: PathBuf,
}

pub struct IPCReceiver {
    pub(crate) pipe_path: PathBuf,
    pub(crate) sender: mpsc::Sender<ThreadResult>,
    timer_number: u64,
    refresh_duration: Duration,
    pub(crate) receiver: mpsc::Receiver<ThreadResult>,
    pub(crate) read_handle_opt: Option<JoinHandle<Result<Option<TradingEvent>>>>,
}

impl IPCSender {
    /// Create a new sender (and the relative FIFO)
    pub fn new() -> Self {
        let pb = PathBuf::from(PIPE_PATH);
        try_remove_pipe(&pb);

        // Create the named pipe
        unistd::mkfifo(&pb, stat::Mode::S_IWUSR | stat::Mode::S_IRUSR)
            .expect("Can create the pipe.");

        IPCSender { pipe_path: pb }
    }
}

impl IPCReceiver {
    /// Create a new sender (and the relative FIFO)
    pub fn new(refresh_duration: Duration) -> Self {
        let pb = PathBuf::from(PIPE_PATH);

        // Create channel for informing about IO
        let (sender, receiver) = channel::<ThreadResult>();

        IPCReceiver {
            pipe_path: pb,
            sender,
            receiver,
            refresh_duration,
            read_handle_opt: None,
            timer_number: 0,
        }
    }

    pub fn restart(&mut self) {
        // Create a new channel between internal threads
        let (sender, receiver) = channel::<ThreadResult>();
        self.sender = sender;
        self.receiver = receiver;

        // Let the other handle terminate on its own
        self.read_handle_opt = None;
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

impl Default for IPCReceiver {
    fn default() -> Self {
        Self::new(Duration::from_millis(500))
    }
}
