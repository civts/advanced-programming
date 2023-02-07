use crate::trader::SOLTrader;
use ipc_utils::trader_state::TraderState;
use ipc_utils::trading_event::TradingEvent;
use ipc_utils::trading_event_details::{TradeType, TradingEventDetails};
use unitn_market_2022::good::good_kind::GoodKind;

#[derive(Debug, Clone)]
pub(crate) enum TradeEvent {
    Locked,
    Finalized,
}

#[derive(Debug, Clone)]
pub(crate) struct Arbitrage {
    pub(crate) buying_market_name: String,
    pub(crate) selling_market_name: String,
    pub(crate) good_kind: GoodKind,
    pub(crate) qty: f32,
    pub(crate) buy_price: f32,
    pub(crate) sell_price: f32,
    pub(crate) benefits: f32,
    pub(crate) margin: f32,
}

impl Arbitrage {
    pub(crate) fn new(
        buying_market_name: String,
        selling_market_name: String,
        good_kind: GoodKind,
        qty: f32,
        buy_price: f32,
        sell_price: f32,
        benefits: f32,
        margin: f32,
    ) -> Self {
        Self {
            buying_market_name,
            selling_market_name,
            good_kind,
            qty,
            buy_price,
            sell_price,
            benefits,
            margin,
        }
    }

    // TODO: interact with ipc_utils
    pub(crate) fn log_trade_event(
        &self,
        trader: &SOLTrader,
        trade_event: TradeEvent,
        successful: bool,
        trade_type: TradeType,
    ) {
        let trading_event = TradingEvent {
            details: match trade_event {
                 TradeEvent::Locked => TradingEventDetails::AskedLock {
                    successful,
                    trade_type: trade_type.clone(),
                    good_kind: self.good_kind,
                    quantity: self.qty,
                    price: match trade_type {
                        TradeType::Buy => self.buy_price,
                        TradeType::Sell => self.sell_price,
                    },
                },
                TradeEvent::Finalized => TradingEventDetails::TradeFinalized {
                    successful,
                    trade_type: trade_type.clone(),
                    good_kind: self.good_kind,
                    quantity: self.qty,
                    price: match trade_type {
                        TradeType::Buy => self.buy_price,
                        TradeType::Sell => self.sell_price,
                    },
                },
            },

            market_name: match trade_type {
                TradeType::Buy => self.buying_market_name.clone(),
                TradeType::Sell => self.selling_market_name.clone(),
            },

            trader_state: TraderState::new(
                trader
                    .goods
                    .iter()
                    .map(|(k, g)| (k.clone(), g.get_qty()))
                    .collect(),
                "Arbitrage trader".to_string(), // TODO: Maybe initialize trader with a specific name
            ),
        };
        println!("{trading_event:?}"); // TODO: Replace with real communication with ipc_utils
    }
}
