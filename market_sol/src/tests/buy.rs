mod test_buy {
    use crate::market::SOLMarket;
    use std::cell::RefCell;
    use std::rc::Rc;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind::{self, *};
    use unitn_market_2022::market::{BuyError, LockBuyError, Market, MarketGetterError};

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
