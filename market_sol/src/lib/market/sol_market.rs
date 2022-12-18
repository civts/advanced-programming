use crate::lib::domain::market_meta::MarketMeta;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;

/// The name of the market. #default value = SOL
pub(crate) const MARKET_NAME: &str = "SOL";

/// The validity period for the token given in days. #default value = 15
pub(crate) const TOKEN_DURATION: u32 = 15;

/// The maximum amount of locks for buy and sell operations. #default value = 10
pub(crate) const LOCK_LIMIT: u32 = 10;

/// The margin this market applies on buy orders. #default value = 0.06
pub(crate) const MARKET_MARGIN: f32 = 0.06;

/// The array of all available goods for the market
pub(crate) const ALL_GOOD_KINDS: [GoodKind; 4] =
    [GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

/// SOL Market struct, represents the market data with it's goods, subscribers and metadata.
pub struct SOLMarket {
    pub(crate) goods: HashMap<GoodKind, Good>,
    pub(crate) subscribers: Vec<Box<dyn Notifiable>>,
    pub(crate) meta: MarketMeta,
}

impl SOLMarket {
    /// Returns [`SOLMarket`] with given quantites and optional file path to which the market will be serialized.
    pub(crate) fn new_with_quantities_and_path(
        eur: f32,
        yen: f32,
        usd: f32,
        yuan: f32,
        optional_path: Option<&str>,
    ) -> Rc<RefCell<SOLMarket>> {
        if eur < 0.0 {
            panic!("Tried to initialize the market with a negative quantity of eur");
        }
        if usd < 0.0 {
            panic!("Tried to initialize the market with a negative quantity of usd");
        }
        if yen < 0.0 {
            panic!("Tried to initialize the market with a negative quantity of yen");
        }
        if yuan < 0.0 {
            panic!("Tried to initialize the market with a negative quantity of yuan");
        }
        //Initialize the market
        let mut goods = HashMap::new();
        //Use a for-match to ensure we always do all of them
        for gk in ALL_GOOD_KINDS {
            match gk {
                GoodKind::EUR => goods.insert(gk, Good::new(gk, eur)),
                GoodKind::YEN => goods.insert(gk, Good::new(gk, yen)),
                GoodKind::USD => goods.insert(gk, Good::new(gk, usd)),
                GoodKind::YUAN => goods.insert(gk, Good::new(gk, yuan)),
            };
        }

        log(format!("MARKET_INITIALIZATION\nEUR: {eur:+e}\nUSD: {usd:+e}\nYEN: {yen:+e}\nYUAN: {yuan:+e}\nEND_MARKET_INITIALIZATION"));

        let goods_vec = Vec::from_iter(goods.values().cloned());
        Rc::new(RefCell::new(SOLMarket {
            goods,
            subscribers: vec![],
            meta: MarketMeta::new(goods_vec, optional_path),
        }))
    }

    /// Returns how much of the asked [`GoodKind`] is available (not locked)
    pub(crate) fn get_available_quantity(&self, good_kind: GoodKind) -> f32 {
        let good = self.goods.get(&good_kind).expect("Should be initialized");
        good.get_qty()
    }

    /// Exchange rate (EUR/[`GoodKind`]) for a given good_kind
    fn get_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        if good_kind == DEFAULT_GOOD_KIND {
            return 1.0;
        }
        let stocastic_price = self
            .meta
            .stocastic_price
            .borrow_mut()
            .get_price(&good_kind, self.meta.current_day);
        let quantity_price = self
            .meta
            .quantity_price
            .get_exchange_rate(&good_kind, Vec::from_iter(self.goods.values().cloned()));
        let other_markets_price = self.meta.other_markets.get_exchange_rate(&good_kind);
        //Compute the weighted average of the three
        let stochastic_weight: f32 = 1.0;
        let quantity_weight: f32 = 1.0;
        let others_weight: f32 = 1.0;
        let total_weight = stochastic_weight + quantity_weight + others_weight;
        let weighted_sum = (stocastic_price * stochastic_weight)
            + (quantity_price * quantity_weight)
            + (other_markets_price * others_weight);
        weighted_sum / total_weight
    }

    /// Return the rate applied when the trader wants to BUY the good from this market
    /// The rate is EUR/goodkind
    pub(crate) fn get_good_buy_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        //we divide, since the rate is eur/kind and not kind/eur
        self.get_exchange_rate(good_kind)
    }

    /// Return the rate applied when the trader wants to SELL the good to this market
    /// The rate is EUR/goodkind
    pub(crate) fn get_good_sell_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        if good_kind == DEFAULT_GOOD_KIND {
            return 1.0;
        }
        self.get_exchange_rate(good_kind) / (1.0 + MARKET_MARGIN)
    }

    /// Returns vector of [`GoodLabel`] for each [`GoodKind`] availabale on the market
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
}

/// Verifies if lock limit is still valid (below LOCK_LIMIT const)
pub(crate) fn lock_limit_exceeded(num_of_locks: u32) -> bool {
    num_of_locks + 1 > LOCK_LIMIT
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