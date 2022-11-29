use std::collections::HashMap;
use unitn_market_2022::good::good_kind::GoodKind;
use crate::good_lock_meta::GoodLockMeta;

#[derive(Debug)]
pub struct MarketMeta {
    // Key is token
    pub locked_buys: HashMap<String, GoodLockMeta>,
    // Key is token
    pub locked_sells: HashMap<String, GoodLockMeta>,
    pub min_bid: HashMap<GoodKind, f32>,
    pub current_day: u32
}

impl MarketMeta {
    pub fn new() -> Self {
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            min_bid: Default::default(), // todo: come up with min bid for each goods
            current_day: 0
        }
    }
}