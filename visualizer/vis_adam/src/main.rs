mod vizualization;

use ipc_utils::trading_event::TradingEvent;
use ipc_utils::trading_event_details::TradingEventDetails;
use vizualization::*;

fn main() {
    viz().expect("Visualizer should start without problems!");
}
