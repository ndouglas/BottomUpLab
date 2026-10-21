//! Guo et al. (iScience 2023; arXiv 2307.01123 and its supplement's source): games with R = 1,
//! P = 0 on the (T, S) plane, bots that cooperate with probability φ, and y bots per human. The
//! replicator dynamics (Eqs 3–5, Theorems 1–3) and the pairwise-comparison master equation
//! (Eqs 6–9) are exact references; Algorithms I and II follow in this file's second part.

use super::exact::{logistic, roots, Root};
use serde::{Deserialize, Serialize};

/// A game with R = 1 and P = 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub t: f64,
    pub s: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    FullC,
    FullD,
    Coexistence,
    Bistable,
}

impl Plane {
    /// The row player's payoff (true = cooperate).
    pub fn payoff(&self, me: bool, other: bool) -> f64 {
        match (me, other) {
            (true, true) => 1.0,
            (true, false) => self.s,
            (false, true) => self.t,
            (false, false) => 0.0,
        }
    }

    /// ρ̂ = (P − S)/(R − T − S + P), the interior rest point without bots.
    pub fn rho_hat(&self) -> f64 {
        -self.s / (1.0 - self.t - self.s)
    }

    /// z = (ρ + yφ)/(1 + y), the share of all players cooperating.
    pub fn z(rho: f64, y: f64, phi: f64) -> f64 {
        (rho + y * phi) / (1.0 + y)
    }

    /// π_C − π_D = z(R − T − S + P) + S − P (Eq 4).
    pub fn gap(&self, rho: f64, y: f64, phi: f64) -> f64 {
        Self::z(rho, y, phi) * (1.0 - self.t - self.s) + self.s
    }

    /// ρ̇ under replicator dynamics (Eq 3): ρ(1 − ρ)(π_C − π_D)/(1 + y).
    pub fn replicator(&self, rho: f64, y: f64, phi: f64) -> f64 {
        rho * (1.0 - rho) * self.gap(rho, y, phi) / (1.0 + y)
    }

    /// ρ̇ = Q⁺ − Q⁻ under pairwise comparison with imitation strength K (Eqs 6–9): the reference
    /// is drawn from all 1 + y units, bots included, credited with their action's payoff.
    pub fn master(&self, rho: f64, y: f64, phi: f64, k: f64) -> f64 {
        let g = self.gap(rho, y, phi);
        let p1 = rho * (1.0 - rho + y * (1.0 - phi)) / (1.0 + y);
        let p2 = (rho + y * phi) * (1.0 - rho) / (1.0 + y);
        p2 * logistic(k * g) - p1 * logistic(-k * g)
    }

    /// T > R, S > P.
    pub fn snowdrift(&self) -> bool {
        self.t > 1.0 && self.s > 0.0
    }

    /// R > T, P > S.
    pub fn stag_hunt(&self) -> bool {
        self.t < 1.0 && self.s < 0.0
    }

    /// Theorems 1–3. z runs from yφ/(1 + y) (no human cooperates) to (1 + yφ)/(1 + y).
    pub fn phase_closed(&self, y: f64, phi: f64) -> Phase {
        let lo = y * phi / (1.0 + y);
        let hi = (1.0 + y * phi) / (1.0 + y);
        let h = self.rho_hat();
        if self.snowdrift() {
            if hi <= h {
                Phase::FullC
            } else if lo >= h {
                Phase::FullD
            } else {
                Phase::Coexistence
            }
        } else if self.stag_hunt() {
            if lo >= h {
                Phase::FullC
            } else if hi <= h {
                Phase::FullD
            } else {
                Phase::Bistable
            }
        } else if self.t > 1.0 {
            Phase::FullD
        } else {
            Phase::FullC
        }
    }

    /// The distance in z from the nearer closed-form boundary; G1 compares only where it exceeds
    /// 1e-9.
    pub fn boundary_distance(&self, y: f64, phi: f64) -> f64 {
        let h = self.rho_hat();
        (y * phi / (1.0 + y) - h)
            .abs()
            .min(((1.0 + y * phi) / (1.0 + y) - h).abs())
    }

    /// The fewest bots for full cooperation in snowdrift, at φ = 0: (T − R)/(S − P).
    pub fn y_min(&self) -> f64 {
        (self.t - 1.0) / self.s
    }
}

/// The phase read off the numerical roots of Eq 3 on [0, 1] (1,000 cells); None if the stable set
/// is none of the four.
pub fn phase_numeric(plane: &Plane, y: f64, phi: f64) -> Option<Phase> {
    let stable: Vec<f64> = roots(|r| plane.replicator(r, y, phi), 0.0, 1.0, 1000)
        .into_iter()
        .filter(|r| r.stable)
        .map(|r| r.x)
        .collect();
    match stable.as_slice() {
        [x] if *x == 1.0 => Some(Phase::FullC),
        [x] if *x == 0.0 => Some(Phase::FullD),
        [x] if *x > 0.0 && *x < 1.0 => Some(Phase::Coexistence),
        [a, b] if *a == 0.0 && *b == 1.0 => Some(Phase::Bistable),
        _ => None,
    }
}

/// The master equation's roots on [0, 1], 10,000 cells.
pub fn master_roots(plane: &Plane, y: f64, phi: f64, k: f64) -> Vec<Root> {
    roots(|r| plane.master(r, y, phi, k), 0.0, 1.0, 10_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SD: Plane = Plane { t: 1.2, s: 0.5 };
    const SH: Plane = Plane { t: 0.6, s: -0.2 };
    const PD: Plane = Plane { t: 1.3, s: -0.2 };

    #[test]
    fn eq_4_is_the_payoff_gap_of_eq_2() {
        for (rho, y, phi) in [(0.3, 0.95, 0.25), (0.8, 4.0, 0.9), (0.0, 0.0, 0.0)] {
            for p in [SD, SH, PD] {
                let pi_c = (rho + (1.0 - rho) * p.s) / (1.0 + y)
                    + y * (phi + (1.0 - phi) * p.s) / (1.0 + y);
                let pi_d = rho * p.t / (1.0 + y) + y * phi * p.t / (1.0 + y);
                assert!((p.gap(rho, y, phi) - (pi_c - pi_d)).abs() < 1e-15);
            }
        }
    }

    #[test]
    fn the_interior_root_is_eq_5() {
        let (y, phi) = (0.5, 0.6);
        let h = SD.rho_hat();
        let want = h + y * (h - phi);
        let r = roots(|r| SD.replicator(r, y, phi), 0.0, 1.0, 1000);
        assert!(
            r.iter().any(|r| r.stable && (r.x - want).abs() < 1e-12),
            "{r:?} vs {want}"
        );
        assert!((SD.rho_hat() - 5.0 / 7.0).abs() < 1e-15 && (SD.y_min() - 0.4).abs() < 1e-15);
        assert!((SH.rho_hat() - 1.0 / 3.0).abs() < 1e-15);
    }

    #[test]
    fn theorems_agree_with_the_numerical_roots_away_from_boundaries() {
        // The survey checks the full grid (G1); these cells sit well inside their regions.
        for (plane, y, phi, phase) in [
            (PD, 2.0, 0.9, Phase::FullD),
            (SD, 0.3, 0.0, Phase::Coexistence),
            (SD, 1.0, 0.0, Phase::FullC),
            (SD, 4.0, 0.9, Phase::FullD),
            (SH, 4.0, 0.9, Phase::FullC),
            (SH, 0.5, 0.5, Phase::Bistable),
            (SH, 4.0, 0.0, Phase::FullD),
        ] {
            assert_eq!(plane.phase_closed(y, phi), phase, "{plane:?} y {y} φ {phi}");
            assert_eq!(
                phase_numeric(&plane, y, phi),
                Some(phase),
                "{plane:?} y {y} φ {phi}"
            );
            assert!(plane.boundary_distance(y, phi) > 1e-3);
        }
    }

    #[test]
    fn the_master_equation_runs_from_the_replicator_root_to_phi() {
        let (y, phi) = (0.5, 0.6);
        let want = SD.rho_hat() + y * (SD.rho_hat() - phi);
        let strong = master_roots(&SD, y, phi, 1e4);
        assert!(
            strong.iter().any(|r| r.stable && (r.x - want).abs() < 1e-3),
            "{strong:?}"
        );
        // As K → 0 the comparison is a coin toss and humans drift to the bots' φ.
        let weak = master_roots(&SD, y, phi, 1e-9);
        assert!(
            weak.iter().any(|r| r.stable && (r.x - phi).abs() < 1e-6),
            "{weak:?}"
        );
        assert!(
            SD.master(0.0, y, phi, 10.0) > 0.0 && SD.master(1.0, y, phi, 10.0) < 0.0,
            "bots keep both ends open"
        );
    }

    #[test]
    fn a_root_in_the_cell_next_to_an_exact_endpoint_zero_is_still_found() {
        // ρ* = 0.99971 sits inside the last cell, beside the exact zero at ρ = 1.
        let phase = SD.phase_closed(0.45, 0.08);
        assert_eq!(phase, Phase::Coexistence);
        assert_eq!(phase_numeric(&SD, 0.45, 0.08), Some(phase));
    }
}
