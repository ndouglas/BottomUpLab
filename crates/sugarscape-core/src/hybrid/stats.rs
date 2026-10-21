//! Per-tick series: the humans' cooperation (their cooperating share, or mean strategy), the
//! bots' cooperating share, the bots' share of the nodes, the world's exact reference, and for
//! coloring the conflicts, the solve time and the game clock (NaN where a series does not apply).

use crate::stats::Series;
use serde::Serialize;

pub const SERIES: [&str; 7] = [
    "cooperation",
    "bot_cooperation",
    "bot_share",
    "reference",
    "conflicts",
    "solved_at",
    "game_time",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HybridSnapshot {
    pub tick: u64,
    pub cooperation: f64,
    pub bot_cooperation: f64,
    pub bot_share: f64,
    pub reference: f64,
    pub conflicts: f64,
    pub solved_at: f64,
    pub game_time: f64,
}

impl Default for HybridSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            cooperation: f64::NAN,
            bot_cooperation: f64::NAN,
            bot_share: f64::NAN,
            reference: f64::NAN,
            conflicts: f64::NAN,
            solved_at: f64::NAN,
            game_time: f64::NAN,
        }
    }
}

impl Series for HybridSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "cooperation" => self.cooperation,
            "bot_cooperation" => self.bot_cooperation,
            "bot_share" => self.bot_share,
            "reference" => self.reference,
            "conflicts" => self.conflicts,
            "solved_at" => self.solved_at,
            "game_time" => self.game_time,
            _ => return None,
        })
    }
}

pub fn series_names() -> Vec<String> {
    SERIES.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_series_is_readable_and_defaults_to_nan() {
        let s = HybridSnapshot::default();
        assert_eq!(s.value("tick"), Some(0.0));
        for name in SERIES {
            assert!(s.value(name).unwrap().is_nan(), "{name}");
        }
        assert!(s.value("nope").is_none());
        assert_eq!(series_names().len(), 7);
    }
}
