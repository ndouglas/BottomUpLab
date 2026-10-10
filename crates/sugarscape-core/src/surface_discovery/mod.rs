//! Owned calibration decisions composed with the frozen shared surface physics.
mod kernel;
mod types;
pub use kernel::{advance_one, select_choice, EpisodeState, Step};
pub use types::*;
#[cfg(test)]
mod tests;
