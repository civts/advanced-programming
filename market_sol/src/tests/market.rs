use crate::market::SOLMarket;
use unitn_market_2022::market::Market;

#[test]
fn get_name_should_return_markets_name() {
    // given
    let market = SOLMarket::new_with_quantities(0.0, 0.0, 0.0, 0.0);
    // when
    let name = market.borrow().get_name();
    // then
    assert_eq!("SOL", name)
}
