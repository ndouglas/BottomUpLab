//! Owned calibration decisions composed with the frozen shared surface physics.
mod kernel;
mod types;
pub use kernel::{advance_one, select_choice, EpisodeState, Step};
pub use types::*;
#[cfg(test)]
mod tests;

mod budget;
mod planner;
pub use budget::SearchStats;
pub use planner::{ChoiceValue, CompiledPolicy, Decision};

mod belief;

mod evaluation;
mod replay;
pub use evaluation::*;
pub use replay::Replay;
mod metrics;
pub use metrics::*;
mod diagnostics;
pub use diagnostics::*;

pub(crate) use diagnostics::{
    compiled_bytes_identity, compiled_file_identity, compiled_source_identity,
};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn raw_sha256(bytes: &[u8]) -> String {
    diagnostics::hash::sha256(bytes)
}
