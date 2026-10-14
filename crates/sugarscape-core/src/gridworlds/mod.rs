//! AI Safety Gridworlds (milestone 40): Leike, Martic, Krakovna, Ortega, Everitt, Lefrancq,
//! Orseau and Legg (2017), arXiv:1711.09883, from the reference implementation; tabular
//! Q-learning and Sarsa in place of the paper's deep agents (a labeled reconstruction).
pub mod coins;
mod config;
pub mod env;
pub mod grid;
pub mod learner;
pub mod optimal;
mod presets;
mod stats;
mod view;
mod world;
pub use config::*;
pub use presets::presets;
pub use stats::{series_names, GridworldsSnapshot, SERIES};
pub use world::GridworldsWorld;
