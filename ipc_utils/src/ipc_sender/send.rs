use crate::{domain::trading_event::TradingEvent, IPCSender};
use std::{fs, io::Error};

impl IPCSender {
    /// Send a message to the other process
    pub fn send(&self, event: TradingEvent) -> Result<(), Error> {
        let message = serde_json::to_string(&event)?;
        let message_bytes = message.as_bytes();
        let message_len = message_bytes.len();
        let message_len: u16 = message_len
            .try_into()
            .expect("The size of the message fits in a u16");

        let high_byte: u8 = (message_len >> 8) as u8;
        let low_byte: u8 = (message_len & 0xff) as u8;

        fs::write(&self.pipe_path, [high_byte, low_byte])?;
        fs::write(&self.pipe_path, message_bytes)?;

        Ok(())
    }
}
