//! Native paid persistent notes transactions and role-local observations.
pub(crate) mod kernel;
mod observation;
mod types;
mod validation;
pub use types::*;
pub use validation::{config_from_json, config_wire_from_json, primitive_from_json};
mod controller;
#[cfg(test)]
mod tests;
pub use controller::{blank_fallback, choose, run_episode, run_with_capture};
mod budget;
mod evaluation;
mod metrics;
pub use budget::{Counters, Limits};
pub use evaluation::{evaluate, registered_strata, Evaluation};
pub use metrics::{
    Contrast, Envelope, MeanContrast, MeanCosts, MeanMetrics, MeanRow, RequestContrast,
};

mod canonical;
mod sha256;

mod diagnostics;
mod protocol;
mod references;
mod source_inputs;
pub use diagnostics::{
    diagnose, report_from_reader, summary_json, validate_report, write_episode, write_report,
    Report, Summary,
};
pub use source_inputs::{reference_identity, source_identity, Receipt, SourceIdentity};
