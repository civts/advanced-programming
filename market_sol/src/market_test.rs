use crate::market::SOLMarket;
use std::cell::RefCell;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use unitn_market_2022::event::event::Event;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, SellError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};

const TRADER_NAME: &str = "foobar";

#[test]
fn should_return_markets_name() {
    // given
    let market = SOLMarket::new_with_quantities(0.0, 0.0, 0.0, 0.0);
    // when
    let name = market.borrow().get_name();
    // then
    assert_eq!("SOL", name)
}

#[test]
/// When a market gets created using the `new_with_quantities` constructor,
/// the quantities of each good in the market should correspond to the ones
/// passed as parameters to the constructor.
///
/// SOL group
fn should_initialize_with_right_quantity() {
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::{good_label::GoodLabel, Market};

    //Create market with predefined quantities
    let eur_qty = 12.0;
    let usd_qty = 42.0;
    let yen_qty = 137.0;
    let yuan_qty = 1984.0;
    let market = SOLMarket::new_with_quantities(eur_qty, yen_qty, usd_qty, yuan_qty);
    //Get the goods in the market immediately after creation
    let goods = market.borrow().get_goods();
    {
        //Check USD quantity
        let usd_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::USD)
            .collect();
        assert_eq!(
            usd_vec.len(),
            1,
            "There should be only one GoodLabel for usd"
        );
        let usd_good_label = usd_vec.get(0).unwrap();
        assert_eq!(
            usd_good_label.quantity, usd_qty,
            "The usd quantity in the market should be equal to the one supplied in the constructor"
        );
    }
    {
        //Check EUR quantity
        let eur_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::EUR)
            .collect();
        assert_eq!(
            eur_vec.len(),
            1,
            "There should be only one GoodLabel for eur"
        );
        let eur_good_label = eur_vec.get(0).unwrap();
        assert_eq!(
            eur_good_label.quantity, eur_qty,
            "The eur quantity in the market should be equal to the one supplied in the constructor"
        );
    }
    {
        //Check YEN quantity
        let yen_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::YEN)
            .collect();
        assert_eq!(
            yen_vec.len(),
            1,
            "There should be only one GoodLabel for yen"
        );
        let yen_good_label = yen_vec.get(0).unwrap();
        assert_eq!(
            yen_good_label.quantity, yen_qty,
            "The yen quantity in the market should be equal to the one supplied in the constructor"
        );
    }
    {
        //Check YUAN quantity
        let yuan_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::YUAN)
            .collect();
        assert_eq!(
            yuan_vec.len(),
            1,
            "There should be only one GoodLabel for yuan"
        );
        let yuan_good_label = yuan_vec.get(0).unwrap();
        assert_eq!(
            yuan_good_label.quantity,
            yuan_qty,
            "The yuan quantity in the market should be equal to the one supplied in the constructor"
        );
    }
}

#[test]
/// When a merket gets created using the `new_random` constructor, the total
/// value of the market expressed in default_good_kind and calculated with the
/// default exchange rate shall not exceed the `STARTING_CAPITAL`.
///
/// Since the `new_random` function is, most likely, non-deterministic, this
/// test cannot guarantee that the code is correct, but should catch bugs in the
/// long run.
///
/// Reference to the specs [here](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#market-creation)
///
/// SOL team
fn new_random_should_not_exceeed_starting_capital() {
    use unitn_market_2022::good::consts::*;
    use unitn_market_2022::{good::good_kind::GoodKind, market::Market};

    //Test 10 times to get better chances of catching bugs
    for _ in 0..10 {
        //Create a new market with the random constructor
        let market = SOLMarket::new_random();
        //Immediately get the goods
        let goods = market.borrow().get_goods();
        //Calculate total value of the market with default exchange rate
        let mut total_value = 0.0;
        for good in goods {
            let default_exchnge_rate = match good.good_kind {
                GoodKind::EUR => 1.0,
                GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
                GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
                GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            };
            //The amount of EUR the current good is worth
            let good_market_cap = good.quantity / default_exchnge_rate;
            total_value += good_market_cap;
        }
        assert!(
            total_value <= STARTING_CAPITAL,
            "The value of the market must be under the STARTING_CAPITAL"
        );
    }
}

#[cfg(test)]
mod test_buy {
    use super::*;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind::*;
    use unitn_market_2022::market::{BuyError, LockBuyError, MarketGetterError};

    // Setup a struct with default test value
    struct TestMarketSetup {
        market: Rc<RefCell<dyn Market>>,
        buy_kind: GoodKind,
        init_qty: f32,
        init_bid: f32,
        trader: String,
    }

    impl TestMarketSetup {
        fn new() -> Self {
            let init_qty = 100f32;
            let market = SOLMarket::new_with_quantities(init_qty, init_qty, init_qty, init_qty);
            let buy_kind = USD;
            let init_bid = market
                .borrow()
                .get_goods()
                .iter()
                .find_map(|gl| {
                    if gl.good_kind.eq(&buy_kind) {
                        Some(init_qty / gl.exchange_rate_sell)
                    } else {
                        None
                    }
                })
                .unwrap();
            let trader = "foobar".to_string();
            Self {
                market,
                buy_kind,
                init_qty,
                init_bid,
                trader,
            }
        }
    }

    #[test]
    fn get_buy_price() {
        let s = TestMarketSetup::new();
        let market = s.market.borrow();

        // Fail on negative quantity
        let neg_qty = -1f32;
        let result = market
            .get_buy_price(s.buy_kind.clone(), neg_qty)
            .unwrap_err();
        let expected = MarketGetterError::NonPositiveQuantityAsked;
        assert_eq!(result, expected);

        // Fail on insufficient Quantity
        let extra_qty = s.init_qty + 1f32;
        let result = market
            .get_buy_price(s.buy_kind.clone(), extra_qty)
            .unwrap_err();
        let expected = MarketGetterError::InsufficientGoodQuantityAvailable {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: extra_qty,
            available_good_quantity: s.init_qty,
        };
        assert_eq!(result, expected);

        // Success with total amount
        let kinds = vec![EUR, USD, YEN, YUAN];
        for k in kinds.iter() {
            let result = market.get_buy_price(k.clone(), s.init_qty).unwrap();
            let expected = s.init_qty / k.get_default_exchange_rate(); // market sell price = default exchange rate when init
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn lock_buy() {
        let s = TestMarketSetup::new();
        let mut market = s.market.borrow_mut();

        // Fail on negative quantity (while negative bid to see if priority is maintain)
        let neg_qty = -1f32;
        let neg_bid = neg_qty / s.buy_kind.get_default_exchange_rate();
        let result = market
            .lock_buy(
                s.buy_kind.clone(),
                neg_qty,
                neg_bid - 1f32,
                s.trader.clone(),
            )
            .unwrap_err();
        let expected = LockBuyError::NonPositiveQuantityToBuy {
            negative_quantity_to_buy: neg_qty,
        };
        assert_eq!(result, expected);

        // Fail on negative bid (while insufficient quantity)
        let extra_qty = s.init_qty + 0.1f32;
        let result = market
            .lock_buy(s.buy_kind.clone(), extra_qty, neg_bid, s.trader.clone())
            .unwrap_err();
        let expected = LockBuyError::NonPositiveBid {
            negative_bid: neg_bid,
        };
        assert_eq!(result, expected);

        // Fail on insufficient quantity (while low bid)
        let low_bid = 0f32;
        let result = market
            .lock_buy(s.buy_kind.clone(), extra_qty, low_bid, s.trader.clone())
            .unwrap_err();
        let expected = LockBuyError::InsufficientGoodQuantityAvailable {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: extra_qty,
            available_good_quantity: s.init_qty,
        };
        assert_eq!(result, expected);

        // Fail on low bid
        let low_bid = s.init_bid - 0.1f32;
        let result = market
            .lock_buy(s.buy_kind.clone(), s.init_qty, low_bid, s.trader.clone())
            .unwrap_err();
        let expected = LockBuyError::BidTooLow {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: s.init_qty,
            low_bid,
            lowest_acceptable_bid: s.init_bid,
        };
        assert_eq!(result, expected);

        // Success entire quantity
        let qty_taken = s.init_qty;
        market
            .lock_buy(s.buy_kind.clone(), qty_taken, s.init_bid, s.trader.clone())
            .unwrap();

        // Fail after locking all quantity of USD available
        let qty = 0.1f32;
        let result = market
            .lock_buy(s.buy_kind.clone(), qty, s.init_bid, s.trader.clone())
            .unwrap_err();
        let expected = LockBuyError::InsufficientGoodQuantityAvailable {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: qty,
            available_good_quantity: s.init_qty - qty_taken,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn buy() {
        let s = TestMarketSetup::new();
        let mut market = s.market.borrow_mut();
        let token = market
            .lock_buy(s.buy_kind.clone(), s.init_qty, s.init_bid, s.trader)
            .unwrap();

        // Fail on wrong token (while cash not default to test priority)
        let invalid_token = "".to_string();
        let invalid_kind = USD;
        let result = market
            .buy(
                invalid_token.clone(),
                &mut Good::new(invalid_kind.clone(), s.init_bid),
            )
            .unwrap_err();
        let expected = BuyError::UnrecognizedToken {
            unrecognized_token: invalid_token,
        };
        assert_eq!(result, expected);

        // Fail on cash not default (while quantity insufficient)
        let insufficient_qty = s.init_bid - 0.1f32;
        let mut cash = Good::new(invalid_kind.clone(), insufficient_qty);
        let result = market.buy(token.clone(), &mut cash).unwrap_err();
        let expected = BuyError::GoodKindNotDefault {
            non_default_good_kind: invalid_kind,
        };
        assert_eq!(result, expected);

        // Fail on quantity insufficient
        let mut cash = Good::new(DEFAULT_GOOD_KIND, insufficient_qty);
        let result = market.buy(token.clone(), &mut cash).unwrap_err();
        let expected = BuyError::InsufficientGoodQuantity {
            contained_quantity: insufficient_qty,
            pre_agreed_quantity: s.init_bid,
        };
        assert_eq!(result, expected);

        // Check success
        let mut cash = Good::new(DEFAULT_GOOD_KIND, s.init_bid);
        let result = market.buy(token, &mut cash).unwrap();
        let expected = Good::new(s.buy_kind, s.init_qty);
        assert_eq!(result, expected)
    }
}

#[cfg(test)]
mod test_sell {
    use crate::{market::SOLMarket, market_test::TRADER_NAME};
    use unitn_market_2022::{
        good::{self, consts::DEFAULT_GOOD_KIND, good::Good, good_kind::GoodKind},
        market::{LockSellError, Market, MarketGetterError, SellError},
    };

    #[test]
    fn test_get_sell_price() {
        // identical to test_get_buy_price but with some changes
        let market_start_quantity = 1000.0;
        let preset_quantity = 15.0;

        let mrkt_bind = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );
        let market = mrkt_bind.borrow();

        // Fail on negative quantity
        let neg_qty = -1f32;
        let result = market.get_buy_price(GoodKind::USD, neg_qty).unwrap_err();
        let expected = MarketGetterError::NonPositiveQuantityAsked;
        assert_eq!(result, expected);

        // Fail on insufficient Quantity
        let extra_qty = market_start_quantity + 1f32;
        let result = market.get_buy_price(GoodKind::USD, extra_qty).unwrap_err();
        let expected = MarketGetterError::InsufficientGoodQuantityAvailable {
            requested_good_kind: GoodKind::USD,
            requested_good_quantity: extra_qty,
            available_good_quantity: market_start_quantity,
        };
        assert_eq!(result, expected);

        // Success with total amount
        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds {
            let result = market
                .get_buy_price(kind.clone(), market_start_quantity)
                .unwrap();
            let expected = market_start_quantity / kind.get_default_exchange_rate(); // market sell price = default exchange rate when init
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_lock_sell() {
        let market_start_quantity = 1000.0;
        let preset_quantity = 15.0;

        let mrkt_bind = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );
        let mut market = mrkt_bind.borrow_mut();

        let kind_for_this_test = GoodKind::USD;

        // Fail on negative quantity
        let neg_qty = -1f32;
        let result = market
            .lock_sell(
                kind_for_this_test.clone(),
                neg_qty,
                1.0,
                TRADER_NAME.to_string(),
            )
            .unwrap_err();
        let expected = LockSellError::NonPositiveQuantityToSell {
            negative_quantity_to_sell: neg_qty,
        };
        assert_eq!(result, expected);

        // Fail on negative bid (while insufficient quantity)
        let neg_offer = -10.0;
        let result = market
            .lock_sell(
                kind_for_this_test.clone(),
                preset_quantity,
                neg_offer,
                TRADER_NAME.to_string(),
            )
            .unwrap_err();
        let expected = LockSellError::NonPositiveOffer {
            negative_offer: neg_offer,
        };
        assert_eq!(result, expected);

        // Fail on insufficient default good quantity = not enough money!
        let offer_too_high = market_start_quantity + 1.0;
        let result = market
            .lock_sell(
                kind_for_this_test.clone(),
                preset_quantity,
                offer_too_high,
                TRADER_NAME.to_string(),
            )
            .unwrap_err();
        let expected = LockSellError::InsufficientDefaultGoodQuantityAvailable {
            //change here in case changes in market.rs error are reverted
            offered_good_kind: DEFAULT_GOOD_KIND,
            offered_good_quantity: offer_too_high,
            available_good_quantity: market_start_quantity,
        };
        assert_eq!(result, expected);

        // Fail on offer too high
        let mut good_offer = market
            .get_sell_price(kind_for_this_test.clone(), preset_quantity)
            .ok()
            .unwrap();
        let offer_too_high = good_offer + 1.0;
        let result = market
            .lock_sell(
                kind_for_this_test.clone(),
                preset_quantity,
                offer_too_high,
                TRADER_NAME.to_string(),
            )
            .unwrap_err();
        let expected = LockSellError::OfferTooHigh {
            offered_good_kind: kind_for_this_test.clone(),
            offered_good_quantity: preset_quantity,
            high_offer: offer_too_high,
            highest_acceptable_offer: good_offer,
        };
        assert_eq!(result, expected);

        // // Success entire quantity
        let full_quantity = market_start_quantity;
        let full_price = market
            .get_sell_price(kind_for_this_test.clone(), full_quantity)
            .ok()
            .unwrap();
        market
            .lock_buy(
                kind_for_this_test.clone(),
                full_quantity,
                full_price,
                TRADER_NAME.to_string(),
            )
            .unwrap();
    }

    #[test]
    fn test_sell() {
        let market_start_quantity = 1000.0;

        let mrkt_bind = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );
        let mut market = mrkt_bind.borrow_mut();

        let kind_for_this_test = GoodKind::USD;
        let preset_quantity = 15.0;
        let right_offer = market
            .get_sell_price(kind_for_this_test.clone(), preset_quantity)
            .ok()
            .unwrap();

        let token = market
            .lock_sell(
                kind_for_this_test.clone(),
                preset_quantity,
                right_offer,
                TRADER_NAME.to_string(),
            )
            .unwrap();

        // Fail on wrong token (while cash not default to test priority)
        let invalid_token = "".to_string();
        let invalid_kind = GoodKind::EUR;
        let result = market
            .sell(
                invalid_token.clone(),
                &mut Good::new(invalid_kind.clone(), right_offer),
            )
            .unwrap_err();
        let expected = SellError::UnrecognizedToken {
            unrecognized_token: invalid_token,
        };
        assert_eq!(result, expected);

        // Fail if passed cash
        let mut cash = Good::new(invalid_kind.clone(), preset_quantity);
        let result = market.sell(token.clone(), &mut cash).unwrap_err();
        let expected = SellError::WrongGoodKind {
            wrong_good_kind: invalid_kind.clone(),
            pre_agreed_kind: kind_for_this_test.clone(),
        };
        assert_eq!(result, expected);

        // Fail on quantity insufficient
        let mut good_to_sell = Good::new(kind_for_this_test.clone(), preset_quantity - 1.0);
        let result = market.sell(token.clone(), &mut good_to_sell).unwrap_err();
        let expected = SellError::InsufficientGoodQuantity {
            contained_quantity: preset_quantity - 1.0,
            pre_agreed_quantity: preset_quantity,
        };
        assert_eq!(result, expected);

        // Check success
        let mut good_to_sell = Good::new(kind_for_this_test.clone(), preset_quantity);
        let result = market.sell(token.clone(), &mut good_to_sell).unwrap();
        let expected = Good::new(DEFAULT_GOOD_KIND, right_offer);
        assert_eq!(result, expected);

        //try to reuse the token, it shouldn't be recognized
        let result = market.sell(token.clone(), &mut good_to_sell).unwrap_err();
        let expected = SellError::UnrecognizedToken {
            unrecognized_token: token,
        };
        assert_eq!(result, expected);
    }
}

#[test]
fn price_unsold_decrease_over_time() {
    let market_start_quantity = 1000.0;
    let mrkt_bind = SOLMarket::new_with_quantities(
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
    );

    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN, GoodKind::EUR];
    for kind in kinds {
        let starting_price = mrkt_bind
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        wait_one_day!(mrkt_bind);

        let price_after_waiting = mrkt_bind
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        if kind.eq(&GoodKind::EUR) {
            assert_eq!(starting_price, price_after_waiting);
        } else {
            assert!(starting_price > price_after_waiting);
        }
    }
}

#[test]
fn price_changes_waiting() {
    let market_start_quantity = 1000.0;
    let mrkt_bind = SOLMarket::new_with_quantities(
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
    );

    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN, GoodKind::EUR];
    for kind in kinds {
        let starting_price = mrkt_bind
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        wait_one_day!(mrkt_bind);

        let price_after_waiting = mrkt_bind
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        if kind.eq(&GoodKind::EUR) {
            assert_eq!(starting_price, price_after_waiting);
        } else {
            assert_ne!(starting_price, price_after_waiting);
        }
    }
}

#[test]
fn test_price_change_after_buy() {
    // use crate::good::good_kind::{*};

    let preset_quantity = 15.0;
    let market_start_quantity = 1000.0;
    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

    for kind in kinds {
        let mut market = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        let starting_price = market
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        let bid = market
            .borrow()
            .get_buy_price(kind.clone(), preset_quantity)
            .ok()
            .unwrap();
        let token = market
            .borrow_mut()
            .lock_buy(kind.clone(), preset_quantity, bid, String::from("test"))
            .unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, bid);
        market.borrow_mut().buy(token, &mut cash);

        let price_after_trade = market
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        assert_ne!(starting_price, price_after_trade);
    }

    // do it again!
    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    for kind in kinds {
        let mut market = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        let starting_price = market
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        let bid = market
            .borrow()
            .get_buy_price(kind.clone(), preset_quantity)
            .ok()
            .unwrap();
        let token = market
            .borrow_mut()
            .lock_buy(kind.clone(), preset_quantity, bid, String::from("test"))
            .unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, bid);
        market.borrow_mut().buy(token, &mut cash);

        let price_after_trade = market
            .borrow()
            .get_buy_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        assert_ne!(starting_price, price_after_trade);
    }
}

// test if the selling price of a good changes after selling that good
// SOL group
#[test]
fn test_price_change_after_sell() {
    // use crate::good::good_kind::{*};
    // use crate::good::{good::Good, consts::DEFAULT_GOOD_KIND};

    let preset_quantity = 15.0;
    let market_start_quantity = 1000.0;

    //execute the test for each goodkind
    //EUR has been excluded
    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    for kind in kinds {
        //init again for each good trade
        let mut market = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        let starting_price = market
            .borrow()
            .get_sell_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        //sell the good
        let offer = market
            .borrow()
            .get_sell_price(kind.clone(), preset_quantity)
            .ok()
            .unwrap();
        let token = market
            .borrow_mut()
            .lock_sell(kind.clone(), preset_quantity, offer, String::from("test"))
            .unwrap();
        let mut good_to_sell = Good::new(kind.clone(), preset_quantity);
        market.borrow_mut().sell(token, &mut good_to_sell);

        //get the price to compare
        let price_after_trade = market
            .borrow()
            .get_sell_price(kind.clone(), 1.0)
            .ok()
            .unwrap();

        assert_ne!(starting_price, price_after_trade);
    }
}

#[test]
fn test_token_duration_and_passing_days() {
    let preset_quantity = 15.0;
    let market_start_quantity = 1000.0;

    let mut markt_bind = SOLMarket::new_with_quantities(
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
    );
    // let mut market = markt_bind.borrow_mut();

    let kind_for_this_test = GoodKind::USD;
    let preset_quantity = 15.0;
    let right_bid = markt_bind
        .borrow_mut()
        .get_buy_price(kind_for_this_test.clone(), preset_quantity)
        .ok()
        .unwrap();
    let right_offer = markt_bind
        .borrow_mut()
        .get_sell_price(kind_for_this_test.clone(), preset_quantity)
        .ok()
        .unwrap();

    let expiring_buy_token = markt_bind
        .borrow_mut()
        .lock_buy(
            kind_for_this_test.clone(),
            preset_quantity,
            right_bid,
            TRADER_NAME.to_string(),
        )
        .ok()
        .unwrap();
    let expiring_sell_token = markt_bind
        .borrow_mut()
        .lock_sell(
            kind_for_this_test.clone(),
            preset_quantity,
            right_offer,
            TRADER_NAME.to_string(),
        )
        .ok()
        .unwrap();

    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);
    wait_one_day!(markt_bind);

    let expected_for_buy = BuyError::ExpiredToken {
        expired_token: expiring_buy_token.clone(),
    };
    let expected_for_sell = SellError::ExpiredToken {
        expired_token: expiring_sell_token.clone(),
    };

    let res_buy = markt_bind
        .borrow_mut()
        .buy(
            expiring_buy_token,
            &mut Good::new(DEFAULT_GOOD_KIND, preset_quantity),
        )
        .unwrap_err();
    let res_sell = markt_bind
        .borrow_mut()
        .sell(
            expiring_sell_token,
            &mut Good::new(kind_for_this_test.clone(), preset_quantity),
        )
        .unwrap_err();

    assert_eq!(res_buy, expected_for_buy);
    assert_eq!(res_sell, expected_for_sell);
}

#[test]
fn should_return_err_if_lock_sell_limit_exceeded() {
    // given random market
    let market = SOLMarket::new_random();

    // when 20 buy locks were performed
    (1..21).for_each(|_i: i32| {
        let _r = market
            .borrow_mut()
            .lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
    });
    
    // then 21 lock sell try will return an MaxAllowedLocksReached error
    let result = market
        .borrow_mut()
        .lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
    assert_eq!(result.unwrap_err(), LockSellError::MaxAllowedLocksReached)
}

#[test]
fn should_return_err_if_lock_buy_limit_exceeded() {
    // given random market
    let market = SOLMarket::new_random();
    
    // when 20 buy locks were performed
    (1..21).for_each(|_i: i32| {
        let _r = market
            .borrow_mut()
            .lock_buy(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
    });
    
    // then 21 lock buy try will return an MaxAllowedLocksReached error
    let result = market
        .borrow_mut()
        .lock_buy(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
    assert_eq!(result.unwrap_err(), LockBuyError::MaxAllowedLocksReached)
}
