use ipc_utils::IPCSender;
use std::env;
use trader::trader::strategies::{
    basic_all_random_strategy, basic_best_trade_strategy, do_nothing_strategy, farouk_strategy,
    gianluca_strategy, lose_and_recover_strategy, losing_strategy,
};
use trader::trader::SOLTrader;

// Default iteration number
const DEF_ITERATION: u32 = 100;

/// Call main with arguments:
/// - cargo run <Strategy: farouk | gianluca | basic_best | basic_random | lose | lose_recover> <Visualizer: vis> <Iterations: u32 or "inf">
///
/// Examples:
/// - cargo run farouk vis inf      -> Run trader with lose & recover strategy and visualizer almost forever (u32::MAX / ~4 Billion iterations)
/// - cargo run lose_recover vis    -> Run trader with farouk strategy and visualizer (20 iterations)
/// - cargo run gianluca no_vis inf -> Run trader with gianluca strategy, no visualizer and almost forever
/// - cargo run basic_best          -> Run trader with basic_best strategy and no visualizer, just stdout (20 iterations)
///
/// If no arguments are given the trader will be set with the function `do_nothing` and no visualizer.
///
/// When the arguments for visualizer is `vis`:
/// - another terminal running the visualizer (howie or vis_adam) needs to be executed in parallel
/// - If no visualizer is running in parallel, then the trader will be hanging
pub fn main() {
    let args: Vec<String> = env::args().collect();
    let default = "none".to_string();
    let strategy = args.get(1).unwrap_or(&default).as_str();
    let visualizer = args.get(2).map_or(false, |s| matches!(s.as_str(), "vis"));
    let iterations: u32 = args.get(3).map_or(DEF_ITERATION, |s| match s.as_str() {
        "inf" => u32::MAX,
        s => s.parse::<u32>().unwrap_or(DEF_ITERATION),
    });

    let mut trader: SOLTrader;
    let strategy_fn: fn(&mut SOLTrader, u32);
    let qty = 10_000f32;

    match strategy {
        "farouk" => {
            trader = SOLTrader::new_with_quantities("Farouk".to_string(), qty, qty, qty, qty);
            strategy_fn = farouk_strategy;
        }
        "gianluca" => {
            trader = SOLTrader::new_with_quantities("Gianluca".to_string(), qty, qty, qty, qty);
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
        "lose" => {
            trader = SOLTrader::new("Lose".to_string());
            strategy_fn = losing_strategy;
        }
        "lose_recover" => {
            trader = SOLTrader::new("Lose and Recover".to_string());
            strategy_fn = lose_and_recover_strategy;
        }
        &_ => {
            trader = SOLTrader::new("Lazy".to_string());
            strategy_fn = do_nothing_strategy
        }
    }

    trader.show_all_self_quantities();
    trader.show_all_market_info();

    println!("*** Starting Strategy ({})***", strategy);
    if visualizer {
        trader.set_ipc_sender(IPCSender::new());
        println!("(Be sure to have another terminal running the visualizer in parallel)");
    }
    strategy_fn(&mut trader, iterations);
}
