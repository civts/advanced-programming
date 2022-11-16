use unitn_market_2022::market::MarketTrait;
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