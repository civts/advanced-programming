use std::time::Duration;
use ipc_utils::IPCReceiver;
use ipc_utils::trading_event_details::TradeOperation;
use crate::vizualization::repository::repository::{Balance, save_balance, save_lock_if_successful, save_trade_if_successful};

pub struct Service {
    receiver: IPCReceiver,
}

impl Service {
    pub fn new() -> Service {
        Service {
            receiver: IPCReceiver::new(Duration::from_millis(100))
        }
    }

    pub fn receive(&mut self) {
        let event = self.receiver.receive();

        match event {
            Ok(trade) => {
                match trade {
                    None => {}
                    Some(trade_event) => {
                        let trader_state = trade_event.trader_state;

                        let map = trader_state.cash;

                        for (kind, value) in map {
                            let balance = Balance { value };
                            save_balance(balance, kind);
                        }

                        let market = trade_event.market_name;
                        let details = trade_event.details;

                        match details.operation {
                            TradeOperation::AskedLock => {
                                save_lock_if_successful(market, details.successful, details.trade_type, details.price, details.good_kind, details.quantity)
                            }
                            TradeOperation::TradeFinalized => {
                                save_trade_if_successful(market, details.trade_type, details.quantity, details.price, details.successful, details.good_kind)
                            }
                        };
                        ()
                    }
                };
            }
            Err(_) => {
                ()
            }
        };
        ()
    }
}