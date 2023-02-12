use std::time::Duration;
use ipc_utils::IPCReceiver;
use ipc_utils::trader_state::TraderState;
use ipc_utils::trading_event_details::{TradeOperation, TradingEventDetails};
use unitn_market_2022::good::good_kind::GoodKind;
use crate::vizualization::repository::repository::{Balance, clear_all, find_latest_balance, Lock, read_locks, read_trades, save_balance, save_lock_if_successful, save_trade_if_successful, Trade};

pub struct Service {
    receiver: IPCReceiver,
}

impl Service {
    pub fn new() -> Service {
        Service {
            receiver: IPCReceiver::new(Duration::from_millis(100))
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

    fn save_based_on_operation(&self, market: String, details: TradingEventDetails) {
        match details.operation {
            TradeOperation::AskedLock => {
                save_lock_if_successful(market, details.successful, details.trade_type, details.price, details.good_kind, details.quantity)
            }
            TradeOperation::TradeFinalized => {
                save_trade_if_successful(market, details.trade_type, details.quantity, details.price, details.successful, details.good_kind)
            }
        };
    }

    pub fn get_all_trades(&self) -> Vec<Trade> {
        return read_trades().unwrap();
    }

    pub fn get_all_locks(&self) -> Vec<Lock> {
        return read_locks().unwrap();
    }

    pub fn get_current_balance(&self, gk: GoodKind) -> f32 {
        return find_latest_balance(gk);
    }

}