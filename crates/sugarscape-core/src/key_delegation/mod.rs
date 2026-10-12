//! Registered carrier-only key delegation with private local execution.
mod canonical;
mod controller;
mod kernel;
mod observation;
mod sha256;
mod types;
mod validation;
pub use canonical::trace_hash;
pub use controller::choose;
pub use kernel::{run_episode, run_with_capture};
pub use types::*;
#[cfg(test)]
mod tests;
