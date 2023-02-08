use crate::{domain::trading_event::TradingEvent, Receiver};
use std::{
    fs::File,
    io::{Error, Read, Result},
    ops::Div,
    path::PathBuf,
    thread,
};

pub(crate) enum ThreadResult {
    TimedOut(u64),
    Completed(Result<Option<TradingEvent>>),
}

impl Receiver {
    /// Receive a Trading Event
    pub fn receive(&mut self) -> Result<Option<TradingEvent>> {
        if self.read_handle_opt.is_none() {
            let new_handle = self.spawn_new_read();
            self.read_handle_opt = Some(new_handle);
        } else if self
            .read_handle_opt
            .as_ref()
            .map(|handle| handle.is_finished())
            .unwrap_or(false)
        {
            let thread_execution_result = self.read_handle_opt.take().unwrap().join();
            return thread_execution_result.unwrap_or(Err(Error::new(
                std::io::ErrorKind::Interrupted,
                "Something went wrong with the thread",
            )));
        }

        let current_timer_number = self.spawn_timer();

        let mut should_receive = true;
        while should_receive {
            let thread_result = self.receiver.recv().map_err(|_| {
                Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Something went wrokg receiving",
                )
            })?;
            should_receive = match thread_result {
                ThreadResult::TimedOut(call_number) => call_number < current_timer_number,
                ThreadResult::Completed(result) => return result,
            }
        }

        Ok(None)
    }

    fn spawn_new_read(&self) -> thread::JoinHandle<Result<Option<TradingEvent>>> {
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

    fn spawn_timer(&mut self) -> u64 {
        let current_timer_number = self.timer_number;
        self.timer_number += 1;
        let timer_duration = self.refresh_duration.div(2);
        let sender = self.sender.clone();
        thread::spawn(move || {
            thread::sleep(timer_duration);
            let _ = sender.send(ThreadResult::TimedOut(current_timer_number));
        });
        current_timer_number
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
