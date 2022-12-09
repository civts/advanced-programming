use unitn_market_2022::good::good_kind::GoodKind;

#[derive(PartialEq, Clone, Debug)]
pub struct GoodLockMeta {
    pub kind: GoodKind,
    //TODO Is this the exchange rate or the total amount exchanged in this trade in DEFAULT_GOOD?
    pub price: f32,
    pub quantity: f32,
    pub created_on: u32, // Number of notify calls since creation
                         // lock_type: LockType,
}

// enum LockType {
//     Sell,
//     Buy,
// }
impl GoodLockMeta {
    pub fn new(kind: GoodKind, price: f32, quantity: f32, created_on: u32) -> Self {
        Self {
            kind,
            price,
            quantity,
            created_on,
        }
    }
}
