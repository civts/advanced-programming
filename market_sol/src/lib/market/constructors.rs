use super::sol_market::{log, SOLMarket, ALL_GOOD_KINDS};
use crate::lib::domain::market_meta::MarketMeta;
use crate::lib::domain::strategy_name::StrategyName;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use unitn_market_2022::good::consts::STARTING_CAPITAL;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

impl SOLMarket {
    pub(crate) fn new_random_path(path: Option<&str>) -> Rc<RefCell<Self>> {
        //https://rust-random.github.io/book/guide-rngs.html#cryptographically-secure-pseudo-random-number-generators-csprngs
        let mut rng = ChaCha20Rng::from_entropy();
        //Generate the market cap of each good, randomly
        let mut remaining_market_cap = STARTING_CAPITAL;
        let mut eur_quantity = rng.gen_range(1.0..remaining_market_cap);
        remaining_market_cap -= eur_quantity;
        let yen_mkt_cap = rng.gen_range(0.0..remaining_market_cap);
        remaining_market_cap -= yen_mkt_cap;
        let yuan_mkt_cap = rng.gen_range(0.0..remaining_market_cap);
        remaining_market_cap -= yuan_mkt_cap;
        let usd_mkt_cap = remaining_market_cap;

        //Calculate the quantity of each good
        let mut yen_quantity = yen_mkt_cap * GoodKind::YEN.get_default_exchange_rate();
        let mut yuan_quantity = yuan_mkt_cap * GoodKind::YUAN.get_default_exchange_rate();
        let mut usd_quantity = usd_mkt_cap * GoodKind::USD.get_default_exchange_rate();

        //Fix floating point operation errors
        let real_market_cap = eur_quantity + yen_mkt_cap + yuan_mkt_cap + usd_mkt_cap;
        let exceeding_capital = (real_market_cap - STARTING_CAPITAL) + 1.0;
        if (yen_mkt_cap - exceeding_capital).is_sign_positive() {
            yen_quantity -= exceeding_capital * GoodKind::YEN.get_default_exchange_rate();
        } else if (yuan_mkt_cap - exceeding_capital).is_sign_positive() {
            yuan_quantity -= exceeding_capital * GoodKind::YUAN.get_default_exchange_rate();
        } else if (usd_mkt_cap - exceeding_capital).is_sign_positive() {
            usd_quantity -= exceeding_capital * GoodKind::USD.get_default_exchange_rate();
        } else if (eur_quantity - exceeding_capital).is_sign_positive() {
            eur_quantity -= exceeding_capital;
        } else {
            panic!("We are doing something wrong in this initialization");
        }

        //Get the market
        Self::new_with_quantities_and_path(
            eur_quantity,
            yen_quantity,
            usd_quantity,
            yuan_quantity,
            path,
            HashMap::new(),
        )
    }

    /// Need a constructor that has the SOLMarket type in its signature for our internal tests
    pub(crate) fn new_file_internal(path_str: &str) -> Rc<RefCell<SOLMarket>> {
        let path: &Path = Path::new(path_str);
        let path_exists = std::path::Path::exists(path);
        if path_exists {
            let quantities = Self::read_quantities_from_file(path);
            return match quantities {
                Some(goods) => {
                    let eur = goods
                        .iter()
                        .find(|g| g.get_kind() == GoodKind::EUR)
                        .unwrap()
                        .get_qty();
                    let usd = goods
                        .iter()
                        .find(|g| g.get_kind() == GoodKind::USD)
                        .unwrap()
                        .get_qty();
                    let yen = goods
                        .iter()
                        .find(|g| g.get_kind() == GoodKind::YEN)
                        .unwrap()
                        .get_qty();
                    let yuan = goods
                        .iter()
                        .find(|g| g.get_kind() == GoodKind::YUAN)
                        .unwrap()
                        .get_qty();
                    let weights = Self::read_weights_from_file(path);
                    return Self::new_with_quantities_and_path(
                        eur,
                        yen,
                        usd,
                        yuan,
                        Some(path_str),
                        weights,
                    );
                }
                None => Self::new_random_path(Some(path_str)),
            };
        } else {
            Self::new_random_path(Some(path_str))
        }
    }

    pub(crate) fn new_with_quantities_and_path(
        eur: f32,
        yen: f32,
        usd: f32,
        yuan: f32,
        optional_path: Option<&str>,
        weights: HashMap<StrategyName, f32>,
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
            meta: MarketMeta::new(goods_vec.clone(), optional_path, weights),
            internal_needs: SOLMarket::set_internal_needs(goods_vec),
        }))
    }
}
