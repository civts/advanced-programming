use serde::{Deserialize, Serialize};
use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TradingEventDetails {
    AskedLock {
        successful: bool,
        trade_type: TradeType,
        good_kind: GoodKind,
        quantity: f32,
        price: f32,
    },
    TradeFinalized {
        successful: bool,
        trade_type: TradeType,
        good_kind: GoodKind,
        quantity: f32,
        price: f32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TradeType {
    Buy,
    Sell,
}