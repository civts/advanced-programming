pub mod domain;
pub mod ipc_receiver;
pub mod ipc_sender;

pub use self::domain::*;
pub use self::ipc_receiver::IPCReceiver;
pub use self::ipc_sender::IPCSender;

/// Optionally, we could let the user specify this with a CLI argument
pub(crate) const PIPE_PATH: &str = "/tmp/sol_fifo_pipe";
