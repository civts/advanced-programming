use crate::sol_market::{SOLMarket, LOCK_LIMIT};
use unitn_market_2022::{
    good::good_kind::GoodKind,
    market::{LockBuyError, LockSellError, Market},
};

const TRADER_NAME: &str = "foobar";

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

    // Test than next buy lock returns a MaxAllowedLocksReached error
    let result = market.lock_buy(GoodKind::EUR, 1.0, f32::MAX, TRADER_NAME.to_string());
    assert_eq!(result.unwrap_err(), LockBuyError::MaxAllowedLocksReached);

    // Create the maximum amount of allowed sell locks
    for i in 0..LOCK_LIMIT {
        let r = market.lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
        assert!(r.is_ok(), "Sell lock number {i} should be successful");
    }

    // Test than next lock returns a MaxAllowedLocksReached error
    let result = market.lock_sell(GoodKind::EUR, 1.0, 1.0, TRADER_NAME.to_string());
    assert_eq!(result.unwrap_err(), LockSellError::MaxAllowedLocksReached);
}
