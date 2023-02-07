use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unitn_market_2022::good::good_kind::GoodKind;

pub const ALL_GOOD_KINDS: [GoodKind; 4] =
    [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

/// The situation of a trader in a given moment
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TraderState {
    pub cash: HashMap<GoodKind, f32>,
    pub name: String,
}

impl TraderState {
    pub fn new(
        //Map between each `GoodKind` and how much of it the trader has
        cash: HashMap<GoodKind, f32>,
        trader_name: String,
    ) -> Self {
        ALL_GOOD_KINDS.iter().for_each(|g| {
            assert!(
                cash.contains_key(g),
                "The trader state should contain all goods, but is missing {:?}",
                g
            );
        });
        TraderState {
            cash,
            name: trader_name,
        }
    }
}
