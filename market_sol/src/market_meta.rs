use crate::good_lock_meta::GoodLockMeta;
use std::collections::HashMap;
use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Debug)]
pub struct MarketMeta {
    // Key is token
    pub locked_buys: HashMap<String, GoodLockMeta>,
    // Key is token
    pub locked_sells: HashMap<String, GoodLockMeta>,
    pub min_bid: HashMap<GoodKind, f32>,
    pub current_day: u32,
    pub num_of_sell_locks: u32,
    pub num_of_buy_locks: u32,
}

impl MarketMeta {
    pub fn new() -> Self {
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            min_bid: Default::default(), // todo: come up with min bid for each goods
            current_day: 0u32,
            num_of_sell_locks: 0u32,
            num_of_buy_locks: 0u32,
        }
    }
}
