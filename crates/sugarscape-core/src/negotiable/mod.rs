//! Checked exact calculations for the registered negotiable-assistance foundation.
mod cake;
mod pareto;
mod priority;
mod ratio;

pub use cake::{cake_payoff, cake_rows, registered_weights, CakeRow, Policy};
pub use pareto::{classify, Classification, ParetoKind, Point, Witness, WitnessPart};
pub use priority::{history_weights, update_weights, HistoryResult, Observation};
pub use ratio::{Error, Ratio, WireRatio};

#[cfg(test)]
mod tests;
