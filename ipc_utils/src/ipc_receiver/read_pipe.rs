use crate::{
    domain::trading_event::TradingEvent, ipc_receiver::receive::ThreadResult,
    ipc_receiver::IPCReceiver,
};
use std::{
    fs::File,
    io::{Error, Read, Result},
    path::PathBuf,
    thread,
};

impl IPCReceiver {
    /// Spawns a new thread in which we can run the blocking calls for reading an event
    /// from the pipe and returns its handle
    pub(crate) fn spawn_new_read(&self) -> thread::JoinHandle<Result<Option<TradingEvent>>> {
        let sender = self.sender.clone();
        let path = self.pipe_path.clone();
        thread::spawn(move || {
            let f = open_pipe_read(path).and_then(|mut pipefd| {
                let len = get_next_message_length(&mut pipefd);

                if len == 0 {
                    // No message, for now
                    return Ok(None);
                }

                let len: usize = len.try_into().expect("Len fits in usize");
                let message_res = get_next_message(pipefd, len);

                message_res.and_then(deserialize_message).map(Some)
            });
            let _ = sender.send(ThreadResult::Completed(f));
            Ok(None)
        })
    }
}

fn open_pipe_read(pipe_path: PathBuf) -> std::result::Result<File, Error> {
    File::open(pipe_path)
}

fn deserialize_message(message: String) -> Result<TradingEvent> {
    let event = serde_json::from_str(message.as_str())?;

    Ok(event)
}

fn get_next_message(mut pipefd: File, len: usize) -> Result<String> {
    let mut total_read_bytes: usize = 0;
    let mut message_buf: Vec<u8> = vec![0; len];
    while total_read_bytes < len {
        let read_bytes = pipefd.read(&mut message_buf)?;
        total_read_bytes += read_bytes;
    }

    let message = String::from_utf8(message_buf)
        .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    Ok(message)
}

fn get_next_message_length(pipefd: &mut File) -> u8 {
    let mut len_buf: [u8; 1] = [0; 1];
    let read_bytes = pipefd.read(&mut len_buf).expect("Can read the pipe");

    if read_bytes == 0 {
        0
    } else {
        let len = *len_buf
            .first()
            .expect("The programmer can create a non-empty buffer 👾");
        len
    }
}
