use crate::{domain::trading_event::TradingEvent, ipc_receiver::IPCReceiver};
use std::{
    io::{Error, Result},
    ops::Div,
    thread,
};

pub(crate) enum ThreadResult {
    TimedOut(u64),
    Completed(Result<Option<TradingEvent>>),
}

impl IPCReceiver {
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
