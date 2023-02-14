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
    use std::collections::HashMap;
    use ipc_utils::trader_state::TraderState;
    use ipc_utils::trading_event_details::TradeOperation::AskedLock;
    use ipc_utils::trading_event_details::TradeType::Buy;
    use ipc_utils::trading_event_details::{TradingEventDetails};
    use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};
    use crate::visualization::repository::repository::{Balance, clear_all, read_balance, read_locks, save_balance};
    use crate::visualization::service::service::Service;

    #[test]
    fn should_return_profit() {
        // given
        clear_all();
        save_balance(Balance { value: 10000.00 }, EUR);
        save_balance(Balance { value: 20000.00 }, EUR);
        // when
        let s = Service::new();
        // then
        let result = s.get_profit(EUR);
        assert_eq!(100.00, result)
    }

    #[test]
    fn should_save_balances_from_state() {
        // given
        clear_all();
        let mut cash = HashMap::new();
        cash.insert(YEN, 25.0);
        cash.insert(YUAN, 35.0);
        cash.insert(USD, 45.0);
        cash.insert(EUR, 55.0);

        // when
        let s = Service::new();
        s.save_current_balances(TraderState { cash, name: "test".to_string() });

        // then
        let yen = read_balance(YEN).unwrap().get(0).unwrap().value;
        assert_eq!(25.0, yen);
        let yuan = read_balance(YUAN).unwrap().get(0).unwrap().value;
        assert_eq!(35.0, yuan);
        let usd = read_balance(USD).unwrap().get(0).unwrap().value;
        assert_eq!(45.0, usd);
        let eur = read_balance(EUR).unwrap().get(0).unwrap().value;
        assert_eq!(55.0, eur);
    }

    #[test]
    fn should_save_based_on_operation() {
        // given
        clear_all();
        let given_lock = TradingEventDetails {
            trade_type: Buy,
            operation: AskedLock,
            good_kind: USD,
            price: 10.0,
            successful: true,
            quantity: 1.0,
        };

        // when
        let mut s = Service::new();
        s.save_based_on_operation("PSE".to_string(), given_lock);

        // then
        let result = read_locks().unwrap().get(0).unwrap().clone();
        assert_eq!(result.operation, Buy);
        assert_eq!(result.price, 10.0);
        assert_eq!(result.quantity, 1);
        assert_eq!(result.good_kind, USD);
    }
}