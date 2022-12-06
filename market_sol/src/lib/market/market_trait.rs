use super::sol_market::{SOLMarket, MARKET_NAME};
use crate::lib::market::sol_market::log;
use crate::lib::{
    domain::{good_lock_meta::GoodLockMeta, market_meta::MarketMeta},
    market::sol_market::{lock_limit_exceeded, TOKEN_DURATION},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::Path,
    rc::Rc,
};
use unitn_market_2022::event::event::Event;
use unitn_market_2022::{
    event::event::EventKind,
    good::{
        consts::{DEFAULT_GOOD_KIND, STARTING_CAPITAL},
        good::Good,
        good_kind::GoodKind,
    },
    market::{
        good_label::GoodLabel, BuyError, LockBuyError, LockSellError, Market, MarketGetterError,
        SellError,
    },
};

pub(crate) const MIN_MARGIN_PERCENTAGE: f32 = 0.6;
pub(crate) const MAX_MARGIN_PERCENTAGE: f32 = 3.0;

impl Market for SOLMarket {
    fn new_random() -> Rc<RefCell<dyn Market>> {
        //https://rust-random.github.io/book/guide-rngs.html#cryptographically-secure-pseudo-random-number-generators-csprngs
        let mut rng = ChaCha20Rng::from_entropy();
        //Generate the market cap of each good, randomly
        let mut remaining_market_cap = STARTING_CAPITAL;
        let mut eur_quantity = rng.gen_range(1.0..remaining_market_cap);
        remaining_market_cap -= eur_quantity;
        let mut yen_mkt_cap = rng.gen_range(0.0..remaining_market_cap);
        remaining_market_cap -= yen_mkt_cap;
        let mut yuan_mkt_cap = rng.gen_range(0.0..remaining_market_cap);
        remaining_market_cap -= yuan_mkt_cap;
        let mut usd_mkt_cap = remaining_market_cap;

        //Fix floating point operation errors
        let real_market_cap = eur_quantity + yen_mkt_cap + yuan_mkt_cap + usd_mkt_cap;
        let exceeding_capital = real_market_cap - STARTING_CAPITAL;
        if (yen_mkt_cap - exceeding_capital).is_sign_positive() {
            yen_mkt_cap -= exceeding_capital;
        } else if (yuan_mkt_cap - exceeding_capital).is_sign_positive() {
            yuan_mkt_cap -= exceeding_capital;
        } else if (usd_mkt_cap - exceeding_capital).is_sign_positive() {
            usd_mkt_cap -= exceeding_capital;
        } else if (eur_quantity - exceeding_capital).is_sign_positive() {
            eur_quantity -= exceeding_capital;
        } else {
            panic!("We are doing something wrong in this initialization");
        }

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

    fn get_budget(&self) -> f32 {
        self.goods.iter().fold(0f32, |acc, good| {
            let sell_price = self.meta.min_bid.get(&good.get_kind()).unwrap();
            let good_quantity = good.get_qty();
            let good_market_cap = good_quantity * sell_price;
            acc + good_market_cap
        })
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity.is_sign_negative() {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        //TODO: check that this is the total unlocked quantity!
        let total_quantity_in_the_market = self
            .goods
            .iter()
            .filter(|g| g.get_kind() == kind)
            .fold(0.0, |acc, good| acc + good.get_qty());
        if quantity > total_quantity_in_the_market {
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind,
                requested_good_quantity: quantity,
                available_good_quantity: total_quantity_in_the_market,
            });
        }

        let mut state = self.meta.price_state.borrow_mut();
        let unit_price = state.get_price(&kind, self.meta.current_day);

        let asked_quantity_ratio = quantity / total_quantity_in_the_market;
        let margin_percentage = asked_quantity_ratio
            * (MAX_MARGIN_PERCENTAGE - MIN_MARGIN_PERCENTAGE)
            + MIN_MARGIN_PERCENTAGE;

        let initial_price = unit_price * quantity;
        let margin = initial_price * margin_percentage / 100.0;
        let price = initial_price + margin;
        Ok(price)
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity.is_sign_negative() {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        // let good_label = self
        //     .good_labels
        //     .iter()
        //     .find(|l| l.good_kind.eq(&kind))
        //     .unwrap();

        // Ok(quantity / good_label.exchange_rate_buy) //as discussed in the group with farouk

        let mut state = self.meta.price_state.borrow_mut();
        let unit_price = state.get_price(&kind, self.meta.current_day);
        Ok(unit_price * quantity)
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
            log(log_error);
            return Err(LockBuyError::NonPositiveQuantityToBuy {
                negative_quantity_to_buy: quantity_to_buy,
            });
        }

        // Check positive bid
        if bid.is_sign_negative() {
            log(log_error);
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
            log(log_error);
            return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: quantity_to_buy,
                available_good_quantity: quantity_available,
            });
        }

        // Lock limit check
        let num_of_locks = self.meta.num_of_buy_locks();
        if lock_limit_exceeded(num_of_locks) {
            log(log_error);
            return Err(LockBuyError::MaxAllowedLocksReached);
        }

        // Check bid
        let min_bid = quantity_to_buy / good_label.exchange_rate_sell;
        if bid < min_bid {
            log(log_error);
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

        log(format!("LOCK_BUY-{trader_name}-KIND_TO_BUY:{kind_to_buy}-QUANTITY_TO_BUY:{quantity_to_buy:+e}-BID:{bid:+e}-TOKEN:{token}"));

        Ok(token)
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        // Set error log
        let log_error = format!("BUY-TOKEN:{token}-ERROR");

        // Check token existence
        let good_meta = match self.meta.locked_buys.get(&*token) {
            None => {
                log(log_error);
                return Err(BuyError::UnrecognizedToken {
                    unrecognized_token: token,
                });
            }
            Some(g) => g,
        };

        // Check token validity
        let days_since = self.meta.current_day - good_meta.created_on;
        if days_since > TOKEN_DURATION {
            log(log_error);
            return Err(BuyError::ExpiredToken {
                expired_token: token,
            });
        }

        // Check cash is default
        let kind = cash.get_kind();
        if kind.ne(&DEFAULT_GOOD_KIND) {
            log(log_error);
            return Err(BuyError::GoodKindNotDefault {
                non_default_good_kind: kind,
            });
        }

        // Check cash qty
        let contained_quantity = cash.get_qty();
        let pre_agreed_quantity = good_meta.price;
        if contained_quantity < pre_agreed_quantity {
            log(log_error);
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

        log(format!("BUY-TOKEN:{token}-OK"));

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
            log(log_error);
            return Err(LockSellError::NonPositiveQuantityToSell {
                negative_quantity_to_sell: (quantity_to_sell),
            });
        }

        // Check positive bid
        if offer.is_sign_negative() {
            log(log_error);
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
            log(log_error);
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: quantity_to_sell,
                available_good_quantity: money_available,
            });
        }

        // Lock limit check
        if lock_limit_exceeded(self.meta.num_of_locked_sells()) {
            log(log_error);
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
            log(log_error);
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

        log(format!("LOCK_SELL-{trader_name}-KIND_TO_SELL:{kind_to_sell}-QUANTITY_TO_SELL:{quantity_to_sell:+e}-OFFER:{offer:+e}-TOKEN:{token}"));

        Ok(token)
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        // Set error log
        let log_error = format!("SELL-TOKEN:{token}-ERROR");

        // Check token existence
        let good_meta = match self.meta.locked_sells.get(&*token) {
            None => {
                log(log_error);
                return Err(SellError::UnrecognizedToken {
                    unrecognized_token: token,
                });
            }
            Some(g) => g,
        };

        // Check token validity
        let days_since = self.meta.current_day - good_meta.created_on;
        if days_since > TOKEN_DURATION {
            log(log_error);
            return Err(SellError::ExpiredToken {
                expired_token: token,
            });
        }

        // Check good is the same as we agreed on lock
        let kind = good.get_kind();
        let expected_kind = good_meta.kind;
        if kind.ne(&expected_kind) {
            log(log_error);
            return Err(SellError::WrongGoodKind {
                wrong_good_kind: kind,
                pre_agreed_kind: expected_kind,
            });
        }

        // Check quantity of the good passed in the args, has to match the pre_agreed_quantity during lock
        let contained_quantity = good.get_qty();
        let pre_agreed_quantity = good_meta.quantity;
        if contained_quantity < pre_agreed_quantity {
            log(log_error);
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

        log(format!("SELL-TOKEN:{token}-OK"));

        Ok(give_money)
    }
}
