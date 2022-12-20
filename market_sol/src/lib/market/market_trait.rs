use super::sol_market::{SOLMarket, MARKET_NAME};
use crate::lib::market::sol_market::{get_value_good, log};
use crate::lib::{
    domain::good_lock_meta::GoodLockMeta,
    market::sol_market::{lock_limit_exceeded, TOKEN_DURATION},
};
use std::collections::HashMap;
use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};
use unitn_market_2022::event::event::Event;
use unitn_market_2022::{
    event::event::EventKind,
    good::{consts::DEFAULT_GOOD_KIND, good::Good, good_kind::GoodKind},
    market::{
        good_label::GoodLabel, BuyError, LockBuyError, LockSellError, Market, MarketGetterError,
        SellError,
    },
};

impl Market for SOLMarket {
    fn new_random() -> Rc<RefCell<dyn Market>> {
        Self::new_random_path(None)
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> {
        Self::new_with_quantities_and_path(eur, yen, usd, yuan, None, HashMap::new())
    }

    fn new_file(path_str: &str) -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        Self::new_file_internal(path_str)
    }

    fn get_name(&self) -> &'static str {
        MARKET_NAME
    }

    // [from the specs] returns the quantity of good EUR of the market
    fn get_budget(&self) -> f32 {
        self.goods.get(&DEFAULT_GOOD_KIND).unwrap().get_qty()
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity.is_sign_negative() {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        //TODO: check that this is the total unlocked quantity!
        let total_quantity_in_the_market =
            self.goods.get(&kind).map(|g| g.get_qty()).unwrap_or(0.0);
        if quantity > total_quantity_in_the_market {
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind,
                requested_good_quantity: quantity,
                available_good_quantity: total_quantity_in_the_market,
            });
        }

        let exchange_rate_eur_good = self.get_good_buy_exchange_rate(kind);

        // Adaptive margin
        // pub(crate) const MIN_MARGIN_PERCENTAGE: f32 = 0.6;
        // pub(crate) const MAX_MARGIN_PERCENTAGE: f32 = 3.0;
        // // let asked_quantity_ratio = quantity / total_quantity_in_the_market;
        // // let margin_percentage = asked_quantity_ratio
        // //     * (MAX_MARGIN_PERCENTAGE - MIN_MARGIN_PERCENTAGE)
        // //     + MIN_MARGIN_PERCENTAGE;
        // // let initial_price = unit_price * quantity;
        // // let margin = initial_price * margin_percentage / 100.0;
        // // let price = initial_price + margin;

        //no more using the adaptive margin
        let price = quantity / exchange_rate_eur_good;

        Ok(price)
    }

    //Returns how many of the default good we want to receive for quantity of the given good
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

        let eur_good_exchange_rate = self.get_good_sell_exchange_rate(kind);
        Ok(quantity / eur_good_exchange_rate)
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        self.get_good_labels()
    }

    fn lock_buy(
        &mut self,
        // What we want to buy (e.g., YEN)
        kind_to_buy: GoodKind,
        // How much of kind_to_buy we want to lock (e.g., 100.0)
        good_quantity_to_lock: f32,
        // How much we will pay in DEFAULT_GOOD
        bid: f32,
        trader_name: String,
    ) -> Result<String, LockBuyError> {
        // Set error log
        let log_error = format!("LOCK_BUY-{trader_name}-KIND_TO_BUY:{kind_to_buy}-QUANTITY_TO_BUY:{good_quantity_to_lock:+e}-BID:{bid:+e}-ERROR");

        // Check positive quantity
        if good_quantity_to_lock.is_sign_negative() {
            log(log_error);
            return Err(LockBuyError::NonPositiveQuantityToBuy {
                negative_quantity_to_buy: good_quantity_to_lock,
            });
        }

        // Check positive bid
        if bid.is_sign_negative() {
            log(log_error);
            return Err(LockBuyError::NonPositiveBid { negative_bid: bid });
        }

        // Lock limit check
        let num_of_locks = self.meta.num_of_buy_locks();
        if lock_limit_exceeded(num_of_locks) {
            log(log_error);
            return Err(LockBuyError::MaxAllowedLocksReached);
        }

        // Check quantity available
        let quantity_available = self.get_available_quantity(kind_to_buy);
        if quantity_available < good_quantity_to_lock {
            log(log_error);
            return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: good_quantity_to_lock,
                available_good_quantity: quantity_available,
            });
        }

        // Check bid
        let sell_exchange_rate_eur_good = self.get_good_buy_exchange_rate(kind_to_buy);
        let min_bid = good_quantity_to_lock / sell_exchange_rate_eur_good;
        if bid < min_bid {
            log(log_error);
            return Err(LockBuyError::BidTooLow {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: good_quantity_to_lock,
                low_bid: bid,
                lowest_acceptable_bid: min_bid,
            });
        }

        // Create token
        let mut hasher = DefaultHasher::new();
        let now = chrono::Local::now();
        (
            kind_to_buy,
            good_quantity_to_lock.to_string(),
            bid.to_string(),
            now,
            trader_name.clone(),
        )
            .hash(&mut hasher);
        let token = hasher.finish().to_string();

        // Update good quantity available, todo: Update good buy and sell price (in on_event method)
        // problem with on_event method: the subscribed markets receive the notif, but you don't send the notif to yourself (at the moment) - but you can add that with one line
        // TODO: DISCUSS -> updates should be done only after a successful buy/sell, not locks

        let previous_quantity = self.goods.get(&kind_to_buy).unwrap().get_qty();
        self.goods.insert(
            kind_to_buy,
            Good::new(kind_to_buy, previous_quantity - good_quantity_to_lock),
        );

        // Update meta
        let good_meta = GoodLockMeta::new(
            kind_to_buy,
            bid,
            good_quantity_to_lock,
            self.meta.current_day,
        );

        self.meta.locked_buys.insert(token.clone(), good_meta);

        // Create and spread event
        let e = Event {
            kind: EventKind::LockedBuy,
            good_kind: kind_to_buy,
            quantity: good_quantity_to_lock,
            price: bid,
        };

        self.notify_everyone(e);

        log(format!("LOCK_BUY-{trader_name}-KIND_TO_BUY:{kind_to_buy}-QUANTITY_TO_BUY:{good_quantity_to_lock:+e}-BID:{bid:+e}-TOKEN:{token}"));

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
        let paid_eur = cash.split(pre_agreed_quantity).unwrap();
        let total_quantity =
            self.goods.get(&DEFAULT_GOOD_KIND).unwrap().get_qty() + paid_eur.get_qty();
        self.goods.insert(
            DEFAULT_GOOD_KIND,
            Good::new(DEFAULT_GOOD_KIND, total_quantity),
        );

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

        // Increase need for release good
        self.internal_needs
            .get_mut(&release_good.get_kind())
            .unwrap()
            .increase_need(get_value_good(
                &release_good.get_kind(),
                release_good.get_qty(),
            ));

        // Decrease need for cash
        self.internal_needs
            .get_mut(&paid_eur.get_kind())
            .unwrap()
            .decrease_need(get_value_good(&paid_eur.get_kind(), paid_eur.get_qty()));

        self.notify_everyone(e);

        log(format!("BUY-TOKEN:{token}-OK"));

        Ok(release_good)
    }

    ///Allow the trader to get a lock to SELL a good to this market
    fn lock_sell(
        &mut self,
        kind_to_sell: GoodKind,
        // the quantity of good the trader wants to sell
        quantity_to_sell: f32,
        // the quantity of the default good kind the trader wants in exchange
        // for the good kind_to_sell with quantity quantity_to_sell
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
        let money_available = self.goods.get(&DEFAULT_GOOD_KIND).unwrap().get_qty();
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
        let good_sell_rate = self.get_good_sell_exchange_rate(kind_to_sell);
        let acceptable_eur_we_give_the_trader_on_sell = quantity_to_sell / good_sell_rate;
        if offer > acceptable_eur_we_give_the_trader_on_sell {
            log(log_error);
            return Err(LockSellError::OfferTooHigh {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: quantity_to_sell,
                high_offer: offer,
                highest_acceptable_offer: acceptable_eur_we_give_the_trader_on_sell,
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
        let mut remaining_quantity = self.goods.get(&DEFAULT_GOOD_KIND).unwrap().get_qty();
        remaining_quantity -= offer;
        self.goods.insert(
            DEFAULT_GOOD_KIND,
            Good::new(DEFAULT_GOOD_KIND, remaining_quantity),
        );

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
        let my_good = self.goods.get(&kind).unwrap();
        let final_quantity = my_good.get_qty() + selling_good.get_qty();
        self.goods.insert(kind, Good::new(kind, final_quantity));

        let give_money = Good::new(DEFAULT_GOOD_KIND, good_meta.price);

        // Create and sold event
        let e = Event {
            kind: EventKind::Sold,
            good_kind: good_meta.kind,
            quantity: good_meta.quantity,
            price: good_meta.price,
        };

        // Reset lock
        self.meta.locked_sells.remove(&*token);

        // Increase need for cash
        self.internal_needs
            .get_mut(&give_money.get_kind())
            .unwrap()
            .increase_need(get_value_good(&give_money.get_kind(), give_money.get_qty()));

        // Decrease need for selling good
        self.internal_needs
            .get_mut(&selling_good.get_kind())
            .unwrap()
            .decrease_need(get_value_good(
                &selling_good.get_kind(),
                selling_good.get_qty(),
            ));

        self.notify_everyone(e);

        log(format!("SELL-TOKEN:{token}-OK"));

        Ok(give_money)
    }
}
