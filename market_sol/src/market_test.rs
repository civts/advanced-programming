use std::cell::RefCell;
use crate::market::SOLMarket;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use std::rc::Rc;
use unitn_market_2022::good::{good::Good, good_kind::GoodKind};
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::{LockBuyError, Market};
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use unitn_market_2022::event::event::Event;

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
/// When a market gets created using the `new_with_quantities` constructor,
/// the quantities of each good in the market should correspond to the ones
/// passed as parameters.
fn should_initialize_with_right_quantity() {
    //Create market with predefined quantities
    let eur_qty = 12.0;
    let usd_qty = 42.0;
    let yen_qty = 137.0;
    let yuan_qty = 1984.0;
    let market = SOLMarket::new_with_quantities(eur_qty, yen_qty, usd_qty, yuan_qty);
    let goods = market.borrow().get_goods();
    {
        //Check USD quantity
        let usd_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::USD)
            .collect();
        assert_eq!(
            usd_vec.len(),
            1,
            "There should be only one GoodLabel for usd"
        );
        let usd_good_label = usd_vec.get(0).unwrap();
        assert_eq!(
            usd_good_label.quantity,
            usd_qty,
            "The usd quantity in the market should be equal to the one supplied in the constructor"
        );
    }
    {
        //Check EUR quantity
        let eur_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::EUR)
            .collect();
        assert_eq!(
            eur_vec.len(),
            1,
            "There should be only one GoodLabel for eur"
        );
        let eur_good_label = eur_vec.get(0).unwrap();
        assert_eq!(
            eur_good_label.quantity,
            eur_qty,
            "The eur quantity in the market should be equal to the one supplied in the constructor"
        );
    }
    {
        //Check YEN quantity
        let yen_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::YEN)
            .collect();
        assert_eq!(
            yen_vec.len(),
            1,
            "There should be only one GoodLabel for yen"
        );
        let yen_good_label = yen_vec.get(0).unwrap();
        assert_eq!(
            yen_good_label.quantity,
            yen_qty,
            "The yen quantity in the market should be equal to the one supplied in the constructor"
        );
    }
    {
        //Check YUAN quantity
        let yuan_vec: Vec<&GoodLabel> = goods
            .iter()
            .filter(|g| g.good_kind == GoodKind::YUAN)
            .collect();
        assert_eq!(
            yuan_vec.len(),
            1,
            "There should be only one GoodLabel for yuan"
        );
        let yuan_good_label = yuan_vec.get(0).unwrap();
        assert_eq!(
            yuan_good_label.quantity,
            yuan_qty,
            "The yuan quantity in the market should be equal to the one supplied in the constructor"
        );
    }
}

#[cfg(test)]
mod test_buy {
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind::*;
    use unitn_market_2022::market::{BuyError, LockBuyError, MarketGetterError};
    use super::*;

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
            let init_bid = market.borrow().get_goods().iter().find_map(
                |gl| { if gl.good_kind.eq(&buy_kind) { Some(init_qty / gl.exchange_rate_sell) } else { None } }).unwrap();
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
        let result = market.get_buy_price(s.buy_kind.clone(), neg_qty).unwrap_err();
        let expected = MarketGetterError::NonPositiveQuantityAsked;
        assert_eq!(result, expected);

        // Fail on insufficient Quantity
        let extra_qty = s.init_qty + 1f32;
        let result = market.get_buy_price(s.buy_kind.clone(), extra_qty).unwrap_err();
        let expected = MarketGetterError::InsufficientGoodQuantityAvailable {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: extra_qty,
            available_good_quantity: s.init_qty
        };
        assert_eq!(result, expected);

        // Success with total amount
        let kinds = vec![EUR, USD, YEN, YUAN];
        for k in kinds.iter() {
            let result = market.get_buy_price(k.clone(), s.init_qty).unwrap();
            let expected = s.init_qty / k.get_default_exchange_rate(); // market sell price = default exchange rate when init
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
        let result = market.lock_buy(s.buy_kind.clone(), neg_qty, neg_bid - 1f32, s.trader.clone()).unwrap_err();
        let expected = LockBuyError::NonPositiveQuantityToBuy { negative_quantity_to_buy: neg_qty };
        assert_eq!(result, expected);

        // Fail on negative bid (while insufficient quantity)
        let extra_qty = s.init_qty + 0.1f32;
        let result = market.lock_buy(s.buy_kind.clone(), extra_qty, neg_bid, s.trader.clone()).unwrap_err();
        let expected = LockBuyError::NonPositiveBid { negative_bid: neg_bid };
        assert_eq!(result, expected);

        // Fail on insufficient quantity (while low bid)
        let low_bid = 0f32;
        let result = market.lock_buy(s.buy_kind.clone(), extra_qty, low_bid, s.trader.clone()).unwrap_err();
        let expected = LockBuyError::InsufficientGoodQuantityAvailable {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: extra_qty,
            available_good_quantity: s.init_qty,
        };
        assert_eq!(result, expected);

        // Fail on low bid
        let low_bid = s.init_bid - 0.1f32;
        let result = market.lock_buy(s.buy_kind.clone(), s.init_qty, low_bid, s.trader.clone()).unwrap_err();
        let expected = LockBuyError::BidTooLow {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: s.init_qty,
            low_bid,
            lowest_acceptable_bid: s.init_bid
        };
        assert_eq!(result, expected);

        // Success entire quantity
        let qty_taken = s.init_qty;
        market.lock_buy(s.buy_kind.clone(), qty_taken, s.init_bid, s.trader.clone()).unwrap();

        // Fail after locking all quantity of USD available
        let qty = 0.1f32;
        let result = market.lock_buy(s.buy_kind.clone(), qty, s.init_bid, s.trader.clone()).unwrap_err();
        let expected = LockBuyError::InsufficientGoodQuantityAvailable {
            requested_good_kind: s.buy_kind.clone(),
            requested_good_quantity: qty,
            available_good_quantity: s.init_qty - qty_taken,
        };
        assert_eq!(result, expected);

    }

    #[test]
    fn buy() {
        let s = TestMarketSetup::new();
        let mut market = s.market.borrow_mut();
        let token = market.lock_buy(s.buy_kind.clone(), s.init_qty, s.init_bid, s.trader).unwrap();

        // Fail on wrong token (while cash not default to test priority)
        let invalid_token = "".to_string();
        let invalid_kind = USD;
        let result = market.buy(invalid_token.clone(), &mut Good::new(invalid_kind.clone(), s.init_bid)).unwrap_err();
        let expected = BuyError::UnrecognizedToken { unrecognized_token: invalid_token };
        assert_eq!(result, expected);

        // Fail on cash not default (while quantity insufficient)
        let insufficient_qty = s.init_bid - 0.1f32;
        let mut cash = Good::new(invalid_kind.clone(), insufficient_qty );
        let result = market.buy(token.clone(), &mut cash).unwrap_err();
        let expected = BuyError::GoodKindNotDefault { non_default_good_kind: invalid_kind };
        assert_eq!(result, expected);

        // Fail on quantity insufficient
        let mut cash = Good::new(DEFAULT_GOOD_KIND, insufficient_qty );
        let result = market.buy(token.clone(), &mut cash).unwrap_err();
        let expected = BuyError::InsufficientGoodQuantity { contained_quantity: insufficient_qty, pre_agreed_quantity: s.init_bid };
        assert_eq!(result, expected);

        // Check success
        let mut cash = Good::new(DEFAULT_GOOD_KIND, s.init_bid);
        let result = market.buy(token, &mut cash).unwrap();
        let expected = Good::new(s.buy_kind, s.init_qty);
        assert_eq!(result, expected)
    }

}


#[test]
fn price_unsold_decrease_over_time(){

    //at this time, the exchange rate of EUR does not change over time by just waiting

    let mrkt_bind = SOLMarket::new_random();
    
    let mut USD: f32; let mut YUAN: f32; let mut YEN:f32; let mut EUR:f32;
    {  
        let market = mrkt_bind.borrow();
        USD = market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN = market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN = market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
        EUR = market.get_buy_price(GoodKind::EUR, 1.0).ok().unwrap();
    }
    
    wait_one_day!(mrkt_bind);
    {
        let market = mrkt_bind.borrow();
        assert!(market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD);
        assert!(market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() < YUAN);
        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN);
        assert!(market.get_buy_price(GoodKind::EUR, 1.0).ok().unwrap() == EUR);
        USD = market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN = market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN = market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }

    wait_one_day!(mrkt_bind);
    {
        let market = mrkt_bind.borrow();
        assert!(market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD);
        assert!(market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() < YUAN);
        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN);
        assert!(market.get_buy_price(GoodKind::EUR, 1.0).ok().unwrap() == EUR);
        USD = market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN = market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN = market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }
}

#[test]
fn price_changes_waiting(){
    let mrkt_bind = SOLMarket::new_random();
    
    let mut USD: f32; let mut YUAN: f32; let mut YEN:f32; let mut EUR: f32;
    {  
        let market = mrkt_bind.borrow();
        USD = market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN = market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN = market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
        EUR = market.get_buy_price(GoodKind::EUR, 1.0).ok().unwrap();
        
    }
    
    wait_one_day!(mrkt_bind);
    {
        let market = mrkt_bind.borrow();
        assert!(market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() != USD);
        assert!(market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() != YUAN);
        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() != YEN);
        assert!(market.get_buy_price(GoodKind::EUR, 1.0).ok().unwrap() == EUR);
        USD = market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN = market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN = market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }

    wait_one_day!(mrkt_bind);
    {
        let market = mrkt_bind.borrow();
        assert!(market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() != USD);
        assert!(market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() != YUAN);
        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() != YEN);
        assert!(market.get_buy_price(GoodKind::EUR, 1.0).ok().unwrap() == EUR);
        USD = market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN = market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN = market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }
}


#[test]
fn price_change_after_buy(){
    let mrkt_bind = SOLMarket::new_random();
    
    let mut USD: f32; let mut YUAN: f32; let mut YEN:f32;
    {  
        let market = mrkt_bind.borrow();
        USD = market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN = market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN = market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();        
    }
    let usd_bid = mrkt_bind.borrow().get_goods().iter().find_map(
        |gl| { if gl.good_kind.eq(&GoodKind::USD) { Some(gl.exchange_rate_sell) } else { None } }).unwrap();
    let yen_bid = mrkt_bind.borrow().get_goods().iter().find_map(
        |gl| { if gl.good_kind.eq(&GoodKind::YEN) { Some(gl.exchange_rate_sell) } else { None } }).unwrap();
    let yuan_bid = mrkt_bind.borrow().get_goods().iter().find_map(
        |gl| { if gl.good_kind.eq(&GoodKind::YUAN) { Some(gl.exchange_rate_sell) } else { None } }).unwrap();
    

    // buy USD, assess price change
    {
        let mut market = mrkt_bind.borrow_mut();
        let token = market.lock_buy(GoodKind::USD, 10.0, usd_bid, TRADER_NAME.to_string()).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, 10.0 / usd_bid);
        market.buy(token, &mut cash).unwrap();

        assert!(market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD);
        // assert!(market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() != USD);
    }
    // buy YUAN, assess price change
    {
        let mut market = mrkt_bind.borrow_mut();
        let token = market.lock_buy(GoodKind::YUAN, 10.0, yuan_bid, TRADER_NAME.to_string()).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, 10.0 / yuan_bid);
        market.buy(token, &mut cash).unwrap();

        assert!(market.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() < YUAN);
    }
    // buy YEN, assess price change
    {
        let mut market = mrkt_bind.borrow_mut();
        let token = market.lock_buy(GoodKind::YEN, 10.0, yen_bid, TRADER_NAME.to_string()).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, 10.0 / yen_bid);
        market.buy(token, &mut cash).unwrap();

        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN);
    }

    //do it again
    {
        let mut market = mrkt_bind.borrow_mut();
        let token = market.lock_buy(GoodKind::YEN, 10.0, yen_bid, TRADER_NAME.to_string()).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, 10.0 / yen_bid);
        market.buy(token, &mut cash).unwrap();

        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN);
    }
    {
        let mut market = mrkt_bind.borrow_mut();
        let token = market.lock_buy(GoodKind::YEN, 10.0, yen_bid, TRADER_NAME.to_string()).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, 10.0 / yen_bid);
        market.buy(token, &mut cash).unwrap();

        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN);
    }

}

#[test]
fn price_change_over_time(){

    let a = SOLMarket::new_random();
    let b = SOLMarket::new_random();
    subscribe_each_other!(a, b);

    let mut USD_a: f32; let mut YUAN_a: f32; let mut YEN_a:f32;
    let mut USD_b: f32; let mut YUAN_b: f32; let mut YEN_b:f32;
    {  
        let market_a = a.borrow();
        USD_a = market_a.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN_a = market_a.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN_a = market_a.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
        let market_b = b.borrow();
        USD_b = market_b.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN_b = market_b.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN_b = market_b.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }

    wait_one_day!(a, b);
    {
        let market_a = a.borrow();
        assert!(market_a.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD_a);
        assert!(market_a.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() < YUAN_a);
        assert!(market_a.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN_a);
        USD_a = market_a.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN_a = market_a.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN_a = market_a.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }
    {
        let market_b = b.borrow();
        assert!(market_b.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD_b);
        assert!(market_b.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() < YUAN_b);
        assert!(market_b.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN_b);
        USD_b = market_b.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN_b = market_b.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN_b = market_b.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }

    
    // market_a buys USD, price of USD should decrease
    {   
        let usd_bid = a.borrow().get_goods().iter().find_map(
            |gl| { if gl.good_kind.eq(&GoodKind::USD) { Some(gl.exchange_rate_sell) } else { None } }).unwrap();
        let mut market = a.borrow_mut();
        let token = market.lock_buy(GoodKind::USD, 10.0, usd_bid, TRADER_NAME.to_string()).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, 10.0 / usd_bid);
        market.buy(token, &mut cash).unwrap();

        assert!(market.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD_a);
    }

    wait_one_day!(a, b);
    {
        let market_a = a.borrow();
        assert!(market_a.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD_a);
        assert!(market_a.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() < YUAN_a);
        assert!(market_a.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN_a);
        USD_a = market_a.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN_a = market_a.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN_a = market_a.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }
    {
        let market_b = b.borrow();
        assert!(market_b.get_buy_price(GoodKind::USD, 1.0).ok().unwrap() < USD_b);
        assert!(market_b.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap() < YUAN_b);
        assert!(market_b.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN_b);
        USD_b = market_b.get_buy_price(GoodKind::USD, 1.0).ok().unwrap();
        YUAN_b = market_b.get_buy_price(GoodKind::YUAN, 1.0).ok().unwrap();
        YEN_b = market_b.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap();
    }

    // market_b buys YEN, price of YEN should decrease
    {   
        let yen_bid = b.borrow().get_goods().iter().find_map(
            |gl| { if gl.good_kind.eq(&GoodKind::YEN) { Some(gl.exchange_rate_sell) } else { None } }).unwrap();
        let mut market = b.borrow_mut();
        let token = market.lock_buy(GoodKind::YEN, 10.0, yen_bid, TRADER_NAME.to_string()).unwrap();
        let mut cash = Good::new(DEFAULT_GOOD_KIND, 10.0 / yen_bid);
        market.buy(token, &mut cash).unwrap();

        assert!(market.get_buy_price(GoodKind::YEN, 1.0).ok().unwrap() < YEN_b);
    }
}
