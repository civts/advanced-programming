use unitn_market_2022::{
    good::{consts::DEFAULT_GOOD_KIND, good::Good, good_kind::GoodKind},
    market::Market,
    wait_one_day,
};

use crate::lib::market::sol_market::SOLMarket;

#[test]
fn price_unsold_decrease_over_time() {
    let market_start_quantity = 1000.0;
    let mrkt_bind = SOLMarket::new_with_quantities(
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
    );

    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN, GoodKind::EUR];
    for kind in kinds {
        let starting_price = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        wait_one_day!(mrkt_bind);

        let price_after_waiting = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        if kind.eq(&GoodKind::EUR) {
            assert_eq!(starting_price, price_after_waiting);
        } else {
            assert!(starting_price > price_after_waiting);
        }
    }
}

#[test]
fn price_changes_waiting() {
    let market_start_quantity = 1000.0;
    let mrkt_bind = SOLMarket::new_with_quantities(
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
        market_start_quantity,
    );

    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN, GoodKind::EUR];
    for kind in kinds {
        let starting_price = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        wait_one_day!(mrkt_bind);

        let price_after_waiting = mrkt_bind.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        if kind.eq(&GoodKind::EUR) {
            assert_eq!(starting_price, price_after_waiting);
        } else {
            assert_ne!(starting_price, price_after_waiting);
        }
    }
}

#[test]
fn test_price_change_after_buy() {
    // use crate::good::good_kind::{*};

    let preset_quantity = 15.0;
    let market_start_quantity = 1000.0;
    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

    for kind in kinds {
        let market = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        let starting_price = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        let bid = market
            .borrow()
            .get_buy_price(kind, preset_quantity)
            .ok()
            .unwrap();
        let token = market
            .borrow_mut()
            .lock_buy(kind, preset_quantity, bid, String::from("test"))
            .unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, bid);
        let buy_result = market.borrow_mut().buy(token, &mut cash);
        assert!(buy_result.is_ok());

        let price_after_trade = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        assert_ne!(starting_price, price_after_trade);
    }

    // do it again!
    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    for kind in kinds {
        let market = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        let starting_price = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        let bid = market
            .borrow()
            .get_buy_price(kind, preset_quantity)
            .ok()
            .unwrap();
        let token = market
            .borrow_mut()
            .lock_buy(kind, preset_quantity, bid, String::from("test"))
            .unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, bid);
        let buy_result = market.borrow_mut().buy(token, &mut cash);
        assert!(buy_result.is_ok());

        let price_after_trade = market.borrow().get_buy_price(kind, 1.0).ok().unwrap();

        assert_ne!(starting_price, price_after_trade);
    }
}

// test if the selling price of a good changes after selling that good
// SOL group
#[test]
fn test_price_change_after_sell() {
    // use crate::good::good_kind::{*};
    // use crate::good::{good::Good, consts::DEFAULT_GOOD_KIND};

    let preset_quantity = 15.0;
    let market_start_quantity = 1000.0;

    //execute the test for each goodkind
    //EUR has been excluded
    let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    for kind in kinds {
        //init again for each good trade
        let market = SOLMarket::new_with_quantities(
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
            market_start_quantity,
        );

        let starting_price = market.borrow().get_sell_price(kind, 1.0).ok().unwrap();

        //sell the good
        let offer = market
            .borrow()
            .get_sell_price(kind, preset_quantity)
            .ok()
            .unwrap();
        let token = market
            .borrow_mut()
            .lock_sell(kind, preset_quantity, offer, String::from("test"))
            .unwrap();
        let mut good_to_sell = Good::new(kind, preset_quantity);
        let sell_result = market.borrow_mut().sell(token, &mut good_to_sell);
        assert!(sell_result.is_ok());

        //get the price to compare
        let price_after_trade = market.borrow().get_sell_price(kind, 1.0).ok().unwrap();

        assert_ne!(starting_price, price_after_trade);
    }
}
