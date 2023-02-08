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
        let previous_read_already_finished = self
            .read_handle_opt
            .as_ref()
            .map(|handle| handle.is_finished())
            .unwrap_or(false);

        if previous_read_already_finished {
            // get and return that envent
            let thread_execution_result = self.read_handle_opt.take().unwrap().join();
            thread_execution_result.unwrap_or(Err(Error::new(
                std::io::ErrorKind::Interrupted,
                "Something went wrong with the thread",
            )))
        } else {
            // Ensure one read operation is running
            if self.read_handle_opt.is_none() {
                let new_handle = self.spawn_new_read();
                self.read_handle_opt = Some(new_handle);
            }

            // Create a timer
            let current_timer_number = self.spawn_timer();

            // Receive from either the timer or the read thread.
            // Since we could also get timeouts from previous timers,
            // we loop
            let mut thread_result: ThreadResult = ThreadResult::Completed(Ok(None));
            let mut should_receive_again = true;
            while should_receive_again {
                thread_result = self.receiver.recv().map_err(|_| {
                    Error::new(
                        std::io::ErrorKind::Interrupted,
                        "Something went wrong receiving",
                    )
                })?;

                if let ThreadResult::TimedOut(call_number) = thread_result {
                    should_receive_again = call_number < current_timer_number
                } else {
                    should_receive_again = false;
                }
            }

            match thread_result {
                ThreadResult::TimedOut(_) => Ok(None),
                ThreadResult::Completed(result) => result,
            }
        }
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
