pub(crate) enum AppView {
    WaitingForFirstTrade,
    MainTradingView,
    HelpMenu,
    FarewellScreen,
}

impl Default for AppView {
    fn default() -> Self {
        AppView::WaitingForFirstTrade
    }
}
