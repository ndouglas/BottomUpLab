//! Assistance games (milestone 39): the paperclip game (Hadfield-Menell, Dragan, Abbeel and
//! Russell, NeurIPS 2016, §3.3 and appendix §6) and the off-switch game (IJCAI 2017), exact
//! where the papers are exact, with two equilibrium dynamics that are ours.
mod config;
pub mod dynamics;
pub mod offswitch;
pub mod paperclips;
mod presets;
mod stats;
mod view;
mod world;
pub use config::*;
pub use presets::presets;
pub use stats::{series_names, AssistanceSnapshot, SERIES};
pub use world::{AssistanceWorld, Outcome, PairChange};
