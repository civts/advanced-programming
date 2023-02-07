use super::{trader_state::TraderState, trading_event_details::TradingEventDetails};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradingEvent {
    /// What happened
    pub details: TradingEventDetails,
    /// Name of the market the trader bought/sold/locked goods with
    pub market_name: String,
    /// The situation of a trader after the event happened
    pub trader_state: TraderState,
}
