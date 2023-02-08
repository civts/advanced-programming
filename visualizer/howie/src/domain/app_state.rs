use super::app_view::AppView;

#[derive(Default)]
pub(crate) struct AppState {
    /// The screen the user is viewing
    pub(crate) current_view: AppView,

    /// How many trading events we received so far
    pub(crate) received_messages: u64,
}
