use crate::market::SOLMarket;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::{LockBuyError, Market};

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

#[cfg(test)]
mod test_buy {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::{LockBuyError, Market};
    use crate::market::SOLMarket;

    // Setup a struct with default test value
    struct Setup {
        market: Rc<RefCell<dyn Market>>,
        buy_kind: GoodKind,
        qty: f32,
        bid: f32,
        trader: String,
    }

    impl Setup {
        fn new() -> Self {
            let qty = 100f32;
            let market = SOLMarket::new_with_quantities(qty, qty, qty, qty);
            let buy_kind = GoodKind::USD;
            let bid = market.borrow().get_goods().iter().find_map(
                |gl| { if gl.good_kind.eq(&buy_kind) { Some(gl.exchange_rate_sell) } else { None } }).unwrap();
            let trader = "foobar".to_string();
            Self {
                market,
                buy_kind,
                qty,
                bid,
                trader,
            }
        }
    }


    #[test]
    fn fail_lock_neg_qty() {
        let s = Setup::new();
        let mut market = s.market.borrow_mut();
        let neg_qty = -s.qty;
        let result = market.lock_buy(GoodKind::USD, neg_qty, s.bid, s.trader).unwrap_err();
        let expected = LockBuyError::NonPositiveQuantityToBuy { negative_quantity_to_buy: neg_qty };
        assert_eq!(result, expected);
    }

    #[test]
    fn fail_lock_neg_bid() {
        let s = Setup::new();
        let mut market = s.market.borrow_mut();
        let neg_bid = -s.bid;
        let result = market.lock_buy(s.buy_kind, s.qty, neg_bid, s.trader).unwrap_err();
        let expected = LockBuyError::NonPositiveBid { negative_bid: neg_bid };

        assert_eq!(result, expected)
    }

    #[test]
    fn fail_lock_insufficient_qty() {
        let s = Setup::new();
        let mut market = s.market.borrow_mut();
        let extra_qty = s.qty + 1f32;
        let result = market.lock_buy(s.buy_kind.clone(), extra_qty, s.bid, s.trader).unwrap_err();
        let expected = LockBuyError::InsufficientGoodQuantityAvailable {
            requested_good_kind: s.buy_kind,
            requested_good_quantity: extra_qty,
            available_good_quantity: s.qty,
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn fail_lock_bid_low() {
        let s = Setup::new();
        let mut market = s.market.borrow_mut();
        let low_bid = s.bid - 1f32;
        let result = market.lock_buy(s.buy_kind.clone(), s.qty, low_bid, s.trader).unwrap_err();
        let expected = LockBuyError::BidTooLow {
            requested_good_kind: s.buy_kind,
            requested_good_quantity: s.qty,
            low_bid,
            lowest_acceptable_bid: s.bid
        };

        assert_eq!(result, expected)
    }

    #[test]
    fn success_lock_buy() {
        let s = Setup::new();
        let mut market = s.market.borrow_mut();
        market.lock_buy(s.buy_kind, s.qty, s.bid, s.trader).unwrap();
    }

    // TODO: Implement tests for buy method

    #[test]
    fn success_buy() {
        let s = Setup::new();
        let mut market = s.market.borrow_mut();
        let token = market.lock_buy(s.buy_kind.clone(), s.qty, s.bid, s.trader).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, s.qty / s.bid);
        let result = market.buy(token, &mut cash).unwrap();
        let expected = Good::new(s.buy_kind, s.qty);
        assert_eq!(result, expected)
    }
}
