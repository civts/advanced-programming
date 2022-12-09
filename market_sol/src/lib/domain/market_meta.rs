use crate::lib::{
    domain::good_lock_meta::GoodLockMeta, market::price_strategies::stocastic::StocasticPrice,
};
use std::{cell::RefCell, collections::HashMap, path::Path};

#[derive(Debug)]
pub(crate) struct MarketMeta {
    // Key is token
    pub locked_buys: HashMap<String, GoodLockMeta>,
    // Key is token
    pub locked_sells: HashMap<String, GoodLockMeta>,
    pub current_day: u32,
    pub file_path: Option<String>,
    pub stocastic_price: RefCell<StocasticPrice>,
}

impl MarketMeta {
    pub fn new() -> Self {
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            current_day: 1,
            file_path: None,
            stocastic_price: RefCell::new(StocasticPrice::new()),
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
            current_day: 1,
            file_path: Some(String::from(file_str)),
            stocastic_price: RefCell::new(StocasticPrice::new()),
        }
    }
}
