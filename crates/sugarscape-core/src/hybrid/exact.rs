//! Numerics shared by the exact references: the logistic function, tanh(x/2) and eˣ from the
//! portable exponential, log-sum-exp, ln n!, and bracketing root scans that read the flow's
//! direction on each side of a root.

use crate::portable::{exp_neg, ln};
use serde::Serialize;

/// 1 / (1 + e^{−x}) for any finite x; `exp_neg` only ever sees a non-positive argument.
pub fn logistic(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + exp_neg(-x))
    } else {
        let e = exp_neg(x);
        e / (1.0 + e)
    }
}

/// tanh(x / 2) = (1 − e^{−|x|}) / (1 + e^{−|x|}), odd in x.
pub fn tanh_half(x: f64) -> f64 {
    let e = exp_neg(-x.abs());
    let t = (1.0 - e) / (1.0 + e);
    if x >= 0.0 {
        t
    } else {
        -t
    }
}

/// eˣ (the references need e^{β(T − R)} > 1); infinite past x ≈ 708.
pub fn exp(x: f64) -> f64 {
    if x <= 0.0 {
        exp_neg(x)
    } else {
        1.0 / exp_neg(-x)
    }
}

/// ln Σ e^{xᵢ} without overflow; −∞ for an empty slice.
pub fn log_sum_exp(xs: &[f64]) -> f64 {
    let m = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if m == f64::NEG_INFINITY {
        return m;
    }
    m + ln(xs.iter().map(|&x| exp_neg(x - m)).sum::<f64>())
}

/// ln n!, summed term by term with the portable logarithm.
pub fn ln_factorial(n: u32) -> f64 {
    (2..=n).map(|k| ln(f64::from(k))).sum()
}

/// A zero of a flow ẋ = f(x), and whether the flow returns to it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Root {
    pub x: f64,
    pub stable: bool,
}

/// How far from a root the flow's direction is read.
pub const PROBE: f64 = 1e-12;

/// The zeros of f on [lo, hi]: f is sampled at `cells + 1` evenly spaced points; an exact zero is
/// a root, and each strict sign change between neighboring samples is bisected until the bracket
/// cannot shrink. Beside an exact zero the sign change is sought between the zero's offset point
/// (x ± `PROBE`) and the neighboring sample, so a root strictly inside the adjacent cell is found
/// too (flows such as ρ(1 − ρ)g(ρ) vanish exactly at the ends). A root is stable when the flow
/// points toward it on every side it has inside the interval (f > 0 just left of it, f < 0 just
/// right), read at ±`PROBE`.
pub fn roots(f: impl Fn(f64) -> f64, lo: f64, hi: f64, cells: usize) -> Vec<Root> {
    let at = |k: usize| lo + (hi - lo) * k as f64 / cells as f64;
    // The point where f changes sign in (a, b), given f(a) = fa and a strict sign change.
    let bisect = |mut a: f64, mut b: f64, fa: f64| {
        for _ in 0..200 {
            let m = 0.5 * (a + b);
            if m <= a || m >= b {
                break;
            }
            let fm = f(m);
            if fm == 0.0 {
                a = m;
                b = m;
                break;
            }
            if (fm < 0.0) == (fa < 0.0) {
                a = m;
            } else {
                b = m;
            }
        }
        0.5 * (a + b)
    };
    let mut xs = vec![];
    let (mut px, mut pf) = (at(0), f(at(0)));
    if pf == 0.0 {
        xs.push(px);
    }
    for k in 1..=cells {
        let x = at(k);
        let fx = f(x);
        if fx == 0.0 {
            // Left of an exact zero: from the previous sample to just short of it.
            let xl = x - PROBE;
            if pf != 0.0 && xl > px {
                let fl = f(xl);
                if fl != 0.0 && pf * fl < 0.0 {
                    xs.push(bisect(px, xl, pf));
                }
            }
            xs.push(x);
        } else if pf == 0.0 {
            // Right of an exact zero: from just past it to this sample.
            let xr = px + PROBE;
            if xr < x {
                let fr = f(xr);
                if fr != 0.0 && fr * fx < 0.0 {
                    xs.push(bisect(xr, x, fr));
                }
            }
        } else if pf * fx < 0.0 {
            xs.push(bisect(px, x, pf));
        }
        (px, pf) = (x, fx);
    }
    xs.into_iter()
        .map(|x| {
            let left = (x - PROBE >= lo).then(|| f(x - PROBE) > 0.0);
            let right = (x + PROBE <= hi).then(|| f(x + PROBE) < 0.0);
            let stable = (left.is_some() || right.is_some())
                && left.unwrap_or(true)
                && right.unwrap_or(true);
            Root { x, stable }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logistic_tanh_and_exp_match_literal_values() {
        assert_eq!(logistic(0.0), 0.5);
        assert!((logistic(2.0) - 0.880_797_077_977_882_3).abs() < 1e-15);
        assert!((logistic(-2.0) - 0.119_202_922_022_117_6).abs() < 1e-15);
        assert!(logistic(-800.0) < 1e-300 && logistic(800.0) == 1.0);
        assert!((tanh_half(1.0) - 0.462_117_157_260_009_7).abs() < 1e-15);
        assert!((tanh_half(-1.0) + 0.462_117_157_260_009_7).abs() < 1e-15);
        assert!((exp(1.0) - std::f64::consts::E).abs() < 1e-15);
        assert!((exp(-1.0) - 0.367_879_441_171_442_3).abs() < 1e-16);
        assert!((log_sum_exp(&[0.0, 0.0]) - std::f64::consts::LN_2).abs() < 1e-15);
        assert_eq!(log_sum_exp(&[]), f64::NEG_INFINITY);
        assert!((ln_factorial(10) - 15.104_412_573_075_516).abs() < 1e-12);
        assert_eq!(ln_factorial(0), 0.0);
    }

    #[test]
    fn roots_find_a_root_inside_the_cell_next_to_an_exact_zero() {
        // x(x − 0.0004): an exact zero at 0 and a root strictly inside the first cell (0, 0.001).
        let r = roots(|x| x * (x - 0.0004), 0.0, 1.0, 1000);
        assert_eq!(r.len(), 2, "{r:?}");
        assert!(r[0].x == 0.0 && r[0].stable, "{r:?}");
        assert!((r[1].x - 0.0004).abs() < 1e-15 && !r[1].stable, "{r:?}");
        // The mirror image at the upper end: (x − 1)(x − 0.9996) has its roots at 1 and 0.9996.
        let r = roots(|x| (x - 1.0) * (x - 0.9996), 0.0, 1.0, 1000);
        assert_eq!(r.len(), 2, "{r:?}");
        assert!((r[0].x - 0.9996).abs() < 1e-15 && r[0].stable, "{r:?}");
        assert!(r[1].x == 1.0 && !r[1].stable, "{r:?}");
    }

    #[test]
    fn roots_find_crossings_and_read_stability() {
        // ẋ = x(1 − x)(x − 0.3): 0 and 1 stable, 0.3 unstable.
        let r = roots(|x| x * (1.0 - x) * (x - 0.3), 0.0, 1.0, 1000);
        assert_eq!(r.len(), 3, "{r:?}");
        assert!(
            r[0].x.abs() < 1e-15 && (r[1].x - 0.3).abs() < 1e-15 && (r[2].x - 1.0).abs() < 1e-15
        );
        assert_eq!(
            r.iter().map(|r| r.stable).collect::<Vec<_>>(),
            vec![true, false, true]
        );
        // A crossing between samples is bisected to adjacent floats.
        let one = roots(|x| 0.123_456_789 - x, 0.0, 1.0, 10);
        assert_eq!(one.len(), 1);
        assert!((one[0].x - 0.123_456_789).abs() < 1e-15 && one[0].stable);
    }
}
