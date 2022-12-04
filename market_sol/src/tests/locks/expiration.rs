use unitn_market_2022::{
    good::{consts::DEFAULT_GOOD_KIND, good::Good, good_kind::GoodKind},
    market::{BuyError, Market, SellError},
    wait_one_day,
};

use crate::market::{SOLMarket, TOKEN_DURATION};

const TRADER_NAME: &str = "foobar";

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
