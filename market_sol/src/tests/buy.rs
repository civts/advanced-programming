mod test_buy {
    use crate::lib::market::sol_market::{SOLMarket, MARKET_MARGIN};
    use std::cell::RefCell;
    use std::rc::Rc;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind::{self, *};
    use unitn_market_2022::market::{BuyError, Market, MarketGetterError};

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
                        let price = market.borrow().get_buy_price(buy_kind, init_qty).unwrap();
                        Some(price)
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
            let get_buy_price_opt = market.get_buy_price(*k, s.init_qty);
            assert!(get_buy_price_opt.is_ok());
        }
    }

    #[test]
    fn should_use_a_fixed_margin_on_buy() {
        let s = TestMarketSetup::new();
        let market = s.market.borrow();

        let quantity = s.init_qty / 2.0;
        let buy_price_result = market.get_buy_price(s.buy_kind, quantity);
        assert!(buy_price_result.is_ok());
        let buy_price = buy_price_result.unwrap();
        let buy_price_plus_margin = buy_price * (1.0 + MARKET_MARGIN);
        let sell_price_result = market.get_sell_price(s.buy_kind, quantity);
        assert!(sell_price_result.is_ok());
        let sell_price = sell_price_result.unwrap();
        assert!(sell_price > buy_price);
        let difference = (buy_price_plus_margin - sell_price).abs();
        let epsilon = buy_price / 10000.0;
        assert!(difference < epsilon);
    }

    #[test]
    fn multiple_calls_to_get_buy_price_yield_same_result() {
        let s = TestMarketSetup::new();
        let market = s.market.borrow();

        let quantity = s.init_qty / 2.0;
        let buy_price_result1 = market.get_buy_price(s.buy_kind, quantity);
        assert!(buy_price_result1.is_ok());
        let buy_price1 = buy_price_result1.unwrap();
        for _ in 0..10 {
            let buy_price_result2 = market.get_buy_price(s.buy_kind, quantity);
            assert!(buy_price_result2.is_ok());
            let buy_price2 = buy_price_result2.unwrap();
            assert_eq!(buy_price2, buy_price1);
        }
    }

    #[test]
    fn multiple_calls_to_get_sell_price_yield_same_result() {
        let s = TestMarketSetup::new();
        let market = s.market.borrow();

        let quantity = s.init_qty / 2.0;
        let sell_price_result1 = market.get_sell_price(s.buy_kind, quantity);
        assert!(sell_price_result1.is_ok());
        let sell_price1 = sell_price_result1.unwrap();
        for _ in 0..10 {
            let sell_price_result2 = market.get_sell_price(s.buy_kind, quantity);
            assert!(sell_price_result2.is_ok());
            let sell_price2 = sell_price_result2.unwrap();
            assert_eq!(sell_price2, sell_price1);
        }
    }

    mod lock_buy {
        use crate::tests::buy::test_buy::TestMarketSetup;
        use unitn_market_2022::market::LockBuyError;

        mod on_negative_quantity {
            use crate::tests::buy::test_buy::TestMarketSetup;
            use unitn_market_2022::market::LockBuyError;

            #[test]
            fn fails() {
                let s = TestMarketSetup::new();
                let mut market = s.market.borrow_mut();

                // Fail on negative quantity
                let neg_qty = -1f32;
                let bid = f32::MAX;
                let result = market
                    .lock_buy(s.buy_kind, neg_qty, bid, s.trader.clone())
                    .unwrap_err();
                let expected = LockBuyError::NonPositiveQuantityToBuy {
                    negative_quantity_to_buy: neg_qty,
                };
                assert_eq!(result, expected);
            }

            #[test]
            fn fails_on_negative_quantity_before_negative_bid() {
                let s = TestMarketSetup::new();
                let mut market = s.market.borrow_mut();

                // Fail on negative quantity (while negative bid to see if priority is maintained)
                let neg_qty = -1f32;
                let neg_bid = neg_qty / s.buy_kind.get_default_exchange_rate();
                let result = market
                    .lock_buy(s.buy_kind, neg_qty, neg_bid - 1f32, s.trader.clone())
                    .unwrap_err();
                let expected = LockBuyError::NonPositiveQuantityToBuy {
                    negative_quantity_to_buy: neg_qty,
                };
                assert_eq!(result, expected);
            }
        }

        mod on_negative_bid {
            use crate::tests::buy::test_buy::TestMarketSetup;
            use unitn_market_2022::market::LockBuyError;

            #[test]
            fn fails_with_non_positive_bid_error() {
                let s = TestMarketSetup::new();
                let mut market = s.market.borrow_mut();

                // Fail on negative bid (while insufficient quantity)
                let neg_bid = -1.0;
                let result = market.lock_buy(s.buy_kind, 0.1, neg_bid, s.trader.clone());
                assert!(result.is_err());
                let expected = LockBuyError::NonPositiveBid {
                    negative_bid: neg_bid,
                };
                assert_eq!(result.unwrap_err(), expected);
            }

            #[test]
            fn fails_before_quantity_check() {
                let s = TestMarketSetup::new();
                let mut market = s.market.borrow_mut();

                // Fail on negative bid (while insufficient quantity)
                let neg_bid = -12.0;
                let result = market.lock_buy(s.buy_kind, f32::MAX, neg_bid, s.trader.clone());
                assert!(result.is_err());
                let expected = LockBuyError::NonPositiveBid {
                    negative_bid: neg_bid,
                };
                assert_eq!(result.unwrap_err(), expected);
            }
        }

        mod on_low_bid {
            use crate::tests::buy::test_buy::TestMarketSetup;
            use unitn_market_2022::market::LockBuyError;

            #[test]
            fn fails_on_insufficient_quantity_before_low_bid() {
                let s = TestMarketSetup::new();
                let mut market = s.market.borrow_mut();
                // Fail on insufficient quantity (while low bid)
                let low_bid = 0f32;
                let extra_qty = f32::MAX;
                let result = market
                    .lock_buy(s.buy_kind, extra_qty, low_bid, s.trader.clone())
                    .unwrap_err();
                let expected = LockBuyError::InsufficientGoodQuantityAvailable {
                    requested_good_kind: s.buy_kind,
                    requested_good_quantity: extra_qty,
                    available_good_quantity: s.init_qty,
                };
                assert_eq!(result, expected);
            }

            #[test]
            fn fails_on_low_bid() {
                let s = TestMarketSetup::new();
                let mut market = s.market.borrow_mut();

                // Fail on low bid
                let low_bid = 0f32;
                let result = market
                    .lock_buy(s.buy_kind, s.init_qty, low_bid, s.trader.clone())
                    .unwrap_err();
                let expected = LockBuyError::BidTooLow {
                    requested_good_kind: s.buy_kind,
                    requested_good_quantity: s.init_qty,
                    low_bid,
                    lowest_acceptable_bid: market.get_buy_price(s.buy_kind, s.init_qty).unwrap(),
                };
                assert_eq!(result, expected);
            }
        }

        #[test]
        fn fails_upon_locking_everything() {
            let s = TestMarketSetup::new();
            let mut market = s.market.borrow_mut();

            // Lock the entire quantity of a good
            let qty_taken = s.init_qty;
            let first_lock_result =
                market.lock_buy(s.buy_kind, qty_taken, s.init_bid, s.trader.clone());
            assert!(first_lock_result.is_ok());

            // Fail after locking all quantity of USD available
            let qty = 0.1f32;
            let second_lock_result = market.lock_buy(s.buy_kind, qty, s.init_bid, s.trader.clone());
            assert!(second_lock_result.is_err());
            let expected = LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: s.buy_kind,
                requested_good_quantity: qty,
                available_good_quantity: s.init_qty - qty_taken,
            };
            assert_eq!(second_lock_result.unwrap_err(), expected);
        }

        #[test]
        fn succeeds_when_bid_is_exactly_the_buy_price() {
            let s = TestMarketSetup::new();
            let mut market = s.market.borrow_mut();

            let quantity_to_buy = 0.5;
            let buy_price = market.get_buy_price(s.buy_kind, quantity_to_buy);
            assert!(buy_price.is_ok());
            let first_lock_result = market.lock_buy(
                s.buy_kind,
                quantity_to_buy,
                buy_price.unwrap(),
                s.trader.clone(),
            );
            assert!(first_lock_result.is_ok());
        }
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
