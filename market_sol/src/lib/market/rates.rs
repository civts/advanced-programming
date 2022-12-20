use super::sol_market::{SOLMarket, MARKET_MARGIN};
use crate::lib::domain::strategy_name::StrategyName;
use unitn_market_2022::good::{consts::DEFAULT_GOOD_KIND, good_kind::GoodKind};

impl SOLMarket {
    /// Exchange rate (EUR/goodkind) for this good
    fn get_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        let stocastic_rate = self.get_stocastic_rate(good_kind);
        let quantity_rate = self.get_quantity_rate(good_kind);
        let other_markets_rate = self.get_other_rate(good_kind);
        //Compute the weighted average of the three
        let stochastic_weight: f32 = *self
            .meta
            .weights
            .get(&StrategyName::Stocastic)
            .unwrap_or(&1.0);
        let quantity_weight: f32 = *self
            .meta
            .weights
            .get(&StrategyName::Quantity)
            .unwrap_or(&1.0);
        let others_weight: f32 = *self.meta.weights.get(&StrategyName::Others).unwrap_or(&1.0);
        let total_weight = stochastic_weight.abs() + quantity_weight.abs() + others_weight.abs();
        assert!(total_weight > 0.0);
        let weighted_sum = f32::max(0.0, stocastic_rate * stochastic_weight)
            + f32::max(0.0, quantity_rate * quantity_weight)
            + f32::max(0.0, other_markets_rate * others_weight);
        weighted_sum / total_weight
    }

    pub fn get_other_rate(&self, good_kind: GoodKind) -> f32 {
        self.meta.other_markets.get_exchange_rate(&good_kind)
    }

    pub fn get_quantity_rate(&self, good_kind: GoodKind) -> f32 {
        self.meta
            .quantity_price
            .get_exchange_rate(&good_kind, Vec::from_iter(self.goods.values().cloned()))
    }

    pub fn get_stocastic_rate(&self, good_kind: GoodKind) -> f32 {
        self.meta
            .stocastic_price
            .borrow_mut()
            .get_rate(&good_kind, self.meta.current_day)
    }

    /// Return the rate applied when the trader wants to BUY the good from this market
    /// The rate is EUR/goodkind
    pub(crate) fn get_good_buy_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        if good_kind == DEFAULT_GOOD_KIND {
            1.0
        } else {
            //we divide, since the rate is eur/kind and not kind/eur
            self.get_exchange_rate(good_kind)
        }
    }

    /// Return the rate applied when the trader wants to SELL the good to this market
    /// The rate is EUR/goodkind
    pub(crate) fn get_good_sell_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        if good_kind == DEFAULT_GOOD_KIND {
            1.0
        } else {
            self.get_exchange_rate(good_kind) / (1.0 + MARKET_MARGIN)
        }
    }
}
