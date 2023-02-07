use domain::trading_event::TradingEvent;
use nix::{sys::stat, unistd};
use receive::ThreadResult;
use std::{
    fs,
    io::Result,
    path::PathBuf,
    sync::mpsc::{self, channel},
    thread::JoinHandle,
};

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
pub struct Sender {
    pub(crate) pipe_path: PathBuf,
}

pub struct Receiver {
    pub(crate) pipe_path: PathBuf,
    pub(crate) sender: mpsc::Sender<ThreadResult>,
    timer_number: u64,
    pub(crate) receiver: mpsc::Receiver<ThreadResult>,
    pub(crate) read_handle_opt: Option<JoinHandle<Result<Option<TradingEvent>>>>,
}

impl Sender {
    /// Create a new sender (and the relative FIFO)
    pub fn new() -> Self {
        let pb = PathBuf::from(PIPE_PATH);
        try_remove_pipe(&pb);

        // Create the named pipe
        unistd::mkfifo(&pb, stat::Mode::S_IWUSR | stat::Mode::S_IRUSR)
            .expect("Can create the pipe.");

        Sender { pipe_path: pb }
    }
}

impl Receiver {
    /// Create a new sender (and the relative FIFO)
    pub fn new() -> Self {
        let pb = PathBuf::from(PIPE_PATH);

        // Create channel for informing about IO
        let (sender, receiver) = channel::<ThreadResult>();

        Receiver {
            pipe_path: pb,
            sender,
            receiver,
            read_handle_opt: None,
            timer_number: 0,
        }
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

impl Default for Sender {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        let pb = PathBuf::from(PIPE_PATH);
        try_remove_pipe(&pb);
    }
}

impl Default for Receiver {
    fn default() -> Self {
        Self::new()
    }
}
