//! Native paid persistent notes transactions and role-local observations.
pub(crate) mod kernel;
mod observation;
mod types;
mod validation;
pub use types::*;
pub use validation::{config_from_json, primitive_from_json};
#[cfg(test)]
mod tests;
