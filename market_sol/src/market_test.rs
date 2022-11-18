use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{MarketError, MarketTrait};
use crate::market::Market;

#[test]
fn should_return_markets_name() {
    // given
    let market = Market::new();
    // when
    let name = market.get_market_name();
    // then
    assert_eq!("SOL", name)
}

#[test]
fn fail_lock_buy_on_locked() {
    let mut market = Market::new();
    let token = market.lock_trader_buy_from_market(GoodKind::USD, 1.01, 1f32, "Farouk".to_string());
    let result = market.lock_trader_buy_from_market(GoodKind::USD, 1.01, 100f32, "Farouk".to_string()).expect_err("DID NOT FAILED");
    let expected = MarketError::GoodAlreadyLocked();

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)

}

#[test]
fn fail_lock_buy_quantity() {
    let mut market = Market::new();
    let result = market.lock_trader_buy_from_market(GoodKind::USD, 1.04, 100.01, "Farouk".to_string()).expect_err("DID NOT FAILED");
    let expected = MarketError::NotEnoughQuantity();

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)
}

#[test]
fn fail_lock_buy_price() {
    let mut market = Market::new();
    let result = market.lock_trader_buy_from_market(GoodKind::USD, 1.00, 99.9, "Farouk".to_string()).expect_err("DID NOT FAILED");
    let expected = MarketError::OfferTooLow();

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)
}

#[test]
fn success_lock_buy() {
    let mut market = Market::new();

    let g = GoodKind::USD;
    let p = 1.01;
    let q = 100f32;
    let d = "Farouk".to_string();

    let result = market.lock_trader_buy_from_market(g.clone(), p, q, d.clone()).ok().unwrap();
    let mut hasher = DefaultHasher::new();
    (g.to_string(), p.to_string(), q.to_string(), d).hash(&mut hasher);
    let expected = hasher.finish().to_string();

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)
}

#[test]
fn success_buy() {
    let mut market = Market::new();

    let g = GoodKind::USD;
    let p = 1.01;
    let q = 100f32;
    let d = "Farouk".to_string();

    let token = market.lock_trader_buy_from_market(g.clone(), p, q, d.clone()).ok().unwrap();
    let result = market.trader_buy_from_market(token, &mut Good::new(GoodKind::EUR, q / p));
    let expected = Good::new(g, q);

    // Can't assert_eq because: `MarketError` cannot be formatted using `{:?}` because it doesn't implement `Debug`
    // assert_eq!(result, expected)
}

