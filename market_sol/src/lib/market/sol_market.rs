use crate::lib::domain::market_meta::MarketMeta;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;

pub(crate) const MARKET_NAME: &str = "SOL";
pub(crate) const TOKEN_DURATION: u32 = 15;
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
        let goods = vec![
            Good::new(GoodKind::EUR, eur),
            Good::new(GoodKind::YEN, yen),
            Good::new(GoodKind::YUAN, yuan),
            Good::new(GoodKind::USD, usd),
        ];
        let good_labels: Vec<GoodLabel> = goods
            .iter()
            .map(|g| GoodLabel {
                good_kind: g.get_kind(),
                quantity: g.get_qty(),
                exchange_rate_buy: g.get_kind().get_default_exchange_rate(),
                // Selling price should always be slightly lower
                exchange_rate_sell: g.get_kind().get_default_exchange_rate() * 0.98,
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
