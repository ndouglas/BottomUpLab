//! The two worlds' closed forms: utility U, constraint C (feasible where C ≤ 0), their
//! gradients, the sensitivity (∂U/∂sᵢ)(∂C/∂sᵢ)⁻¹, feasibility with a floor under every
//! attribute, the human optimum, the fixed-proxy limit (Theorem 1's endpoint: unmentioned
//! attributes at the floor, proxy attributes equal on the surface), the impact-minimizing limit
//! (Proposition 2's: unmentioned attributes held at the start) and the correction that returns a
//! point to C = 0 along ∇C. Pure; no RNG.
use crate::portable;
use serde::{Deserialize, Serialize};

pub const L: usize = 4;
pub type State = [f64; L];
pub type Set = [bool; L];
pub const NAMES: [&str; L] = ["a", "b", "c", "d"];
/// |C| at or below this counts as on the surface; a floor violation below it is tolerated.
pub const SURFACE_TOLERANCE: f64 = 1e-9;
/// The recommender's sensitivity 1/(2sᵢ) is infinite at the floor 0; it is evaluated at this value instead.
const SENSITIVITY_FLOOR: f64 = 1e-9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum World {
    /// The paper's §2.2: U = a + b + c + d, C = a² + b² + c² + d² − capacity.
    Recommender,
    /// Ours: U = ln a + ln b + ln c + ln d, C = a + b + c + d − capacity; every hypothesis of
    /// Theorem 2 and Proposition 1 holds on s > 0, and the limit utility has no lower bound.
    Logarithmic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub world: World,
    pub capacity: f64,
    pub floor: f64,
}

pub fn count(set: Set) -> usize {
    set.iter().filter(|&&b| b).count()
}

pub fn complement(set: Set) -> Set {
    [!set[0], !set[1], !set[2], !set[3]]
}

fn dot(a: &State, b: &State) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

impl Shape {
    pub fn utility(&self, s: &State) -> f64 {
        match self.world {
            World::Recommender => s.iter().sum(),
            World::Logarithmic => s.iter().map(|&x| portable::ln(x)).sum(),
        }
    }

    pub fn constraint(&self, s: &State) -> f64 {
        match self.world {
            World::Recommender => s.iter().map(|x| x * x).sum::<f64>() - self.capacity,
            World::Logarithmic => s.iter().sum::<f64>() - self.capacity,
        }
    }

    pub fn utility_gradient(&self, s: &State) -> State {
        match self.world {
            World::Recommender => [1.0; L],
            World::Logarithmic => s.map(|x| 1.0 / x),
        }
    }

    pub fn constraint_gradient(&self, s: &State) -> State {
        match self.world {
            World::Recommender => s.map(|x| 2.0 * x),
            World::Logarithmic => [1.0; L],
        }
    }

    /// (∂U/∂sᵢ)(∂C/∂sᵢ)⁻¹: how much the human cares about attribute i at s.
    pub fn sensitivities(&self, s: &State) -> State {
        match self.world {
            World::Recommender => s.map(|x| 1.0 / (2.0 * x.max(SENSITIVITY_FLOOR))),
            World::Logarithmic => s.map(|x| 1.0 / x),
        }
    }

    pub fn feasible(&self, s: &State) -> bool {
        s.iter()
            .all(|&x| x.is_finite() && x >= self.floor - SURFACE_TOLERANCE)
            && self.constraint(s) <= SURFACE_TOLERANCE
    }

    /// Equal sensitivities on the surface: every attribute at the same value.
    pub fn optimum(&self) -> (State, f64) {
        let each = match self.world {
            World::Recommender => (self.capacity / L as f64).sqrt(),
            World::Logarithmic => self.capacity / L as f64,
        };
        let s = [each; L];
        (s, self.utility(&s))
    }

    /// The proxy attributes equal on the surface when the unmentioned ones hold `held`'s values.
    fn equalized(&self, proxy: Set, held: &State) -> State {
        let j = count(proxy).max(1) as f64;
        let used: f64 = (0..L)
            .filter(|&i| !proxy[i])
            .map(|i| match self.world {
                World::Recommender => held[i] * held[i],
                World::Logarithmic => held[i],
            })
            .sum();
        let each = match self.world {
            World::Recommender => ((self.capacity - used) / j).max(0.0).sqrt(),
            World::Logarithmic => (self.capacity - used) / j,
        };
        let mut s = *held;
        for i in 0..L {
            if proxy[i] {
                s[i] = each;
            }
        }
        s
    }

    /// Theorem 1's endpoint under complete optimization of a fixed proxy.
    pub fn fixed_proxy_limit(&self, proxy: Set) -> (State, f64) {
        let s = self.equalized(proxy, &[self.floor; L]);
        (s, self.utility(&s))
    }

    /// Proposition 2's endpoint: the unmentioned attributes stay where they started.
    pub fn impact_limit(&self, start: &State, proxy: Set) -> (State, f64) {
        let s = self.equalized(proxy, start);
        (s, self.utility(&s))
    }

    /// Ũ(s_J) = U(s_J, s_K^ref): the proxy from the state, the rest from the reference.
    pub fn proxy_utility(&self, s: &State, proxy: Set, reference: &State) -> f64 {
        let mut mixed = *reference;
        for i in 0..L {
            if proxy[i] {
                mixed[i] = s[i];
            }
        }
        self.utility(&mixed)
    }

    /// The largest value one attribute can take with the others at the floor (the frame's scale).
    pub fn maximum(&self) -> f64 {
        match self.world {
            World::Recommender => self.capacity.sqrt(),
            World::Logarithmic => self.capacity - (L - 1) as f64 * self.floor,
        }
    }

    /// Returns `s` to C = 0 by moving along −∇C (evaluated at `s`) over the free attributes,
    /// Newton on the one-dimensional C(s − t·n). A free attribute the move would take below the
    /// floor is set to the floor, frozen, and the rest pay; up to L rounds.
    pub fn correct(&self, mut s: State, mut free: Set) -> State {
        for _ in 0..=L {
            let mut n = self.constraint_gradient(&s);
            for i in 0..L {
                if !free[i] {
                    n[i] = 0.0;
                }
            }
            if dot(&n, &n) == 0.0 {
                return s;
            }
            let mut t = 0.0;
            for _ in 0..50 {
                let p = point(&s, &n, t);
                let c = self.constraint(&p);
                if c.abs() <= 1e-15 {
                    break;
                }
                let slope = dot(&self.constraint_gradient(&p), &n);
                if slope == 0.0 {
                    break;
                }
                t += c / slope;
            }
            let p = point(&s, &n, t);
            let mut clamped = false;
            for i in 0..L {
                if free[i] && p[i] < self.floor {
                    s[i] = self.floor;
                    free[i] = false;
                    clamped = true;
                }
            }
            if !clamped {
                return p;
            }
        }
        s
    }
}

fn point(s: &State, n: &State, t: f64) -> State {
    let mut p = *s;
    for i in 0..L {
        p[i] -= t * n[i];
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    const AB: Set = [true, true, false, false];
    const A: Set = [true, false, false, false];
    const ABC: Set = [true, true, true, false];

    fn recommender() -> Shape {
        Shape { world: World::Recommender, capacity: 100.0, floor: 0.0 }
    }
    fn logarithmic() -> Shape {
        Shape { world: World::Logarithmic, capacity: 40.0, floor: 1.0 }
    }
    fn close(a: &State, b: &State, tol: f64) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < tol)
    }

    #[test]
    fn the_recommender_is_the_papers_sum_and_sphere() {
        let s = recommender();
        let start = [1.0, 1.0, 7.0, 7.0];
        assert_eq!(s.utility(&start), 16.0);
        assert_eq!(s.constraint(&start), 0.0);
        assert_eq!(s.constraint_gradient(&start), [2.0, 2.0, 14.0, 14.0]);
        assert_eq!(s.utility_gradient(&start), [1.0; 4]);
        assert_eq!(s.sensitivities(&start), [0.5, 0.5, 1.0 / 14.0, 1.0 / 14.0]);
        assert!(s.feasible(&start) && !s.feasible(&[1.0, 1.0, 7.0, 7.1]) && !s.feasible(&[-0.1, 0.0, 0.0, 0.0]));
        assert!(s.sensitivities(&[0.0; 4])[0].is_finite(), "the floor is clamped at 1e-9");
    }

    #[test]
    fn the_logarithmic_world_is_a_budget_with_log_utility() {
        let s = logarithmic();
        let start = [4.0, 4.0, 16.0, 16.0];
        assert!((s.utility(&start) - (2.0 * 4.0f64.ln() + 2.0 * 16.0f64.ln())).abs() < 1e-12);
        assert_eq!(s.constraint(&start), 0.0);
        assert_eq!(s.constraint_gradient(&start), [1.0; 4]);
        assert_eq!(s.utility_gradient(&start), [0.25, 0.25, 0.0625, 0.0625]);
        assert_eq!(s.sensitivities(&start), [0.25, 0.25, 0.0625, 0.0625]);
        assert!(!s.feasible(&[0.5, 4.0, 16.0, 16.0]), "below the floor");
    }

    #[test]
    fn optima_are_equal_shares_on_the_surface() {
        let (s, u) = recommender().optimum();
        assert_eq!((s, u), ([5.0; 4], 20.0));
        let (s, u) = logarithmic().optimum();
        assert_eq!(s, [10.0; 4]);
        assert!((u - 4.0 * 10.0f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn fixed_proxy_limits_put_the_unmentioned_at_the_floor_and_split_the_rest() {
        let r = recommender();
        let (s, u) = r.fixed_proxy_limit(AB);
        assert!(close(&s, &[50f64.sqrt(), 50f64.sqrt(), 0.0, 0.0], 1e-12));
        assert!((u - 10.0 * 2f64.sqrt()).abs() < 1e-12);
        assert!((r.fixed_proxy_limit(A).1 - 10.0).abs() < 1e-12);
        assert!((r.fixed_proxy_limit(ABC).1 - 10.0 * 3f64.sqrt()).abs() < 1e-12);
        let l = logarithmic();
        let (s, u) = l.fixed_proxy_limit(AB);
        assert!(close(&s, &[19.0, 19.0, 1.0, 1.0], 1e-12));
        assert!((u - 2.0 * 19f64.ln()).abs() < 1e-12);
        let low = Shape { floor: 0.001, ..l };
        assert!(low.fixed_proxy_limit(AB).1 < -5.0, "unbounded below as the floor falls");
    }

    #[test]
    fn impact_limits_freeze_the_unmentioned_at_the_start() {
        let r = recommender();
        let (s, u) = r.impact_limit(&[1.0, 1.0, 7.0, 7.0], AB);
        assert!(close(&s, &[1.0, 1.0, 7.0, 7.0], 1e-12) && (u - 16.0).abs() < 1e-12, "no slack: the start");
        let (s, u) = r.impact_limit(&[1.0, 1.0, 6.0, 6.0], AB);
        assert!(close(&s, &[14f64.sqrt(), 14f64.sqrt(), 6.0, 6.0], 1e-12));
        assert!((u - (12.0 + 2.0 * 14f64.sqrt())).abs() < 1e-12);
        let (s, _) = logarithmic().impact_limit(&[4.0, 4.0, 16.0, 16.0], [true, false, true, false]);
        assert!(close(&s, &[10.0, 4.0, 10.0, 16.0], 1e-12));
    }

    #[test]
    fn proxy_utility_reads_the_proxy_from_the_state_and_the_rest_from_the_reference() {
        let r = recommender();
        let reference = [1.0, 1.0, 7.0, 7.0];
        assert_eq!(r.proxy_utility(&[3.0, 4.0, 0.0, 0.0], AB, &reference), 3.0 + 4.0 + 7.0 + 7.0);
        assert_eq!(r.proxy_utility(&reference, AB, &reference), 16.0);
    }

    #[test]
    fn correction_returns_to_the_surface_along_the_constraint_gradient() {
        let r = recommender();
        let free = [true; 4];
        let s = r.correct([1.1, 1.1, 7.0, 7.0], free);
        assert!(r.constraint(&s).abs() < 1e-12);
        assert!((s[0] / s[2] - 1.1 / 7.0).abs() < 1e-12, "radial: ratios are kept");
        let s = r.correct([5.0, 5.0, 5.0, 4.0], [true, true, true, false]);
        assert_eq!(s[3], 4.0, "a frozen attribute does not move");
        assert!(r.constraint(&s).abs() < 1e-12);
        let l = logarithmic();
        let s = l.correct([5.0, 5.0, 16.0, 16.0], free);
        assert!(close(&s, &[4.5, 4.5, 15.5, 15.5], 1e-12), "equal shares of the overrun");
        let clamped = l.correct([10.0, 10.0, 1.2, 20.0], [true, true, true, false]);
        assert_eq!(clamped[2], 1.0, "an attribute that would cross the floor stops there and the others pay");
        assert!(l.constraint(&clamped).abs() < 1e-12 && clamped[3] == 20.0);
    }

    #[test]
    fn set_helpers_count_and_complement() {
        assert_eq!(count(AB), 2);
        assert_eq!(complement(AB), [false, false, true, true]);
        assert_eq!(NAMES, ["a", "b", "c", "d"]);
        let r = recommender();
        assert_eq!(r.maximum(), 10.0);
        assert_eq!(logarithmic().maximum(), 37.0);
    }
}
