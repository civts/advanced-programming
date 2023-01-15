//the main runs the trader forever

//take the markets out of the trader into a separate playground object?
//maybe too complicated

use std::{borrow::Borrow, collections::HashMap};

use std::{thread, time};
use trader::trader::SOLTrader;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::{good::good_kind::GoodKind, wait_one_day};

//NOTES
// TRADER OBJECT manages trader quantities (default or other init) and the markets (methods to read the markets, trade with the markets)
// MAIN manages history of prices, history display, next choices (can puut into separate objects later)
//can define type history = ...

type History = Vec<HashMap<String, HashMap<GoodKind, f32>>>;

pub fn main() {
    let generic_init_quantity = 10000.0;
    let mut trader = SOLTrader::new_with_quantities(
        generic_init_quantity,
        generic_init_quantity,
        generic_init_quantity,
        generic_init_quantity,
    );
    trader.subscribe_markets_to_one_another();

    trader.show_all_self_quantities();

    trader.show_all_market_info();

    //trader main loop, each loop a different trade
    // basic_all_random_strategy(&mut trader, 6);
    // do_nothing_strategy(&mut trader, 6);
    basic_best_trade_strategy(&mut trader, 6);
}

fn basic_all_random_strategy(trader: &mut SOLTrader, iterations: u32) {
    let mut history_buy: History = Vec::new();
    let mut history_sell: History = Vec::new();
    let mut buy_deltas: History = Vec::new();
    let mut sell_deltas: History = Vec::new();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    for _ in 0..iterations {
        make_trade_all_random(trader);

        show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());
        buy_deltas.push(get_delta_last_day(history_buy.clone()).unwrap());
        sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

        println!("\nBUY DELTAS\n{:?}", buy_deltas[buy_deltas.len() - 1]);
        println!("\nSELL DELTAS\n{:?}", sell_deltas[sell_deltas.len() - 1]);

        let (a, b) = get_best_buy_delta(&buy_deltas);
        println!("\n today's best BUY delta is {} {}", a, b);
        let (a, b) = get_best_sell_delta(&sell_deltas);
        println!("\n today's best SELL delta is {} {}", a, b);

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

//first makes one random trade, than looks at the deltas and starts making the best trades possible
//best trade means either
///buy (market,goodkind) with the lowest delta (bargain)
//or sell (market,goodkind) with the highest delta (amke the most out of what you bought)
//the quantities are still random
fn basic_best_trade_strategy(trader: &mut SOLTrader, iterations: u32) {
    let mut history_buy: History = Vec::new();
    let mut history_sell: History = Vec::new();
    let mut buy_deltas: History = Vec::new();
    let mut sell_deltas: History = Vec::new();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    make_trade_all_random(trader);
    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());
    buy_deltas.push(get_delta_last_day(history_buy.clone()).unwrap());
    sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

    for _ in 0..iterations - 1 {
        make_best_trade(trader, &buy_deltas, &sell_deltas);

        show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());
        buy_deltas.push(get_delta_last_day(history_buy.clone()).unwrap());
        sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

        println!("\nBUY DELTAS\n{:?}", buy_deltas[buy_deltas.len() - 1]);
        println!("\nSELL DELTAS\n{:?}", sell_deltas[sell_deltas.len() - 1]);

        let (a, b) = get_best_buy_delta(&buy_deltas);
        println!("\n today's best BUY delta is {} {}", a, b);
        let (a, b) = get_best_sell_delta(&sell_deltas);
        println!("\n today's best SELL delta is {} {}", a, b);

        // thread::sleep(time::Duration::from_secs(5))
        trader.show_all_self_quantities();
    }
}

fn make_best_trade(trader: &mut SOLTrader, buy_deltas: &History, sell_deltas: &History) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let (kind_buy, name_buy) = get_best_buy_delta(buy_deltas);

    let (kind_sell, name_sell) = get_best_sell_delta(sell_deltas);

    //select next quantity
    let qty = rng.gen_range(500..1000) as f32;
    //trade!

    //still selects the kind of trade randomly
    if rng.gen_range(0..2) == 0 {
        trader.buy_from_market(name_buy.to_owned(), kind_buy, qty);
    } else {
        trader.sell_to_market(name_sell.to_owned(), kind_sell, qty);
    }
}

fn do_nothing_strategy(trader: &mut SOLTrader, iterations: u32) {
    let mut history_buy: History = Vec::new();
    let mut history_sell: History = Vec::new();
    let mut buy_deltas: History = Vec::new();
    let mut sell_deltas: History = Vec::new();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    for _ in 0..iterations {
        fake_trade(trader);

        show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());

        sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

        println!("\n{:?}", sell_deltas[sell_deltas.len() - 1]);

        // thread::sleep(time::Duration::from_secs(5))
        trader.show_all_self_quantities();
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

fn get_best_sell_delta(h: &History) -> (GoodKind, String) {
    let delta = &h[h.len() - 1];
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut max_found: f32 = delta["DogeMarket"][&GoodKind::USD];

    for (market, map) in delta {
        for (good, delta) in map {
            if *good != DEFAULT_GOOD_KIND {
                if *delta > max_found {
                    res_kind = *good;
                    res_market = market.clone();
                    max_found = *delta;
                }
            }
        }
    }
    (res_kind, res_market)
}

fn get_best_buy_delta(h: &History) -> (GoodKind, String) {
    let delta = &h[h.len() - 1];
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut min_found: f32 = delta["DogeMarket"][&GoodKind::USD];

    for (market, map) in delta {
        for (good, delta) in map {
            if *good != DEFAULT_GOOD_KIND {
                if *delta < min_found {
                    res_kind = *good;
                    res_market = market.clone();
                    min_found = *delta;
                }
            }
        }
    }
    (res_kind, res_market)
}
