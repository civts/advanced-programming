use std::io;

pub(crate) enum AppView {
    WaitingForFirstTrade,
    MainTradingView,
    HelpMenu,
    ErrorView(io::Error),
}

impl Default for AppView {
    fn default() -> Self {
        AppView::WaitingForFirstTrade
    }
}
