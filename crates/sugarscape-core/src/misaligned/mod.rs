//! Consequences of misaligned AI (milestone 43): Zhuang and Hadfield-Menell's resource world
//! (NeurIPS 2020), a robot that raises a proxy on a subset of the attributes the human cares
//! about, the paper's recommender example, and the mitigations. Closed forms are the targets;
//! the climber, the starts and the human's interval are ours.
mod config;
pub mod human;
pub mod robots;
mod stats;
mod view;
mod world;
pub mod worlds;
pub use config::*;
pub use stats::{series_names, MisalignedSnapshot, SERIES};
pub use world::{MisalignedWorld, Outcome, ProxyChange};
