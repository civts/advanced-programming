use crate::lib::domain::good_lock_meta::GoodLockMeta;
use std::{cell::RefCell, collections::HashMap, path::Path};
use unitn_market_2022::good::good_kind::GoodKind;

use super::price_state::PriceState;

#[derive(Debug)]
pub(crate) struct MarketMeta {
    // Key is token
    pub locked_buys: HashMap<String, GoodLockMeta>,
    // Key is token
    pub locked_sells: HashMap<String, GoodLockMeta>,
    pub min_bid: HashMap<GoodKind, f32>,
    pub current_day: u32,
    pub file_path: Option<String>,
    pub price_state: RefCell<PriceState>,
}

impl MarketMeta {
    pub fn new() -> Self {
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            min_bid: Default::default(), // todo: come up with min bid for each goods
            current_day: 1,
            file_path: None,
            price_state: RefCell::new(PriceState::new()),
        }
    }

    pub fn num_of_locked_sells(&self) -> u32 {
        self.locked_sells.len() as u32
    }

    pub fn num_of_buy_locks(&self) -> u32 {
        self.locked_buys.len() as u32
    }

    pub fn new_with_file(f: &Path) -> Self {
        let file_str = f.to_str().unwrap();
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            min_bid: Default::default(), // todo: come up with min bid for each goods
            current_day: 1,
            file_path: Some(String::from(file_str)),
            price_state: RefCell::new(PriceState::new()),
        }
    }
}
