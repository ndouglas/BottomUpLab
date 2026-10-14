mod cake;
mod pareto;
mod priority;
mod ratio;
mod reference;

use super::{history_weights, update_weights, Error, HistoryResult, Observation, Ratio, WireRatio};

fn r(num: i128, den: i128) -> Ratio {
    Ratio::new(num, den).unwrap()
}
