use crate::lib::market::sol_market::SOLMarket;

#[test]
/// When a market gets created using the `new_with_quantities` constructor,
/// the quantities of each good in the market should correspond to the ones
/// passed as parameters to the constructor.
///
/// SOL group
fn should_initialize_with_right_quantity() {
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::{good_label::GoodLabel, Market};

    //Create market with predefined quantities
    let eur_qty = 12.0;
    let usd_qty = 42.0;
    let yen_qty = 137.0;
    let yuan_qty = 1984.0;
    let market = SOLMarket::new_with_quantities(eur_qty, yen_qty, usd_qty, yuan_qty);
    //Get the goods in the market immediately after creation
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
            usd_good_label.quantity, usd_qty,
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
            eur_good_label.quantity, eur_qty,
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
            yen_good_label.quantity, yen_qty,
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

#[test]
/// When a merket gets created using the `new_random` constructor, the total
/// value of the market expressed in default_good_kind and calculated with the
/// default exchange rate shall not exceed the `STARTING_CAPITAL`.
///
/// Since the `new_random` function is, most likely, non-deterministic, this
/// test cannot guarantee that the code is correct, but should catch bugs in the
/// long run.
///
/// Reference to the specs [here](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#market-creation)
///
/// SOL team
fn new_random_should_not_exceeed_starting_capital() {
    use unitn_market_2022::good::consts::*;
    use unitn_market_2022::{good::good_kind::GoodKind, market::Market};

    //Test 10 times to get better chances of catching bugs
    for _ in 0..10 {
        //Create a new market with the random constructor
        let market = SOLMarket::new_random();
        //Immediately get the goods
        let goods = market.borrow().get_goods();
        //Calculate total value of the market with default exchange rate
        let mut total_value = 0.0;
        for good in goods {
            let default_exchnge_rate = match good.good_kind {
                GoodKind::EUR => 1.0,
                GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
                GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
                GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
            };
            //The amount of EUR the current good is worth
            let good_market_cap = good.quantity / default_exchnge_rate;
            total_value += good_market_cap;
        }
        assert!(
            total_value <= STARTING_CAPITAL,
            "The value of the market must be under the STARTING_CAPITAL"
        );
    }
}
