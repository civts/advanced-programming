use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Debug)]
pub struct GoodLockMeta {
    pub kind: GoodKind,
    pub price: f32,
    pub quantity: f32,
    pub created_on: u32, // Number of notify calls since creation
}

impl GoodLockMeta {
    pub fn new(kind: GoodKind, price: f32, quantity: f32, created_on: u32) -> Self {
        Self {
            kind,
            price,
            quantity,
            created_on
        }
    }
}