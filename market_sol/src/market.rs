use crate::good_meta::GoodMeta;
use crate::market_metadata::MarketMetadata;
use crate::misc::banner::BANNER;

use crate::good_lock_meta::GoodLockMeta;
use crate::market_meta::MarketMeta;
use log::info;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::rc::Rc;
use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::*;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::{
    BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError,
};

const MARKET_NAME: &str = "SOL";
pub(crate) const TOKEN_DURATION: u32 = 15; // TODO: Either token duration need to be > Lock Limit or implement lock_limit per trader
pub(crate) const LOCK_LIMIT: u32 = 10;
mod sol_file_prefixes {
    pub const COMMENT_PREFIX: &str = "#";
    pub const GOOD_PREFIX: &str = "good ";
}

pub struct SOLMarket {
    goods: Vec<Good>,
    good_labels: Vec<GoodLabel>,
    subscribers: Vec<Box<dyn Notifiable>>,
    // Deprecated
    old_meta: MarketMetadata,
    meta: MarketMeta,
}

impl SOLMarket {
    /// Notify every market including ours of an event
    fn notify_everyone(&mut self, e: Event) {
        for subscriber in &mut self.subscribers {
            subscriber.on_event(e.clone())
        }
        // UNCOMMENT THIS LINE TO NOTIFY YOURSELF TOO, AND NOT ONLY YOUR NEIGHBOURS
        self.on_event(e);
    }

    fn new_with_quantities_and_meta(
        eur: f32,
        yen: f32,
        usd: f32,
        yuan: f32,
        meta: MarketMeta,
    ) -> Rc<RefCell<dyn Market>> {
        // Init logger
        log4rs::init_file("logging_config.yaml", Default::default()).unwrap_or_default();

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

        info!("MARKET_INITIALIZATION\nEUR: {eur:+e}\nUSD: {usd:+e}\nYEN: {yen:+e}\nYUAN: {yuan:+e}\nEND_MARKET_INITIALIZATION");

        Rc::new(RefCell::new(SOLMarket {
            goods,
            good_labels,
            subscribers: vec![],
            old_meta: MarketMetadata {
                goods_meta: goods_metadata,
            },
            meta,
        }))
    }

    /// If the market knows about a file, it means it read the state from there.
    /// This function updates such file with the current state of the market.
    fn write_to_file(&self) {
        match &self.meta.file_path {
            Some(pts) => {
                println!("Writing market info to file 📝");
                let path = Path::new(pts);
                let exists = Path::exists(path);
                //If needed, create target file
                if !exists {
                    let parent = Path::parent(path);
                    //If needed, create parent directory
                    if let Some(directory_path) = parent {
                        let parent_exists = Path::exists(directory_path);
                        if !parent_exists {
                            match fs::create_dir_all(directory_path) {
                                Ok(_) => {}
                                Err(_) => {
                                    panic!("Could not create directory for SOL market status file");
                                }
                            }
                        }
                    }
                    println!("SOL Market file at {} does not seem to exist", pts);
                    return;
                }
                //Get string contents
                let contents = self.serialize_to_file_string();
                match fs::write(path, contents) {
                    Ok(_) => {
                        //Success
                    }
                    Err(_) => {
                        println!("SOL market could not write to its file. Next run contents will not be resotred");
                    }
                }
            }
            None => {
                println!("Not writing the market info to file");
            }
        }
    }

    fn serialize_to_file_string(&self) -> String {
        let mut contents = String::new();
        for banner_line in BANNER {
            let mut s = String::from(sol_file_prefixes::COMMENT_PREFIX);
            s += &" ".repeat(4);
            s += banner_line;
            s += "\n";
            contents.push_str(&s);
        }
        contents.push('\n');
        for good in self.goods.iter() {
            contents.push_str(sol_file_prefixes::GOOD_PREFIX);
            let kind = match good.get_kind() {
                GoodKind::EUR => "EUR",
                GoodKind::YEN => "YEN",
                GoodKind::USD => "USD",
                GoodKind::YUAN => "YUAN",
            };
            contents.push_str(kind);
            contents.push(' ');
            contents.push_str(good.get_qty().to_string().as_str());
            contents.push(' ');
            let exchange_rate = self.meta.min_bid.get(&good.get_kind());
            if exchange_rate.is_none() {
                println!("⚠️ Exchange rate should be something at this point");
            }
            let exchange_rate = match exchange_rate {
                Some(e) => *e,
                None => match good.get_kind() {
                    GoodKind::EUR => 1f32,
                    GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
                    GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
                    GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
                },
            };
            contents.push_str(exchange_rate.to_string().as_str());
            contents.push('\n');
        }
        contents
    }

    /// Reads the file at the provided path and optionally returns a tuple with 4
    /// f32 numbers representing, respectively, the amount of euros, yens, dollars
    /// and yuan that the SOL Market represented in that file has.
    ///
    /// If there is an error reading or parsing the file, None is returned.
    fn read_quantities_from_file(path: &Path) -> Option<(f32, f32, f32, f32)> {
        use sol_file_prefixes::*;

        let pts = path.to_str().unwrap_or("invalid path");
        let exists = Path::exists(path);
        if !exists {
            println!("SOL Market file at {} does not seem to exist", pts);
            return None;
        }
        let contents = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Should have been able to read the file at {pts}"));
        let mut reading_failed = false;
        let mut goodmap: HashMap<GoodKind, f32> = HashMap::new();
        for (line_number, line) in contents.split('\n').into_iter().enumerate() {
            if line.starts_with(COMMENT_PREFIX) {
                continue;
            } else if line.starts_with(GOOD_PREFIX) {
                let parts = line.replace(GOOD_PREFIX, "");
                let parts: Vec<&str> = parts.split(' ').collect();
                let good_kind = match parts.first() {
                    Some(ticket) => match *ticket {
                        "USD" => GoodKind::USD,
                        "YEN" => GoodKind::YEN,
                        "EUR" => GoodKind::EUR,
                        "YUAN" => GoodKind::YUAN,
                        _ => {
                            println!("Line {line_number} should have a known good kind, but has '{ticket}'");
                            reading_failed = true;
                            break;
                        }
                    },
                    None => {
                        println!("Line {line_number} should declare a good in the correct format");
                        reading_failed = true;
                        break;
                    }
                };
                let quantity: f32 = match parts.get(1) {
                    Some(quantity_str) => {
                        let qty_result = quantity_str.parse();
                        match qty_result {
                            Ok(qt) => qt,
                            Err(_) => {
                                println!("Line {line_number} should have a valid good quantity, but has '{quantity_str}'");
                                reading_failed = true;
                                break;
                            }
                        }
                    }
                    None => {
                        println!("Line {line_number} should declare a good in the correct format");
                        reading_failed = true;
                        break;
                    }
                };
                if quantity < 0.0 {
                    println!("Line {line_number} should not declare a negative good quanity");
                }
                goodmap.insert(good_kind, quantity);
            }
        }
        let usd_qty = *goodmap.get(&GoodKind::USD).unwrap_or(&-1.0);
        if usd_qty < 0.0 {
            println!("Invalid quantity of usd in the SOL market file");
            reading_failed = true;
        }
        let eur_qty = *goodmap.get(&GoodKind::EUR).unwrap_or(&-1.0);
        if eur_qty < 0.0 {
            println!("Invalid quantity of eur in the SOL market file");
            reading_failed = true;
        }
        let yen_qty = *goodmap.get(&GoodKind::YEN).unwrap_or(&-1.0);
        if yen_qty < 0.0 {
            println!("Invalid quantity of yen in the SOL market file");
            reading_failed = true;
        }
        let yuan_qty = *goodmap.get(&GoodKind::YUAN).unwrap_or(&-1.0);
        if yuan_qty < 0.0 {
            println!("Invalid quantity of yuan in the SOL market file");
            reading_failed = true;
        }

        if reading_failed {
            None
        } else {
            Some((eur_qty, yen_qty, usd_qty, yuan_qty))
        }
    }
}

impl Notifiable for SOLMarket {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        self.subscribers.push(subscriber);
    }

    fn on_event(&mut self, event: Event) {
        match event.kind {
            EventKind::Bought => {
                //Update price after successful buy, slightly decrease the price as qnty increases
                self.good_labels.iter_mut().for_each(|gl| {
                    if gl.good_kind.eq(&event.good_kind) {
                        gl.exchange_rate_sell *= 1.05;
                    }
                });
            }

            EventKind::Sold => {
                //Update price after successful sell, slightly increase the price as qnty increases
                // i'm just chaniging the price :/
                self.good_labels.iter_mut().for_each(|gl| {
                    if gl.good_kind.eq(&event.good_kind) {
                        gl.exchange_rate_buy *= 0.95;
                        // println!("ciaoo {}", gl.exchange_rate_buy);
                    }
                });
            }

            EventKind::LockedBuy => {}
            EventKind::LockedSell => {}
            EventKind::Wait => {
                // change some exchange rate -> buy_prices - as for now it's enough to decrease the price a bit
                // as time goes on with goods left unsold you tend to decrease the price
                self.good_labels.iter_mut().for_each(|gl| {
                    if gl.good_kind.ne(&GoodKind::EUR) {
                        gl.exchange_rate_sell *= 1.05;
                    }
                });
            }
        }
        //progress one day in any case
        self.meta.current_day += 1;

        // Reinstate any good which has an expired token
        for (_, meta) in self.meta.locked_buys.iter() {
            let days_since = self.meta.current_day - meta.created_on;
            if days_since == TOKEN_DURATION {
                let good = self
                    .good_labels
                    .iter_mut()
                    .find(|l| l.good_kind.eq(&meta.kind))
                    .unwrap();
                good.quantity += meta.quantity;
            }
        }
        for (_, meta) in self.meta.locked_sells.iter() {
            let days_since = self.meta.current_day - meta.created_on;
            if days_since == TOKEN_DURATION {
                let default_good = self
                    .good_labels
                    .iter_mut()
                    .find(|l| l.good_kind.eq(&DEFAULT_GOOD_KIND))
                    .unwrap();
                default_good.quantity += meta.price;
            }
        }
    }
}

impl Market for SOLMarket {
    fn new_random() -> Rc<RefCell<dyn Market>> {
        //https://rust-random.github.io/book/guide-rngs.html#cryptographically-secure-pseudo-random-number-generators-csprngs
        let mut rng = ChaCha20Rng::from_entropy();
        //Generate the market cap of each good, randomly
        let mut remaining_market_cap = STARTING_CAPITAL;
        let eur_quantity = rng.gen_range(1.0..remaining_market_cap);
        remaining_market_cap -= eur_quantity;
        let yen_mkt_cap = rng.gen_range(0.0..remaining_market_cap);
        remaining_market_cap -= yen_mkt_cap;
        let yuan_mkt_cap = rng.gen_range(0.0..remaining_market_cap);
        remaining_market_cap -= yuan_mkt_cap;
        let mut usd_mkt_cap = remaining_market_cap;

        //Fix floating point operation errors
        let real_market_cap = eur_quantity + yen_mkt_cap + yuan_mkt_cap + usd_mkt_cap;
        let exceeding_capital = real_market_cap - STARTING_CAPITAL;
        usd_mkt_cap -= exceeding_capital + 1f32;

        // TODO: Check if usd_mkt_cap < 0

        //Calculate the quantity of each good
        let yen_quantity = GoodKind::get_default_exchange_rate(&GoodKind::YEN) * yen_mkt_cap;
        let yuan_quantity = GoodKind::get_default_exchange_rate(&GoodKind::YUAN) * yuan_mkt_cap;
        let usd_quantity = GoodKind::get_default_exchange_rate(&GoodKind::USD) * usd_mkt_cap;
        //Get the market
        Self::new_with_quantities(eur_quantity, yen_quantity, usd_quantity, yuan_quantity)
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> {
        Self::new_with_quantities_and_meta(eur, yen, usd, yuan, MarketMeta::new())
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        let path: &Path = Path::new(path);
        let path_exists = std::path::Path::exists(path);
        if path_exists {
            let quantities = Self::read_quantities_from_file(path);
            return match quantities {
                Some(q) => {
                    let meta = MarketMeta::new_with_file(path);
                    return Self::new_with_quantities_and_meta(q.0, q.1, q.2, q.3, meta);
                }
                None => Self::new_random(),
            };
        } else {
            Self::new_random()
        }
    }

    fn get_name(&self) -> &'static str {
        MARKET_NAME
    }

    // TODO: Check is we need to sum up all goods -> Specs not so clear
    fn get_budget(&self) -> f32 {
        self.goods.iter().fold(0f32, |acc, good| {
            let value = good.get_qty()
                * self
                    .old_meta
                    .goods_meta
                    .get(&good.get_kind())
                    .unwrap()
                    .sell_price;
            acc + value
        })
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity.is_sign_negative() {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        let good_label = self
            .good_labels
            .iter()
            .find(|g| g.good_kind.eq(&kind))
            .unwrap();

        let qty_available = good_label.quantity;
        if qty_available < quantity {
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind,
                requested_good_quantity: quantity,
                available_good_quantity: qty_available,
            });
        }

        Ok(quantity / good_label.exchange_rate_sell)
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity.is_sign_negative() {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        let good_label = self
            .good_labels
            .iter()
            .find(|l| l.good_kind.eq(&kind))
            .unwrap();

        Ok(quantity / good_label.exchange_rate_buy) //as discussed in the group with farouk
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        self.good_labels.clone()
    }

    fn lock_buy(
        &mut self,
        kind_to_buy: GoodKind,
        quantity_to_buy: f32,
        bid: f32,
        trader_name: String,
    ) -> Result<String, LockBuyError> {
        // Set error log
        let log_error = format!("LOCK_BUY-{trader_name}-KIND_TO_BUY:{kind_to_buy}-QUANTITY_TO_BUY:{quantity_to_buy:+e}-BID:{bid:+e}-ERROR");

        // Check positive quantity
        if quantity_to_buy.is_sign_negative() {
            info!("{log_error}");
            return Err(LockBuyError::NonPositiveQuantityToBuy {
                negative_quantity_to_buy: quantity_to_buy,
            });
        }

        // Check positive bid
        if bid.is_sign_negative() {
            info!("{log_error}");
            return Err(LockBuyError::NonPositiveBid { negative_bid: bid });
        }

        // Check quantity available
        let good_label = self
            .good_labels
            .iter_mut()
            .find(|g| g.good_kind.eq(&kind_to_buy))
            .unwrap();
        let quantity_available = good_label.quantity;
        if quantity_available < quantity_to_buy {
            info!("{log_error}");
            return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: quantity_to_buy,
                available_good_quantity: quantity_available,
            });
        }

        // Lock limit check
        let num_of_locks = self.meta.num_of_buy_locks();
        if lock_limit_exceeded(num_of_locks) {
            let b = lock_limit_exceeded(num_of_locks);
            print!("{b}");
            return Err(LockBuyError::MaxAllowedLocksReached);
        }

        // Check bid
        let min_bid = quantity_to_buy / good_label.exchange_rate_sell;
        if bid < min_bid {
            info!("{log_error}");
            return Err(LockBuyError::BidTooLow {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: quantity_to_buy,
                low_bid: bid,
                lowest_acceptable_bid: min_bid,
            });
        }

        // Create token
        let mut hasher = DefaultHasher::new();
        let now = chrono::Local::now();
        (
            kind_to_buy,
            quantity_to_buy.to_string(),
            bid.to_string(),
            now,
            trader_name.clone(),
        )
            .hash(&mut hasher);
        let token = hasher.finish().to_string();

        // Update good quantity available, todo: Update good buy and sell price (in on_event method)
        // problem with on_event method: the subscribed markets receive the notif, but you don't send the notif to yourself (at the moment) - but you can add that with one line
        // TODO: DISCUSS -> updates should be done only after a successful buy/sell, not locks

        good_label.quantity -= quantity_to_buy;

        // Update meta
        let good_meta = GoodLockMeta::new(kind_to_buy, bid, quantity_to_buy, self.meta.current_day);

        self.meta.locked_buys.insert(token.clone(), good_meta);

        // Create and spread event
        let e = Event {
            kind: EventKind::LockedBuy,
            good_kind: kind_to_buy,
            quantity: quantity_to_buy,
            price: bid,
        };

        self.notify_everyone(e);

        // Success log
        info!("LOCK_BUY-{trader_name}-KIND_TO_BUY:{kind_to_buy}-QUANTITY_TO_BUY:{quantity_to_buy:+e}-BID:{bid:+e}-TOKEN:{token}");

        Ok(token)
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        // Set error log
        let log_error = format!("BUY-TOKEN:{token}-ERROR");

        // Check token existence
        let good_meta = match self.meta.locked_buys.get(&*token) {
            None => {
                info!("{log_error}");
                return Err(BuyError::UnrecognizedToken {
                    unrecognized_token: token,
                });
            }
            Some(g) => g,
        };

        // Check token validity
        let days_since = self.meta.current_day - good_meta.created_on;
        if days_since > TOKEN_DURATION {
            info!("{log_error}");
            return Err(BuyError::ExpiredToken {
                expired_token: token,
            });
        }

        // Check cash is default
        let kind = cash.get_kind();
        if kind.ne(&DEFAULT_GOOD_KIND) {
            info!("{log_error}");
            return Err(BuyError::GoodKindNotDefault {
                non_default_good_kind: kind,
            });
        }

        // Check cash qty
        let contained_quantity = cash.get_qty();
        let pre_agreed_quantity = good_meta.price;
        if contained_quantity < pre_agreed_quantity {
            info!("{log_error}");
            return Err(BuyError::InsufficientGoodQuantity {
                contained_quantity,
                pre_agreed_quantity,
            });
        }

        // Cash in, todo: Update good buy and sell price (in on_event method)
        let eur = cash.split(pre_agreed_quantity).unwrap();
        let default = self
            .good_labels
            .iter_mut()
            .find(|g| g.good_kind.eq(&eur.get_kind()))
            .unwrap();
        default.quantity += eur.get_qty();

        let release_good = Good::new(good_meta.kind, good_meta.quantity);

        // Create and spread event
        let e = Event {
            kind: EventKind::Bought,
            good_kind: release_good.get_kind(),
            quantity: release_good.get_qty(),
            price: good_meta.price,
        };

        // Reset lock
        self.meta.locked_buys.remove(&*token);

        self.notify_everyone(e);

        // Success log
        info!("BUY-TOKEN:{token}-OK");

        Ok(release_good)
    }

    fn lock_sell(
        &mut self,
        kind_to_sell: GoodKind,
        quantity_to_sell: f32,
        offer: f32,
        trader_name: String,
    ) -> Result<String, LockSellError> {
        // Set error log
        let log_error = format!("LOCK_SELL-{trader_name}-KIND_TO_SELL:{kind_to_sell}-QUANTITY_TO_SELL:{quantity_to_sell:+e}-OFFER:{offer:+e}-ERROR");

        // Check positive quantity
        if quantity_to_sell.is_sign_negative() {
            info!("{log_error}");
            return Err(LockSellError::NonPositiveQuantityToSell {
                negative_quantity_to_sell: (quantity_to_sell),
            });
        }

        // Check positive bid
        if offer.is_sign_negative() {
            info!("{log_error}");
            return Err(LockSellError::NonPositiveOffer {
                negative_offer: offer,
            });
        }

        // Check money available
        let money_available = self
            .good_labels
            .iter_mut()
            .find(|gl| gl.good_kind.eq(&DEFAULT_GOOD_KIND))
            .unwrap()
            .quantity;
        if money_available < offer {
            info!("{log_error}");
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: quantity_to_sell,
                available_good_quantity: money_available,
            });
        }

        // Lock limit check
        if lock_limit_exceeded(self.meta.num_of_locked_sells()) {
            return Err(LockSellError::MaxAllowedLocksReached);
        }

        // Check offer not too high
        let good_buying_rate = self
            .good_labels
            .iter()
            .find(|l| l.good_kind.eq(&kind_to_sell))
            .unwrap()
            .exchange_rate_buy;
        let max_offer = quantity_to_sell / good_buying_rate;
        if offer > max_offer {
            info!("{log_error}");
            return Err(LockSellError::OfferTooHigh {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: quantity_to_sell,
                high_offer: offer,
                highest_acceptable_offer: max_offer,
            });
        }

        let mut hasher = DefaultHasher::new();
        let now = chrono::Local::now();
        (
            kind_to_sell,
            quantity_to_sell.to_string(),
            offer.to_string(),
            now,
            trader_name.clone(),
        )
            .hash(&mut hasher);
        let token = hasher.finish().to_string();

        // Update default good quantity available, todo: Update good buy and sell price (in on_event method)
        // also: updates should be done only after a successful buy/sell, not locks
        let default_good = self
            .good_labels
            .iter_mut()
            .find(|gl| gl.good_kind.eq(&DEFAULT_GOOD_KIND))
            .unwrap();
        default_good.quantity -= offer;

        // Update meta
        let good_meta =
            GoodLockMeta::new(kind_to_sell, offer, quantity_to_sell, self.meta.current_day);

        self.meta.locked_sells.insert(token.clone(), good_meta);

        // Create and spread event
        let e = Event {
            kind: EventKind::LockedSell,
            good_kind: kind_to_sell,
            quantity: quantity_to_sell,
            price: offer,
        };

        self.notify_everyone(e);

        // Success log
        info!("LOCK_SELL-{trader_name}-KIND_TO_SELL:{kind_to_sell}-QUANTITY_TO_SELL:{quantity_to_sell:+e}-OFFER:{offer:+e}-TOKEN:{token}");

        Ok(token)
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        // Set error log
        let log_error = format!("SELL-TOKEN:{token}-ERROR");

        // Check token existence
        let good_meta = match self.meta.locked_sells.get(&*token) {
            None => {
                info!("{log_error}");
                return Err(SellError::UnrecognizedToken {
                    unrecognized_token: token,
                });
            }
            Some(g) => g,
        };

        // Check token validity
        let days_since = self.meta.current_day - good_meta.created_on;
        if days_since > TOKEN_DURATION {
            info!("{log_error}");
            return Err(SellError::ExpiredToken {
                expired_token: token,
            });
        }

        // Check good is the same as we agreed on lock
        let kind = good.get_kind();
        let expected_kind = good_meta.kind;
        if kind.ne(&expected_kind) {
            info!("{log_error}");
            return Err(SellError::WrongGoodKind {
                wrong_good_kind: kind,
                pre_agreed_kind: expected_kind,
            });
        }

        // Check quantity of the good passed in the args, has to match the pre_agreed_quantity during lock
        let contained_quantity = good.get_qty();
        let pre_agreed_quantity = good_meta.quantity;
        if contained_quantity < pre_agreed_quantity {
            info!("{log_error}");
            return Err(SellError::InsufficientGoodQuantity {
                contained_quantity,
                pre_agreed_quantity,
            });
        }

        // Get your good now
        let selling_good = good.split(pre_agreed_quantity).unwrap();
        let my_good = self
            .good_labels
            .iter_mut()
            .find(|l| l.good_kind.eq(&selling_good.get_kind()))
            .unwrap();
        my_good.quantity += selling_good.get_qty();

        let give_money = Good::new(DEFAULT_GOOD_KIND, good_meta.price);

        // Create and spread event
        let e = Event {
            kind: EventKind::Sold,
            good_kind: good_meta.kind,
            quantity: good_meta.quantity,
            price: good_meta.price,
        };

        // Reset lock
        self.meta.locked_sells.remove(&*token);

        self.notify_everyone(e);

        // Sucess log
        info!("SELL-TOKEN:{token}-OK");

        Ok(give_money)
    }
}

fn lock_limit_exceeded(num_of_locks: u32) -> bool {
    num_of_locks + 1 > LOCK_LIMIT
}

impl Drop for SOLMarket {
    fn drop(&mut self) {
        println!("Looks like it is time to say farewell my friend 👋");
        self.write_to_file();
        println!("Thank you for using the {} market 😌", MARKET_NAME);
    }
}
