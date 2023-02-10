pub mod arbitrage;
mod misc;

use crate::trader::strategies::arbitrage::Arbitrages;
use crate::trader::strategies::misc::{
    fake_trade, get_best_buy_delta, get_best_buy_delta_from_historical_avg, get_best_sell_delta,
    get_delta_from_historical_avg, get_delta_last_day, get_historical_average,
    make_best_historical_trade, make_best_trade, make_trade_all_random, show_delta,
};
use crate::trader::SOLTrader;
use std::collections::HashMap;
use unitn_market_2022::good::good_kind::GoodKind;

type History = Vec<HashMap<String, HashMap<GoodKind, f32>>>;

pub fn farouk_strategy(trader: &mut SOLTrader, iterations: u32) {
    let worth_before = trader.get_current_worth();
    for _ in 0..iterations {
        trader.exploit_pse_market();
    }
    let worth_after = trader.get_current_worth();
    let profit = worth_after - worth_before;
    let margin_percentage = (profit / worth_before) * 100f32;
    println!(
        "\n*** Arbitrage results ***\n\
    Trader's worth before: {worth_before}\n\
    Trader's worth after: {worth_after}\n\
    Profit: {margin_percentage}%"
    );
}

pub fn gianluca_strategy(trader: &mut SOLTrader, iterations: u32) {
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

pub fn basic_all_random_strategy(trader: &mut SOLTrader, iterations: u32) {
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

///first makes one random trade, than looks at the deltas and starts making the best trades possible
///best trade means either
///buy (market,goodkind) with the lowest delta (bargain)
///or sell (market,goodkind) with the highest delta (amke the most out of what you bought)
///the quantities are still random
pub fn basic_best_trade_strategy(trader: &mut SOLTrader, iterations: u32) {
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

pub fn do_nothing_strategy(trader: &mut SOLTrader, iterations: u32) {
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
