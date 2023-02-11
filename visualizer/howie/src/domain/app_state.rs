use ipc_utils::trading_event::TradingEvent;

use super::{app_view::AppView, stats::Stats};

#[derive(Default)]
pub(crate) struct AppState {
    /// The screen the user is viewing
    pub(crate) current_view: AppView,

    /// How many trading events we received so far
    pub(crate) received_messages: u64,

    /// The events we received from the trader
    pub(crate) events: Vec<TradingEvent>,

    /// The trading statistics relative to this session
    pub(crate) stats: Stats,

    pub(crate) paused: bool,

    /// If we are showing the trading volume chart
    pub(crate) trading_volume_chart_visible: bool,
}

impl AppState {
    pub(crate) fn update(&mut self, event: &TradingEvent) {
        self.received_messages += 1;
        self.stats.update(event.clone());
        self.events.push(event.clone());
    }
}
