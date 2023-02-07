use std::{
    fs::File,
    io::{Error, Read, Result},
    path::PathBuf,
};

use crate::{domain::trading_event::TradingEvent, IpcUtils, PIPE_PATH};

impl IpcUtils {
    /// Receive a Trading Event
    pub fn receive() -> Result<Option<TradingEvent>> {
        // fs::read;
        let mut pipefd = open_pipe_read()?;

        let len = get_next_message_length(&mut pipefd);

        if len == 0 {
            // No message, for now
            return Ok(None);
        }

        let len: usize = len.try_into().expect("Len fits in usize");
        let message = get_next_message(pipefd, len)?;

        deserialize_message(message).map(Some)
    }
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

fn open_pipe_read() -> std::result::Result<File, Error> {
    File::open(PathBuf::from(PIPE_PATH))
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
