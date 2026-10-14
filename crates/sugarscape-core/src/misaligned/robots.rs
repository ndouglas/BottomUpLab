//! One move of each robot, given the state, the proxy set, the proxy's reference values, the
//! frozen set and (for the climber) the shared RNG. Pure otherwise.
//!
//! The climber is the paper's black box as we reconstruct it: a Gaussian proposal on every
//! attribute that is not frozen, kept when it is feasible and raises the proxy. The gradient
//! robot is the efficient one: steepest ascent of the proxy, projected onto the constraint
//! surface once it binds, so that whatever the proxy attributes gain the unmentioned ones pay
//! for and nothing is destroyed for its own sake. The wasteful robot is Proposition 3's: it
//! lowers the unmentioned attributes for a sliver of proxy.
use super::worlds::{Set, Shape, State, L, SURFACE_TOLERANCE};
use crate::assistance::dynamics::standard_normal;
use crate::rng::SimRng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Robot {
    Climber,
    Gradient,
    Wasteful,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub state: State,
    pub accepted: bool,
}

fn stay(s: &State) -> Move {
    Move {
        state: *s,
        accepted: false,
    }
}

fn dot(a: &State, b: &State) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// A Gaussian proposal on the free attributes; the draws happen for every free attribute before
/// the test, so a move always consumes the same number of draws.
pub fn climb(
    shape: &Shape,
    s: &State,
    proxy: Set,
    reference: &State,
    frozen: Set,
    step: f64,
    rng: &mut SimRng,
) -> Move {
    let mut next = *s;
    for i in 0..L {
        if !frozen[i] {
            next[i] += step * standard_normal(rng);
        }
    }
    if shape.feasible(&next)
        && shape.proxy_utility(&next, proxy, reference) > shape.proxy_utility(s, proxy, reference)
    {
        Move {
            state: next,
            accepted: true,
        }
    } else {
        stay(s)
    }
}

/// Steepest ascent of the proxy at most `rate` long: straight up while the constraint is slack
/// (shortened to the surface if the move would cross it), then along the surface's tangent over
/// the free attributes, with attributes at the floor that would move down frozen for the move,
/// and the correction back onto C = 0. The gradient of Ũ on J is ∇U on J because both worlds'
/// utilities are additively separable; `reference` is accepted so the robots share a call shape.
pub fn ascend(
    shape: &Shape,
    s: &State,
    proxy: Set,
    reference: &State,
    frozen: Set,
    rate: f64,
) -> Move {
    let _ = reference;
    let mut free = [false; L];
    for i in 0..L {
        free[i] = !frozen[i];
    }
    let full = shape.utility_gradient(s);
    let mut g = [0.0; L];
    for i in 0..L {
        if proxy[i] && free[i] {
            g[i] = full[i];
        }
    }
    let capped = |d: State| -> State {
        let norm = dot(&d, &d).sqrt();
        let scale = rate / norm.max(1.0);
        d.map(|x| x * scale)
    };
    if shape.constraint(s) < -SURFACE_TOLERANCE {
        if dot(&g, &g) == 0.0 {
            return stay(s);
        }
        let step = capped(g);
        let mut next = *s;
        for i in 0..L {
            next[i] += step[i];
        }
        if shape.constraint(&next) > 0.0 {
            // Shorten to the surface: C is continuous and changes sign on [0, 1].
            let (mut lo, mut hi) = (0.0, 1.0);
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                let mut p = *s;
                for i in 0..L {
                    p[i] += mid * step[i];
                }
                if shape.constraint(&p) > 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            for i in 0..L {
                next[i] = s[i] + lo * step[i];
            }
        }
        return Move {
            state: next,
            accepted: true,
        };
    }
    let d = loop {
        let mut n = shape.constraint_gradient(s);
        for i in 0..L {
            if !free[i] {
                n[i] = 0.0;
            }
        }
        let nn = dot(&n, &n);
        if nn == 0.0 {
            return stay(s);
        }
        let lambda = dot(&g, &n) / nn;
        let mut d = [0.0; L];
        for i in 0..L {
            if free[i] {
                d[i] = g[i] - lambda * n[i];
            }
        }
        let mut froze = false;
        for i in 0..L {
            if free[i] && s[i] <= shape.floor + SURFACE_TOLERANCE && d[i] < 0.0 {
                free[i] = false;
                froze = true;
            }
        }
        if !froze {
            break d;
        }
    };
    if dot(&d, &d).sqrt() < 1e-12 {
        return stay(s);
    }
    let step = capped(d);
    let mut next = *s;
    for i in 0..L {
        next[i] += step[i];
    }
    Move {
        state: shape.correct(next, free),
        accepted: true,
    }
}

/// Proposition 3's robot: every unmentioned attribute down by `rate` (not below the floor), every
/// proxy attribute up by `rate × waste` when that stays feasible.
pub fn waste(shape: &Shape, s: &State, proxy: Set, frozen: Set, rate: f64, waste: f64) -> Move {
    let mut next = *s;
    for k in 0..L {
        if !proxy[k] && !frozen[k] {
            next[k] = (next[k] - rate).max(shape.floor);
        }
    }
    let mut raised = next;
    for j in 0..L {
        if proxy[j] {
            raised[j] += rate * waste;
        }
    }
    if shape.feasible(&raised) {
        next = raised;
    }
    Move {
        accepted: next != *s,
        state: next,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::misaligned::worlds::World;
    use crate::rng;

    const AB: Set = [true, true, false, false];
    const NONE: Set = [false; 4];

    fn recommender() -> Shape {
        Shape { world: World::Recommender, capacity: 100.0, floor: 0.0 }
    }
    fn logarithmic() -> Shape {
        Shape { world: World::Logarithmic, capacity: 40.0, floor: 1.0 }
    }

    #[test]
    fn the_climber_rejects_infeasible_and_proxy_lowering_proposals() {
        let (r, l) = (recommender(), logarithmic());
        let mut rng = rng::seeded(3);
        let start = [1.0, 1.0, 7.0, 7.0];
        let (mut s, mut accepted) = (start, 0);
        for _ in 0..2000 {
            let m = climb(&r, &s, AB, &start, NONE, 1.0, &mut rng);
            if m.accepted {
                accepted += 1;
                assert!(r.proxy_utility(&m.state, AB, &start) > r.proxy_utility(&s, AB, &start));
                assert!(r.feasible(&m.state));
            } else {
                assert_eq!(m.state, s);
            }
            s = m.state;
        }
        // A step of 1 reaches the limit in a few dozen accepted moves; after that nearly every
        // proposal is rejected (at seed 3: 25 accepted, the rest infeasible or proxy-lowering).
        assert!(accepted > 10 && accepted < 200, "{accepted}");
        assert!(s[0] + s[1] > 13.0, "the proxy climbs to near its limit 14.14: {s:?}");
        // The logarithmic world near its floor: proposals at or below zero are rejected, nothing is NaN.
        let mut s = [1.0, 1.0, 19.0, 19.0];
        for _ in 0..500 {
            let m = climb(&l, &s, AB, &s, NONE, 3.0, &mut rng);
            assert!(m.state.iter().all(|x| x.is_finite() && *x >= 1.0 - SURFACE_TOLERANCE));
            s = m.state;
        }
        // Frozen attributes never move.
        let m = climb(&r, &start, AB, &start, [false, false, true, true], 1.0, &mut rng);
        assert_eq!((m.state[2], m.state[3]), (7.0, 7.0));
    }

    #[test]
    fn the_gradient_robot_walks_the_great_circle_through_the_optimum_to_the_limit() {
        let r = recommender();
        let start = [1.0, 1.0, 7.0, 7.0];
        let mut s = start;
        let (mut best, mut proxy_before) = (0.0f64, 2.0);
        for _ in 0..5000 {
            let m = ascend(&r, &s, AB, &start, NONE, 0.1);
            let proxy = r.proxy_utility(&m.state, AB, &start);
            assert!(proxy >= proxy_before - 1e-12, "the proxy never falls");
            assert!(r.constraint(&m.state).abs() < 1e-9, "stays on the sphere");
            best = best.max(r.utility(&m.state));
            s = m.state;
            proxy_before = proxy;
        }
        assert!((best - 20.0).abs() < 1e-3, "passes within 1e-3 of the optimum: {best}");
        let (limit, _) = r.fixed_proxy_limit(AB);
        assert!(s.iter().zip(&limit).all(|(x, y)| (x - y).abs() < 1e-3), "{s:?}");
        let settled = ascend(&r, &s, AB, &start, NONE, 0.1);
        assert!(settled.state.iter().zip(&s).all(|(x, y)| (x - y).abs() < 1e-6), "at the limit a move is negligible");
    }

    #[test]
    fn the_gradient_robot_stands_still_at_a_proxy_optimum() {
        let r = recommender();
        let start = [1.0, 1.0, 7.0, 7.0];
        // Impact minimizing from the sphere with a, b already equal: no slack, no direction.
        let m = ascend(&r, &start, AB, &start, [false, false, true, true], 0.1);
        assert!(!m.accepted && m.state == start);
        // From inside the sphere with slack it climbs, and then trades c, d down proportionally.
        let inside = [1.0, 1.0, 6.0, 6.0];
        let m = ascend(&r, &inside, AB, &inside, NONE, 0.1);
        assert!(m.accepted && m.state[0] > 1.0 && m.state[2] == 6.0, "slack: straight up");
        let mut s = inside;
        for _ in 0..3000 {
            s = ascend(&r, &s, AB, &inside, [false, false, true, true], 0.1).state;
        }
        let (limit, _) = r.impact_limit(&inside, AB);
        assert!(s.iter().zip(&limit).all(|(x, y)| (x - y).abs() < 1e-3), "{s:?}");
    }

    #[test]
    fn the_gradient_robot_freezes_attributes_at_the_floor() {
        let l = logarithmic();
        let start = [4.0, 4.0, 16.0, 16.0];
        let mut s = start;
        for _ in 0..5000 {
            let m = ascend(&l, &s, AB, &start, NONE, 0.1);
            assert!(m.state.iter().all(|&x| x >= 1.0 - SURFACE_TOLERANCE));
            s = m.state;
        }
        assert!((s[2] - 1.0).abs() < 1e-9 && (s[3] - 1.0).abs() < 1e-9, "{s:?}");
        assert!((s[0] - 19.0).abs() < 1e-3 && (s[1] - 19.0).abs() < 1e-3, "{s:?}");
        // The recommender with a negative floor: c and d stop at 0, where C stops increasing.
        let r = Shape { floor: -5.0, ..recommender() };
        let start = [1.0, 1.0, 7.0, 7.0];
        let mut s = start;
        for _ in 0..5000 {
            s = ascend(&r, &s, AB, &start, NONE, 0.1).state;
        }
        assert!(s[2].abs() < 1e-3 && s[3].abs() < 1e-3, "{s:?}");
    }

    #[test]
    fn the_wasteful_robot_destroys_the_unmentioned_and_stops_at_the_floor() {
        let r = recommender();
        let start = [1.0, 1.0, 7.0, 7.0];
        let m = waste(&r, &start, AB, NONE, 0.1, 0.1);
        assert!(m.accepted);
        assert!((m.state[2] - 6.9).abs() < 1e-12 && (m.state[3] - 6.9).abs() < 1e-12);
        assert!((m.state[0] - 1.01).abs() < 1e-12);
        let mut s = start;
        for _ in 0..200 {
            s = waste(&r, &s, AB, NONE, 0.1, 0.1).state;
        }
        assert_eq!((s[2], s[3]), (0.0, 0.0));
        assert!(r.utility(&s) < 16.0);
        let m = waste(&r, &s, AB, NONE, 0.1, 0.1);
        assert!(m.accepted, "the proxy still has room");
        assert!(r.feasible(&m.state));
    }
}
