//! Registered paid gate assistance, with one own-state controller and kernel.
mod canonical;
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

/// Explicit compiled dependency order for gate source receipts.
pub const SOURCE_INPUTS: &[(&str, &[u8])] = &[
    (
        "crates/sugarscape-core/src/gate_assistance/mod.rs",
        include_bytes!("mod.rs"),
    ),
    (
        "crates/sugarscape-core/src/gate_assistance/types.rs",
        include_bytes!("types.rs"),
    ),
    (
        "crates/sugarscape-core/src/gate_assistance/controller.rs",
        include_bytes!("controller.rs"),
    ),
    (
        "crates/sugarscape-core/src/gate_assistance/kernel.rs",
        include_bytes!("kernel.rs"),
    ),
    (
        "crates/sugarscape-core/src/gate_assistance/observation.rs",
        include_bytes!("observation.rs"),
    ),
    (
        "crates/sugarscape-core/src/gate_assistance/canonical.rs",
        include_bytes!("canonical.rs"),
    ),
    (
        "crates/sugarscape-core/src/surface_discovery/mod.rs",
        include_bytes!("../surface_discovery/mod.rs"),
    ),
    (
        "crates/sugarscape-core/src/surface_discovery/types.rs",
        include_bytes!("../surface_discovery/types.rs"),
    ),
    (
        "crates/sugarscape-core/src/surface_discovery/diagnostics.rs",
        include_bytes!("../surface_discovery/diagnostics.rs"),
    ),
    (
        "crates/sugarscape-core/src/surface_discovery/diagnostics/hash.rs",
        include_bytes!("../surface_discovery/diagnostics/hash.rs"),
    ),
];
