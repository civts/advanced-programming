use crate::{domain::trading_event::TradingEvent, IpcUtils};
use std::{fs, io::Error};

impl IpcUtils {
    /// Send a message to the other process
    pub fn send(&self, event: TradingEvent) -> Result<(), Error> {
        let message = serde_json::to_string(&event)?;
        let message_bytes = message.as_bytes();
        let message_len: u8 = message_bytes
            .len()
            .try_into()
            .expect("The size of the message fits in a u8");

        fs::write(&self.pipe_path, [message_len])?;
        fs::write(&self.pipe_path, message_bytes)?;

        Ok(())
    }
}
