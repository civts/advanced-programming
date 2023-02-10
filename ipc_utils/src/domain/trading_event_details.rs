use serde::{Deserialize, Serialize};
use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradingEventDetails {
    pub operation: TradeOperation,
    pub successful: bool,
    pub trade_type: TradeType,
    pub good_kind: GoodKind,
    pub quantity: f32,
    pub price: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TradeOperation {
    AskedLock,
    TradeFinalized,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TradeType {
    Buy,
    Sell,
}
