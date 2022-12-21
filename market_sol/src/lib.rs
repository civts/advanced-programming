#![doc = include_str!("../README.md")]

pub mod sol_market;

pub use self::sol_market::SOLMarket;

#[cfg(test)]
mod tests;
