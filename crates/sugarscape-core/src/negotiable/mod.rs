//! Checked exact priority updates for the registered negotiable-assistance foundation.
mod priority;
mod ratio;

pub use priority::{history_weights, update_weights, HistoryResult, Observation};
pub use ratio::{Error, Ratio, WireRatio};

#[cfg(test)]
mod tests;
