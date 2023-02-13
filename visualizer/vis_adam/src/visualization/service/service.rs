use std::time::Duration;

use chrono::Utc;
use ipc_utils::IPCReceiver;
use ipc_utils::trader_state::TraderState;
use ipc_utils::trading_event_details::{TradeOperation, TradingEventDetails};
use unitn_market_2022::good::good_kind::GoodKind;

use crate::visualization::repository::repository::{Balance, find_latest_balance, Lock, save_balance, save_lock, save_trade, Trade};

pub struct Service {
    receiver: IPCReceiver,
    pub failed_locks: usize,
    pub failed_trades: usize,
}

impl Service {
    pub fn new() -> Service {
        Service {
            receiver: IPCReceiver::new(Duration::from_millis(100)),
            failed_locks: 0,
            failed_trades: 0,
        }
    }

    pub fn get_profit(&self, gk: GoodKind) -> f32 {
        let current_balance = find_latest_balance(gk);
        let starting_balance = 10000.00;
        return (current_balance - starting_balance) / starting_balance * 100.00;
    }

    pub fn receive(&mut self) {
        let event = self.receiver.receive();

        match event {
            Ok(trade) => {
                match trade {
                    None => {}
                    Some(trade_event) => {
                        let trader_state = trade_event.trader_state;
                        Self::save_current_balances(self, trader_state);
                        let market = trade_event.market_name;
                        let details = trade_event.details;
                        Self::save_based_on_operation(self, market, details);
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

    fn save_current_balances(&self, trader_state: TraderState) {
        let map = trader_state.cash;

        for (kind, value) in map {
            let balance = Balance { value };
            save_balance(balance, kind);
        }
    }

    fn save_based_on_operation(&mut self, market: String, details: TradingEventDetails) {
        match details.operation {
            TradeOperation::AskedLock => {
                if details.successful {
                    let lock = Lock { quantity: details.quantity as i32, good_kind: details.good_kind, market, price: details.price, operation: details.trade_type, timestamp: Utc::now() };
                    save_lock(lock)
                } else {
                    self.failed_locks += 1
                }
            }
            TradeOperation::TradeFinalized => {
                let trade = Trade { quantity: details.quantity as usize, good_kind: details.good_kind, market, price: details.price, operation: details.trade_type, timestamp: Utc::now() };
                if details.successful {
                    save_trade(trade);
                } else {
                    self.failed_trades += 1;
                }
            }
        };
    }
}

mod test {

    #[test]
    fn should_return_profit() {
        // when

    }

}