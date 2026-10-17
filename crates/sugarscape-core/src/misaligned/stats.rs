//! Per-tick series: utility and the proxy, the gain over the start, the four attributes and
//! their sensitivities, the gaps to the optimum and to the current proxy's limit, the
//! interaction count, the OFF flag and the climber's acceptance share (NaN for other robots).
use crate::stats::Series;
use serde::Serialize;

pub const SERIES: [&str; 16] = [
    "utility",
    "proxy",
    "gain",
    "a",
    "b",
    "c",
    "d",
    "sensitivity_a",
    "sensitivity_b",
    "sensitivity_c",
    "sensitivity_d",
    "optimum_gap",
    "limit_gap",
    "interactions",
    "switched_off",
    "accepted",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MisalignedSnapshot {
    pub tick: u64,
    pub utility: f64,
    pub proxy: f64,
    pub gain: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub sensitivity_a: f64,
    pub sensitivity_b: f64,
    pub sensitivity_c: f64,
    pub sensitivity_d: f64,
    pub optimum_gap: f64,
    pub limit_gap: f64,
    pub interactions: f64,
    pub switched_off: f64,
    pub accepted: f64,
}

impl Default for MisalignedSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            utility: f64::NAN,
            proxy: f64::NAN,
            gain: f64::NAN,
            a: f64::NAN,
            b: f64::NAN,
            c: f64::NAN,
            d: f64::NAN,
            sensitivity_a: f64::NAN,
            sensitivity_b: f64::NAN,
            sensitivity_c: f64::NAN,
            sensitivity_d: f64::NAN,
            optimum_gap: f64::NAN,
            limit_gap: f64::NAN,
            interactions: f64::NAN,
            switched_off: f64::NAN,
            accepted: f64::NAN,
        }
    }
}

impl Series for MisalignedSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "utility" => self.utility,
            "proxy" => self.proxy,
            "gain" => self.gain,
            "a" => self.a,
            "b" => self.b,
            "c" => self.c,
            "d" => self.d,
            "sensitivity_a" => self.sensitivity_a,
            "sensitivity_b" => self.sensitivity_b,
            "sensitivity_c" => self.sensitivity_c,
            "sensitivity_d" => self.sensitivity_d,
            "optimum_gap" => self.optimum_gap,
            "limit_gap" => self.limit_gap,
            "interactions" => self.interactions,
            "switched_off" => self.switched_off,
            "accepted" => self.accepted,
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
        let s = MisalignedSnapshot::default();
        assert_eq!(s.value("tick"), Some(0.0));
        for name in SERIES {
            assert!(s.value(name).unwrap().is_nan(), "{name}");
        }
        assert!(s.value("nope").is_none());
        assert_eq!(series_names().len(), SERIES.len());
        assert_eq!(SERIES.len(), 16);
    }
}
