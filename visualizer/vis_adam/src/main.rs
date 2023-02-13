mod visualization;

use ipc_utils::trader_state::ALL_GOOD_KINDS;
use unitn_market_2022::good::good_kind::GoodKind;
use visualization::*;
use crate::visualization::service::plotlib::generate_all_balances_plot;


fn main() {
    let visualisation = Visualization::new();
    visualisation.start().expect("Visualizer should start!");
}
