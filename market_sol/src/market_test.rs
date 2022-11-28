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

#[test]
fn fail_lock_buy_on_locked() {
    let q = 20.0;
    let trader_name = String::from(TRADER_NAME);
    let binding = SOLMarket::new_with_quantities(q, q, q, q);
    let mut market = binding.borrow_mut();
    let token_result = market.lock_buy(GoodKind::USD, 1.01, 1f32, trader_name);
    assert!(token_result.is_ok());
    let token = token_result.unwrap();
    let trader_name = String::from(TRADER_NAME);
    let result = market
        .lock_buy(GoodKind::USD, 1.01, 100f32, trader_name)
        .expect_err("DID NOT FAIL");
    let expected = LockBuyError::GoodAlreadyLocked { token: token };
}

#[test]
fn fail_lock_buy_quantity() {
    let trader_name = String::from(TRADER_NAME);

    let q = 20.0;
    let binding = SOLMarket::new_with_quantities(q, q, q, q);
    let mut market = binding.borrow_mut();
    let requested_quantity = q + 0.1;
    let bid = f32::MAX;
    let result = market
        .lock_buy(GoodKind::USD, requested_quantity, bid, trader_name)
        .expect_err("DID NOT FAIL");
    let expected = LockBuyError::InsufficientGoodQuantityAvailable {
        requested_good_kind: GoodKind::USD,
        requested_good_quantity: requested_quantity,
        available_good_quantity: q,
    };
}

#[test]
fn fail_lock_buy_price() {
    let binding = SOLMarket::new_with_quantities(0.0, 0.0, 0.0, 0.0);
    let mut market = binding.borrow_mut();
    let trader_name = String::from(TRADER_NAME);
    let quantity_to_buy = 1.00;
    let bid = 99.9;
    let result = market
        .lock_buy(GoodKind::USD, quantity_to_buy, bid, trader_name)
        .expect_err("DID NOT FAIL");
    let expected = LockBuyError::BidTooLow {
        requested_good_kind: GoodKind::USD,
        requested_good_quantity: quantity_to_buy,
        low_bid: bid,
        lowest_acceptable_bid: 999.0, //TODO
    };

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)
}

#[test]
fn success_lock_buy() {
    let binding = SOLMarket::new_with_quantities(0.0, 0.0, 0.0, 0.0);
    let mut market = binding.borrow_mut();
    let trader_name = String::from(TRADER_NAME);

    let g = GoodKind::USD;
    let p = 1.01;
    let q = 100f32;
    let d = trader_name;

    let result = market.lock_buy(g.clone(), p, q, d.clone()).ok().unwrap();
    let mut hasher = DefaultHasher::new();
    (g.to_string(), p.to_string(), q.to_string(), d).hash(&mut hasher);
    let expected = hasher.finish().to_string();

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)
}

#[test]
fn success_buy() {
    let binding = SOLMarket::new_with_quantities(0.0, 0.0, 0.0, 0.0);
    let mut market = binding.borrow_mut();
    let trader_name = String::from(TRADER_NAME);

    let g = GoodKind::USD;
    let p = 1.01;
    let q = 100f32;
    let d = trader_name;

    let token = market.lock_buy(g.clone(), p, q, d.clone()).ok().unwrap();
    let result = market.buy(token, &mut Good::new(GoodKind::EUR, q / p));
    let expected = Good::new(g, q);

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)
}
