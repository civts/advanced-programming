use crate::lib::domain::good_meta::GoodMeta;
use crate::lib::domain::market_meta::MarketMeta;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::Market;

pub(crate) const MARKET_NAME: &str = "SOL";
pub(crate) const TOKEN_DURATION: u32 = 15; // TODO: Either token duration need to be > Lock Limit or implement lock_limit per trader
pub(crate) const LOCK_LIMIT: u32 = 10;

pub struct SOLMarket {
    pub(crate) goods: Vec<Good>,
    pub(crate) good_labels: Vec<GoodLabel>,
    pub(crate) subscribers: Vec<Box<dyn Notifiable>>,
    pub(crate) meta: MarketMeta,
}

impl SOLMarket {
    pub(crate) fn new_with_quantities_and_meta(
        eur: f32,
        yen: f32,
        usd: f32,
        yuan: f32,
        meta: MarketMeta,
    ) -> Rc<RefCell<dyn Market>> {
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
        let goods = vec![
            Good::new(GoodKind::EUR, eur),
            Good::new(GoodKind::YEN, yen),
            Good::new(GoodKind::YUAN, yuan),
            Good::new(GoodKind::USD, usd),
        ];
        fn to_map_item(good: &Good) -> (GoodKind, GoodMeta) {
            let kind = good.get_kind();
            let meta = GoodMeta::new(kind.get_default_exchange_rate(), good.get_qty());
            (kind, meta)
        }
        let goods_metadata: HashMap<GoodKind, GoodMeta> = goods.iter().map(to_map_item).collect();
        let good_labels: Vec<GoodLabel> = goods_metadata
            .iter()
            .map(|(k, g)| GoodLabel {
                good_kind: *k,
                quantity: g.quantity_available,
                exchange_rate_buy: g.buy_price,
                exchange_rate_sell: g.sell_price,
            })
            .collect();

        log(format!("MARKET_INITIALIZATION\nEUR: {eur:+e}\nUSD: {usd:+e}\nYEN: {yen:+e}\nYUAN: {yuan:+e}\nEND_MARKET_INITIALIZATION"));

        Rc::new(RefCell::new(SOLMarket {
            goods,
            good_labels,
            subscribers: vec![],
            meta,
        }))
    }
}

pub(crate) fn lock_limit_exceeded(num_of_locks: u32) -> bool {
    num_of_locks + 1 > LOCK_LIMIT
}

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
