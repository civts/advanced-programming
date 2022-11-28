use unitn_market_2022::good::{good::Good, good_kind::GoodKind};

pub(crate) struct GoodMeta {
    pub sell_price: f32,
    buy_price: f32,
    quantity_locked: f32,
    price_locked: f32,
    quantity_available: f32,
    token: String,
}

impl GoodMeta {
    fn new(price: f32, quantity: f32) -> Self {
        Self {
            sell_price: price,
            buy_price: price * 0.99, // TODO: Come with a better option
            quantity_locked: 0.0,
            price_locked: price,
            quantity_available: quantity,
            token: "".to_string(),
        }
    }

    fn is_locked(&self) -> bool {
        self.quantity_locked > 0f32
    }

    pub fn fromGood(g: &Good) -> (GoodKind, GoodMeta) {
        let kind = g.get_kind();
        let meta = GoodMeta::new(1.01, g.get_qty());
        (kind, meta)
    }
}
