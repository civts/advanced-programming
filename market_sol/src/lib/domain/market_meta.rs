use crate::lib::domain::good_lock_meta::GoodLockMeta;
use crate::lib::market::sol_market::TOKEN_DURATION;
use std::{collections::HashMap, path::Path};

#[derive(Debug)]
pub struct MarketMeta {
    // Key is token
    pub locked_buys: HashMap<String, GoodLockMeta>,
    // Key is token
    pub locked_sells: HashMap<String, GoodLockMeta>,
    pub current_day: u32,
    pub file_path: Option<String>,
}

impl MarketMeta {
    pub fn new() -> Self {
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            current_day: 1,
            file_path: None,
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

    pub fn new_with_file(f: &Path) -> Self {
        let file_str = f.to_str().unwrap();
        Self {
            locked_buys: Default::default(),
            locked_sells: Default::default(),
            current_day: 1,
            file_path: Some(String::from(file_str)),
        }
    }
}
