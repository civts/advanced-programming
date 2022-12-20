use super::trade_role::TradeRole;
use crate::lib::market::sol_market::{get_value_good, SOLMarket};
use std::collections::HashMap;
use unitn_market_2022::good::{good::Good, good_kind::GoodKind};

impl SOLMarket {
    /// Perform an internal trade if needed
    ///
    /// Example: An importer has a positive need and an exporter has a surplus
    pub(crate) fn internal_trade_if_needed(&mut self) {
        // Find good that need a refill and the one capable of refilling
        let mut max_need = 0f32;
        let mut max_ability = 0f32;
        let mut kind_need_refill: Option<GoodKind> = None;
        let mut kind_able_refill: Option<GoodKind> = None;
        for (kind, role) in self.internal_needs.iter() {
            match role {
                TradeRole::Importer { need } => {
                    let n = *need;
                    if n > max_need {
                        max_need = n;
                        kind_need_refill = Some(*kind);
                    }
                }
                TradeRole::Exporter { need } => {
                    let ability = if need.is_sign_negative() {
                        need.abs()
                    } else {
                        -need
                    };
                    // Market ability in case the good is locked and need has not been updated yet
                    let market_ability =
                        get_value_good(kind, self.goods.get(kind).unwrap().get_qty());
                    if ability > max_ability && market_ability > max_ability {
                        max_ability = market_ability.min(ability);
                        kind_able_refill = Some(*kind);
                    }
                }
            }
        }

        // Refill if possible/needed
        if let Some(scr_kind) = kind_able_refill {
            if let Some(dst_kind) = kind_need_refill {
                let value = (max_ability.min(10_000f32)).min(max_need);
                self.internal_trade(scr_kind, dst_kind, value);
            }
        }
    }

    /// Perform an internal trade
    fn internal_trade(&mut self, src_kind: GoodKind, dst_kind: GoodKind, value_in_eur: f32) {
        // Decrease good qty from source
        let src_qty = value_in_eur * src_kind.get_default_exchange_rate();
        self.goods
            .get_mut(&src_kind)
            .unwrap()
            .split(src_qty)
            .unwrap();

        // Increase need to source
        self.internal_needs
            .get_mut(&src_kind)
            .unwrap()
            .increase_need(value_in_eur);

        // Increase good qty to destination (+25% of default exchange rate)
        let dst_qty = value_in_eur * dst_kind.get_default_exchange_rate() * 1.25;
        self.goods
            .get_mut(&dst_kind)
            .unwrap()
            .merge(Good::new(dst_kind, dst_qty))
            .unwrap();

        // Decrease need to destination
        self.internal_needs
            .get_mut(&dst_kind)
            .unwrap()
            .decrease_need(value_in_eur);
    }

    /// Set internal needs according to the EUR value of a certain good and the total value of the market (in EUR)
    ///
    /// Example:
    ///
    /// Market has:
    ///     - 100 EUR  (value: 100€)
    ///     - 100 USD  (value: 96.55€)
    ///     - 100 YEN  (value: 0.70€)
    ///     - 100 YUAN (value: 13.59€)
    ///
    /// Total Value: 210.84€
    /// Ideal Value of each goods: (210.84 / 4) = 52.71€
    ///     - 100 EUR  (value: 100€)    -> need: (52.71 - 100)      = -47.29    -> Exporter
    ///     - 100 USD  (value: 96.55€)  -> need: (52.71 - 96.55)    = -43.84    -> Exporter
    ///     - 100 YEN  (value: 0.70€)   -> need: (52.71 - 0.70)     = 52.01     -> Importer
    ///     - 100 YUAN (value: 13.59€)  -> need: (52.71 - 13.59)    = 39.12     -> Importer
    pub(crate) fn set_internal_needs(goods_vec: Vec<Good>) -> HashMap<GoodKind, TradeRole> {
        let total_value_market = goods_vec.iter().fold(0f32, |acc, g| {
            acc + get_value_good(&g.get_kind(), g.get_qty())
        });
        let ideal_value_per_good = total_value_market / goods_vec.len() as f32;

        let mut internal_needs: HashMap<GoodKind, TradeRole> = HashMap::new();
        for g in goods_vec.iter() {
            let need = ideal_value_per_good - get_value_good(&g.get_kind(), g.get_qty());
            // Set goods with needs as importers
            if need > 0f32 {
                internal_needs.insert(g.get_kind(), TradeRole::Importer { need });
            }
            // Set goods with negative needs (surplus) as Exporters
            else {
                internal_needs.insert(g.get_kind(), TradeRole::Exporter { need });
            }
        }
        internal_needs
    }
}
