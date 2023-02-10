use ipc_utils::IPCSender;
use std::collections::HashMap;
use std::env;
use trader::trader::arbitrage::Arbitrages;
use trader::trader::SOLTrader;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good_kind::GoodKind;

type History = Vec<HashMap<String, HashMap<GoodKind, f32>>>;

/// Call main with arguments:
/// - cargo run <Strategy: farouk | gianluca | basic_best | basic_random > <Visualizer: yes | no>.
///
/// Examples:
/// - cargo run farouk yes  -> Run trader with farouk strategy and visualizer
/// - cargo run gianluca no -> Run trader with gianluca strategy and no visualizer (just stdout)
///
/// If no arguments are given the trader will be set with the function `do_nothing` and no visualizer.
///
/// When the arguments for visualizer is `yes`:
/// - another terminal running the visualizer (howie or vis_adam) needs to be executed in parallel
/// - If no visualizer is running in parallel, then the trader will be stuck
pub fn main() {
    let args: Vec<String> = env::args().collect();
    let default = "None given".to_string();
    let strategy = args.get(1).unwrap_or(&default).as_str();
    let visualizer = args.get(2).map_or(false, |s| match s.as_str() {
        "yes" => true,
        _ => false,
    });

    let mut trader: SOLTrader;
    let strategy_fn: fn(&mut SOLTrader, u32);
    let mut iterations: u32 = 20;

    match strategy {
        "farouk" => {
            trader = SOLTrader::new("Farouk".to_string());
            strategy_fn = farouk_strategy;
            iterations = 100;
        }
        "gianluca" => {
            let generic_init_quantity = 10000.0;
            trader = SOLTrader::new_with_quantities(
                "Gianluca".to_string(),
                generic_init_quantity,
                generic_init_quantity,
                generic_init_quantity,
                generic_init_quantity,
            );
            strategy_fn = gianluca_strategy;
        }
        "basic_random" => {
            trader = SOLTrader::new("Random".to_string());
            strategy_fn = basic_all_random_strategy;
        }
        "basic_best" => {
            trader = SOLTrader::new("Basic".to_string());
            strategy_fn = basic_best_trade_strategy;
        }
        &_ => {
            trader = SOLTrader::new("Lazy".to_string());
            strategy_fn = do_nothing_strategy
        }
    }

    trader.subscribe_markets_to_one_another();
    trader.show_all_self_quantities();
    trader.show_all_market_info();

    if visualizer {
        trader.set_ipc_sender(IPCSender::new());
    }

    println!("*** Starting Strategy ({})***", strategy);
    strategy_fn(&mut trader, iterations);
}

fn farouk_strategy(trader: &mut SOLTrader, iterations: u32) {
    let worth_before = trader.get_current_worth();
    for _ in 0..iterations {
        trader.exploit_pse_market();
    }
    let worth_after = trader.get_current_worth();
    let profit = worth_after - worth_before;
    let margin_percentage = (profit / worth_before) * 100f32;
    println!(
        "*** Arbitrage results ***\n\
    Trader's worth before: {worth_before}\n\
    Trader's worth after: {worth_after}\n\
    Profit: {margin_percentage}%"
    );
}

fn basic_all_random_strategy(trader: &mut SOLTrader, iterations: u32) {
    let max_qty = 100;
    let mut history_buy: History = Vec::new();
    let mut history_sell: History = Vec::new();
    let mut buy_deltas: History = Vec::new();
    let mut sell_deltas: History = Vec::new();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    for _ in 0..iterations {
        make_trade_all_random(trader, max_qty);

        show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());
        buy_deltas.push(get_delta_last_day(history_buy.clone()).unwrap());
        sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

        println!("\nBUY DELTAS\n{:?}", buy_deltas[buy_deltas.len() - 1]);
        println!("\nSELL DELTAS\n{:?}", sell_deltas[sell_deltas.len() - 1]);

        let (a, b, _) = get_best_buy_delta(&buy_deltas);
        println!("\n today's best BUY delta is {} {}", a, b);
        let (a, b, _) = get_best_sell_delta(&sell_deltas);
        println!("\n today's best SELL delta is {} {}", a, b);

        // thread::sleep(time::Duration::from_secs(5))
        trader.show_all_self_quantities();
    }
}

//here we can implement the stategy of the trader
pub fn make_trade_all_random(trader: &mut SOLTrader, max_qty: i32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let market_names = ["DogeMarket", "PSE_Market", "Baku stock exchange"];
    //select next trade partner
    let name = market_names[rng.gen_range(0..market_names.len())];

    let all_kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    //select next good
    let kind = all_kinds[rng.gen_range(0..all_kinds.len())];
    //select next quantity
    let qty = rng.gen_range(1..max_qty) as f32;
    //trade!

    if rng.gen_range(0..2) == 0 {
        trader.buy_from_market(name.to_owned(), kind, qty);
    } else {
        trader.sell_to_market(name.to_owned(), kind, qty);
    }
}

///first makes one random trade, than looks at the deltas and starts making the best trades possible
///best trade means either
///buy (market,goodkind) with the lowest delta (bargain)
///or sell (market,goodkind) with the highest delta (amke the most out of what you bought)
///the quantities are still random
fn basic_best_trade_strategy(trader: &mut SOLTrader, iterations: u32) {
    let max_qty = 100;
    let mut history_buy: History = Vec::new();
    let mut history_sell: History = Vec::new();
    let mut buy_deltas: History = Vec::new();
    let mut sell_deltas: History = Vec::new();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    make_trade_all_random(trader, max_qty);
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

        // println!("\nBUY DELTAS\n{:?}", buy_deltas[buy_deltas.len() - 1]);
        // println!("\nSELL DELTAS\n{:?}", sell_deltas[sell_deltas.len() - 1]);

        // let (a, b) = get_best_buy_delta(&buy_deltas);
        // println!("\n today's best BUY delta is {} {}", a, b);
        // let (a, b) = get_best_sell_delta(&sell_deltas);
        // println!("\n today's best SELL delta is {} {}", a, b);

        // thread::sleep(time::Duration::from_secs(5))
        trader.show_all_self_quantities();

        println!("history\n{:?}", history_buy);
        println!("average");
        println!("{:?}", get_historical_average(&history_buy));
        println!("delta from history avg");
        println!("{:?}", get_delta_from_historical_avg(&history_buy));
        let (a, b, c) = get_best_buy_delta_from_historical_avg(&history_buy);
        println!("bestbuy delta from history avg: \n {} {} {}", a, b, c);
    }
}

fn make_best_trade(trader: &mut SOLTrader, buy_deltas: &History, sell_deltas: &History) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let (kind_buy, name_buy, b_delta) = get_best_buy_delta(buy_deltas);

    let (kind_sell, name_sell, s_delta) = get_best_sell_delta(sell_deltas);

    //select next quantity
    let qty = rng.gen_range(500..1000) as f32;

    //trade!
    if b_delta.abs() > s_delta {
        trader.buy_from_market(name_buy.to_owned(), kind_buy, qty);
    } else {
        trader.sell_to_market(name_sell.to_owned(), kind_sell, qty);
    }
}

fn gianluca_strategy(trader: &mut SOLTrader, iterations: u32) {
    //how this strategy works: make a small random trade to start collecting some historical data
    // choose the next operation based on best delta from historical average
    //choose quantity based on delta

    //this var here avoids doing nothins for too many days
    let mut do_nothing_count = 0;

    let mut history_buy: History = Vec::new();
    let mut history_sell: History = Vec::new();
    let mut buy_deltas: History = Vec::new();
    let mut sell_deltas: History = Vec::new();

    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());

    //day 1, small random trade
    make_trade_all_random(trader, 10);
    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());
    buy_deltas.push(get_delta_last_day(history_buy.clone()).unwrap());
    sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

    //day 2, small random trade
    make_trade_all_random(trader, 10);
    history_buy.push(trader.get_all_current_buy_rates());
    history_sell.push(trader.get_all_current_sell_rates());
    buy_deltas.push(get_delta_last_day(history_buy.clone()).unwrap());
    sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

    //for all the other days make best historical trade
    for _ in 0..iterations - 2 {
        make_best_historical_trade(trader, &history_buy, &history_sell, &mut do_nothing_count);

        // show_delta();

        history_buy.push(trader.get_all_current_buy_rates());
        history_sell.push(trader.get_all_current_sell_rates());
        buy_deltas.push(get_delta_last_day(history_buy.clone()).unwrap());
        sell_deltas.push(get_delta_last_day(history_sell.clone()).unwrap());

        // println!("\nBUY DELTAS\n{:?}", buy_deltas[buy_deltas.len() - 1]);
        // println!("\nSELL DELTAS\n{:?}", sell_deltas[sell_deltas.len() - 1]);

        // let (a, b) = get_best_buy_delta(&buy_deltas);
        // println!("\n today's best BUY delta is {} {}", a, b);
        // let (a, b) = get_best_sell_delta(&sell_deltas);
        // println!("\n today's best SELL delta is {} {}", a, b);

        // thread::sleep(time::Duration::from_secs(5))
        trader.show_all_self_quantities();

        // println!("history\n{:?}",history_buy);
        // println!("average");
        // println!("{:?}", get_historical_average(&history_buy));
        // println!("delta from history avg");
        // println!("{:?}", get_delta_from_historical_avg(&history_buy));
        // let(a,b,c) = get_best_buy_delta_from_historical_avg(&history_buy);
        // println!("bestbuy delta from history avg: \n {} {} {}",a,b,c);
    }
}

fn make_best_historical_trade(
    trader: &mut SOLTrader,
    h_buy: &History,
    h_sell: &History,
    do_nothing_count: &mut i32,
) {
    let (kind_buy, name_buy, b_delta) = get_best_buy_delta_from_historical_avg(h_buy);

    let (kind_sell, name_sell, s_delta) = get_best_sell_delta_from_historical_avg(h_sell);

    //select next quantity
    let std_qty = 10.0;
    let threshold = 1.05;

    //new condition: if the delta is too small, don't make any trade
    //but if you haven't made any trade for too long, then force a trade to shuffle the markets
    if (*do_nothing_count < 5 && (b_delta > threshold || s_delta > threshold))
        || (*do_nothing_count >= 5)
    {
        *do_nothing_count = 0;

        if b_delta.abs() > s_delta {
            println!("buy {} {} {}", b_delta, name_buy, kind_buy);

            let mut qty = {
                if b_delta > 5.0 {
                    std_qty * b_delta
                } else if b_delta > 1.0 {
                    std_qty * b_delta.powi(2).round()
                } else {
                    std_qty
                }
            };

            let upperbound = trader.get_cur_good_qty_from_market(&kind_buy, name_buy.clone()) / 2.0; //upperbound for buy is the market qty
            if qty > upperbound {
                qty = upperbound;
            }

            println!("qty {}", qty);
            trader.buy_from_market(name_buy.to_owned(), kind_buy, qty);
        } else {
            println!("sell {} {} {}", s_delta, name_sell, kind_sell);

            let mut qty = {
                if s_delta > 5.0 {
                    std_qty * s_delta
                } else if s_delta > 1.0 {
                    std_qty * s_delta.powi(2).round()
                } else {
                    std_qty
                }
            };
            //upperbound for sell is my own qty
            let upperbound = trader.get_cur_good_qty(&kind_sell) / 2.0;
            if qty > upperbound {
                qty = upperbound;
            }

            println!("qty {}", qty);

            trader.sell_to_market(name_sell.to_owned(), kind_sell, qty);
        }
    } else {
        *do_nothing_count += 1;
        println!("i'm doing nothing today");
        trader.all_wait_one_day();
    }
}

fn do_nothing_strategy(trader: &mut SOLTrader, iterations: u32) {
    let mut history_buy: History = Vec::new();
    let mut history_sell: History = Vec::new();
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

fn get_historical_average(h: &History) -> Option<HashMap<String, HashMap<GoodKind, f32>>> {
    if h.len() >= 2 {
        let mut avg: HashMap<String, HashMap<GoodKind, f32>> = h[0].clone();
        let days = h.len() as f32;

        for day in h[1..].iter() {
            for (market, rates) in day {
                for (good, single_rate) in rates {
                    let tmp = avg.get_mut(&market[..]).unwrap().get_mut(good).unwrap();
                    *tmp += *single_rate;
                }
            }
        }

        for (market, rates) in avg.clone() {
            for (good, _) in rates {
                let tmp = avg.get_mut(&market[..]).unwrap().get_mut(&good).unwrap();
                *tmp /= days;
            }
        }

        return Some(avg);
    }
    None
}

fn get_delta_from_historical_avg(h: &History) -> Option<HashMap<String, HashMap<GoodKind, f32>>> {
    if h.len() >= 2 {
        let mut delta = h[h.len() - 1].clone(); //assign last day
        let mut avg = get_historical_average(h).unwrap();

        for (market, rates) in avg.clone() {
            for (good, _) in rates {
                let tmp = delta.get_mut(&market[..]).unwrap().get_mut(&good).unwrap();
                let tmp2 = avg.get_mut(&market[..]).unwrap().get_mut(&good).unwrap();
                *tmp = tmp.abs() - tmp2.abs();
            }
        }

        return Some(delta);
    }
    None
}

fn get_best_buy_delta_from_historical_avg(h: &History) -> (GoodKind, String, f32) {
    let delta_buy = get_delta_from_historical_avg(h).unwrap();
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut min_found: f32 = delta_buy["DogeMarket"][&GoodKind::USD];

    for (market, rates) in delta_buy {
        for (good, delta) in rates {
            if good != DEFAULT_GOOD_KIND {
                if delta < min_found {
                    res_kind = good;
                    res_market = market.clone();
                    min_found = delta;
                }
            }
        }
    }

    //return the abs()
    (res_kind, res_market, min_found.abs())
}

//again i could have made only one function, but i would have had to encode the selected operation somehow. it's just cleaner this way
fn get_best_sell_delta_from_historical_avg(h: &History) -> (GoodKind, String, f32) {
    let delta_sell = get_delta_from_historical_avg(h).unwrap();
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut max_found: f32 = delta_sell["DogeMarket"][&GoodKind::USD];

    for (market, rates) in delta_sell {
        for (good, delta) in rates {
            if good != DEFAULT_GOOD_KIND {
                //it's important that i use no abs() here
                if delta > max_found {
                    res_kind = good;
                    res_market = market.clone();
                    max_found = delta;
                }
            }
        }
    }

    (res_kind, res_market, max_found)
}

fn show_delta() {}

fn get_best_sell_delta(h: &History) -> (GoodKind, String, f32) {
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
    (res_kind, res_market, max_found)
}

fn get_best_buy_delta(h: &History) -> (GoodKind, String, f32) {
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
    (res_kind, res_market, min_found)
}
