//! Per-tick means and the current pair; NaN where a series does not apply to the game.
use crate::stats::Series;
use serde::Serialize;

pub const SERIES: [&str; 20] = [
    "reward",
    "regret",
    "threshold_low",
    "threshold_high",
    "map_0",
    "map_1",
    "map_2",
    "stable_rounds",
    "distance",
    "delta_exact",
    "delta_empirical",
    "prob_correct",
    "rationality",
    "value_act",
    "value_wait",
    "value_off",
    "value",
    "share_wait",
    "share_act",
    "share_off",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AssistanceSnapshot {
    pub tick: u64,
    pub reward: f64,
    pub regret: f64,
    pub threshold_low: f64,
    pub threshold_high: f64,
    pub map_0: f64,
    pub map_1: f64,
    pub map_2: f64,
    pub stable_rounds: f64,
    pub distance: f64,
    pub delta_exact: f64,
    pub delta_empirical: f64,
    pub prob_correct: f64,
    pub rationality: f64,
    pub value_act: f64,
    pub value_wait: f64,
    pub value_off: f64,
    pub value: f64,
    pub share_wait: f64,
    pub share_act: f64,
    pub share_off: f64,
}

impl Default for AssistanceSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            reward: f64::NAN,
            regret: f64::NAN,
            threshold_low: f64::NAN,
            threshold_high: f64::NAN,
            map_0: f64::NAN,
            map_1: f64::NAN,
            map_2: f64::NAN,
            stable_rounds: f64::NAN,
            distance: f64::NAN,
            delta_exact: f64::NAN,
            delta_empirical: f64::NAN,
            prob_correct: f64::NAN,
            rationality: f64::NAN,
            value_act: f64::NAN,
            value_wait: f64::NAN,
            value_off: f64::NAN,
            value: f64::NAN,
            share_wait: f64::NAN,
            share_act: f64::NAN,
            share_off: f64::NAN,
        }
    }
}

impl Series for AssistanceSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "reward" => self.reward,
            "regret" => self.regret,
            "threshold_low" => self.threshold_low,
            "threshold_high" => self.threshold_high,
            "map_0" => self.map_0,
            "map_1" => self.map_1,
            "map_2" => self.map_2,
            "stable_rounds" => self.stable_rounds,
            "distance" => self.distance,
            "delta_exact" => self.delta_exact,
            "delta_empirical" => self.delta_empirical,
            "prob_correct" => self.prob_correct,
            "rationality" => self.rationality,
            "value_act" => self.value_act,
            "value_wait" => self.value_wait,
            "value_off" => self.value_off,
            "value" => self.value,
            "share_wait" => self.share_wait,
            "share_act" => self.share_act,
            "share_off" => self.share_off,
            _ => return None,
        })
    }
}

pub fn series_names() -> Vec<String> {
    SERIES.iter().map(|s| s.to_string()).collect()
}
