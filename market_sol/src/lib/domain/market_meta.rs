use super::strategy_name::{StrategyName, ALL_STRATEGY_NAMES};
use crate::lib::domain::good_lock_meta::GoodLockMeta;
use crate::lib::market::price_strategies::other_markets::OtherMarketsPrice;
use crate::lib::market::price_strategies::quantity::QuantityPrice;
use crate::lib::market::price_strategies::stocastic::StocasticPrice;
use crate::lib::market::sol_market::TOKEN_DURATION;
use rand::Rng;
use std::{cell::RefCell, collections::HashMap};
use unitn_market_2022::good::good::Good;

#[derive(Debug)]
pub(crate) struct MarketMeta {
    // Key is token
    pub locked_buys: HashMap<String, GoodLockMeta>,
    // Key is token
    pub locked_sells: HashMap<String, GoodLockMeta>,
    pub current_day: u32,
    pub file_path: Option<String>,
    pub stocastic_price: RefCell<StocasticPrice>,
    pub quantity_price: QuantityPrice,
    pub other_markets: OtherMarketsPrice,
    /// The priority of each pricing strategy relative to the others
    pub weights: HashMap<StrategyName, f32>,
}

impl MarketMeta {
    pub fn new(goods: Vec<Good>, path: Option<&str>, weights: HashMap<StrategyName, f32>) -> Self {
        let mut r = rand::thread_rng();
        let range = 0.0..=1.0;
        let mut all_weights: HashMap<StrategyName, f32> = weights;
        for strategy in ALL_STRATEGY_NAMES {
            all_weights
                .entry(strategy)
                .or_insert_with(|| r.gen_range(range.clone()));
        }
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            current_day: 1,
            file_path: path.map(String::from),
            stocastic_price: RefCell::new(StocasticPrice::new()),
            quantity_price: QuantityPrice::new(goods),
            other_markets: OtherMarketsPrice::new(),
            weights: all_weights,
        }
    }

    /// Return the number of buy locks that are not expired
    pub fn num_of_locked_sells(&self, trader_name: &str) -> u32 {
        let locks_of_this_trader = self
            .locked_sells
            .values()
            .filter(|lock| lock.trader_name == trader_name);
        let not_expired_locks = locks_of_this_trader.filter(|lock| {
            let days_since = self.current_day - lock.created_on;
            days_since <= TOKEN_DURATION
        });
        not_expired_locks.count().try_into().unwrap()
    }

    /// Return the number of buy locks that are not expired
    pub fn num_of_buy_locks(&self, trader_name: &str) -> u32 {
        let locks_of_this_trader = self
            .locked_buys
            .values()
            .filter(|lock| lock.trader_name == trader_name);
        let not_expired_locks = locks_of_this_trader.filter(|lock| {
            let days_since = self.current_day - lock.created_on;
            days_since <= TOKEN_DURATION
        });
        not_expired_locks.count().try_into().unwrap()
    }
}
