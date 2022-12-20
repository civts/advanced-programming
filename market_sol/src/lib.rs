//! # SOL Market
//! Welcome to the SOL market library 🌞
//!
//! This crate implements a "market" following the
//! [specifications](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/20e6bd88eb8911932e4f374cefbaed13deb4ba82/market-protocol-specifications.md)
//! of the Working Group.
//!
//! You can find the source code or file any issue you may have in the official
//! [GitHub repository](https://github.com/civts/advanced-programming).
//!
//! # Constructors
//!
//! The SOL market exposes the three cosntructors you are used to from the specs:
//! - `new_random`
//! - `new_qith_quantities`
//! - `new_file`
//!
//! Note that when you use `new_file`, the market will save its state on the
//! provided path when it is dropped.
//! If a file was already present on that path, it will be renamed to avoid
//! overwriting it.
//!
//! # Price change logic
//! We have three separate strategies that determine the price. We called them
//! "stocastic", "quantity" and "others".
//!
//! Let us denote the stocastic price with $p_s$, the quantity price with $p_q$,
//! and the price based on the other markets with $p_o$.
//! The price that the trader sees is given by their weighted average -with $w_s$,
//! $w_q$, and $w_o$ being the weights-.
//!
//! $price=\frac {(p_s*w_s)+(p_q*w_q)+(p_o*w_o)} {w_s+w_q+w_o}$
//!
//! The value of the weights is initialized randomly every time an instance of
//! `SOLMarket` is created. Each weight will be between 0 and 1, and the sampling
//!  is from a uniform distribution.
//!
//! An exception to this is when you initialize from a file. In that case, the
//! weights will be read from there.
//!
//! Let us now consider the logic behind each price changing strategy.
//!
//! ## Stocastic
//!  
//! With this strategy, the market goes in **seasons**. More specifically, each
//! good changes accordingly to its season, independently from the others.
//! A season can last from 20 to 365 days.
//!
//! At the start of the season, the market decides how much the price will change,
//! in percentage, sampling from a gaussian distribution. Then, for every day of
//! the season, the price will fall exactly on the line that connects the point
//! (starting_day, starting_price), to (ending_day, ending_price) plus a gaussian
//! noise.
//!
//! ## Quantity
//!
//! In this strategy, the price of a good increases as its supply decreases and
//! vice-versa.
//!
//! ## Others
//!
//! In this strategy, the price of a good is equal to the latest price of that
//! good (independently from the fact that it was bought or sold) on the other
//! markets.
//!
//! # Internal trading
//!
//! The market will try to reach a "perfect" distribution of the goods.
//! By perfect distribution we mean that each good should have the same market
//! cap in DEFAULT_GOOD.
//!
//! Every 100 days the market :
//!   - Set, for each good, its role as an `Importer` or `Exporter` with some `needs`
//!     - If a good has a `positive need`, its value in EUR is lower than the perfect distribution
//!       - The good will be set as an `Importer` with `needs` equal to the amount of
//!         goods in EUR needed to reach the perfect distribution
//!     - If a good has a `negative need`, its value in EUR is higher than the perfect distribution
//!       - The good will be set as an `Exporter` with `needs` equal to the amount
//!         of goods in EUR needed to reach the perfect distribution
//!
//! If no Exporter has any more supply at a given moment, then internal trades
//! won't be possible until either
//!
//!   - A trader refill the supply by selling to the market
//!   - The market reset the roles (every 100 days)
//!
//! The `Exporter` with the lowest negative needs (maximum surplus) gets rid of
//! some of its goods to fill the needs of the `Importer` with the highest
//! positive needs.
//!
//! Everytime a trader buy/sell a good from the market, we adjust its need
//! accordingly.

pub mod sol_market;

pub use self::sol_market::SOLMarket;

#[cfg(test)]
mod tests;
