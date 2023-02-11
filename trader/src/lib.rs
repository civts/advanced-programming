pub mod logging;
pub mod trader;

pub use self::logging::*;

#[cfg(test)]
mod tests;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
