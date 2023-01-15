//the main runs the trader forever

//take the markets out of the trader into a separate playground object?
//maybe too complicated

use std::{borrow::Borrow, collections::HashMap};

use std::{thread, time};
use trader::trader::SOLTrader;
use unitn_market_2022::{good::good_kind::GoodKind, wait_one_day};

//NOTES
// TRADER OBJECT manages trader quantities (default or other init) and the markets (methods to read the markets, trade with the markets)
// MAIN manages history of prices, history display, next choices (can puut into separate objects later)
//can define type history = ...

pub fn main() {
    let mut history_buy: Vec<HashMap<String, HashMap<GoodKind, f32>>> = Vec::new();
    let mut history_sell: Vec<HashMap<String, HashMap<GoodKind, f32>>> = Vec::new();
    let mut history_delta: Vec<HashMap<String, HashMap<GoodKind, f32>>> = Vec::new();

    let generic_init_quantity = 1000.0;
    let mut trader = SOLTrader::new_with_quantities(
        generic_init_quantity,
        generic_init_quantity,
        generic_init_quantity,
        generic_init_quantity,
    );
    trader.subscribe_markets_to_one_another();

    trader.show_all_self_quantities();

    trader.show_all_market_info();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    //trader main loop, each loop a different trade
    for _ in 0..6 {
        make_trade_all_random(&mut trader);

        show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());
        history_delta.push(get_delta_last_day(history_buy.clone()).unwrap());

        // let d = get_delta_last_day(history_buy.clone()).unwrap();
        println!("\n{:?}", history_delta[history_delta.len() - 1]);

        // thread::sleep(time::Duration::from_secs(5))
        trader.show_all_self_quantities();
    }
}

//here we can implement the stategy of the trader
pub fn make_trade_all_random(trader: &mut SOLTrader) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let market_names = ["DogeMarket", "SOL", "Baku stock exchange"];
    //select next trade partner
    let name = market_names[rng.gen_range(0..market_names.len())];

    let all_kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    //select next good
    let kind = all_kinds[rng.gen_range(0..all_kinds.len())];
    //select next quantity
    let qty = rng.gen_range(0..100) as f32;
    //trade!

    if rng.gen_range(0..2) == 0 {
        trader.buy_from_market(name.to_owned(), kind, qty);
    } else {
        trader.sell_to_market(name.to_owned(), kind, qty);
    }
}

pub fn fake_trade(trader: &SOLTrader) {
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

fn get_best_sell_delta() -> (GoodKind, String) {
    todo!()
}

fn get_best_buy_delta() -> (GoodKind, String) {
    todo!()
}
