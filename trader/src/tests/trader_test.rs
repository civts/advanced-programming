#[cfg(test)]
mod trader_tests {
    use crate::trader::strategies::arbitrage::Arbitrages;
    use crate::trader::SOLTrader;
    use std::rc::Rc;

    #[test]
    fn test_get_market_by_name() {
        let trader = SOLTrader::new("Testing_Name".to_string());

        let my_m = "DogeMarket";
        let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());

        let my_m = "Baku stock exchange";
        let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());

        let my_m = "PSE_Market";
        let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());
    }

    #[test]
    fn test_subscription() {
        let trader = SOLTrader::new("Testing_Subscription".to_string());
        let mut strong_count: usize;
        let mut weak_count: usize;

        let nb_sub_per_market = trader.markets.len() - 1;
        for market in trader.markets.iter() {
            strong_count = Rc::strong_count(market);
            weak_count = Rc::weak_count(market);
            assert!(strong_count == 1 && weak_count == nb_sub_per_market);
        }
    }

    #[test]
    fn test_exploit_pse() {
        let mut trader: SOLTrader = SOLTrader::new("Testing_Arbitrage".to_string());

        let value_before = trader.get_current_worth();
        for _ in 0..15_000 {
            trader.exploit_pse_market();
        }
        let value_after = trader.get_current_worth();
        let profit = value_after - value_before;
        let margin_percentage = (profit / value_before) * 100f32;
        assert!(value_after > value_before, "Trader is not profitable");
        println!("VALUE BEFORE: {value_before}\nVALUE AFTER: {value_after}\nPROFIT: {margin_percentage}%");
    }

    #[test]
    fn test_losing_money() {
        let mut trader: SOLTrader = SOLTrader::new("Testing_Arbitrage".to_string());

        let value_before = trader.get_current_worth();
        for _ in 0..15_000 {
            trader.lose_all();
        }
        let value_after = trader.get_current_worth();
        let profit = value_after - value_before;
        let margin_percentage = (profit / value_before) * 100f32;
        assert!(value_after < value_before, "Trader is profitable");
        println!("VALUE BEFORE: {value_before}\nVALUE AFTER: {value_after}\nPROFIT: {margin_percentage}%");
    }
}
