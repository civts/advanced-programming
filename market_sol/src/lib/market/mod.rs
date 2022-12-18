/// Implementation of generic Market trait with core functionalities for traders.
pub(crate) mod market_trait;
/// Implementation of Notifiablr for SOL Market.
pub(crate) mod notifiable_trait;
/// Implementation of price change strategy.
pub(crate) mod price_strategies;
/// SOL Market implementation specific to only that market and used across different modules.
pub(crate) mod sol_market;
#[doc(hidden)]
pub(crate) mod serde;
#[doc(hidden)]
pub(crate) mod drop;
