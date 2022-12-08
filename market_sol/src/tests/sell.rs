mod test_sell {
    use crate::lib::market::sol_market::SOLMarket;
    use unitn_market_2022::{
        good::{consts::DEFAULT_GOOD_KIND, good::Good, good_kind::GoodKind},
        market::{Market, MarketGetterError, SellError},
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
            let result = market.get_sell_price(kind, market_start_quantity);
            assert!(result.is_ok());
            let price = result.unwrap();
            assert!(price > 0.0);
        }
    }

    mod lock_sell {
        use unitn_market_2022::{
            good::good_kind::GoodKind,
            market::{LockSellError, Market},
        };

        use crate::{lib::market::sol_market::SOLMarket, tests::sell::test_sell::TRADER_NAME};

        #[test]
        fn should_fail_on_negative_quantity() {
            let market_start_quantity = 1000.0;

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
        }

        #[test]
        fn should_fail_on_negative_bid() {
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
        }

        #[test]
        fn should_fail_on_not_enough_money() {
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
        }

        #[test]
        fn should_fail_on_offer_too_high() {
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

            // Fail on offer too high
            let good_offer = market
                .get_sell_price(kind_for_this_test, preset_quantity)
                .ok()
                .unwrap();
            let offer_too_high = good_offer + 0.1;
            let lock_result = market.lock_sell(
                kind_for_this_test,
                preset_quantity,
                offer_too_high,
                TRADER_NAME.to_string(),
            );
            assert!(lock_result.is_err());
            let error = lock_result.unwrap_err();
            let expected = LockSellError::OfferTooHigh {
                offered_good_kind: kind_for_this_test,
                offered_good_quantity: preset_quantity,
                high_offer: offer_too_high,
                highest_acceptable_offer: good_offer,
            };
            assert_eq!(error, expected);
        }

        #[test]
        fn should_succeed_on_entire_quantity() {
            let market_start_quantity = 1000.0;

            let mrkt_bind = SOLMarket::new_with_quantities(
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
                market_start_quantity,
            );
            let mut market = mrkt_bind.borrow_mut();

            let kind_for_this_test = GoodKind::USD;

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
