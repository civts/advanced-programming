use unitn_market_2022::good::good::Good;

use crate::lib::{
    domain::good_lock_meta::GoodLockMeta,
    market::price_strategies::{quantity::QuantityPrice, stocastic::StocasticPrice},
};
use std::{cell::RefCell, collections::HashMap};

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
}

impl MarketMeta {
    pub fn new(goods: Vec<Good>, path: Option<&str>) -> Self {
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            current_day: 1,
            file_path: path.map(String::from),
            stocastic_price: RefCell::new(StocasticPrice::new()),
            quantity_price: QuantityPrice::new(goods),
        }
    }

    pub fn num_of_locked_sells(&self) -> u32 {
        self.locked_sells.len() as u32
    }

    pub fn num_of_buy_locks(&self) -> u32 {
        self.locked_buys.len() as u32
    }
}
