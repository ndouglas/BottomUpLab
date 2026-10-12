//! Registered carrier-only key delegation with private local execution.
// Strict object wire decoding preserves duplicate/integer checks and required nullable fields.
macro_rules! wire_object {
 ($name:ident { $(pub $field:ident:$ty:ty),* $(,)? }) => {
  #[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
  pub struct $name {$(pub $field:$ty),*}
  impl<'de> serde::Deserialize<'de> for $name {
   fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
    #[derive(serde::Serialize,serde::Deserialize)] #[serde(deny_unknown_fields)]
    struct Wire{$($field:$ty),*}
    let w:Wire=super::validation::decode(super::validation::strict_value(d)?)?;
    Ok(Self{$($field:w.$field),*})
   }
  }
 };
}
mod diagnostics;
mod protocol;
mod references;
mod source_inputs;
pub use diagnostics::*;
pub use source_inputs::{
    reference_identity, reference_receipts, source_identity, source_receipts, Receipt,
    REFERENCE_INPUTS, SOURCE_INPUTS,
};
mod budget;
mod canonical;
mod controller;
mod game;
mod kernel;
mod metrics;
mod observation;
mod sha256;
mod types;
mod validation;
pub use budget::{CompileStats, Limits, PeakWitness};
pub use canonical::trace_hash;
pub use controller::choose;
pub use game::*;
pub use kernel::{run_episode, run_with_capture};
pub use metrics::*;
pub use types::*;
#[cfg(test)]
mod tests;
