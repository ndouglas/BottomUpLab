//! Hybrid human–machine populations (milestone 42): fixed-behavior bots among imitating humans.
//! Four worlds share one engine: Booker et al.'s donation game with Samaritan or discriminatory
//! bots, Guo et al.'s (T, S) plane with probabilistic bots, Si et al.'s r-family Prisoner's
//! Dilemma in three strategy frameworks, and Shirado and Christakis's network coloring game with
//! noisy bots. Exact references are pure functions of the config.
pub mod booker;
pub mod coloring;
mod config;
pub mod engine;
pub mod exact;
pub mod graph;
pub mod guo;
pub mod si;
mod stats;
mod view;
mod world;
pub use config::*;
pub use stats::{series_names, HybridSnapshot, SERIES};
pub use world::{Body, HybridWorld, Runner};
