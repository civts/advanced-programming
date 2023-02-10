use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Debug, Clone)]
pub(crate) struct Arbitrage {
    pub(crate) buying_market_name: String,
    pub(crate) selling_market_name: String,
    pub(crate) good_kind: GoodKind,
    pub(crate) qty: f32,
    pub(crate) benefits: f32,
    pub(crate) margin: f32,
}

impl Arbitrage {
    pub(crate) fn new(
        buying_market_name: String,
        selling_market_name: String,
        good_kind: GoodKind,
        qty: f32,
        benefits: f32,
        margin: f32,
    ) -> Self {
        Self {
            buying_market_name,
            selling_market_name,
            good_kind,
            qty,
            benefits,
            margin,
        }
    }
}
