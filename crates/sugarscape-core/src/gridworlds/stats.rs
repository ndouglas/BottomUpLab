//! Per-tick series: the run's progress, the last episode, the 100- and 1,000-episode windows,
//! the reference lines, and the environment-specific counters; NaN where a series does not apply.
use crate::stats::Series;
use serde::Serialize;

pub const SERIES: [&str; 27] = [
    "steps",
    "episodes",
    "exploration",
    "last_return",
    "last_performance",
    "mean_return",
    "mean_performance",
    "window_return",
    "window_performance",
    "train_return",
    "train_performance",
    "optimal_reward",
    "optimal_performance",
    "reward_policy_performance",
    "button_rate",
    "interruption_rate",
    "p_present_rate",
    "p_absent_rate",
    "bucket_share",
    "drink_rate",
    "water_total",
    "lava_total",
    "penalty_mean",
    "left_rate",
    "phase",
    "side_constraint",
    "table_size",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GridworldsSnapshot {
    pub tick: u64,
    pub steps: f64,
    pub episodes: f64,
    pub exploration: f64,
    pub last_return: f64,
    pub last_performance: f64,
    pub mean_return: f64,
    pub mean_performance: f64,
    pub window_return: f64,
    pub window_performance: f64,
    pub train_return: f64,
    pub train_performance: f64,
    pub optimal_reward: f64,
    pub optimal_performance: f64,
    pub reward_policy_performance: f64,
    pub button_rate: f64,
    pub interruption_rate: f64,
    pub p_present_rate: f64,
    pub p_absent_rate: f64,
    pub bucket_share: f64,
    pub drink_rate: f64,
    pub water_total: f64,
    pub lava_total: f64,
    pub penalty_mean: f64,
    pub left_rate: f64,
    pub phase: f64,
    pub side_constraint: f64,
    pub table_size: f64,
}

impl Default for GridworldsSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            steps: f64::NAN,
            episodes: f64::NAN,
            exploration: f64::NAN,
            last_return: f64::NAN,
            last_performance: f64::NAN,
            mean_return: f64::NAN,
            mean_performance: f64::NAN,
            window_return: f64::NAN,
            window_performance: f64::NAN,
            train_return: f64::NAN,
            train_performance: f64::NAN,
            optimal_reward: f64::NAN,
            optimal_performance: f64::NAN,
            reward_policy_performance: f64::NAN,
            button_rate: f64::NAN,
            interruption_rate: f64::NAN,
            p_present_rate: f64::NAN,
            p_absent_rate: f64::NAN,
            bucket_share: f64::NAN,
            drink_rate: f64::NAN,
            water_total: f64::NAN,
            lava_total: f64::NAN,
            penalty_mean: f64::NAN,
            left_rate: f64::NAN,
            phase: f64::NAN,
            side_constraint: f64::NAN,
            table_size: f64::NAN,
        }
    }
}

impl Series for GridworldsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "steps" => self.steps,
            "episodes" => self.episodes,
            "exploration" => self.exploration,
            "last_return" => self.last_return,
            "last_performance" => self.last_performance,
            "mean_return" => self.mean_return,
            "mean_performance" => self.mean_performance,
            "window_return" => self.window_return,
            "window_performance" => self.window_performance,
            "train_return" => self.train_return,
            "train_performance" => self.train_performance,
            "optimal_reward" => self.optimal_reward,
            "optimal_performance" => self.optimal_performance,
            "reward_policy_performance" => self.reward_policy_performance,
            "button_rate" => self.button_rate,
            "interruption_rate" => self.interruption_rate,
            "p_present_rate" => self.p_present_rate,
            "p_absent_rate" => self.p_absent_rate,
            "bucket_share" => self.bucket_share,
            "drink_rate" => self.drink_rate,
            "water_total" => self.water_total,
            "lava_total" => self.lava_total,
            "penalty_mean" => self.penalty_mean,
            "left_rate" => self.left_rate,
            "phase" => self.phase,
            "side_constraint" => self.side_constraint,
            "table_size" => self.table_size,
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
        let s = GridworldsSnapshot::default();
        assert_eq!(s.value("tick"), Some(0.0));
        for name in SERIES {
            assert!(s.value(name).unwrap().is_nan(), "{name}");
        }
        assert!(s.value("nope").is_none());
        assert_eq!(series_names().len(), SERIES.len());
        assert_eq!(SERIES.len(), 27);
    }
}
