pub(crate) struct GoodMeta {
    pub sell_price: f32,
    pub buy_price: f32,
    pub quantity_locked: f32,
    pub price_locked: f32,
    pub quantity_available: f32,
    pub token: String,
}

impl GoodMeta {
    pub fn new(price: f32, quantity: f32) -> Self {
        Self {
            sell_price: price * 0.98, // Sell price should always be inferior
            buy_price: price,
            quantity_locked: 0.0,
            price_locked: price,
            quantity_available: quantity,
            token: "".to_string(),
        }
    }

    fn is_locked(&self) -> bool {
        self.quantity_locked > 0f32
    }
}
