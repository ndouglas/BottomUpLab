// Wire fields are consumed by later archive/source-binding stages.
pub mod archive;
#[cfg(test)]
mod archive_tests;
pub mod cli;
pub mod gate;
#[cfg(test)]
mod gate_tests;
pub mod io;
#[cfg(test)]
mod io_tests;
pub mod numerics;
#[cfg(test)]
mod numerics_tests;
pub mod observer;
#[cfg(test)]
mod observer_tests;
pub mod source;
#[cfg(test)]
mod source_tests;
#[allow(dead_code)]
pub mod wire;
#[cfg(test)]
mod wire_tests;
