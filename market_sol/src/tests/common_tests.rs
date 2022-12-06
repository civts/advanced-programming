mod extern_test {
    //import here the market_test module and the Market trait
    use unitn_market_2022::market::market_test;
    //import here your implementation of the market
    use crate::lib::market::sol_market::SOLMarket;
    //make an alias to your market
    type MarketType = SOLMarket;
    //test every aspect of your market using the generic function
    #[test]
    fn test_name() {
        market_test::test_name::<MarketType>();
    }
    #[test]
    fn test_get_buy_price_success() {
        market_test::test_get_buy_price_success::<MarketType>();
    }
    #[test]
    fn test_get_buy_price_non_positive_error() {
        market_test::test_get_buy_price_non_positive_error::<MarketType>();
    }
    #[test]
    fn test_get_buy_price_insufficient_qty_error() {
        market_test::test_get_buy_price_insufficient_qty_error::<MarketType>();
    }
    #[test]
    fn test_get_sell_price_success() {
        market_test::test_get_sell_price_success::<MarketType>();
    }
    #[test]
    fn test_get_sell_price_non_positive_error() {
        market_test::test_get_sell_price_non_positive_error::<MarketType>();
    }
    #[test]
    fn test_deadlock_prevention() {
        market_test::test_deadlock_prevention::<MarketType>();
    }
    #[test]
    fn test_new_random() {
        market_test::test_new_random::<MarketType>();
    }
    #[test]
    fn test_price_change_after_buy() {
        market_test::test_price_change_after_buy::<MarketType>();
    }
    #[test]
    fn price_changes_waiting() {
        market_test::price_changes_waiting::<MarketType>();
    }
    #[test]
    fn test_price_change_after_sell() {
        market_test::test_price_change_after_sell::<MarketType>();
    }
    #[test]
    fn should_initialize_with_right_quantity() {
        market_test::should_initialize_with_right_quantity::<MarketType>();
    }
    #[test]
    fn new_random_should_not_exceeed_starting_capital() {
        for _ in 0..1000 {
            market_test::new_random_should_not_exceeed_starting_capital::<MarketType>();
        }
    }
    #[test]
    fn test_sell_success() {
        market_test::test_sell_success::<MarketType>();
    }
    #[test]
    fn test_sell_err_unrecognized_token() {
        market_test::test_sell_err_unrecognized_token::<MarketType>();
    }
    #[test]
    fn test_sell_err_expired_token() {
        market_test::test_sell_err_expired_token::<MarketType>();
    }
    #[test]
    fn test_sell_err_wrong_good_kind() {
        market_test::test_sell_err_wrong_good_kind::<MarketType>();
    }
    #[test]
    fn test_sell_err_insufficient_good_quantity() {
        market_test::test_sell_err_insufficient_good_quantity::<MarketType>();
    }
    #[test]
    fn test_lock_sell_non_positive_offer() {
        market_test::test_lock_sell_nonPositiveOffer::<MarketType>();
    }
    #[test]
    fn test_lock_sell_default_good_already_locked() {
        // Our market allows more than 1 lock per good
        // market_test::test_lock_sell_defaultGoodAlreadyLocked::<MarketType>();
    }
    #[test]
    fn test_lock_sell_max_allowed_locks_reached() {
        market_test::test_lock_sell_maxAllowedLocksReached::<MarketType>();
    }
    #[test]
    fn test_lock_sell_insufficient_default_good_quantity_available() {
        market_test::test_lock_sell_insufficientDefaultGoodQuantityAvailable::<MarketType>();
    }
    #[test]
    fn test_lock_sell_offer_too_high() {
        market_test::test_lock_sell_offerTooHigh::<MarketType>();
    }
    #[test]
    fn test_working_function_lock_sell_token() {
        //test_working_function_lock_sell_token::test_lock_sell_offerTooHigh::<MarketType>();
    }
}
