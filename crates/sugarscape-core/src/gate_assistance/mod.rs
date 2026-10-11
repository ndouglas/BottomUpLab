//! Registered paid gate assistance, with one own-state controller and kernel.
mod budget;
mod canonical;
mod game;
pub use budget::{CompileStats, Limits};
pub use game::{classify, compile, resolve, Analysis, Classification, GameTable, Row, Selector};
mod metrics;
pub use metrics::*;
mod controller;
mod kernel;
mod observation;
mod types;

pub use canonical::trace_hash;
pub use controller::choose;
pub use kernel::{run_episode, run_with_capture};
pub use types::*;

#[cfg(test)]
mod tests;

mod source_inputs;
pub use source_inputs::{REFERENCE_INPUTS, SOURCE_INPUTS};
mod protocol;
pub use protocol::*;
mod diagnostics;
mod references;
pub use diagnostics::{diagnose, reconstruct_episode, report_integrity, summary_json};
