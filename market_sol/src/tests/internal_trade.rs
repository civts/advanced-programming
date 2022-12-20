mod test_internal_trade {
    use crate::lib::market::sol_market::*;
    use std::collections::HashMap;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use unitn_market_2022::wait_one_day;

    #[test]
    fn test_refill_of_nil_qty() {
        let market = SOLMarket::new_with_quantities(1000f32, 0f32, 0f32, 0f32);

        // Wait 3 days and check if quantities of goods has been distributed
        for _ in 0..3 {
            wait_one_day!(market)
        }
        let quantities_per_kind: HashMap<GoodKind, f32> = market
            .borrow()
            .get_goods()
            .iter()
            .map(|gl| (gl.good_kind, gl.quantity))
            .collect();
        for (k, q) in quantities_per_kind.iter() {
            assert!(get_value_good(k, *q) > 0f32);
        }
    }

    #[test]
    fn test_order_refill() {
        // USD has 0€ value & YEN has a 1€ value
        let market = SOLMarket::new_with_quantities(250f32, 145f32, 0f32, 1000f32);

        // Check that USD is the first one to be refilled
        wait_one_day!(market);
        let usd_qty = market
            .borrow()
            .get_goods()
            .iter()
            .find(|g| g.good_kind.eq(&GoodKind::USD))
            .unwrap()
            .quantity;
        assert!(usd_qty > 0f32);

        // Check that yen get refilled afterwards
        wait_one_day!(market);
        let yen_qty = market
            .borrow()
            .get_goods()
            .iter()
            .find(|g| g.good_kind.eq(&GoodKind::YEN))
            .unwrap()
            .quantity;
        assert!(yen_qty > 145f32);
    }

    #[test]
    fn test_switch_role() {
        use GoodKind::*;
        let trader = "test".to_string();

        // USD & YUAN should be exporter (>100€ value) and YEN & EUR should be importers (0€ value)
        let market = SOLMarket::new_with_quantities(0f32, 0f32, 250f32, 1000f32);

        // DAY 1: Lock all USD from market
        let usd_bid = market.borrow().get_buy_price(USD, 250f32).unwrap();
        let token_usd = market
            .borrow_mut()
            .lock_buy(USD, 250f32, usd_bid, trader.clone())
            .unwrap();

        // 1 day has passed so Yuan quantity should have dropped in order to refill EUR or YEN
        let yuan_qty = market
            .borrow()
            .get_goods()
            .iter()
            .find(|gl| gl.good_kind.eq(&YUAN))
            .unwrap()
            .quantity;
        assert!(yuan_qty < 1000f32);

        // DAY 2: Lock all remaining YUAN from market
        let yuan_bid = market.borrow().get_buy_price(YUAN, yuan_qty).unwrap();
        let token_yuan = market
            .borrow_mut()
            .lock_buy(YUAN, yuan_qty, yuan_bid, trader)
            .unwrap();

        // DAY 3 & 4: Buy USD & YUAN
        let mut cash_usd = Good::new(EUR, usd_bid);
        let mut cash_yuan = Good::new(EUR, yuan_bid);
        market.borrow_mut().buy(token_usd, &mut cash_usd).unwrap();
        market.borrow_mut().buy(token_yuan, &mut cash_yuan).unwrap();

        // Now USD & YUAN qty in market should be empty
        // No internal trade should be possible because they were the only 2 exporters
        // DAY 5 -> 99: Check change in quantities
        let yuan_qty = market
            .borrow()
            .get_goods()
            .iter()
            .find(|gl| gl.good_kind.eq(&YUAN))
            .unwrap()
            .quantity;
        let usd_qty = market
            .borrow()
            .get_goods()
            .iter()
            .find(|gl| gl.good_kind.eq(&USD))
            .unwrap()
            .quantity;
        for _ in 5..100 {
            wait_one_day!(market);
            assert_eq!(usd_qty, 0f32);
            assert_eq!(yuan_qty, 0f32)
        }

        // DAY 100 -> 101: USD & YUAN should become importers, therefore their quantities should increase
        wait_one_day!(market);
        wait_one_day!(market);
        wait_one_day!(market);
        assert!(
            yuan_qty
                < market
                    .borrow()
                    .get_goods()
                    .iter()
                    .find(|gl| gl.good_kind.eq(&YUAN))
                    .unwrap()
                    .quantity
        );
        assert!(
            usd_qty
                < market
                    .borrow()
                    .get_goods()
                    .iter()
                    .find(|gl| gl.good_kind.eq(&USD))
                    .unwrap()
                    .quantity
        );
    }
}
