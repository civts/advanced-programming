use crate::{domain::trading_event::TradingEvent, PIPE_PATH};
use std::{
    io::Result,
    path::PathBuf,
    sync::mpsc::{self, channel},
    thread::JoinHandle,
    time::Duration,
};

use self::receive::ThreadResult;

mod read_pipe;
mod receive;

/// `IPCReceiver` allows a process to receive `TradingEvent`s from a sender.
/// The receive operation is blocking, and completes in at most
/// `self.refresh_duration` time.
pub struct IPCReceiver {
    /// Pipe where to read the events from
    pub(crate) pipe_path: PathBuf,

    // Sender and receiver used to communicate amongst the
    // internal threads (for timer and I/O)
    pub(crate) sender: mpsc::Sender<ThreadResult>,
    pub(crate) receiver: mpsc::Receiver<ThreadResult>,

    // Thread handle to check if the read from the pipe has completed
    pub(crate) read_handle_opt: Option<JoinHandle<Result<Option<TradingEvent>>>>,

    /// How many timers we have spawned until now
    timer_number: u64,

    /// If receive takes more than this duration, it will return no event
    refresh_duration: Duration,
}

impl IPCReceiver {
    /// Create a new sender (and the relative FIFO)
    /// # params
    /// `refresh_duration` is the maximum time the receive function
    /// will wait before returning "no event"
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

impl Default for IPCReceiver {
    fn default() -> Self {
        Self::new(Duration::from_millis(500))
    }
}
