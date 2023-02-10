use ipc_utils::{
    trading_event::TradingEvent,
    trading_event_details::{TradeOperation, TradeType},
};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Default)]
pub(crate) struct Stats {
    /// How many trades we performed with this given market
    pub(crate) trades_with_market: HashMap<String, u64>,

    /// The capital our trader started with
    pub(crate) starting_capital: f32,

    pub(crate) total_locks: HashMap<TradeType, u64>,

    pub(crate) total_trades: HashMap<TradeType, u64>,

    pub(crate) recent_trades: VecDeque<TradingEvent>,

    /// A day and how much money the trader had on that day
    pub(crate) profit_history: VecDeque<(u64, f64)>,

    day: u64,
}

impl Stats {
    pub(crate) fn update(&mut self, trading_event: TradingEvent) {
        let market_name = trading_event.market_name.clone();
        let new_counter = self.trades_with_market.get(&market_name).unwrap_or(&0) + 1;
        self.trades_with_market.insert(market_name, new_counter);

        if self.starting_capital == 0.0 {
            self.starting_capital = get_capital_for_day(&trading_event);
        }

        if trading_event.details.successful {
            let trade_type = trading_event.details.trade_type;
            match trading_event.details.operation {
                TradeOperation::AskedLock => {
                    let count = self.total_locks.get(&trade_type).unwrap_or(&0);
                    self.total_locks.insert(trade_type, count + 1);
                    //self.recent_trades.push_back(trading_event.clone());
                }
                TradeOperation::TradeFinalized => {
                    let count = self.total_trades.get(&trade_type).unwrap_or(&0);
                    self.total_trades.insert(trade_type, count + 1);
                    self.recent_trades.push_back(trading_event.clone());
                }
            }
        }

        self.profit_history
            .push_back((self.day, get_capital_for_day(&trading_event).into()));

        self.day += 1;
    }
}

fn get_capital_for_day(trading_event: &TradingEvent) -> f32 {
    trading_event
        .trader_state
        .cash
        .iter()
        .fold(0.0, |acc, (gk, amount)| {
            acc + amount / gk.get_default_exchange_rate()
        })
}
