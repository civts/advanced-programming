use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Debug, Clone)]
pub(crate) struct Arbitrage {
    pub(crate) buying_market_name: String,
    pub(crate) selling_market_name: String,
    pub(crate) good_kind: GoodKind,
    pub(crate) benefits: f32,
    pub(crate) margin: f32,
    pub(crate) max_qty: f32,
}

impl Arbitrage {
    pub(crate) fn new(buying_market_name: String, selling_market_name: String, good_kind: GoodKind, benefits: f32,  margin: f32, max_qty: f32) -> Self {
        Self {
            buying_market_name,
            selling_market_name,
            good_kind,
            benefits,
            margin,
            max_qty
        }
    }
}