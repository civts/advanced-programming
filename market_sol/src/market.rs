use crate::good_meta::GoodMeta;
use crate::market_metadata::MarketMetadata;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::{DEFAULT_GOOD_KIND, STARTING_CAPITAL};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError};
use unitn_market_2022::market::good_label::GoodLabel;
use crate::good_lock_meta::GoodLockMeta;
use crate::market_meta::MarketMeta;

const MARKET_NAME: &str = "SOL";
const TOKEN_DURATION: u32 = 5;

pub struct SOLMarket {
    name: String,
    goods: Vec<Good>,
    // Deprecated
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
        self.on_event(e.clone());
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
                        } });
            },

            EventKind::Sold => {
                //Update price after successful sell, slightly increase the price as qnty increases
                // i'm just chaniging the price :/
                self.good_labels.iter_mut().for_each(|gl| {
                    if gl.good_kind.eq(&event.good_kind) {
                        gl.exchange_rate_buy *= 0.95;
                        // println!("ciaoo {}", gl.exchange_rate_buy);
                    } });
            },

            EventKind::LockedBuy => {
            },
            EventKind::LockedSell => {},
            EventKind::Wait => { 
                // change some exchange rate -> buy_prices - as for now it's enough to decrease the price a bit
                // as time goes on with goods left unsold you tend to decrease the price
                self.good_labels.iter_mut().for_each(|gl| {
                        if gl.good_kind.ne(&GoodKind::EUR){
                            gl.exchange_rate_sell *= 1.05;
                        }
                    });
            },
        }
        //progress one day in any case
        self.meta.current_day += 1;
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
        usd_mkt_cap -= exceeding_capital;

        //Calculate the quantity of each good
        let yen_quantity = GoodKind::get_default_exchange_rate(&GoodKind::YEN) * yen_mkt_cap;
        let yuan_quantity = GoodKind::get_default_exchange_rate(&GoodKind::YUAN) * yuan_mkt_cap;
        let usd_quantity = GoodKind::get_default_exchange_rate(&GoodKind::USD) * usd_mkt_cap;
        //Get the market
        return Self::new_with_quantities(eur_quantity, yen_quantity, usd_quantity, yuan_quantity);
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> {
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
                good_kind: k.clone(),
                quantity: g.quantity_available,
                exchange_rate_buy: g.buy_price,
                exchange_rate_sell: g.sell_price,
            })
            .collect();

        Rc::new(RefCell::new(SOLMarket {
            name: String::from(MARKET_NAME),
            goods,
            good_labels,
            subscribers: vec![],
            old_meta: MarketMetadata {
                goods_meta: goods_metadata,
            },
            meta: MarketMeta::new(),
        }))
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>>
        where
            Self: Sized,
    { todo!() }

    fn get_name(&self) -> &'static str {
        return MARKET_NAME;
    }

    fn get_budget(&self) -> f32 {
        self.goods.iter().fold(0f32, |acc, good| {
            let value = good.get_qty() * self.old_meta.goods_meta.get(&good.get_kind()).unwrap().sell_price;
            acc + value
        })
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity.is_sign_negative() { return Err(MarketGetterError::NonPositiveQuantityAsked); }

        let good_label = self.good_labels.iter().find(|l| l.good_kind.eq(&kind)).unwrap();

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
        if quantity.is_sign_negative() { return Err(MarketGetterError::NonPositiveQuantityAsked); }

        let good_label = self.good_labels.iter().find(|l| l.good_kind.eq(&kind)).unwrap();

        let qty_available = good_label.quantity;
        if qty_available < quantity {
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind,
                requested_good_quantity: quantity,
                available_good_quantity: qty_available,
            });
        }

        Ok(quantity / good_label.exchange_rate_buy) //as discussed in the group with farouk 
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        self.good_labels.clone()
    }

    fn lock_buy(&mut self, kind_to_buy: GoodKind, quantity_to_buy: f32, bid: f32, trader_name: String) -> Result<String, LockBuyError> {
        // Check positive quantity
        if quantity_to_buy.is_sign_negative() { return Err(LockBuyError::NonPositiveQuantityToBuy { negative_quantity_to_buy: quantity_to_buy }); }

        // Check positive bid
        if bid.is_sign_negative() { return Err(LockBuyError::NonPositiveBid { negative_bid: bid }); }

        // Check quantity available
        let good_label = self.good_labels.iter_mut().find(|l| l.good_kind.eq(&kind_to_buy)).unwrap();
        let quantity_available = good_label.quantity;
        if quantity_available < quantity_to_buy {
            return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: quantity_to_buy,
                available_good_quantity: quantity_available,
            });
        }

        // todo: Maximum locks reached (see Market Deadlock section)

        // Check bid
        let min_bid = quantity_to_buy / good_label.exchange_rate_sell;
        if bid < min_bid {
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
        (kind_to_buy.clone(), quantity_to_buy.to_string(), bid.to_string(), now, trader_name).hash(&mut hasher);
        let token = hasher.finish().to_string();

        // Update good quantity available, todo: Update good buy and sell price (in on_event method)
        //problem with on_event method: the subscribed markets receive the notif, but you don't send the notif to yourself (at the moment) - but you can add that with one line
        // also: updates should be done only after a successful buy/sell, not locks

        good_label.quantity -= quantity_to_buy;

        // Update meta
        let good_meta = GoodLockMeta::new(kind_to_buy.clone(), bid, quantity_to_buy, self.meta.current_day);
        self.meta.locked_buys.insert(token.clone(), good_meta);

        // Create and spread event
        let e = Event {
            kind: EventKind::LockedBuy,
            good_kind: kind_to_buy,
            quantity: quantity_to_buy,
            price: bid,
        };

        self.notify_everyone(e);

        Ok(token)
    }


    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        // Check token existence
        let good_meta = match self.meta.locked_buys.get(&*token) {
            None => { return Err(BuyError::UnrecognizedToken { unrecognized_token: token }); }
            Some(g) => { g }
        };

        // Check token validity
        let days_since = self.meta.current_day - good_meta.created_on;
        if days_since > TOKEN_DURATION { return Err(BuyError::ExpiredToken { expired_token: token }); }

        // Check cash is default
        let kind = cash.get_kind();
        if kind.ne(&DEFAULT_GOOD_KIND) { return Err(BuyError::GoodKindNotDefault { non_default_good_kind: kind }); }

        // Check cash qty
        let contained_quantity = cash.get_qty();
        let pre_agreed_quantity = good_meta.price;
        if contained_quantity < pre_agreed_quantity { return Err(BuyError::InsufficientGoodQuantity { contained_quantity, pre_agreed_quantity }); }

        // Cash in, todo: Update good buy and sell price (in on_event method)
        let eur = cash.split(pre_agreed_quantity).unwrap();
        let default = self.good_labels.iter_mut().find(|l| l.good_kind.eq(&eur.get_kind())).unwrap();
        default.quantity += eur.get_qty();

        let release_good = Good::new(good_meta.kind.clone(), good_meta.quantity);

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

        Ok(release_good)
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        // Check positive quantity
        if quantity_to_sell.is_sign_negative() { return Err(LockSellError::NonPositiveQuantityToSell { negative_quantity_to_sell: (quantity_to_sell) }); }

        // Check positive bid
        if offer.is_sign_negative() { return Err(LockSellError::NonPositiveOffer { negative_offer: offer }); }

        // Check money available
        let money_available = self.good_labels.iter_mut().find(|gl| gl.good_kind.eq(&DEFAULT_GOOD_KIND)).unwrap().quantity;
        if money_available < offer {
            // return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable {
            //     offered_good_kind: kind_to_sell,
            //     offered_good_quantity: quantity_to_sell,
            //     available_good_quantity: money_available,
            // });
            // changed, maybe revert changes later on
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable {
                offered_good_kind: DEFAULT_GOOD_KIND,
                offered_good_quantity: offer,
                available_good_quantity: money_available,
            });
        }

        let good_label = self.good_labels.iter_mut().find(|l| l.good_kind.eq(&kind_to_sell)).unwrap();
        // todo: Maximum locks reached (see Market Deadlock section)

        // Check offer not too high
        let max_offer = quantity_to_sell / good_label.exchange_rate_buy;
        if offer > max_offer {
            return Err(LockSellError::OfferTooHigh {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: quantity_to_sell,
                high_offer: offer,
                highest_acceptable_offer: max_offer,
            });
        }

        let mut hasher = DefaultHasher::new();
        let now = chrono::Local::now();
        (kind_to_sell.clone(), quantity_to_sell.to_string(), offer.to_string(), now, trader_name).hash(&mut hasher);
        let token = hasher.finish().to_string();

        // Update good quantity available, todo: Update good buy and sell price (in on_event method)
        // also: updates should be done only after a successful buy/sell, not locks

        good_label.quantity += quantity_to_sell;

        // Update meta
        let good_meta = GoodLockMeta::new(kind_to_sell.clone(), offer, quantity_to_sell, self.meta.current_day);
        self.meta.locked_sells.insert(token.clone(), good_meta);

        // Create and spread event
        let e = Event {
            kind: EventKind::LockedSell,
            good_kind: kind_to_sell,
            quantity: quantity_to_sell,
            price: offer,
        };

        self.notify_everyone(e);

        Ok(token)
        
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        //if buy returns a good, then sell returns EUR!

        // Check token existence
        let good_meta = match self.meta.locked_sells.get(&*token) {
            None => { return Err(SellError::UnrecognizedToken { unrecognized_token: token }); }
            Some(g) => { g }
        };

        // Check token validity
        let days_since = self.meta.current_day - good_meta.created_on;
        if days_since > TOKEN_DURATION { return Err(SellError::ExpiredToken { expired_token: token }); }

        // Check good is not default
        let kind = good.get_kind();
        if kind.eq(&DEFAULT_GOOD_KIND) { return Err(SellError::WrongGoodKind { wrong_good_kind: kind.clone(), pre_agreed_kind: good_meta.kind.clone() }); }

        // Check quantity of the good passed in the args, has to match the pre_agreed_quantity during lock
        let contained_quantity = good.get_qty();
        let pre_agreed_quantity = good_meta.quantity;
        if contained_quantity < pre_agreed_quantity { return Err(SellError::InsufficientGoodQuantity { contained_quantity, pre_agreed_quantity }); }

        // Get your good now
        let selling_good = good.split(pre_agreed_quantity).unwrap();
        let my_good = self.good_labels.iter_mut().find(|l| l.good_kind.eq(&selling_good.get_kind())).unwrap();
        my_good.quantity += selling_good.get_qty();

        let give_money = Good::new(DEFAULT_GOOD_KIND, good_meta.price);

        // Create and spread event
        let e = Event {
            kind: EventKind::Sold,
            good_kind: good_meta.kind.clone(),
            quantity: good_meta.quantity,
            price: good_meta.price,
        };

        // Reset lock
        self.meta.locked_sells.remove(&*token);

        self.notify_everyone(e);

        Ok(give_money)
    }
}