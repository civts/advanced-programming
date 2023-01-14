//the main runs the trader forever

//take the markets out of the trader into a separate playground object?
//maybe too complicated

use std::{borrow::Borrow, collections::HashMap};

use std::{thread, time};
use trader::trader::SOLTrader;
use unitn_market_2022::{good::good_kind::GoodKind, wait_one_day};

pub fn main() {
    let mut history_buy: Vec<HashMap<String, HashMap<GoodKind, f32>>> = Vec::new();
    let mut history_sell: Vec<HashMap<String, HashMap<GoodKind, f32>>> = Vec::new();

    let generic_init_quantity = 1000.0;
    let mut trader = SOLTrader::new_with_quantities(generic_init_quantity, 0.0, 0.0, 0.0);
    trader.subscribe_markets_to_one_another();

    trader.show_all_self_quantities();

    trader.show_all_market_info();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    //trader main loop, each loop a different trade
    for _ in 0..3 {
        make_trade(&trader);

        show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());

        let d = get_delta_last_day(history_buy.clone()).unwrap();
        println!("\n{:?}", d);

        // thread::sleep(time::Duration::from_secs(5))
    }
    for _ in 0..3 {
        make_sell(&trader);

        show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());

        let d = get_delta_last_day(history_buy.clone()).unwrap();
        println!("\n{:?}", d);

        // thread::sleep(time::Duration::from_secs(5))
    }
}

//here we can implement the stategy of the trader
pub fn make_trade(trader: &SOLTrader) {
    //select next trade partner
    let name = "DogeMarket";
    //select next good
    let kind = GoodKind::USD;
    //select next quantity
    let qty = 1.0;
    //trade!
    trader.buy_from_market(name.to_owned(), kind, qty)
}

pub fn make_sell(trader: &SOLTrader) {
    //select next trade partner
    let name = "DogeMarket";
    //select next good
    let kind = GoodKind::USD;
    //select next quantity
    let qty = 1.0;
    //trade!
    trader.sell_to_market(name.to_owned(), kind, qty)
}

pub fn fake_trade(trader: &SOLTrader){
    trader.all_wait_one_day();
}

//tested: delta is zero qith no trades
fn get_delta_last_day(
    history: Vec<HashMap<String, HashMap<GoodKind, f32>>>,
) -> Option<HashMap<String, HashMap<GoodKind, f32>>> {
    if history.len() >= 2 {
        // if at least one day has passed

        let mut delta: HashMap<String, HashMap<GoodKind, f32>> = HashMap::new();

        let yesterday = history[history.len() - 1].clone();

        for (name, abc) in history[history.len() - 2].clone() {
            let mut tmp: HashMap<GoodKind, f32> = HashMap::new();

            for (good, rate) in abc {
                // let acbabca = rate-yesterday[&name][&good];
                tmp.insert(good.clone(), rate - yesterday[&name][&good]);
            }

            delta.insert(name.clone(), tmp);
        }

        return Some(delta);
    }
    None
}

fn show_delta() {}

fn print_history() {}
