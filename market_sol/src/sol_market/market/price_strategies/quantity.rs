use crate::sol_market::ALL_GOOD_KINDS;
use std::{collections::HashMap, fmt::Debug};
use unitn_market_2022::good::{good::Good, good_kind::GoodKind};

#[derive(Debug)]
pub(crate) struct QuantityPrice {
    initial_quantities: HashMap<GoodKind, f32>,
}

impl QuantityPrice {
    pub(crate) fn new(goods: Vec<Good>) -> Self {
        let mut initial_quantities = HashMap::with_capacity(goods.len());
        for good in goods {
            initial_quantities.insert(good.get_kind(), good.get_qty());
        }
        for g in ALL_GOOD_KINDS {
            if !initial_quantities.contains_key(&g) {
                panic!("Should have had key for {g}");
            }
        }
        QuantityPrice { initial_quantities }
    }

    /// Returns the exchange rate EUR/Good for the given good
    pub(crate) fn get_exchange_rate(&self, good_kind: &GoodKind, goods: Vec<Good>) -> f32 {
        let rate = good_kind.get_default_exchange_rate();
        let quantity_now = goods
            .iter()
            .find(|g| g.get_kind() == *good_kind)
            .unwrap()
            .get_qty();
        let initial_quantity = self.initial_quantities.get(good_kind).unwrap();
        let remaining_percentage = quantity_now.max(0.000001) / initial_quantity.max(0.000001);
        rate / remaining_percentage.abs().sqrt()
    }
}
