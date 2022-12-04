#[cfg(test)]
mod solmarket_tests {
    use crate::market::{SOLMarket, LOCK_LIMIT, TOKEN_DURATION};
    use std::cell::RefCell;
    use std::rc::Rc;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
    use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, SellError};
    use unitn_market_2022::wait_one_day;
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
            let result = market.get_buy_price(s.buy_kind, neg_qty).unwrap_err();
            let expected = MarketGetterError::NonPositiveQuantityAsked;
            assert_eq!(result, expected);

            // Fail on insufficient Quantity
            let extra_qty = s.init_qty + 1f32;
            let result = market.get_buy_price(s.buy_kind, extra_qty).unwrap_err();
            let expected = MarketGetterError::InsufficientGoodQuantityAvailable {
                requested_good_kind: s.buy_kind,
                requested_good_quantity: extra_qty,
                available_good_quantity: s.init_qty,
            };
            assert_eq!(result, expected);

            // Success with total amount
            let kinds = vec![EUR, USD, YEN, YUAN];
            for k in kinds.iter() {
                let result = market.get_buy_price(*k, s.init_qty).unwrap();
                let market_sell_price = market
                    .get_goods()
                    .iter()
                    .find(|gl| gl.good_kind.eq(k))
                    .unwrap()
                    .exchange_rate_sell;
                let expected = s.init_qty / market_sell_price;
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
                .lock_buy(s.buy_kind, neg_qty, neg_bid - 1f32, s.trader.clone())
                .unwrap_err();
            let expected = LockBuyError::NonPositiveQuantityToBuy {
                negative_quantity_to_buy: neg_qty,
            };
            assert_eq!(result, expected);

            // Fail on negative bid (while insufficient quantity)
            let extra_qty = s.init_qty + 0.1f32;
            let result = market
                .lock_buy(s.buy_kind, extra_qty, neg_bid, s.trader.clone())
                .unwrap_err();
            let expected = LockBuyError::NonPositiveBid {
                negative_bid: neg_bid,
            };
            assert_eq!(result, expected);

            // Fail on insufficient quantity (while low bid)
            let low_bid = 0f32;
            let result = market
                .lock_buy(s.buy_kind, extra_qty, low_bid, s.trader.clone())
                .unwrap_err();
            let expected = LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: s.buy_kind,
                requested_good_quantity: extra_qty,
                available_good_quantity: s.init_qty,
            };
            assert_eq!(result, expected);

            // Fail on low bid
            let low_bid = s.init_bid - 0.1f32;
            let result = market
                .lock_buy(s.buy_kind, s.init_qty, low_bid, s.trader.clone())
                .unwrap_err();
            let expected = LockBuyError::BidTooLow {
                requested_good_kind: s.buy_kind,
                requested_good_quantity: s.init_qty,
                low_bid,
                lowest_acceptable_bid: s.init_bid,
            };
            assert_eq!(result, expected);

            // Success entire quantity
            let qty_taken = s.init_qty;
            market
                .lock_buy(s.buy_kind, qty_taken, s.init_bid, s.trader.clone())
                .unwrap();

            // Fail after locking all quantity of USD available
            let qty = 0.1f32;
            let result = market
                .lock_buy(s.buy_kind, qty, s.init_bid, s.trader.clone())
                .unwrap_err();
            let expected = LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: s.buy_kind,
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
                .lock_buy(s.buy_kind, s.init_qty, s.init_bid, s.trader)
                .unwrap();

            // Fail on wrong token (while cash not default to test priority)
            let invalid_token = "".to_string();
            let invalid_kind = USD;
            let result = market
                .buy(
                    invalid_token.clone(),
                    &mut Good::new(invalid_kind, s.init_bid),
                )
                .unwrap_err();
            let expected = BuyError::UnrecognizedToken {
                unrecognized_token: invalid_token,
            };
            assert_eq!(result, expected);

            // Fail on cash not default (while quantity insufficient)
            let insufficient_qty = s.init_bid - 0.1f32;
            let mut cash = Good::new(invalid_kind, insufficient_qty);
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
        use crate::market::SOLMarket;
        use unitn_market_2022::{
            good::{consts::DEFAULT_GOOD_KIND, good::Good, good_kind::GoodKind},
            market::{LockSellError, Market, MarketGetterError, SellError},
        };
        const TRADER_NAME: &str = "foobar";

        #[test]
        fn test_get_sell_price() {
            // identical to test_get_buy_price but with some changes
            let market_start_quantity = 1000.0;

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
                let result = market.get_sell_price(kind, market_start_quantity).unwrap();
                let market_buy_price = market
                    .get_goods()
                    .iter()
                    .find(|gl| gl.good_kind.eq(&kind))
                    .unwrap()
                    .exchange_rate_buy;
                let expected = market_start_quantity / market_buy_price;
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
                .lock_sell(kind_for_this_test, neg_qty, 1.0, TRADER_NAME.to_string())
                .unwrap_err();
            let expected = LockSellError::NonPositiveQuantityToSell {
                negative_quantity_to_sell: neg_qty,
            };
            assert_eq!(result, expected);

            // Fail on negative bid (while insufficient quantity)
            let neg_offer = -10.0;
            let result = market
                .lock_sell(
                    kind_for_this_test,
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
                    kind_for_this_test,
                    preset_quantity,
                    offer_too_high,
                    TRADER_NAME.to_string(),
                )
                .unwrap_err();
            let expected = LockSellError::InsufficientDefaultGoodQuantityAvailable {
                //change here in case changes in market.rs error are reverted
                offered_good_kind: kind_for_this_test,
                offered_good_quantity: preset_quantity,
                available_good_quantity: market_start_quantity,
            };
            assert_eq!(result, expected);

            // Fail on offer too high
            let good_offer = market
                .get_sell_price(kind_for_this_test, preset_quantity)
                .ok()
                .unwrap();
            let offer_too_high = good_offer + 1.0;
            let result = market
                .lock_sell(
                    kind_for_this_test,
                    preset_quantity,
                    offer_too_high,
                    TRADER_NAME.to_string(),
                )
                .unwrap_err();
            let expected = LockSellError::OfferTooHigh {
                offered_good_kind: kind_for_this_test,
                offered_good_quantity: preset_quantity,
                high_offer: offer_too_high,
                highest_acceptable_offer: good_offer,
            };
            assert_eq!(result, expected);

            // // Success entire quantity
            let full_quantity = market_start_quantity;
            let full_price = market
                .get_sell_price(kind_for_this_test, full_quantity)
                .ok()
                .unwrap();
            market
                .lock_sell(
                    kind_for_this_test,
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
                .get_sell_price(kind_for_this_test, preset_quantity)
                .ok()
                .unwrap();

            let token = market
                .lock_sell(
                    kind_for_this_test,
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
                    &mut Good::new(invalid_kind, right_offer),
                )
                .unwrap_err();
            let expected = SellError::UnrecognizedToken {
                unrecognized_token: invalid_token,
            };
            assert_eq!(result, expected);

            // Fail if passed cash
            let mut cash = Good::new(invalid_kind, preset_quantity);
            let result = market.sell(token.clone(), &mut cash).unwrap_err();
            let expected = SellError::WrongGoodKind {
                wrong_good_kind: invalid_kind,
                pre_agreed_kind: kind_for_this_test,
            };
            assert_eq!(result, expected);

            // Fail on quantity insufficient
            let mut good_to_sell = Good::new(kind_for_this_test, preset_quantity - 1.0);
            let result = market.sell(token.clone(), &mut good_to_sell).unwrap_err();
            let expected = SellError::InsufficientGoodQuantity {
                contained_quantity: preset_quantity - 1.0,
                pre_agreed_quantity: preset_quantity,
            };
            assert_eq!(result, expected);

            // Check success
            let mut good_to_sell = Good::new(kind_for_this_test, preset_quantity);
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
            let starting_price = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

            wait_one_day!(mrkt_bind);

            let price_after_waiting = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

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
            let starting_price = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

            wait_one_day!(mrkt_bind);

            let price_after_waiting = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

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
            let market = SOLMarket::new_with_quantities(
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
            );

            let starting_price = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

            let bid = market
                .borrow()
                .get_buy_price(kind, preset_quantity)
                .ok()
                .unwrap();
            let token = market
                .borrow_mut()
                .lock_buy(kind, preset_quantity, bid, String::from("test"))
                .unwrap();
            let mut cash = Good::new(DEFAULT_GOOD_KIND, bid);
            let buy_result = market.borrow_mut().buy(token, &mut cash);
            assert!(buy_result.is_ok());

            let price_after_trade = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

            assert_ne!(starting_price, price_after_trade);
        }

        // do it again!
        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds {
            let market = SOLMarket::new_with_quantities(
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
            );

            let starting_price = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

            let bid = market
                .borrow()
                .get_buy_price(kind, preset_quantity)
                .ok()
                .unwrap();
            let token = market
                .borrow_mut()
                .lock_buy(kind, preset_quantity, bid, String::from("test"))
                .unwrap();
            let mut cash = Good::new(DEFAULT_GOOD_KIND, bid);
            let buy_result = market.borrow_mut().buy(token, &mut cash);
            assert!(buy_result.is_ok());

            let price_after_trade = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

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
            let market = SOLMarket::new_with_quantities(
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
            );

            let starting_price = market.borrow().get_sell_price(kind, 1.0).ok().unwrap();

            //sell the good
            let offer = market
                .borrow()
                .get_sell_price(kind, preset_quantity)
                .ok()
                .unwrap();
            let token = market
                .borrow_mut()
                .lock_sell(kind, preset_quantity, offer, String::from("test"))
                .unwrap();
            let mut good_to_sell = Good::new(kind, preset_quantity);
            let sell_result = market.borrow_mut().sell(token, &mut good_to_sell);
            assert!(sell_result.is_ok());

            //get the price to compare
            let price_after_trade = market.borrow().get_sell_price(kind, 1.0).ok().unwrap();

            assert_ne!(starting_price, price_after_trade);
        }
    }

    #[test]
    fn buy_locks_expire() {
        let market_start_quantity = 1000.0;
        let kind_for_this_test = GoodKind::USD;
        let preset_quantity = 15.0;

        let markt_bind = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        //Create a buy lock
        let right_bid = markt_bind
            .borrow_mut()
            .get_buy_price(kind_for_this_test, preset_quantity)
            .unwrap();
        let expiring_buy_token = markt_bind
            .borrow_mut()
            .lock_buy(
                kind_for_this_test,
                preset_quantity,
                right_bid,
                TRADER_NAME.to_string(),
            )
            .unwrap();

        //Wait the minimum days to make the locks expire (TOKEN_DURATION)
        for _ in 0..TOKEN_DURATION {
            wait_one_day!(markt_bind);
        }

        //Have to re-declare it here otherwise wait_one_day will panic due to two mutable references
        let mut market = markt_bind.borrow_mut();

        //Try to finish the buy
        let res_buy = market
            .buy(
                expiring_buy_token.clone(),
                &mut Good::new(DEFAULT_GOOD_KIND, preset_quantity),
            )
            .unwrap_err();

        //Compute the expected error
        let expected_for_buy = BuyError::ExpiredToken {
            expired_token: expiring_buy_token,
        };

        //Check we got this error
        assert_eq!(res_buy, expected_for_buy);
    }

    #[test]
    fn sell_locks_expire() {
        let market_start_quantity = 1000.0;
        let kind_for_this_test = GoodKind::USD;
        let preset_quantity = 15.0;

        let markt_bind = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        //Create a sell lock
        let right_offer = markt_bind
            .borrow_mut()
            .get_sell_price(kind_for_this_test, preset_quantity)
            .unwrap();
        let expiring_sell_token = markt_bind
            .borrow_mut()
            .lock_sell(
                kind_for_this_test,
                preset_quantity,
                right_offer,
                TRADER_NAME.to_string(),
            )
            .unwrap();

        //Wait the minimum days to make the locks expire (TOKEN_DURATION)
        for _ in 0..TOKEN_DURATION {
            wait_one_day!(markt_bind);
        }

        //Have to re-declare it here otherwise wait_one_day will panic due to two mutable references
        let mut market = markt_bind.borrow_mut();

        //Try to finish the sell
        let res_sell = market
            .sell(
                expiring_sell_token.clone(),
                &mut Good::new(kind_for_this_test, preset_quantity),
            )
            .unwrap_err();

        //Compute the expected error
        let expected_for_sell = SellError::ExpiredToken {
            expired_token: expiring_sell_token,
        };

        //Check we got those error
        assert_eq!(res_sell, expected_for_sell);
    }

    #[test]
    fn should_return_err_if_lock_sell_limit_exceeded() {
        // given random market
        let market_ref = SOLMarket::new_random();
        let mut market = market_ref.borrow_mut();

        // Create the maximum amount of allowed sell locks
        for i in 0..LOCK_LIMIT {
            let r = market.lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
            assert!(r.is_ok(), "Lock number {i} should be successful");
        }

        // Test than next lock returns a MaxAllowedLocksReached error
        let result = market.lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
        assert_eq!(result.unwrap_err(), LockSellError::MaxAllowedLocksReached);
    }

    #[test]
    fn should_return_err_if_lock_buy_limit_exceeded() {
        // given random market
        let market_ref = SOLMarket::new_random();
        let mut market = market_ref.borrow_mut();

        // Create the maximum amount of allowed buy locks
        for i in 0..LOCK_LIMIT {
            let r = market.lock_buy(GoodKind::EUR, 1.0, f32::MAX, TRADER_NAME.to_string());
            assert!(r.is_ok(), "Lock number {i} should be successful");
        }

        // Test than next lock returns a MaxAllowedLocksReached error
        let result = market.lock_buy(GoodKind::EUR, 1.0, f32::MAX, TRADER_NAME.to_string());
        assert_eq!(result.unwrap_err(), LockBuyError::MaxAllowedLocksReached);
    }

    #[test]
    fn should_return_err_if_lock_buy_and_sell_limit_exceeded() {
        // given random market
        let market_ref = SOLMarket::new_random();
        let mut market = market_ref.borrow_mut();

        // Create the maximum amount of allowed buy locks
        for i in 0..LOCK_LIMIT {
            let r = market.lock_buy(GoodKind::EUR, 1.0, f32::MAX, TRADER_NAME.to_string());
            assert!(r.is_ok(), "Buy lock number {i} should be successful");
        }

        // Create the maximum amount of allowed sell locks
        for i in 0..LOCK_LIMIT {
            let r = market.lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
            assert!(r.is_ok(), "Sell lock number {i} should be successful");
        }

        // Test than next buy lock returns a MaxAllowedLocksReached error
        let result = market.lock_buy(GoodKind::EUR, 1.0, f32::MAX, TRADER_NAME.to_string());
        assert_eq!(result.unwrap_err(), LockBuyError::MaxAllowedLocksReached);

        // Test than next lock returns a MaxAllowedLocksReached error
        let result = market.lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
        assert_eq!(result.unwrap_err(), LockSellError::MaxAllowedLocksReached);
    }

    #[cfg(test)]
    mod extern_test {
        //import here the market_test module and the Market trait
        use unitn_market_2022::market::market_test;
        //import here your implementation of the market
        use crate::market::SOLMarket;
        //make an alias to your market
        type MarketType = SOLMarket;
        //test every aspect of your market using the generic function
        #[test]
        fn test_name() {
            market_test::test_name::<MarketType>();
        }
        #[test]
        fn test_get_buy_price_success() {
            market_test::test_get_buy_price_success::<MarketType>();
        }
        #[test]
        fn test_get_buy_price_non_positive_error() {
            market_test::test_get_buy_price_non_positive_error::<MarketType>();
        }
        #[test]
        fn test_get_buy_price_insufficient_qty_error() {
            market_test::test_get_buy_price_insufficient_qty_error::<MarketType>();
        }
        #[test]
        fn test_get_sell_price_success() {
            market_test::test_get_sell_price_success::<MarketType>();
        }
        #[test]
        fn test_get_sell_price_non_positive_error() {
            market_test::test_get_sell_price_non_positive_error::<MarketType>();
        }
        #[test]
        fn test_deadlock_prevention() {
            market_test::test_deadlock_prevention::<MarketType>();
        }
        #[test]
        fn test_new_random() {
            market_test::test_new_random::<MarketType>();
        }
        #[test]
        fn test_price_change_after_buy() {
            market_test::test_price_change_after_buy::<MarketType>();
        }
        #[test]
        fn price_changes_waiting() {
            market_test::price_changes_waiting::<MarketType>();
        }
        #[test]
        fn test_price_change_after_sell() {
            market_test::test_price_change_after_sell::<MarketType>();
        }
        #[test]
        fn should_initialize_with_right_quantity() {
            market_test::should_initialize_with_right_quantity::<MarketType>();
        }
        #[test]
        fn new_random_should_not_exceeed_starting_capital() {
            market_test::new_random_should_not_exceeed_starting_capital::<MarketType>();
        }
        #[test]
        fn test_sell_success() {
            market_test::test_sell_success::<MarketType>();
        }
        #[test]
        fn test_sell_err_unrecognized_token() {
            market_test::test_sell_err_unrecognized_token::<MarketType>();
        }
        #[test]
        fn test_sell_err_expired_token() {
            market_test::test_sell_err_expired_token::<MarketType>();
        }
        #[test]
        fn test_sell_err_wrong_good_kind() {
            market_test::test_sell_err_wrong_good_kind::<MarketType>();
        }
        #[test]
        fn test_sell_err_insufficient_good_quantity() {
            market_test::test_sell_err_insufficient_good_quantity::<MarketType>();
        }
        #[test]
        fn test_lock_sell_non_positive_offer() {
            market_test::test_lock_sell_nonPositiveOffer::<MarketType>();
        }
        #[test]
        fn test_lock_sell_default_good_already_locked() {
            // Our market allows more than 1 lock per good
            // market_test::test_lock_sell_defaultGoodAlreadyLocked::<MarketType>();
        }
        #[test]
        fn test_lock_sell_max_allowed_locks_reached() {
            market_test::test_lock_sell_maxAllowedLocksReached::<MarketType>();
        }
        #[test]
        fn test_lock_sell_insufficient_default_good_quantity_available() {
            market_test::test_lock_sell_insufficientDefaultGoodQuantityAvailable::<MarketType>();
        }
        #[test]
        fn test_lock_sell_offer_too_high() {
            market_test::test_lock_sell_offerTooHigh::<MarketType>();
        }
        #[test]
        fn test_working_function_lock_sell_token() {
            //test_working_function_lock_sell_token::test_lock_sell_offerTooHigh::<MarketType>();
        }
    }
}
