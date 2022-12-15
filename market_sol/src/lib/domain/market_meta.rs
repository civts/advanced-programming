use crate::lib::domain::good_lock_meta::GoodLockMeta;
use crate::lib::market::price_strategies::other_markets::OtherMarketsPrice;
use crate::lib::market::price_strategies::quantity::QuantityPrice;
use crate::lib::market::price_strategies::stocastic::StocasticPrice;
use crate::lib::market::sol_market::TOKEN_DURATION;
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
            other_markets: OtherMarketsPrice::new(),
        }
    }

    /// Return the number of buy locks that are not expired
    pub fn num_of_locked_sells(&self) -> u32 {
        let mut not_expired_locks = 0u32;
        for (_, glm) in self.locked_sells.iter() {
            let days_since = self.current_day - glm.created_on;
            if days_since <= TOKEN_DURATION {
                not_expired_locks += 1
            }
        }
        not_expired_locks
    }

    /// Return the number of buy locks that are not expired
    pub fn num_of_buy_locks(&self) -> u32 {
        let mut not_expired_locks = 0u32;
        for (_, glm) in self.locked_buys.iter() {
            let days_since = self.current_day - glm.created_on;
            if days_since <= TOKEN_DURATION {
                not_expired_locks += 1
            }
        }
        not_expired_locks
    }
}
