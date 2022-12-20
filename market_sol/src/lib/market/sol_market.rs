use super::internal_trading::trade_role::TradeRole;
use crate::lib::domain::market_meta::MarketMeta;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;

pub(crate) const MARKET_NAME: &str = "SOL";
pub(crate) const TOKEN_DURATION: u32 = 15;
pub(crate) const LOCK_LIMIT: u32 = 10;
// The margin this market applies on buy orders
pub(crate) const MARKET_MARGIN: f32 = 0.06;

pub(crate) const ALL_GOOD_KINDS: [GoodKind; 4] =
    [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

pub struct SOLMarket {
    pub(crate) goods: HashMap<GoodKind, Good>,
    pub(crate) subscribers: Vec<Box<dyn Notifiable>>,
    pub(crate) meta: MarketMeta,
    pub(crate) internal_needs: HashMap<GoodKind, TradeRole>,
}

impl SOLMarket {
    /// Returns how much of the asked GoodKind is available (not locked)
    pub(crate) fn get_available_quantity(&self, good_kind: GoodKind) -> f32 {
        let good = self.goods.get(&good_kind).expect("Should be initialized");
        good.get_qty()
    }

    pub(crate) fn get_good_labels(&self) -> Vec<GoodLabel> {
        let values = self.goods.values();
        let iter = values.map(|g: &Good| -> GoodLabel {
            let good_kind = g.get_kind();
            GoodLabel {
                good_kind,
                quantity: g.get_qty(),
                exchange_rate_buy: self.get_good_buy_exchange_rate(good_kind),
                exchange_rate_sell: self.get_good_sell_exchange_rate(good_kind),
            }
        });
        Vec::from_iter(iter)
    }

    pub(crate) fn lock_limit_exceeded(num_of_locks: u32) -> bool {
        num_of_locks + 1 > LOCK_LIMIT
    }
}

/// Append log code to file according to specifications
pub(crate) fn log(log_code: String) {
    let filename = format!("log_{}.txt", MARKET_NAME);
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .unwrap();

    let time = chrono::Local::now()
        .format("%y:%m:%d:%H:%M:%S:%3f")
        .to_string();

    if let Err(e) = writeln!(file, "{}|{}|{}", MARKET_NAME, time, log_code) {
        eprintln!("Error while writing to file {}", e);
    }
}

/// Return the value in DEFAULT_GOOD_KIND of a good
pub(crate) fn get_value_good(kind: &GoodKind, qty: f32) -> f32 {
    qty / kind.get_default_exchange_rate()
}
