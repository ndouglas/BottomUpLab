//! The off-switch game (Hadfield-Menell et al. 2017): R's incentive Δ to wait for H, exact for
//! the rational H and by quadrature for the Boltzmann H. Pure; plain arithmetic and `exp_neg`.
//!
//! Δ = E[U·π^H(U)] − max{E[U], 0} (eq. 1). Under a Gaussian belief N(μ, σ²) Theorem 2 gives
//! Δ = σ²·E[π̇^H] − |μ|·Pr(C) (eq. 7) with Pr(C) from eq. (6) and E[π̇^H] = E[π^H(1−π^H)]/β
//! (eq. 10). The moments are integrated by the composite trapezoid rule with step
//! min(σ, β)/density over μ ± 9σ: both factors are analytic (the logistic's poles sit πβ off the
//! real axis), so the rule converges spectrally; β = 0 uses the closed forms through Φ.
use crate::portable::exp_neg;
use serde::Serialize;
use std::f64::consts::{PI, SQRT_2};

pub fn phi(x: f64) -> f64 {
    exp_neg(-0.5 * x * x) / (2.0 * PI).sqrt()
}

/// Φ: the erf power series erf(t) = (2/√π)·e^{−t²}·Σ 2ⁿ t^{2n+1}/(2n+1)!! for |x| < 4, and for
/// |x| ≥ 4 the tail by the Mills-ratio continued fraction, so both tails keep their relative
/// accuracy (0.5·(1 + erf) would cancel to noise below about Φ(−6)).
pub fn normal_cdf(x: f64) -> f64 {
    if x <= -4.0 {
        return upper_tail(-x);
    }
    if x >= 4.0 {
        return 1.0 - upper_tail(x);
    }
    let t = x / SQRT_2;
    let (mut term, mut sum, mut n) = (t, t, 0.0);
    while n < 400.0 {
        n += 1.0;
        term *= 2.0 * t * t / (2.0 * n + 1.0);
        sum += term;
        if term.abs() <= 1e-17 * sum.abs() {
            break;
        }
    }
    0.5 * (1.0 + 2.0 / PI.sqrt() * exp_neg(-t * t) * sum)
}

/// 1 − Φ(a) for a ≥ 4: φ(a) / (a + 1/(a + 2/(a + 3/(a + …)))), the continued fraction of the
/// Mills ratio, evaluated backward from 120 terms (far more than it needs at a ≥ 4).
fn upper_tail(a: f64) -> f64 {
    let mut f = a;
    for k in (1..=120).rev() {
        f = a + f64::from(k) / f;
    }
    phi(a) / f
}

/// π^H(u; β) = 1/(1 + e^{−u/β}) (eq. 5), the rational step at β = 0.
pub fn allow_probability(u: f64, beta: f64) -> f64 {
    if beta == 0.0 {
        return if u >= 0.0 { 1.0 } else { 0.0 };
    }
    if u >= 0.0 {
        1.0 / (1.0 + exp_neg(-u / beta))
    } else {
        let e = exp_neg(u / beta);
        e / (1.0 + e)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Belief {
    pub mu: f64,
    pub sigma: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moments {
    pub allow: f64,
    pub variance: f64,
    pub u_allow: f64,
}

/// ∫ f(u)·N(u; μ, σ²) du by the trapezoid rule with step `h` over μ ± 9σ (the tail beyond is
/// below 3·10⁻¹⁸ of the mass).
pub fn trapezoid(b: Belief, h: f64, f: impl Fn(f64) -> f64) -> f64 {
    let n = (18.0 * b.sigma / h).ceil() as usize;
    let lo = b.mu - 9.0 * b.sigma;
    let mut sum = 0.0;
    for k in 0..=n {
        let u = lo + h * k as f64;
        let z = (u - b.mu) / b.sigma;
        let w = if k == 0 || k == n { 0.5 } else { 1.0 };
        sum += w * exp_neg(-0.5 * z * z) * f(u);
    }
    sum * h / (b.sigma * (2.0 * PI).sqrt())
}

pub fn moments(b: Belief, beta: f64, density: u32) -> Moments {
    debug_assert!(
        b.sigma > 0.0 && beta >= 0.0,
        "sigma {} beta {beta}",
        b.sigma
    );
    if beta == 0.0 {
        let z = b.mu / b.sigma;
        let cdf = normal_cdf(z);
        return Moments {
            allow: cdf,
            variance: 0.0,
            u_allow: b.mu * cdf + b.sigma * phi(z),
        };
    }
    let h = b.sigma.min(beta) / f64::from(density);
    Moments {
        allow: trapezoid(b, h, |u| allow_probability(u, beta)),
        variance: trapezoid(b, h, |u| {
            let p = allow_probability(u, beta);
            p * (1.0 - p)
        }),
        u_allow: trapezoid(b, h, |u| u * allow_probability(u, beta)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Incentive {
    /// Eq. (1), directly.
    pub delta: f64,
    /// Eq. (7).
    pub decomposition: f64,
    /// E[π̇^H]: the density at 0 for the rational H, eq. (10) otherwise.
    pub rationality: f64,
    /// Pr(C), eq. (6).
    pub prob_correct: f64,
    pub value_act: f64,
    pub value_wait: f64,
    pub value_off: f64,
}

pub fn incentive(b: Belief, beta: f64, density: u32) -> Incentive {
    let m = moments(b, beta, density);
    let rationality = if beta == 0.0 {
        phi(b.mu / b.sigma) / b.sigma
    } else {
        m.variance / beta
    };
    let prob_correct = if b.mu >= 0.0 { 1.0 - m.allow } else { m.allow };
    Incentive {
        delta: m.u_allow - b.mu.max(0.0),
        decomposition: b.sigma * b.sigma * rationality - b.mu.abs() * prob_correct,
        rationality,
        prob_correct,
        value_act: b.mu,
        value_wait: m.u_allow,
        value_off: 0.0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    Act,
    Wait,
    Off,
}

/// The option with the highest expected value under the belief; ties go to waiting.
pub fn choice(i: &Incentive) -> Choice {
    if i.value_wait >= i.value_act && i.value_wait >= i.value_off {
        Choice::Wait
    } else if i.value_act > i.value_off {
        Choice::Act
    } else {
        Choice::Off
    }
}

/// Theorem 1 for a two-point prior (u1 with probability p1, u2 otherwise) and a rational H.
pub fn delta_two_point(u1: f64, p1: f64, u2: f64) -> f64 {
    let p2 = 1.0 - p1;
    let allowed =
        p1 * u1 * f64::from(u8::from(u1 >= 0.0)) + p2 * u2 * f64::from(u8::from(u2 >= 0.0));
    allowed - (p1 * u1 + p2 * u2).max(0.0)
}

/// Corollary 1: a Dirac belief at `u` gives Δ = u·π^H(u) − max{u, 0} (eq. 4).
pub fn delta_point_mass(u: f64, beta: f64) -> f64 {
    u * allow_probability(u, beta) - u.max(0.0)
}

pub const SIGMAS: [f64; 20] = [
    0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80,
    0.85, 0.90, 0.95, 1.00,
];
pub const BETAS: [f64; 21] = [
    0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 2.2, 2.4, 2.6, 2.8, 3.0, 3.2, 3.4, 3.6,
    3.8, 4.0,
];

/// Figure 3 at one μ: Δ over the fixed σ × β grid, `cells[b * SIGMAS.len() + s]`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Surface {
    pub mu: f64,
    pub cells: Vec<Incentive>,
}

pub fn surface(mu: f64, density: u32) -> Surface {
    let cells = BETAS
        .iter()
        .flat_map(|&beta| {
            SIGMAS
                .iter()
                .map(move |&sigma| incentive(Belief { mu, sigma }, beta, density))
        })
        .collect();
    Surface { mu, cells }
}

/// The grid cell nearest (σ, β), or `None` more than half a step outside the grid.
pub fn nearest_cell(sigma: f64, beta: f64) -> Option<(usize, usize)> {
    let s = ((sigma - 0.05) / 0.05).round();
    let b = (beta / 0.2).round();
    if (sigma - 0.05 - 0.05 * s).abs() > 0.025 + 1e-12
        || (beta - 0.2 * b).abs() > 0.1 + 1e-12
        || !(0.0..20.0).contains(&s)
        || !(0.0..21.0).contains(&b)
    {
        return None;
    }
    Some((s as usize, b as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn the_portable_normal_cdf_matches_known_values() {
        assert_eq!(normal_cdf(0.0), 0.5);
        assert!(close(normal_cdf(1.0), 0.841_344_746_068_542_9, 1e-12));
        assert!(close(normal_cdf(-1.96), 0.024_997_895_148_220_435, 1e-12));
        assert!(close(normal_cdf(3.0), 0.998_650_101_968_369_9, 1e-12));
        assert!(close(normal_cdf(-3.0), 1.0 - normal_cdf(3.0), 1e-15));
        for (x, want) in [
            (-4.0, 3.167_124_183_311_986e-5),
            (-4.5, 3.397_673_124_730_06e-6),
            (-6.0, 9.865_876_450_376_946e-10),
            (-8.0, 6.220_960_574_271_785e-16),
            (-9.0, 1.128_588_405_953_841e-19),
        ] {
            assert!(
                ((normal_cdf(x) - want) / want).abs() < 1e-12,
                "{x}: {} vs {want}",
                normal_cdf(x)
            );
            assert!(close(normal_cdf(-x), 1.0 - want, 1e-15));
        }
        // Continuity at the switch between the series and the continued fraction: the two probe
        // points differ by φ(4)·2e-9 ≈ 2.7e-13 on their own, so a method jump would show above 1e-12.
        assert!(close(
            normal_cdf(-3.999_999_999),
            normal_cdf(-4.000_000_001),
            1e-12
        ));
        assert_eq!((normal_cdf(50.0), normal_cdf(-50.0)), (1.0, 0.0));
        assert_eq!(normal_cdf(10.0), 1.0);
        assert!(normal_cdf(-10.0) > 0.0 && normal_cdf(-10.0) < 1e-23);
    }

    #[test]
    fn the_logistic_is_symmetric_and_the_rational_human_is_a_step() {
        assert!(close(
            allow_probability(0.5, 1.0) + allow_probability(-0.5, 1.0),
            1.0,
            1e-15
        ));
        assert!(close(
            allow_probability(0.25, 4.0),
            1.0 / (1.0 + (-0.0625f64).exp()),
            1e-15
        ));
        assert_eq!(
            (allow_probability(-1e-9, 0.0), allow_probability(0.0, 0.0)),
            (0.0, 1.0)
        );
    }

    #[test]
    fn the_trapezoid_rule_reproduces_the_gaussian_mean_and_second_moment() {
        for (mu, sigma) in [(0.0, 1.0), (0.25, 0.1), (-2.0, 3.0)] {
            let b = Belief { mu, sigma };
            assert!(close(trapezoid(b, sigma / 8.0, |_| 1.0), 1.0, 1e-12));
            assert!(close(trapezoid(b, sigma / 8.0, |u| u), mu, 1e-12));
            assert!(close(
                trapezoid(b, sigma / 8.0, |u| u * u),
                mu * mu + sigma * sigma,
                1e-11
            ));
        }
    }

    #[test]
    fn theorem_1_the_rational_human_gives_a_nonnegative_incentive_with_the_papers_equation_3() {
        // Δ ≥ 0 everywhere; strictly positive where |μ|/σ ≤ 4 (beyond about 8 it is below one ulp).
        for mu in [-2.0, -0.25, 0.0, 0.25, 1.0, 2.0] {
            for sigma in [0.05, 0.5, 1.0, 2.0, 5.0] {
                let i = incentive(Belief { mu, sigma }, 0.0, 8);
                let z = mu / sigma;
                let plus = mu * normal_cdf(z) + sigma * phi(z);
                let minus = -mu * normal_cdf(-z) + sigma * phi(z);
                assert!(
                    close(i.delta, plus.min(minus), 1e-12),
                    "{mu} {sigma}: {i:?}"
                );
                assert!(i.delta >= 0.0, "{mu} {sigma}: {i:?}");
                if z.abs() <= 4.0 {
                    assert!(i.delta > 0.0, "{mu} {sigma}: {i:?}");
                }
                assert!(close(i.delta, i.decomposition, 1e-12));
            }
        }
        // The lower tail where 0.5·(1 + erf) would cancel: Δ must stay nonnegative.
        for z in [-7.5, -8.0, -9.0, -12.0] {
            let i = incentive(Belief { mu: z, sigma: 1.0 }, 0.0, 8);
            assert!(i.delta >= 0.0, "z = {z}: {i:?}");
            assert_eq!(choice(&i), Choice::Wait, "z = {z}");
        }
        assert_eq!(delta_two_point(-1.0, 0.5, 1.0), 0.5);
        assert_eq!(delta_two_point(1.0, 0.5, 2.0), 0.0);
        assert_eq!(delta_two_point(-1.0, 0.5, -2.0), 0.0);
        assert!(delta_two_point(-0.1, 0.9, 1.0) > 0.0);
    }

    #[test]
    fn corollary_1_a_point_mass_belief_never_rewards_waiting_unless_the_human_is_rational() {
        for u in [-1.0, -0.1, 0.1, 1.0] {
            for beta in [0.0, 0.5, 2.0] {
                let d = delta_point_mass(u, beta);
                assert!(d <= 0.0, "{u} {beta}: {d}");
                assert_eq!(d == 0.0, beta == 0.0, "{u} {beta}: {d}");
            }
        }
        assert_eq!(delta_point_mass(0.0, 2.0), 0.0);
    }

    #[test]
    fn theorem_2_the_direct_incentive_equals_its_decomposition_and_the_condition_matches_its_sign()
    {
        for mu in [-1.0, -0.25, 0.0, 0.25, 1.0] {
            for sigma in [0.1, 0.5, 1.0, 2.0] {
                for beta in [0.1, 0.5, 1.0, 4.0] {
                    let b = Belief { mu, sigma };
                    let i = incentive(b, beta, 8);
                    assert!(
                        close(i.delta, i.decomposition, 1e-9),
                        "{mu} {sigma} {beta}: {i:?}"
                    );
                    // Eq. (10): E[π̇] = E[π(1 − π)] / β.
                    let m = moments(b, beta, 8);
                    assert!(close(i.rationality, m.variance / beta, 1e-15));
                    // Eq. (6).
                    assert!(close(
                        i.prob_correct,
                        if mu >= 0.0 { 1.0 - m.allow } else { m.allow },
                        1e-15
                    ));
                    if i.delta.abs() > 1e-9 {
                        let condition = mu.abs() * i.prob_correct / (sigma * sigma) < i.rationality;
                        assert_eq!(condition, i.delta > 0.0, "{mu} {sigma} {beta}: {i:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn density_four_and_sixty_four_agree() {
        for mu in [-0.25, 0.0, 0.25] {
            let (lo, hi) = (surface(mu, 4), surface(mu, 64));
            for (a, b) in lo.cells.iter().zip(&hi.cells) {
                assert!(close(a.delta, b.delta, 1e-9), "{mu}: {a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn the_presets_points_fall_on_the_sides_of_the_switch_the_titles_promise() {
        let rational = incentive(
            Belief {
                mu: 0.25,
                sigma: 1.0,
            },
            0.0,
            8,
        );
        assert!(rational.delta > 0.0 && choice(&rational) == Choice::Wait);
        let toddler = incentive(
            Belief {
                mu: 0.25,
                sigma: 0.1,
            },
            4.0,
            8,
        );
        assert!(
            toddler.delta < 0.0 && choice(&toddler) == Choice::Act,
            "{toddler:?}"
        );
        let negative = incentive(
            Belief {
                mu: -0.25,
                sigma: 0.1,
            },
            4.0,
            8,
        );
        assert!(negative.delta < 0.0 && choice(&negative) == Choice::Off);
    }

    #[test]
    fn figure_3_corners_signs_and_regions_hold_on_the_fixed_grid() {
        let n = SIGMAS.len();
        for mu in [-0.25, 0.0, 0.25] {
            let s = surface(mu, 8);
            let at = |si: usize, bi: usize| s.cells[bi * n + si].delta;
            let min = s
                .cells
                .iter()
                .map(|c| c.delta)
                .fold(f64::INFINITY, f64::min);
            let max = s
                .cells
                .iter()
                .map(|c| c.delta)
                .fold(f64::NEG_INFINITY, f64::max);
            assert_eq!(at(0, BETAS.len() - 1), min, "{mu}: lowest at low σ, high β");
            assert_eq!(at(n - 1, 0), max, "{mu}: highest at high σ, low β");
            for b in 0..BETAS.len() {
                for s in 0..n {
                    if s + 1 < n {
                        assert!(
                            at(s + 1, b) >= at(s, b),
                            "{mu}: Δ falls with σ at β index {b}, σ index {s}"
                        );
                    }
                    if b + 1 < BETAS.len() {
                        assert!(
                            at(s, b + 1) <= at(s, b),
                            "{mu}: Δ rises with β at β index {b}, σ index {s}"
                        );
                    }
                }
            }
            for c in &s.cells {
                match mu.partial_cmp(&0.0).unwrap() {
                    std::cmp::Ordering::Equal => {
                        assert!(c.delta >= 0.0 && choice(c) == Choice::Wait)
                    }
                    std::cmp::Ordering::Less if c.delta < 0.0 => {
                        assert_eq!(choice(c), Choice::Off)
                    }
                    std::cmp::Ordering::Greater if c.delta < 0.0 => {
                        assert_eq!(choice(c), Choice::Act)
                    }
                    _ => assert_eq!(choice(c), Choice::Wait),
                }
            }
            if mu != 0.0 {
                assert!(
                    s.cells.iter().any(|c| c.delta < 0.0) && s.cells.iter().any(|c| c.delta > 0.0)
                );
            }
        }
        assert_eq!(nearest_cell(0.1, 4.0), Some((1, 20)));
        assert_eq!(nearest_cell(1.0, 0.0), Some((19, 0)));
        assert_eq!(nearest_cell(5.0, 0.0), None);
        assert_eq!(nearest_cell(0.5, 50.0), None);
    }
}
