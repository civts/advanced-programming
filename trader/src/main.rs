use ipc_utils::IPCSender;
use std::env;
use trader::trader::strategies::{
    basic_all_random_strategy, basic_best_trade_strategy, do_nothing_strategy, farouk_strategy,
    gianluca_strategy,
};
use trader::trader::SOLTrader;

/// Call main with arguments:
/// - cargo run <Strategy: farouk | gianluca | basic_best | basic_random> <Visualizer: vis>.
///
/// Examples:
/// - cargo run farouk yes  -> Run trader with farouk strategy and visualizer
/// - cargo run gianluca no -> Run trader with gianluca strategy and no visualizer (just stdout)
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
    let visualizer = args.get(2).map_or(false, |s| match s.as_str() {
        "vis" => true,
        _ => false,
    });

    let mut trader: SOLTrader;
    let strategy_fn: fn(&mut SOLTrader, u32);
    let mut iterations: u32 = 100;
    let qty = 10_000f32;

    match strategy {
        "farouk" => {
            trader = SOLTrader::new_with_quantities("Farouk".to_string(), qty, qty, qty, qty);
            strategy_fn = farouk_strategy;
            iterations = 100;
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
