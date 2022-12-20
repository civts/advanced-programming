use crate::lib::domain::market_meta::MarketMeta;
use crate::lib::domain::strategy_name::StrategyName;
use crate::lib::market::trade_role::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::{DEFAULT_GOOD_KIND, STARTING_CAPITAL};
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
            internal_needs: set_internal_needs(goods_vec),
        }))
    }

    /// Returns how much of the asked GoodKind is available (not locked)
    pub(crate) fn get_available_quantity(&self, good_kind: GoodKind) -> f32 {
        let good = self.goods.get(&good_kind).expect("Should be initialized");
        good.get_qty()
    }

    /// Exchange rate (EUR/goodkind) for this good
    fn get_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        let stocastic_rate = self.get_stocastic_rate(good_kind);
        let quantity_rate = self.get_quantity_rate(good_kind);
        let other_markets_rate = self.get_other_rate(good_kind);
        //Compute the weighted average of the three
        let stochastic_weight: f32 = *self
            .meta
            .weights
            .get(&StrategyName::Stocastic)
            .unwrap_or(&1.0);
        let quantity_weight: f32 = *self
            .meta
            .weights
            .get(&StrategyName::Quantity)
            .unwrap_or(&1.0);
        let others_weight: f32 = *self.meta.weights.get(&StrategyName::Others).unwrap_or(&1.0);
        let total_weight = stochastic_weight.abs() + quantity_weight.abs() + others_weight.abs();
        assert!(total_weight > 0.0);
        let weighted_sum = f32::max(0.0, stocastic_rate * stochastic_weight)
            + f32::max(0.0, quantity_rate * quantity_weight)
            + f32::max(0.0, other_markets_rate * others_weight);
        weighted_sum / total_weight
    }

    pub fn get_other_rate(&self, good_kind: GoodKind) -> f32 {
        self.meta.other_markets.get_exchange_rate(&good_kind)
    }

    pub fn get_quantity_rate(&self, good_kind: GoodKind) -> f32 {
        self.meta
            .quantity_price
            .get_exchange_rate(&good_kind, Vec::from_iter(self.goods.values().cloned()))
    }

    pub fn get_stocastic_rate(&self, good_kind: GoodKind) -> f32 {
        self.meta
            .stocastic_price
            .borrow_mut()
            .get_rate(&good_kind, self.meta.current_day)
    }

    /// Return the rate applied when the trader wants to BUY the good from this market
    /// The rate is EUR/goodkind
    pub(crate) fn get_good_buy_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        if good_kind == DEFAULT_GOOD_KIND {
            1.0
        } else {
            //we divide, since the rate is eur/kind and not kind/eur
            self.get_exchange_rate(good_kind)
        }
    }

    /// Return the rate applied when the trader wants to SELL the good to this market
    /// The rate is EUR/goodkind
    pub(crate) fn get_good_sell_exchange_rate(&self, good_kind: GoodKind) -> f32 {
        if good_kind == DEFAULT_GOOD_KIND {
            1.0
        } else {
            self.get_exchange_rate(good_kind) / (1.0 + MARKET_MARGIN)
        }
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

    /// Perform an internal trade if needed
    ///
    /// Example: An importer has a positive need and an exporter has a surplus
    pub(crate) fn internal_trade_if_needed(&mut self) {
        // Find good that need a refill and the one capable of refilling
        let mut max_need = 0f32;
        let mut max_ability = 0f32;
        let mut kind_need_refill: Option<GoodKind> = None;
        let mut kind_able_refill: Option<GoodKind> = None;
        for (kind, role) in self.internal_needs.iter() {
            match role {
                TradeRole::Importer { need } => {
                    let n = *need;
                    if n > max_need {
                        max_need = n;
                        kind_need_refill = Some(*kind);
                    }
                }
                TradeRole::Exporter { need } => {
                    let ability = if need.is_sign_negative() {
                        need.abs()
                    } else {
                        -need
                    };
                    // Market ability in case the good is locked and need has not been updated yet
                    let market_ability =
                        get_value_good(kind, self.goods.get(kind).unwrap().get_qty());
                    if ability > max_ability && market_ability > max_ability {
                        max_ability = market_ability.min(ability);
                        kind_able_refill = Some(*kind);
                    }
                }
            }
        }

        // Refill if possible/needed
        if let Some(scr_kind) = kind_able_refill {
            if let Some(dst_kind) = kind_need_refill {
                let value = (max_ability.min(10_000f32)).min(max_need);
                self.internal_trade(scr_kind, dst_kind, value);
            }
        }
    }

    /// Perform an internal trade
    fn internal_trade(&mut self, src_kind: GoodKind, dst_kind: GoodKind, value_in_eur: f32) {
        // Decrease good qty from source
        let src_qty = value_in_eur * src_kind.get_default_exchange_rate();
        self.goods
            .get_mut(&src_kind)
            .unwrap()
            .split(src_qty)
            .unwrap();

        // Increase need to source
        self.internal_needs
            .get_mut(&src_kind)
            .unwrap()
            .increase_need(value_in_eur);

        // Increase good qty to destination (+25% of default exchange rate)
        let dst_qty = value_in_eur * dst_kind.get_default_exchange_rate() * 1.25;
        self.goods
            .get_mut(&dst_kind)
            .unwrap()
            .merge(Good::new(dst_kind, dst_qty))
            .unwrap();

        // Decrease need to destination
        self.internal_needs
            .get_mut(&dst_kind)
            .unwrap()
            .decrease_need(value_in_eur);
    }
}

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

/// Return the value in DEFAULT_GOOD_KIND of a good
pub(crate) fn get_value_good(kind: &GoodKind, qty: f32) -> f32 {
    qty / kind.get_default_exchange_rate()
}

/// Set internal needs according to the EUR value of a certain good and the total value of the market (in EUR)
///
/// Example:
///
/// Market has:
///     - 100 EUR  (value: 100€)
///     - 100 USD  (value: 96.55€)
///     - 100 YEN  (value: 0.70€)
///     - 100 YUAN (value: 13.59€)
///
/// Total Value: 210.84€
/// Ideal Value of each goods: (210.84 / 4) = 52.71€
///     - 100 EUR  (value: 100€)    -> need: (52.71 - 100)      = -47.29    -> Exporter
///     - 100 USD  (value: 96.55€)  -> need: (52.71 - 96.55)    = -43.84    -> Exporter
///     - 100 YEN  (value: 0.70€)   -> need: (52.71 - 0.70)     = 52.01     -> Importer
///     - 100 YUAN (value: 13.59€)  -> need: (52.71 - 13.59)    = 39.12     -> Importer
pub(crate) fn set_internal_needs(goods_vec: Vec<Good>) -> HashMap<GoodKind, TradeRole> {
    let total_value_market = goods_vec.iter().fold(0f32, |acc, g| {
        acc + get_value_good(&g.get_kind(), g.get_qty())
    });
    let ideal_value_per_good = total_value_market / goods_vec.len() as f32;

    let mut internal_needs: HashMap<GoodKind, TradeRole> = HashMap::new();
    for g in goods_vec.iter() {
        let need = ideal_value_per_good - get_value_good(&g.get_kind(), g.get_qty());
        // Set goods with needs as importers
        if need > 0f32 {
            internal_needs.insert(g.get_kind(), TradeRole::Importer { need });
        }
        // Set goods with negative needs (surplus) as Exporters
        else {
            internal_needs.insert(g.get_kind(), TradeRole::Exporter { need });
        }
    }
    internal_needs
}
