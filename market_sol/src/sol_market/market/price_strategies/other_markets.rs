use std::{collections::HashMap, fmt::Debug};
use unitn_market_2022::good::good_kind::GoodKind;

/// This strategy returns the latest price used by other markets.
/// An improvement could be tracking which are the market the traders trade the most with and follow them more closely
#[derive(Debug)]
pub(crate) struct OtherMarketsPrice {
    ///Exchange rate EUR-goodkind of the other markets
    exchange_rates: HashMap<GoodKind, f32>,
}

impl OtherMarketsPrice {
    pub(crate) fn new() -> Self {
        OtherMarketsPrice {
            exchange_rates: HashMap::new(),
        }
    }

    pub(crate) fn update(&mut self, good_kind: &GoodKind, exchange_rate: f32) {
        self.exchange_rates.insert(*good_kind, exchange_rate);
    }

    pub(crate) fn get_exchange_rate(&self, good_kind: &GoodKind) -> f32 {
        *self
            .exchange_rates
            .get(good_kind)
            .unwrap_or(&good_kind.get_default_exchange_rate())
    }
}
