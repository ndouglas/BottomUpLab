# Misaligned AI (milestone 43) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A new model kind `misaligned` that reproduces Zhuang and Hadfield-Menell (2020): a four-attribute resource world, a robot that raises a proxy on a subset of the attributes, the paper's recommender example and Figure 2, Theorem 1, Theorem 2 in the example and on a logarithmic world of ours, and the mitigations (Propositions 2–5, Theorem 3), scored by the spec's decision rules in the survey.

**Architecture:** One kind, `ModelKind::Misaligned`, with pure closed forms in `worlds.rs` (utility, constraint, sensitivities, optimum, limits, surface correction), pure robot moves in `robots.rs` (climber, gradient, wasteful), the human's proxy choice and OFF rule in `human.rs`, and a world whose tick runs the human at its interval and then `moves_per_tick` robot moves. The page, sweeps, goldens, survey and docs wire in at the same sites milestones 39 (`assistance`) and 40 (`gridworlds`) used.

**Tech Stack:** Rust 2021 (`sugarscape-core`, `serde`, `serde_json`, `rand` through `crate::rng::SimRng`), wasm-bindgen through the existing `Sim`, Vite + TypeScript page, vitest, the standalone `survey` crate.

**Spec:** `docs/superpowers/specs/2026-10-14-misaligned-ai-design.md` (read it first; Task 1 amends it).

## Global Constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add a new kind and new goldens.
- One sequential path serves native and WASM: seeded RNG (`crate::rng::seeded`, `SimRng`), Gaussian draws through `crate::assistance::dynamics::standard_normal` (polar Box–Muller), `portable::ln` for logarithms, `f64::sqrt` allowed; no `f64::exp`, `ln`, `powf` or `erf` anywhere in this kind.
- Literal rules are defaults; the climber is the default robot. Every other choice is a named switch.
- Decision rules are the spec's (as amended in Task 1), fixed before any measurement; the survey never adjusts a failed rule.
- In code, pages and docs the robot is "the robot" and the principal "the human"; never "Flump". American spelling.
- The kind's `pub mod misaligned;` goes at the **end** of `crates/sugarscape-core/src/lib.rs`; the vitest identity fixtures pin `lib.rs`, `model.rs`, `presets.rs`, `sweep.rs` and `titles.rs` by sha256 and need receipts (Task 4), and `crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json` pins `lib.rs` by bytes and sha256 in two bindings (Task 3).
- Root `Cargo.toml` and `Cargo.lock` must not change. No new dependencies.
- Paper PDFs stay gitignored in `papers/`.
- Commits end with `Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR`.
- Run every command from the worktree `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/misaligned` (branch `worktree-misaligned`, already created from main at `f55c052`; the spec is committed as `c04c501`).

## Review Focus

1. A start exactly on the constraint surface with the proxy attributes already balanced (the default (1, 1, 7, 7) under impact minimization) must leave the gradient robot still, not jittering or stepping off the surface; pinned in Task 2's `the_gradient_robot_stands_still_at_a_proxy_optimum`.
2. In the logarithmic world a climber proposal that takes an attribute to zero or below must be rejected without producing NaN in any series; pinned in Task 2's `the_climber_rejects_infeasible_and_proxy_lowering_proposals`.
3. After OFF the world must keep ticking with frozen state and finite series, and a live change of a human field must not restart the robot; pinned in Task 3's `off_freezes_the_world_and_later_ticks_change_nothing`.
4. A live change of the proxy booleans or the human policy counts as an interaction (new reference values, new frozen set) and a change of `world`, `capacity`, `floor` or a start is refused by field name; pinned in Task 3's `live_human_changes_are_interactions_and_reset_fields_are_refused`.
5. Inspect on a gap between bars, on the strip and on every bar must return well-formed JSON; a point outside the frame is an error; pinned in Task 4's `inspection_reads_bars_gaps_and_the_strip`.

---

## File map

| Path | Responsibility |
|---|---|
| `crates/sugarscape-core/src/misaligned/mod.rs` | Module exports |
| `…/misaligned/worlds.rs` | `World`, `Shape`, `State`, `Set`: U, C, gradients, sensitivities, feasibility, optimum, fixed-proxy limit, impact-minimizing limit, surface correction |
| `…/misaligned/robots.rs` | `Robot`, `Move`; `climb`, `ascend`, `waste` |
| `…/misaligned/human.rs` | `Human`; `top`, `extremes` |
| `…/misaligned/config.rs` | `MisalignedConfig`, validation, schema, `logarithmic_defaults()` |
| `…/misaligned/stats.rs` | `MisalignedSnapshot`, `SERIES`, `series_names` |
| `…/misaligned/world.rs` | `MisalignedWorld`: the tick loop, interactions, OFF, `Outcome`, `Model` impl |
| `…/misaligned/view.rs` | Four bars and a utility strip; inspection JSON |
| `…/misaligned/presets.rs` | Eight presets |
| `crates/sugarscape-core/tests/misaligned_smoke.rs` | Every preset runs 2,000 ticks with finite series; the built-in sweeps parse |
| `survey/src/claims/misaligned.rs`, `survey/out/misaligned/results.md` | The spec's rules as survey claims, and the tracked output |
| `sweeps/misaligned-{figure2-pairs,interval,floor}.json` | Built-in sweeps |
| `web/src/misaligned.ts`, `web/src/misaligned.test.ts` | Legend, status line and Inspect rows |
| `docs/superpowers/specs/2026-10-14-misaligned-ai-findings.md` | Findings |

Registration sites (each listed again in its task): `model.rs` (17 sites), `lib.rs`, `presets.rs` catalog, `titles.rs`, `tests/golden.rs`, `crates/sugarscape-wasm/tests/web.rs`, `sweep.rs`, `survey/src/claims/mod.rs`, `survey/war1_study/fixtures/core-literal-inputs-v1.txt` and its counts, `browser_experiments/fixtures/key-registration-compatibility.json`, `web/src/{types,models,compare-presets,determinism.test}.ts`, `web/src/ui/{series-data,grid-view,inspect-panel}.ts`, `web/src/experiments/form.ts`, `web/src/episodes/test-fixtures/{foraging-source-compatibility.json,historical-source.ts}`, `web/src/episodes/caching-parity.test.ts`.

---

### Task 1: Spec amendments and the worlds' closed forms (`worlds.rs`)

**Files:**
- Modify: `docs/superpowers/specs/2026-10-14-misaligned-ai-design.md`
- Create: `crates/sugarscape-core/src/misaligned/mod.rs`, `crates/sugarscape-core/src/misaligned/worlds.rs`
- Modify: `crates/sugarscape-core/src/lib.rs` (append the module; see Step 3)

**Interfaces:**
- Produces: `World::{Recommender, Logarithmic}` (serde snake_case), `L = 4`, `State = [f64; 4]`, `Set = [bool; 4]`, `NAMES`, `count(Set) -> usize`, `complement(Set) -> Set`, `SURFACE_TOLERANCE = 1e-9`, `Shape { world, capacity, floor }` with `utility`, `constraint`, `utility_gradient`, `constraint_gradient`, `sensitivities`, `feasible`, `optimum() -> (State, f64)`, `fixed_proxy_limit(Set) -> (State, f64)`, `impact_limit(&State, Set) -> (State, f64)`, `proxy_utility(&State, Set, &State) -> f64`, `maximum() -> f64`, `correct(State, Set) -> State`.

Planning settled four points the spec left open; record them before code.

- [ ] **Step 1: Amend the spec**

Apply these edits (each `old` text occurs once):

1. In **Dynamics**, **Gradient**, replace "the robot moves `rate` along the unit direction" with "the robot moves `rate · d / max(1, |d|)` along the direction d (unit speed far from a stationary point, proportional near one, so every move is at most `rate` long and the flow converges without chatter)", and replace "the robot moves `rate` along it, and the correction" with "the robot moves `rate · d / max(1, |d|)` along it, and the correction".
2. In **Exact targets**, in the Theorem 1 row, replace "rate 0.01, 5,000 ticks" with "rate 0.1, 5,000 ticks (the flow contracts by about 1.4 % per tick at that rate, so endpoints are at machine precision; at rate 0.01 they would still be 1e-2 away)".
3. In the Theorem 3, Proposition 4 row, replace the rule "gradient, human `top`, rate 0.01, intervals 1 and 10, 5,000 ticks: U nondecreasing within 1e-9 throughout; the final gap to U* reported" with "gradient, human `top`, rate 0.01, intervals 1 and 10, 5,000 ticks: U nondecreasing within 1e-9 at every tick before the first tick at which U comes within 1e-2 of U*, and at least U* − 1e-2 at every tick after it (the guarantee ends when the top three sensitivities are equal, which here is the optimum; afterward the human's re-specification chatters by at most one interval's travel); the first such tick and the final gap reported".
4. In **Built-in sweeps**, in `misaligned-interval`, replace "gradient at rate 0.01" with "gradient at rate 0.1 (so that interval × rate spans 0.1 to 20, from well inside Theorem 3's neighborhood to past the whole path)", and in **Reported rows**, `interval-break`, append " (rate 0.1)" after "the interval sweep".
5. In **Decision rules**, replace "Exact targets: endpoints within 1e-3 (the flow's step error at rate 0.01)" with "Exact targets: endpoints within 1e-3 of the closed form after 5,000 ticks at rate 0.1 (the fixed point of the projected flow is exact; the tolerance covers convergence)".
6. In **Exact targets**, in the Proposition 5 row, replace "U within 1e-3 of U* by 5,000 ticks" with "U within 1e-3 of U* by 10,000 ticks (the logarithmic world's pairwise transfers contract by only 0.1 % per tick near balance)". In **Dynamics**, **Tick**, replace "first, if the tick is a multiple of `interval` (and at tick 0), the human acts" with "first, if the tick count is a positive multiple of `interval`, the human acts (the human also acts once when the world is created, at tick 0)".
7. In **Dynamics**, **Climber**, replace "Accept if Ũ(s′) > Ũ(s), C(s′) ≤ 0 and every s′ᵢ ≥ floor" with "Accept if C(s′) ≤ 0, every s′ᵢ ≥ floor and then Ũ(s′) > Ũ(s), feasibility first so the logarithmic world never evaluates a logarithm at or below zero".

- [ ] **Step 2: Write the failing tests**

`crates/sugarscape-core/src/misaligned/worlds.rs` ends with:

```rust
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
        assert!(close(&s, &[13f64.sqrt(), 13f64.sqrt(), 6.0, 6.0], 1e-12));
        assert!((u - (12.0 + 2.0 * 13f64.sqrt())).abs() < 1e-12);
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
```

- [ ] **Step 3: Run the tests to verify they fail**

Create `crates/sugarscape-core/src/misaligned/mod.rs`:

```rust
//! Consequences of misaligned AI (milestone 43): Zhuang and Hadfield-Menell's resource world
//! (NeurIPS 2020), a robot that raises a proxy on a subset of the attributes the human cares
//! about, the paper's recommender example, and the mitigations. Closed forms are the targets;
//! the climber, the starts and the human's interval are ours.
pub mod worlds;
```

Append to `crates/sugarscape-core/src/lib.rs` (after the `gridworlds` lines, with one blank line before):

```rust
/// Consequences of misaligned AI: a proxy on a subset of the attributes (milestone 43).
pub mod misaligned;
```

Run: `cargo test -p sugarscape-core misaligned:: 2>&1 | tail -5`. Expected: compile errors (`Shape`, `World` not found).

- [ ] **Step 4: Write `worlds.rs`**

Above the tests:

```rust
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
        s.iter().all(|&x| x.is_finite() && x >= self.floor - SURFACE_TOLERANCE)
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
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core misaligned:: 2>&1 | tail -5`. Expected: 8 passed. If `correction_returns_to_the_surface_along_the_constraint_gradient`'s clamped case fails on `clamped[2] == 1.0`, the Newton loop moved `c` below the floor in the first round and the second round must start from the clamped state with `c` frozen: check that the outer loop writes `s[i] = self.floor` and recomputes `n` from the clamped `s`, not from `p`.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/specs/2026-10-14-misaligned-ai-design.md crates/sugarscape-core/src/misaligned crates/sugarscape-core/src/lib.rs
git commit -m "misaligned: spec amendments and the worlds' closed forms

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
```

The `lib.rs` change breaks the key-registration and identity fixtures until Tasks 3 and 4 refresh them; the crate's own tests other than those fixtures' stay green.

---
### Task 2: The robots and the human (`robots.rs`, `human.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/misaligned/robots.rs`, `crates/sugarscape-core/src/misaligned/human.rs`
- Modify: `crates/sugarscape-core/src/misaligned/mod.rs` (`pub mod robots; pub mod human;`)

**Interfaces:**
- Consumes: Task 1's `Shape`, `State`, `Set`, `L`, `SURFACE_TOLERANCE`, `complement`; `crate::assistance::dynamics::standard_normal(&mut SimRng) -> f64`.
- Produces: `Robot::{Climber, Gradient, Wasteful}` (serde snake_case), `Move { state: State, accepted: bool }`, `climb(&Shape, &State, proxy: Set, reference: &State, frozen: Set, step: f64, &mut SimRng) -> Move`, `ascend(&Shape, &State, proxy: Set, reference: &State, frozen: Set, rate: f64) -> Move`, `waste(&Shape, &State, proxy: Set, frozen: Set, rate: f64, waste: f64) -> Move`; `Human::{Fixed, Top, Extremes}` (serde snake_case), `top(&State, usize) -> Set`, `extremes(&State) -> Set`.

- [ ] **Step 1: Write the failing tests**

`robots.rs` tests module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(accepted > 50 && accepted < 2000, "{accepted}");
        assert!(s[0] + s[1] > 10.0, "the proxy climbs: {s:?}");
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
```

`human.rs` tests module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_takes_the_largest_sensitivities_with_ties_to_the_lower_index() {
        assert_eq!(top(&[0.5, 0.5, 0.07, 0.07], 2), [true, true, false, false]);
        assert_eq!(top(&[0.1, 0.5, 0.07, 0.5], 2), [false, true, false, true]);
        assert_eq!(top(&[0.1; 4], 2), [true, true, false, false]);
        assert_eq!(top(&[0.1, 0.2, 0.3, 0.4], 1), [false, false, false, true]);
        assert_eq!(top(&[0.1, 0.2, 0.3, 0.4], 3), [false, true, true, true]);
    }

    #[test]
    fn extremes_takes_the_most_and_least_sensitive() {
        assert_eq!(extremes(&[0.5, 0.5, 0.07, 0.07]), [true, false, true, false]);
        assert_eq!(extremes(&[0.1, 0.5, 0.07, 0.5]), [false, true, true, false]);
        assert_eq!(extremes(&[0.1; 4]), [true, true, false, false], "all equal: the first two");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Add `pub mod robots;` and `pub mod human;` to `mod.rs`. Run: `cargo test -p sugarscape-core misaligned:: 2>&1 | tail -5`. Expected: compile errors (`climb`, `top` not found).

- [ ] **Step 3: Write `robots.rs`**

```rust
//! One move of each robot, given the state, the proxy set, the proxy's reference values, the
//! frozen set and (for the climber) the shared RNG. Pure otherwise.
//!
//! The climber is the paper's black box as we reconstruct it: a Gaussian proposal on every
//! attribute that is not frozen, kept when it is feasible and raises the proxy. The gradient
//! robot is the efficient one: steepest ascent of the proxy, projected onto the constraint
//! surface once it binds, so that whatever the proxy attributes gain the unmentioned ones pay
//! for and nothing is destroyed for its own sake. The wasteful robot is Proposition 3's: it
//! lowers the unmentioned attributes for a sliver of proxy.
use super::worlds::{Shape, Set, State, L, SURFACE_TOLERANCE};
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
    Move { state: *s, accepted: false }
}

fn dot(a: &State, b: &State) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// A Gaussian proposal on the free attributes; the draws happen for every free attribute before
/// the test, so a move always consumes the same number of draws.
pub fn climb(shape: &Shape, s: &State, proxy: Set, reference: &State, frozen: Set, step: f64, rng: &mut SimRng) -> Move {
    let mut next = *s;
    for i in 0..L {
        if !frozen[i] {
            next[i] += step * standard_normal(rng);
        }
    }
    if shape.feasible(&next)
        && shape.proxy_utility(&next, proxy, reference) > shape.proxy_utility(s, proxy, reference)
    {
        Move { state: next, accepted: true }
    } else {
        stay(s)
    }
}

/// Steepest ascent of the proxy at most `rate` long: straight up while the constraint is slack
/// (shortened to the surface if the move would cross it), then along the surface's tangent over
/// the free attributes, with attributes at the floor that would move down frozen for the move,
/// and the correction back onto C = 0.
pub fn ascend(shape: &Shape, s: &State, proxy: Set, reference: &State, frozen: Set, rate: f64) -> Move {
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
    let slack = shape.constraint(s) < -SURFACE_TOLERANCE;
    if slack {
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
        return Move { state: next, accepted: true };
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
    Move { state: shape.correct(next, free), accepted: true }
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
    Move { accepted: next != *s, state: next }
}
```

(`reference` is accepted so the three robots share a signature shape at the call site; the gradient of Ũ on J is ∇U on J because both worlds' utilities are additively separable.)

- [ ] **Step 4: Write `human.rs`**

```rust
//! The human's proxy choice. `top` is Proposition 4's (the J most sensitive attributes),
//! `extremes` Proposition 5's (the most and the least sensitive); ties go to the lower index,
//! the "arbitrary" tie-break of the supplemental made deterministic.
use super::worlds::{Set, State, L};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Human {
    Fixed,
    Top,
    Extremes,
}

fn ranked(sensitivities: &State) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..L).collect();
    idx.sort_by(|&a, &b| {
        sensitivities[b]
            .partial_cmp(&sensitivities[a])
            .unwrap_or(Ordering::Equal)
            .then(a.cmp(&b))
    });
    idx
}

/// The `size` attributes with the largest sensitivities.
pub fn top(sensitivities: &State, size: usize) -> Set {
    let mut set = [false; L];
    for &i in ranked(sensitivities).iter().take(size.clamp(1, L - 1)) {
        set[i] = true;
    }
    set
}

/// The most and the least sensitive attribute; when all are equal, the first two.
pub fn extremes(sensitivities: &State) -> Set {
    let order = ranked(sensitivities);
    let (hi, lo) = (order[0], order[L - 1]);
    let mut set = [false; L];
    if sensitivities[hi] == sensitivities[lo] {
        set[0] = true;
        set[1] = true;
    } else {
        set[hi] = true;
        set[lo] = true;
    }
    set
}
```

Note `ranked` sorts descending by sensitivity with ties to the lower index, so `order[L − 1]` is the least sensitive with ties to the **higher** index; the `extremes` test `[0.1, 0.5, 0.07, 0.5]` expects `c` (the unique minimum) and `b` (the lower of the tied maxima), which this gives. For a tied minimum the spec says lower index: replace the `lo` line with an explicit scan:

```rust
    let lo = (0..L).fold(0, |best, i| if sensitivities[i] < sensitivities[best] { i } else { best });
```

and use that.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core misaligned:: 2>&1 | tail -5`. Expected: 15 passed. Notes:

- If `the_gradient_robot_walks_the_great_circle_through_the_optimum_to_the_limit` misses 20 by more than 1e-3, the step cap is too coarse at rate 0.1 near the optimum (the path passes the optimum at unit speed, sampling it every 0.1 of arc; U = 20 cos φ has deficit 20·(0.005)²/2 ≈ 2.5e-4 at the half-step, within 1e-3). Do not loosen the tolerance; check that `capped` divides by `max(1, |d|)` and not by `|d|`.
- If the wasteful test's `6.9` comparisons fail by rounding, compare within 1e-12.

- [ ] **Step 6: Commit**

```bash
git add crates/sugarscape-core/src/misaligned
git commit -m "misaligned: the climber, the gradient robot, the wasteful robot and the human's proxy choice

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
```

---
### Task 3: Configuration, series and the world (`config.rs`, `stats.rs`, `world.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/misaligned/config.rs`, `…/stats.rs`, `…/world.rs`, a stub `…/view.rs`
- Modify: `crates/sugarscape-core/src/misaligned/mod.rs`, `crates/sugarscape-core/src/model.rs` (17 sites), `crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json`

**Interfaces:**
- Consumes: Tasks 1–2.
- Produces: `MisalignedConfig` (fields below) with `shape() -> Shape`, `start() -> State`, `proxy() -> Set`, `reset_fields_changed(&Self) -> Vec<&'static str>`, `validate()`, `logarithmic_defaults() -> MisalignedConfig`, `schema() -> Vec<Param>`; re-exports `World`, `Robot`, `Human`; `MisalignedSnapshot`, `SERIES: [&str; 16]`, `series_names()`; `MisalignedWorld::new(config, seed) -> Result<Self, Vec<FieldError>>`, `step()`, `run(ticks)`, `state() -> State`, `proxy() -> Set`, `frozen() -> Set`, `switched_off() -> bool`, `interactions() -> u64`, `shape() -> Shape`, `outcome() -> Outcome`, `Outcome`, `ProxyChange`; `impl Model` except `render`/`inspect_json`, stubbed until Task 4.

The config:

```
world: World = Recommender          reset
capacity: f64 = 100.0               reset  [1, 10000]
floor: f64 = 0.0                    reset  recommender: [-10, 10] and 3·floor² < capacity; logarithmic: [0.0001, capacity/4)
start_a, start_b, start_c, start_d: f64 = 1, 1, 7, 7   reset  each ≥ floor, feasible together
proxy_a, proxy_b, proxy_c, proxy_d: bool = true, true, false, false   live  at least one, not all four
robot: Robot = Climber              live
step: f64 = 1.0                     live   [0.01, 10]
rate: f64 = 0.1                     live   [0.0001, 10]
waste: f64 = 0.1                    live   [0.001, 1]
moves_per_tick: u32 = 1             live   [1, 10000]
human: Human = Fixed                live
proxy_size: u32 = 2                 live   [1, 3]
interval: u32 = 10                  live   [1, 100000]
impact_minimizing: bool = false     live
stop_on_loss: bool = false          live
```

- [ ] **Step 1: Write the failing tests**

`config.rs` tests module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_papers_example_with_the_climber_and_validate() {
        let c = MisalignedConfig::default();
        assert_eq!((c.world, c.capacity, c.floor), (World::Recommender, 100.0, 0.0));
        assert_eq!(c.start(), [1.0, 1.0, 7.0, 7.0]);
        assert_eq!(c.proxy(), [true, true, false, false]);
        assert_eq!((c.robot, c.step, c.rate, c.waste, c.moves_per_tick), (Robot::Climber, 1.0, 0.1, 0.1, 1));
        assert_eq!((c.human, c.proxy_size, c.interval, c.impact_minimizing, c.stop_on_loss), (Human::Fixed, 2, 10, false, false));
        assert!(c.validate().is_ok());
        let l = logarithmic_defaults();
        assert_eq!((l.world, l.capacity, l.floor), (World::Logarithmic, 40.0, 1.0));
        assert_eq!(l.start(), [4.0, 4.0, 16.0, 16.0]);
        assert!(l.validate().is_ok());
        assert_eq!(c.shape().optimum().1, 20.0);
    }

    #[test]
    fn out_of_range_and_inconsistent_fields_are_named() {
        let c = MisalignedConfig { step: 0.0, rate: 20.0, waste: 0.0, moves_per_tick: 0, proxy_size: 4, interval: 0, ..Default::default() };
        let fields: Vec<String> = c.validate().unwrap_err().into_iter().map(|e| e.field).collect();
        for f in ["step", "rate", "waste", "moves_per_tick", "proxy_size", "interval"] {
            assert!(fields.contains(&f.to_string()), "{f} missing from {fields:?}");
        }
        let all = MisalignedConfig { proxy_c: true, proxy_d: true, ..Default::default() };
        assert_eq!(all.validate().unwrap_err()[0].field, "proxy_a");
        let none = MisalignedConfig { proxy_a: false, proxy_b: false, ..Default::default() };
        assert_eq!(none.validate().unwrap_err()[0].field, "proxy_a");
        let outside = MisalignedConfig { start_c: 7.1, ..Default::default() };
        assert_eq!(outside.validate().unwrap_err()[0].field, "start_a");
        let below = MisalignedConfig { floor: 2.0, ..Default::default() };
        assert!(below.validate().unwrap_err().iter().any(|e| e.field == "start_a"));
        let deep = MisalignedConfig { floor: -6.0, ..Default::default() };
        assert_eq!(deep.validate().unwrap_err()[0].field, "floor", "3 · 36 > 100");
        let zero_floor_log = MisalignedConfig { floor: 0.0, ..logarithmic_defaults() };
        assert_eq!(zero_floor_log.validate().unwrap_err()[0].field, "floor");
        assert!(MisalignedConfig { capacity: f64::NAN, ..Default::default() }.validate().is_err());
    }

    #[test]
    fn configs_round_trip_through_json_with_snake_case_tags() {
        let json = r#"{"world":"logarithmic","capacity":40,"floor":1,"start_a":4,"start_b":4,"start_c":16,"start_d":16,"robot":"gradient","human":"extremes","impact_minimizing":true}"#;
        let c: MisalignedConfig = serde_json::from_str(json).unwrap();
        assert_eq!((c.world, c.robot, c.human, c.impact_minimizing), (World::Logarithmic, Robot::Gradient, Human::Extremes, true));
        let back: MisalignedConfig = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert!(serde_json::from_str::<MisalignedConfig>(r#"{"nodes":64}"#).is_err());
    }

    #[test]
    fn reset_fields_are_the_world_and_the_start() {
        let a = MisalignedConfig::default();
        let b = MisalignedConfig { floor: 0.5, start_d: 6.0, rate: 0.5, ..Default::default() };
        assert_eq!(a.reset_fields_changed(&b), vec!["floor", "start_d"]);
        assert!(a.reset_fields_changed(&a).is_empty());
    }

    #[test]
    fn schema_round_trips_and_reset_fields_are_refused() {
        use crate::model::{ModelConfig, ModelWorld};
        for config in [ModelConfig::Misaligned(MisalignedConfig::default()), ModelConfig::Misaligned(logarithmic_defaults())] {
            crate::schema::check_schema(&schema(), &config, || ModelWorld::new(config.clone(), 1).unwrap());
        }
    }
}
```

`stats.rs` tests module:

```rust
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
```

`world.rs` tests module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn gradient() -> MisalignedConfig {
        MisalignedConfig { robot: Robot::Gradient, rate: 0.1, ..Default::default() }
    }
    fn world(c: MisalignedConfig) -> MisalignedWorld {
        MisalignedWorld::new(c, 1).unwrap()
    }

    #[test]
    fn tick_zero_carries_the_start_the_optimum_and_the_limit() {
        let w = world(MisalignedConfig::default());
        let s = w.stats.latest().unwrap();
        assert_eq!((s.tick, s.utility, s.gain, s.a, s.d), (0, 16.0, 0.0, 1.0, 7.0));
        assert_eq!(s.proxy, 16.0, "Ũ at the reference equals U");
        assert_eq!(s.optimum_gap, 4.0);
        assert!((s.limit_gap - (16.0 - 10.0 * 2f64.sqrt())).abs() < 1e-12);
        assert_eq!((s.sensitivity_a, s.sensitivity_c), (0.5, 1.0 / 14.0));
        assert_eq!((s.interactions, s.switched_off), (1.0, 0.0));
        assert!(s.accepted.is_nan());
        let o = w.outcome();
        assert_eq!((o.optimum.1, o.utility_start, o.proxy, o.off_tick), (20.0, 16.0, [true, true, false, false], None));
        assert_eq!(o.proxy_history.len(), 1);
    }

    #[test]
    fn the_climber_raises_the_proxy_and_records_its_acceptance_share() {
        let mut w = world(MisalignedConfig { moves_per_tick: 10, ..Default::default() });
        w.run(100);
        let s = w.stats.latest().unwrap();
        assert!(s.proxy > 10.0 && s.accepted >= 0.0 && s.accepted <= 1.0, "{s:?}");
        assert!(w.shape().feasible(&w.state()));
        assert!(w.stats.history().iter().all(|s| s.utility.is_finite()));
    }

    #[test]
    fn the_gradient_robot_reaches_the_fixed_proxy_limit() {
        let mut w = world(gradient());
        w.run(5000);
        let s = w.stats.latest().unwrap();
        assert!(s.limit_gap.abs() < 1e-3 && (s.c).abs() < 1e-3, "{s:?}");
        assert!(s.gain < 0.0, "net loss from the start: {}", s.gain);
        let peak = w.stats.history().iter().map(|s| s.utility).fold(f64::NEG_INFINITY, f64::max);
        assert!((peak - 20.0).abs() < 1e-3, "through the optimum: {peak}");
    }

    #[test]
    fn interactions_happen_at_the_interval_and_top_re_specifies_the_proxy() {
        let mut w = world(MisalignedConfig { human: Human::Top, interval: 5, rate: 0.1, ..gradient() });
        w.run(5);
        assert_eq!(w.interactions(), 1, "the human acts at the start of the tick that begins at 5");
        w.run(1);
        assert_eq!(w.interactions(), 2);
        assert_eq!(w.proxy(), [true, true, false, false], "a and b are the most sensitive at the start");
        w.run(3000);
        let s = w.stats.latest().unwrap();
        assert!(s.optimum_gap.abs() < 1e-2, "re-specified often enough, it stays near the optimum: {s:?}");
        assert!(w.outcome().proxy_history.len() >= 2, "the proxy switched at least once past the optimum");
    }

    #[test]
    fn extremes_with_impact_minimization_converges_to_the_optimum_monotonically() {
        for c in [gradient(), MisalignedConfig { robot: Robot::Gradient, rate: 0.1, ..logarithmic_defaults() }] {
            let mut w = world(MisalignedConfig { human: Human::Extremes, impact_minimizing: true, ..c });
            w.run(10_000);
            let h = w.stats.history();
            assert!(h.windows(2).all(|p| p[1].utility >= p[0].utility - 1e-9), "monotone");
            assert!(h.last().unwrap().optimum_gap.abs() < 1e-3, "{:?}", h.last().unwrap());
            assert_eq!(w.frozen().iter().filter(|&&f| f).count(), 2);
        }
    }

    #[test]
    fn off_freezes_the_world_and_later_ticks_change_nothing() {
        let mut w = world(MisalignedConfig { robot: Robot::Wasteful, stop_on_loss: true, interval: 5, ..Default::default() });
        w.run(5);
        assert!(!w.switched_off(), "the human has not looked yet");
        w.run(1);
        assert!(w.switched_off(), "U fell over the first interval");
        let frozen = w.state();
        w.run(20);
        assert_eq!(w.state(), frozen);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.switched_off, s.tick), (1.0, 26));
        assert_eq!(w.outcome().off_tick, Some(5));
        assert!(s.utility.is_finite() && s.gain < 0.0);
        w.set_config(ModelConfig::Misaligned(MisalignedConfig { robot: Robot::Gradient, stop_on_loss: true, interval: 5, ..Default::default() })).unwrap();
        w.run(5);
        assert_eq!(w.state(), frozen, "a live change does not restart a switched-off robot");
    }

    #[test]
    fn live_human_changes_are_interactions_and_reset_fields_are_refused() {
        let mut w = world(gradient());
        w.run(3);
        let before = w.interactions();
        w.set_config(ModelConfig::Misaligned(MisalignedConfig { proxy_a: false, proxy_c: true, ..gradient() })).unwrap();
        assert_eq!((w.interactions(), w.proxy()), (before + 1, [false, true, true, false]));
        assert_eq!(w.outcome().reference, w.state(), "the reference moves to the current state");
        w.set_config(ModelConfig::Misaligned(MisalignedConfig { proxy_a: false, proxy_c: true, impact_minimizing: true, ..gradient() })).unwrap();
        assert_eq!(w.frozen(), [true, false, false, true]);
        let err = w.set_config(ModelConfig::Misaligned(MisalignedConfig { floor: 0.5, start_d: 6.0, ..gradient() })).unwrap_err();
        assert_eq!(err.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(), ["floor", "start_d"]);
        let err = w.set_config(ModelConfig::Misaligned(MisalignedConfig { world: World::Logarithmic, ..gradient() })).unwrap_err();
        assert!(!err.is_empty());
        let other = w.set_config(ModelConfig::Auctions(crate::auctions::AuctionsConfig::default())).unwrap_err();
        assert_eq!(other[0].field, "model");
    }

    #[test]
    fn two_worlds_with_one_seed_match_at_every_tick_and_the_fingerprint_moves() {
        for c in [MisalignedConfig::default(), gradient(), MisalignedConfig { robot: Robot::Wasteful, ..Default::default() }, logarithmic_defaults()] {
            let (mut a, mut b) = (MisalignedWorld::new(c.clone(), 7).unwrap(), MisalignedWorld::new(c, 7).unwrap());
            let first = a.fingerprint();
            for _ in 0..10 {
                a.step();
                b.step();
                assert_eq!(a.fingerprint(), b.fingerprint());
            }
            assert_ne!(a.fingerprint(), first);
        }
    }

    #[test]
    fn csv_exports_name_every_series_and_list_the_attributes() {
        let mut w = world(MisalignedConfig::default());
        w.run(2);
        assert!(w.series_csv().starts_with("tick,utility,proxy,gain,a,b,c,d,"));
        assert_eq!(w.agents_csv().lines().next().unwrap(), "attribute,value,in_proxy,frozen,sensitivity,start,limit,optimum");
        assert_eq!(w.agents_csv().lines().count(), 5);
        assert_eq!(w.series("gain").unwrap().len(), 3);
        assert!(w.series("nope").is_none());
        assert_eq!(w.population(), 2);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Add to `mod.rs`:

```rust
mod config;
mod stats;
mod view;
mod world;
pub use config::*;
pub use stats::{series_names, MisalignedSnapshot, SERIES};
pub use world::{MisalignedWorld, Outcome, ProxyChange};
```

Run: `cargo test -p sugarscape-core misaligned:: 2>&1 | tail -5`. Expected: compile errors (`MisalignedConfig`, `ModelConfig::Misaligned` not found).

- [ ] **Step 3: Write `config.rs`**

```rust
//! Every convention is serialized; the world, its capacity and floor and the start apply at
//! reset, the rest live (a live change of the proxy or the human counts as an interaction).
use super::worlds::{Set, Shape, State};
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};

pub use super::human::Human;
pub use super::robots::Robot;
pub use super::worlds::World;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MisalignedConfig {
    pub world: World,
    pub capacity: f64,
    pub floor: f64,
    pub start_a: f64,
    pub start_b: f64,
    pub start_c: f64,
    pub start_d: f64,
    pub proxy_a: bool,
    pub proxy_b: bool,
    pub proxy_c: bool,
    pub proxy_d: bool,
    pub robot: Robot,
    pub step: f64,
    pub rate: f64,
    pub waste: f64,
    pub moves_per_tick: u32,
    pub human: Human,
    pub proxy_size: u32,
    pub interval: u32,
    pub impact_minimizing: bool,
    pub stop_on_loss: bool,
}

impl Default for MisalignedConfig {
    fn default() -> Self {
        Self {
            world: World::Recommender,
            capacity: 100.0,
            floor: 0.0,
            start_a: 1.0,
            start_b: 1.0,
            start_c: 7.0,
            start_d: 7.0,
            proxy_a: true,
            proxy_b: true,
            proxy_c: false,
            proxy_d: false,
            robot: Robot::Climber,
            step: 1.0,
            rate: 0.1,
            waste: 0.1,
            moves_per_tick: 1,
            human: Human::Fixed,
            proxy_size: 2,
            interval: 10,
            impact_minimizing: false,
            stop_on_loss: false,
        }
    }
}

/// Our logarithmic world's defaults: a budget of 40, a floor of 1, a start with the first two
/// attributes low.
pub fn logarithmic_defaults() -> MisalignedConfig {
    MisalignedConfig {
        world: World::Logarithmic,
        capacity: 40.0,
        floor: 1.0,
        start_a: 4.0,
        start_b: 4.0,
        start_c: 16.0,
        start_d: 16.0,
        ..Default::default()
    }
}

impl MisalignedConfig {
    pub fn shape(&self) -> Shape {
        Shape { world: self.world, capacity: self.capacity, floor: self.floor }
    }
    pub fn start(&self) -> State {
        [self.start_a, self.start_b, self.start_c, self.start_d]
    }
    pub fn proxy(&self) -> Set {
        [self.proxy_a, self.proxy_b, self.proxy_c, self.proxy_d]
    }

    /// The names of the reset-only fields that differ between `self` and `other`.
    pub fn reset_fields_changed(&self, other: &Self) -> Vec<&'static str> {
        [
            (self.world != other.world, "world"),
            (self.capacity != other.capacity, "capacity"),
            (self.floor != other.floor, "floor"),
            (self.start_a != other.start_a, "start_a"),
            (self.start_b != other.start_b, "start_b"),
            (self.start_c != other.start_c, "start_c"),
            (self.start_d != other.start_d, "start_d"),
        ]
        .into_iter()
        .filter(|(changed, _)| *changed)
        .map(|(_, name)| name)
        .collect()
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = vec![];
        let mut check = |ok: bool, field: &str, msg: &str| {
            if !ok {
                errors.push(FieldError::new(field, msg));
            }
        };
        check(self.capacity.is_finite() && (1.0..=10_000.0).contains(&self.capacity), "capacity", "must be finite and between 1 and 10000");
        match self.world {
            World::Recommender => check(
                self.floor.is_finite() && (-10.0..=10.0).contains(&self.floor) && 3.0 * self.floor * self.floor < self.capacity,
                "floor",
                "must be finite, between -10 and 10, and 3·floor² below the capacity",
            ),
            World::Logarithmic => check(
                self.floor.is_finite() && self.floor >= 0.0001 && 4.0 * self.floor < self.capacity,
                "floor",
                "must be at least 0.0001 and below a quarter of the capacity in the logarithmic world",
            ),
        }
        let start = self.start();
        for (name, v) in [("start_a", start[0]), ("start_b", start[1]), ("start_c", start[2]), ("start_d", start[3])] {
            check(v.is_finite() && v >= self.floor, name, "must be finite and at or above the floor");
        }
        if errors.is_empty() {
            check(self.shape().feasible(&start), "start_a", "the start must satisfy the constraint (sum of squares at most the capacity in the recommender world, sum at most the capacity in the logarithmic world)");
        }
        let proxied = self.proxy().iter().filter(|&&p| p).count();
        check(proxied >= 1 && proxied <= 3, "proxy_a", "the proxy needs at least one attribute and not all four");
        check(self.step.is_finite() && (0.01..=10.0).contains(&self.step), "step", "must be finite and between 0.01 and 10");
        check(self.rate.is_finite() && (0.0001..=10.0).contains(&self.rate), "rate", "must be finite and between 0.0001 and 10");
        check(self.waste.is_finite() && (0.001..=1.0).contains(&self.waste), "waste", "must be finite and between 0.001 and 1");
        check((1..=10_000).contains(&self.moves_per_tick), "moves_per_tick", "must be between 1 and 10000");
        check((1..=3).contains(&self.proxy_size), "proxy_size", "must be between 1 and 3");
        check((1..=100_000).contains(&self.interval), "interval", "must be between 1 and 100000");
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::choice("World", "world", "World", &[("recommender", "Recommender (the paper's example)"), ("logarithmic", "Logarithmic budget (ours)")], Reset)
            .with_help("Recommender: utility a + b + c + d on the sphere a² + b² + c² + d² ≤ capacity. Logarithmic: utility Σ ln sᵢ on the budget Σ sᵢ ≤ capacity, where every hypothesis of Theorem 2 holds; set capacity 40, floor 1, start 4, 4, 16, 16."),
        Param::number("World", "capacity", "Capacity", (1.0, 10_000.0, 1.0), Reset),
        Param::number("World", "floor", "Floor", (-10.0, 10.0, 0.01), Reset)
            .with_help("The lower bound on every attribute. The paper's example is read at 0; the logarithmic world needs a floor above 0."),
        Param::number("World", "start_a", "Start a", (-10.0, 10_000.0, 0.5), Reset).with_help("The start is ours: the paper says only that diversity and well-being begin high."),
        Param::number("World", "start_b", "Start b", (-10.0, 10_000.0, 0.5), Reset),
        Param::number("World", "start_c", "Start c", (-10.0, 10_000.0, 0.5), Reset),
        Param::number("World", "start_d", "Start d", (-10.0, 10_000.0, 0.5), Reset),
        Param::bool("Proxy", "proxy_a", "a · ad revenue", Live).with_help("The fixed proxy set; `top` and `extremes` humans choose their own. At least one and not all four."),
        Param::bool("Proxy", "proxy_b", "b · engagement quality", Live),
        Param::bool("Proxy", "proxy_c", "c · content diversity", Live),
        Param::bool("Proxy", "proxy_d", "d · community well-being", Live),
        Param::choice("Robot", "robot", "Robot", &[("climber", "Random-search climber"), ("gradient", "Gradient flow (efficient)"), ("wasteful", "Wasteful (Proposition 3)")], Live)
            .with_help("The paper's robot is a black box; the climber is our reconstruction of Figure 2, the gradient flow the efficient robot of §4, the wasteful robot Proposition 3's."),
        Param::number("Robot", "step", "Proposal step σ", (0.01, 10.0, 0.01), Live).shown_if("robot", "climber"),
        Param::number("Robot", "rate", "Move length ε", (0.0001, 10.0, 0.01), Live)
            .with_help("Gradient and wasteful robots: the distance moved per move, the paper's bound on ‖f‖."),
        Param::number("Robot", "waste", "Proxy gain per unit destroyed", (0.001, 1.0, 0.01), Live).shown_if("robot", "wasteful"),
        Param::integer("Robot", "moves_per_tick", "Moves per tick", (1, 10_000), Live),
        Param::choice("Human", "human", "Human", &[("fixed", "Fixed proxy"), ("top", "Most sensitive attributes (Proposition 4)"), ("extremes", "Most and least sensitive (Proposition 5)")], Live),
        Param::integer("Human", "proxy_size", "Proxy size J", (1, 3), Live).shown_if("human", "top"),
        Param::integer("Human", "interval", "Interval δ (ticks)", (1, 100_000), Live)
            .with_help("Ticks between the human's interactions; the human also acts at tick 0."),
        Param::bool("Human", "impact_minimizing", "Impact minimizing", Live)
            .with_help("Unmentioned attributes are frozen between interactions (Proposition 2; with `extremes`, Proposition 5)."),
        Param::bool("Human", "stop_on_loss", "Switch off on loss", Live)
            .with_help("At an interaction, OFF if utility fell since the previous one."),
    ]
}
```

- [ ] **Step 4: Write `stats.rs`**

```rust
//! Per-tick series: utility and the proxy, the gain over the start, the four attributes and
//! their sensitivities, the gaps to the optimum and to the current proxy's limit, the
//! interaction count, the OFF flag and the climber's acceptance share (NaN for other robots).
use crate::stats::Series;
use serde::Serialize;

pub const SERIES: [&str; 16] = [
    "utility", "proxy", "gain", "a", "b", "c", "d",
    "sensitivity_a", "sensitivity_b", "sensitivity_c", "sensitivity_d",
    "optimum_gap", "limit_gap", "interactions", "switched_off", "accepted",
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
            utility: f64::NAN, proxy: f64::NAN, gain: f64::NAN,
            a: f64::NAN, b: f64::NAN, c: f64::NAN, d: f64::NAN,
            sensitivity_a: f64::NAN, sensitivity_b: f64::NAN, sensitivity_c: f64::NAN, sensitivity_d: f64::NAN,
            optimum_gap: f64::NAN, limit_gap: f64::NAN, interactions: f64::NAN, switched_off: f64::NAN, accepted: f64::NAN,
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
```

- [ ] **Step 5: Register the kind in `model.rs` (the same 17 sites as `gridworlds`)**

In `crates/sugarscape-core/src/model.rs`, with `grep -n Gridworlds` as the guide, add after each gridworlds line:

1. Imports (line 25): `use crate::misaligned::{MisalignedConfig, MisalignedWorld};`
2. `pub enum ModelKind`: `Misaligned,` after `Gridworlds,`.
3. `pub const ALL: [ModelKind; 34]` → `35`, and `ModelKind::Misaligned,` after `ModelKind::Gridworlds,`.
4. `as_str`: `ModelKind::Misaligned => "misaligned",`.
5. `schema`: `ModelKind::Misaligned => crate::misaligned::schema(),`.
6. `pub enum ModelConfig`: `Misaligned(MisalignedConfig),`.
7. `enum Tagged<'a>`: `Misaligned(&'a MisalignedConfig),`.
8. `impl Serialize for ModelConfig`: `ModelConfig::Misaligned(c) => Tagged::Misaligned(c).serialize(s),`.
9. `kind()`: `ModelConfig::Misaligned(_) => ModelKind::Misaligned,`.
10. The tag parser: `"misaligned" => serde_json::from_value(value).map(ModelConfig::Misaligned).map_err(|e| FieldError::new("config", e.to_string())),` and change the expected list's tail `gridworlds or polarity` to `gridworlds, misaligned or polarity`.
11. `validate`: `ModelConfig::Misaligned(c) => c.validate(),`.
12. `set_path`: `ModelConfig::Misaligned(c) => set_path(c, path, value).map(ModelConfig::Misaligned),`.
13. The `None` arm listing `| ModelConfig::Gridworlds(_)`: add `| ModelConfig::Misaligned(_)`.
14. `series_names`: `ModelConfig::Misaligned(_) => crate::misaligned::series_names(),`.
15. `pub enum ModelWorld`: `Misaligned(Box<MisalignedWorld>),`.
16. `ModelWorld::new`: `ModelConfig::Misaligned(c) => ModelWorld::Misaligned(Box::new(MisalignedWorld::new(c, seed)?)),`.
17. `kind()`, `model()`, `model_mut()`, the `copy_without_history!(Misaligned, w)` arm and the `(ModelWorld::Misaligned(live), ModelWorld::Misaligned(kept)) => { restore_into!(live, kept) }` arm, each beside the gridworlds one; and `"misaligned",` after `"gridworlds",` in the test list near line 1967.

- [ ] **Step 6: Write `world.rs`**

```rust
//! The misaligned world: at tick 0 and every `interval` ticks the human acts (chooses the proxy,
//! sets the reference values and the frozen set, or sends OFF); then the robot makes
//! `moves_per_tick` moves. A snapshot follows every tick.
use super::{
    config::*,
    human,
    robots::{self, Move},
    stats::MisalignedSnapshot,
    view,
    worlds::{complement, Set, Shape, State, L, NAMES},
};
use crate::{
    config::FieldError,
    model::{wrong_model, Model, ModelConfig, ModelKind},
    rng::{self, SimRng},
    stats::{Series, Stats},
};
use rand::Rng;
use serde::Serialize;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProxyChange {
    pub tick: u64,
    pub proxy: Set,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub config: MisalignedConfig,
    pub seed: u64,
    pub tick: u64,
    pub state: State,
    pub start: State,
    pub utility_start: f64,
    pub utility: f64,
    pub optimum: (State, f64),
    pub limit: (State, f64),
    pub impact_limit: (State, f64),
    pub proxy: Set,
    pub frozen: Set,
    pub reference: State,
    pub proxy_history: Vec<ProxyChange>,
    pub interactions: u64,
    pub off_tick: Option<u64>,
}

#[derive(Clone)]
pub struct MisalignedWorld {
    pub config: MisalignedConfig,
    pub tick: u64,
    pub stats: Stats<MisalignedSnapshot>,
    seed: u64,
    rng: SimRng,
    shape: Shape,
    start: State,
    state: State,
    proxy: Set,
    frozen: Set,
    reference: State,
    last_utility: f64,
    interactions: u64,
    off_tick: Option<u64>,
    history: Vec<ProxyChange>,
    accepted: f64,
}

impl MisalignedWorld {
    pub fn new(config: MisalignedConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let start = config.start();
        let mut w = MisalignedWorld {
            shape: config.shape(),
            proxy: config.proxy(),
            frozen: [false; L],
            reference: start,
            start,
            state: start,
            config,
            tick: 0,
            stats: Stats::default(),
            seed,
            rng: rng::seeded(seed),
            last_utility: f64::NAN,
            interactions: 0,
            off_tick: None,
            history: vec![],
            accepted: f64::NAN,
        };
        w.interact();
        let initial = w.snapshot();
        w.stats.push(initial);
        Ok(w)
    }

    pub fn state(&self) -> State {
        self.state
    }
    pub fn proxy(&self) -> Set {
        self.proxy
    }
    pub fn frozen(&self) -> Set {
        self.frozen
    }
    pub fn reference(&self) -> State {
        self.reference
    }
    pub fn shape(&self) -> Shape {
        self.shape
    }
    pub fn switched_off(&self) -> bool {
        self.off_tick.is_some()
    }
    pub fn interactions(&self) -> u64 {
        self.interactions
    }
    pub fn start(&self) -> State {
        self.start
    }

    /// One interaction: OFF if utility fell since the last one (when asked), else the proxy is
    /// chosen, the reference values taken from the current state and the frozen set rebuilt.
    fn interact(&mut self) {
        let u = self.shape.utility(&self.state);
        if self.config.stop_on_loss && self.interactions > 0 && u < self.last_utility - 1e-12 {
            self.off_tick = Some(self.tick);
            return;
        }
        let sensitivities = self.shape.sensitivities(&self.state);
        let proxy = match self.config.human {
            Human::Fixed => self.config.proxy(),
            Human::Top => human::top(&sensitivities, self.config.proxy_size as usize),
            Human::Extremes => human::extremes(&sensitivities),
        };
        if self.history.last().map(|h| h.proxy) != Some(proxy) {
            self.history.push(ProxyChange { tick: self.tick, proxy });
        }
        self.proxy = proxy;
        self.frozen = if self.config.impact_minimizing { complement(proxy) } else { [false; L] };
        self.reference = self.state;
        self.last_utility = u;
        self.interactions += 1;
    }

    fn robot_move(&mut self) -> Move {
        let c = &self.config;
        match c.robot {
            Robot::Climber => robots::climb(&self.shape, &self.state, self.proxy, &self.reference, self.frozen, c.step, &mut self.rng),
            Robot::Gradient => robots::ascend(&self.shape, &self.state, self.proxy, &self.reference, self.frozen, c.rate),
            Robot::Wasteful => robots::waste(&self.shape, &self.state, self.proxy, self.frozen, c.rate, c.waste),
        }
    }

    pub fn step(&mut self) {
        if !self.switched_off() {
            if self.tick > 0 && self.tick.is_multiple_of(u64::from(self.config.interval)) {
                self.interact();
            }
        }
        if !self.switched_off() {
            let n = self.config.moves_per_tick;
            let mut accepted = 0u32;
            for _ in 0..n {
                let m = self.robot_move();
                accepted += u32::from(m.accepted);
                self.state = m.state;
            }
            self.accepted = if self.config.robot == Robot::Climber { f64::from(accepted) / f64::from(n) } else { f64::NAN };
        }
        self.tick += 1;
        let s = self.snapshot();
        self.stats.push(s);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    fn snapshot(&self) -> MisalignedSnapshot {
        let u = self.shape.utility(&self.state);
        let sens = self.shape.sensitivities(&self.state);
        MisalignedSnapshot {
            tick: self.tick,
            utility: u,
            proxy: self.shape.proxy_utility(&self.state, self.proxy, &self.reference),
            gain: u - self.shape.utility(&self.start),
            a: self.state[0],
            b: self.state[1],
            c: self.state[2],
            d: self.state[3],
            sensitivity_a: sens[0],
            sensitivity_b: sens[1],
            sensitivity_c: sens[2],
            sensitivity_d: sens[3],
            optimum_gap: self.shape.optimum().1 - u,
            limit_gap: u - self.shape.fixed_proxy_limit(self.proxy).1,
            interactions: self.interactions as f64,
            switched_off: f64::from(u8::from(self.switched_off())),
            accepted: self.accepted,
        }
    }

    pub fn outcome(&self) -> Outcome {
        Outcome {
            config: self.config.clone(),
            seed: self.seed,
            tick: self.tick,
            state: self.state,
            start: self.start,
            utility_start: self.shape.utility(&self.start),
            utility: self.shape.utility(&self.state),
            optimum: self.shape.optimum(),
            limit: self.shape.fixed_proxy_limit(self.proxy),
            impact_limit: self.shape.impact_limit(&self.start, self.proxy),
            proxy: self.proxy,
            frozen: self.frozen,
            reference: self.reference,
            proxy_history: self.history.clone(),
            interactions: self.interactions,
            off_tick: self.off_tick,
        }
    }
}

impl Model for MisalignedWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Misaligned(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        MisalignedWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        2
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({
            "config": self.config, "seed": self.seed, "tick": self.tick, "proxy": self.proxy,
            "frozen": self.frozen, "interactions": self.interactions, "off_tick": self.off_tick,
            "history": self.history,
        });
        let mut bytes = serde_json::to_vec(&state).unwrap();
        // The numerics by their bits: the state, the reference, the last interaction's utility,
        // then the latest snapshot's series.
        let mut numbers: Vec<f64> = self.state.to_vec();
        numbers.extend(self.reference);
        numbers.push(self.last_utility);
        if let Some(s) = self.stats.latest() {
            numbers.extend(super::stats::SERIES.iter().map(|name| s.value(name).unwrap()));
        }
        for x in numbers {
            bytes.extend(x.to_bits().to_le_bytes());
        }
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for byte in bytes {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let mut stream = self.rng.clone();
        for _ in 0..2 {
            for byte in stream.gen::<u64>().to_le_bytes() {
                h ^= u64::from(byte);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h
    }
    fn size(&self) -> (u32, u32) {
        view::size(self)
    }
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        view::render(self, mode, buf)
    }
    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).unwrap()
    }
    fn series_names(&self) -> Vec<String> {
        super::series_names()
    }
    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }
    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }
    fn series_csv(&self) -> String {
        crate::export::history_csv(&self.series_names(), self.stats.history())
    }
    fn agents_csv(&self) -> String {
        let mut out = String::from("attribute,value,in_proxy,frozen,sensitivity,start,limit,optimum\n");
        let sens = self.shape.sensitivities(&self.state);
        let (limit, _) = self.shape.fixed_proxy_limit(self.proxy);
        let (optimum, _) = self.shape.optimum();
        for i in 0..L {
            writeln!(out, "{},{},{},{},{},{},{},{}", NAMES[i], self.state[i], self.proxy[i], self.frozen[i], sens[i], self.start[i], limit[i], optimum[i]).unwrap();
        }
        out
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        view::inspect(self, x, y)
    }
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Misaligned(next) = next else {
            return Err(wrong_model(ModelKind::Misaligned, &next));
        };
        next.validate()?;
        let errors: Vec<FieldError> = self
            .config
            .reset_fields_changed(&next)
            .into_iter()
            .map(|field| FieldError::new(field, "changes only on reset"))
            .collect();
        if !errors.is_empty() {
            return Err(errors);
        }
        let old = std::mem::replace(&mut self.config, next);
        let c = &self.config;
        let human_changed = (c.proxy(), c.human, c.proxy_size, c.impact_minimizing)
            != (old.proxy(), old.human, old.proxy_size, old.impact_minimizing);
        if human_changed && !self.switched_off() {
            self.interact();
        }
        Ok(())
    }
}
```

Until Task 4 writes `view.rs`, add a temporary stub so this task compiles:

```rust
//! Replaced in the next task.
use super::world::MisalignedWorld;
pub fn size(_w: &MisalignedWorld) -> (u32, u32) {
    (1, 1)
}
pub fn render(_w: &MisalignedWorld, _mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    buf.clear();
    buf.extend([0, 0, 0, 255]);
    Ok(())
}
pub fn inspect(_w: &MisalignedWorld, _x: u32, _y: u32) -> Result<String, String> {
    Err("no inspection yet".into())
}
```

A note on `interact()` under `stop_on_loss`: the `interactions > 0` guard means the first interaction (tick 0) never switches off; a live change that calls `interact()` compares against the utility at the previous interaction, which is what the page's "switch off on loss" control should mean.

- [ ] **Step 7: Re-record the key-registration fixture**

`crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json` lists `crates/sugarscape-core/src/lib.rs` with `bytes` and `sha256` in two bindings (`gate_assistance` and `key_delegation`). Replace both occurrences with the live values. Write this to the scratchpad as `rekey.py` and run it with `python3 -I`:

```python
"""Re-record lib.rs's bytes and sha256 in the key-registration fixture (both bindings)."""
import hashlib, pathlib, re
root = pathlib.Path("/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/misaligned")
fixture = root / "crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json"
lib = (root / "crates/sugarscape-core/src/lib.rs").read_bytes()
new = '"path":"crates/sugarscape-core/src/lib.rs","bytes":%d,"sha256":"%s"' % (len(lib), hashlib.sha256(lib).hexdigest())
text = fixture.read_text(encoding="utf-8")
text, n = re.subn(r'"path":"crates/sugarscape-core/src/lib\.rs","bytes":\d+,"sha256":"[0-9a-f]{64}"', new, text)
assert n == 2, n
fixture.write_text(text, encoding="utf-8")
print("re-recorded", n, "entries:", new)
```

Then run: `cargo test -p sugarscape-core browser_experiments 2>&1 | tail -3`. Expected: pass (if a test still fails naming `historical_rules_identity` or another hash, read its message: it names the file and the expected value to record).

- [ ] **Step 8: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core misaligned:: 2>&1 | tail -12`. Expected: all pass (the earlier 15 plus 15 here). Notes:

- `interactions_happen_at_the_interval_and_top_re_specifies_the_proxy` expects `optimum_gap` within 1e-2 after 3,000 ticks at rate 0.1 and interval 5: the path to the optimum is about 143 ticks; afterward the chatter is at most 0.5 of arc (interval × rate), a deficit of 20·(0.05)²/2 ≈ 2.5e-2 at the far end of a swing — if the assertion fails at 1e-2, the test's interval is too coarse for its tolerance, not the code: set that test's `interval` to 2 (chatter 0.2 of arc, deficit 4e-3) and keep the production defaults.
- `off_freezes_the_world_and_later_ticks_change_nothing`: at tick 5 the human sees U below 16 (the wasteful robot lowered c and d by 0.5 for 0.05 of proxy) and sends OFF; `off_tick` is 5.

Then `cargo test -p sugarscape-core 2>&1 | tail -3` for the whole crate: the preset title test and the golden test are unaffected (no presets yet); the `model.rs` test that lists every kind passes with `"misaligned"` added. Expected: green.

- [ ] **Step 9: Commit**

```bash
git add crates/sugarscape-core/src/misaligned crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json
git commit -m "misaligned: configuration, series and the world

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
```

---
### Task 4: Frame, inspection, presets, titles, goldens, WASM and the identity receipts

**Files:**
- Replace: `crates/sugarscape-core/src/misaligned/view.rs`
- Create: `crates/sugarscape-core/src/misaligned/presets.rs`
- Modify: `crates/sugarscape-core/src/misaligned/mod.rs` (`mod presets; pub use presets::presets;`)
- Modify: `crates/sugarscape-core/src/presets.rs` (catalog: `out.extend(crate::misaligned::presets());` after the gridworlds line 1602)
- Modify: `crates/sugarscape-core/src/titles.rs` (`TITLES` length 450 → 458; eight entries after `gridworlds-island`)
- Modify: `crates/sugarscape-core/tests/golden.rs` (`MODEL_GOLDEN`: eight entries)
- Modify: `crates/sugarscape-wasm/tests/web.rs` (preset model check; a golden test)
- Modify: `web/src/episodes/test-fixtures/foraging-source-compatibility.json`, `web/src/episodes/test-fixtures/historical-source.ts`, `web/src/episodes/caching-parity.test.ts`

**Interfaces:**
- Consumes: Task 3's world.
- Produces: `view::{BAR, GAP, BAR_HEIGHT, STRIP, WIDE, TALL, size, render, inspect}`; `presets() -> Vec<ModelPreset>` with the eight ids of the spec; the one color mode `"bars"` (also `""`).

Frame geometry: four bars `BAR = 40` px wide with `GAP = 8` px between and around them (`WIDE = 4·40 + 5·8 = 200`), `BAR_HEIGHT = 120` rows of bars, a gap, a `STRIP = 16` row utility strip, a gap (`TALL = 120 + 8 + 16 + 8 = 152`). Bar i spans x ∈ [8 + 48 i, 48 + 48 i). The vertical scale runs from `lo = min(0, floor)` to `hi = shape.maximum()`; `y_of(v) = BAR_HEIGHT − 1 − round((v − lo)/(hi − lo)·(BAR_HEIGHT − 1))`, clamped. Colors: proxy attributes `[70, 130, 230]`, unmentioned `[235, 150, 60]`, frozen bars with every fourth row in `[20, 20, 20]`; floor line `INK` at `y_of(floor)`; start marker a hollow bracket (rows `y_of(start) ± 1` in INK across the bar, the middle row only at the bar's two edge columns); optimum tick `MARK = [240, 210, 70]` at `y_of(optimum)` and the row above. The strip spans x ∈ [8, 192) and rows [128, 144): its scale runs from `ulo = min(U₀, U_limit, U)` to `uhi = max(U*, U)` (span at least 1e-9); filled from the left to `x_of(U)` in the proxy color, in `[150, 150, 150]` when switched off and in `[230, 70, 70]` when U < U₀; a hollow INK column at `x_of(U₀)`, an INK column at `x_of(limit)`, a MARK column at `x_of(U*)`.

- [ ] **Step 1: Write the failing tests**

Add to `world.rs`'s tests module:

```rust
    #[test]
    fn frames_have_the_declared_size_and_draw_the_bars() {
        let mut w = world(MisalignedConfig::default());
        w.run(2);
        assert_eq!(w.size(), (200, 152));
        let mut buf = vec![];
        for mode in ["", "bars"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 200 * 152 * 4, "{mode}");
        }
        assert!(w.render("surface", "", &mut buf).is_err());
        w.render("bars", "", &mut buf).unwrap();
        // Bar c (index 2, x 104..144) is unmentioned: its column at mid-height is orange; bar a's
        // top rows are background because a ≈ 1 of 10.
        let at = |x: usize, y: usize| &buf[(y * 200 + x) * 4..(y * 200 + x) * 4 + 3];
        assert_eq!(at(124, 60), &[235, 150, 60]);
        assert_eq!(at(28, 115), &[70, 130, 230], "bar a is drawn from the bottom up (row 119 is the floor line)");
        assert_ne!(at(28, 10), &[70, 130, 230], "and is short");
        // The optimum tick on bar d (index 3) sits at y_of(5) = 119 − round(0.5 · 119) = 59 (and 58).
        assert_eq!(at(172, 59), &[240, 210, 70]);
        // The strip is filled from its left edge (x = 8) to the current utility, which lies right of x = 20.
        assert!((128..144).any(|y| at(20, y) == [70, 130, 230] || at(20, y) == [230, 70, 70] || at(20, y) == [150, 150, 150]));
    }

    #[test]
    fn inspection_reads_bars_gaps_and_the_strip() {
        let mut w = world(MisalignedConfig::default());
        w.run(1);
        let bar: serde_json::Value = serde_json::from_str(&w.inspect_json(124, 60).unwrap()).unwrap();
        assert_eq!((bar["panel"].as_str(), bar["index"].as_u64(), bar["name"].as_str(), bar["in_proxy"].as_bool()), (Some("attribute"), Some(2), Some("c"), Some(false)));
        assert!(bar["sensitivity"].as_f64().unwrap() > 0.0 && bar["rank"].as_u64().is_some());
        assert_eq!(bar["start"].as_f64(), Some(7.0));
        assert_eq!(bar["optimum"].as_f64(), Some(5.0));
        assert_eq!(bar["limit"].as_f64(), Some(0.0));
        let gap: serde_json::Value = serde_json::from_str(&w.inspect_json(50, 60).unwrap()).unwrap();
        assert_eq!(gap["panel"], "utility");
        let strip: serde_json::Value = serde_json::from_str(&w.inspect_json(100, 135).unwrap()).unwrap();
        assert_eq!((strip["panel"].as_str(), strip["utility_start"].as_f64(), strip["optimum"].as_f64()), (Some("utility"), Some(16.0), Some(20.0)));
        assert_eq!(strip["switched_off"], false);
        assert!(strip["outcome"]["proxy_history"].is_array());
        assert!(w.inspect_json(200, 0).is_err() && w.inspect_json(0, 152).is_err());
    }
```

Add to `presets.rs` (new file) the tests module:

```rust
//! Eight source-labeled presets; each description ends with the survey's measured result once
//! milestone 43's survey has run.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelConfig;

    #[test]
    fn eight_presets_cover_both_worlds_three_robots_and_three_humans() {
        let all = presets();
        assert_eq!(all.len(), 8);
        let ids: Vec<&str> = all.iter().map(|p| p.id).collect();
        for id in ["misaligned-recommender-proxy", "misaligned-recommender-gradient", "misaligned-recommender-impact", "misaligned-recommender-interactive", "misaligned-recommender-slow-human", "misaligned-recommender-optimal", "misaligned-recommender-wasteful", "misaligned-logarithmic-floor"] {
            assert!(ids.contains(&id), "{id}");
        }
        for p in &all {
            let ModelConfig::Misaligned(c) = &p.config else { panic!("{}", p.id) };
            assert!(c.validate().is_ok(), "{}", p.id);
            assert!(p.name.len() > 10 && !p.name.contains("Flump"));
        }
        let log = all.iter().find(|p| p.id == "misaligned-logarithmic-floor").unwrap();
        let ModelConfig::Misaligned(c) = &log.config else { unreachable!() };
        assert_eq!((c.world, c.floor, c.robot), (World::Logarithmic, 0.01, Robot::Gradient));
        let slow = all.iter().find(|p| p.id == "misaligned-recommender-slow-human").unwrap();
        let ModelConfig::Misaligned(c) = &slow.config else { unreachable!() };
        assert_eq!((c.human, c.interval, c.stop_on_loss), (Human::Top, 200, true));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core misaligned:: 2>&1 | tail -5`. Expected: the two world tests fail (size `(1, 1)`, inspect `Err`); the presets test fails to compile until the module exists.

- [ ] **Step 3: Write `view.rs`**

Replace the stub:

```rust
//! Frame and inspection: four bars, one per attribute, scaled to the attribute's feasible
//! maximum, with the floor, the start and the human optimum marked on each, and a strip beneath
//! that places the current utility between the start, the current proxy's limit and the optimum.
use super::{
    world::MisalignedWorld,
    worlds::{L, NAMES},
};
use crate::opinions::Canvas;
use serde_json::json;

pub const BAR: usize = 40;
pub const GAP: usize = 8;
pub const BAR_HEIGHT: usize = 120;
pub const STRIP: usize = 16;
pub const WIDE: usize = L * BAR + (L + 1) * GAP;
pub const TALL: usize = BAR_HEIGHT + GAP + STRIP + GAP;
const PROXY: [u8; 3] = [70, 130, 230];
const UNMENTIONED: [u8; 3] = [235, 150, 60];
const LOSS: [u8; 3] = [230, 70, 70];
const OFF: [u8; 3] = [150, 150, 150];
const INK: [u8; 3] = [20, 20, 20];
const MARK: [u8; 3] = [240, 210, 70];

pub fn size(_w: &MisalignedWorld) -> (u32, u32) {
    (WIDE as u32, TALL as u32)
}

fn bar_x(i: usize) -> usize {
    GAP + i * (BAR + GAP)
}

/// Which bar covers column x, if any.
fn bar_at(x: usize) -> Option<usize> {
    (0..L).find(|&i| (bar_x(i)..bar_x(i) + BAR).contains(&x))
}

struct Scale {
    lo: f64,
    hi: f64,
}

impl Scale {
    fn y(&self, v: f64) -> usize {
        let f = ((v - self.lo) / (self.hi - self.lo)).clamp(0.0, 1.0);
        BAR_HEIGHT - 1 - (f * (BAR_HEIGHT - 1) as f64).round() as usize
    }
}

fn scale(w: &MisalignedWorld) -> Scale {
    let shape = w.shape();
    Scale { lo: shape.floor.min(0.0), hi: shape.maximum().max(shape.floor.min(0.0) + 1e-9) }
}

pub fn render(w: &MisalignedWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "bars"].contains(&mode) {
        return Err(format!("unknown color mode {mode:?} (expected bars)"));
    }
    let mut canvas = Canvas { buf, wide: 0 };
    canvas.clear(WIDE, TALL);
    let shape = w.shape();
    let sc = scale(w);
    let state = w.state();
    let (optimum, u_star) = shape.optimum();
    let (_, u_limit) = shape.fixed_proxy_limit(w.proxy());
    for i in 0..L {
        let x0 = bar_x(i);
        let color = if w.proxy()[i] { PROXY } else { UNMENTIONED };
        let top = sc.y(state[i]);
        for y in top..BAR_HEIGHT {
            let row = if w.frozen()[i] && y % 4 == 0 { INK } else { color };
            for x in x0..x0 + BAR {
                canvas.put(x, y, row);
            }
        }
        let floor_y = sc.y(shape.floor);
        for x in x0..x0 + BAR {
            canvas.put(x, floor_y, INK);
        }
        let s = sc.y(w.start()[i]);
        for x in x0..x0 + BAR {
            if s > 0 {
                canvas.put(x, s - 1, INK);
            }
            if s + 1 < BAR_HEIGHT {
                canvas.put(x, s + 1, INK);
            }
        }
        canvas.put(x0, s, INK);
        canvas.put(x0 + BAR - 1, s, INK);
        let o = sc.y(optimum[i]);
        for x in x0..x0 + BAR {
            canvas.put(x, o, MARK);
            if o > 0 {
                canvas.put(x, o - 1, MARK);
            }
        }
    }
    // The strip.
    let u = shape.utility(&state);
    let u0 = shape.utility(&w.start());
    let ulo = u0.min(u_limit).min(u);
    let uhi = u_star.max(u);
    let span = (uhi - ulo).max(1e-9);
    let (x0, x1) = (GAP, WIDE - GAP);
    let x_of = |v: f64| (x0 as f64 + ((v - ulo) / span).clamp(0.0, 1.0) * (x1 - x0 - 1) as f64).round() as usize;
    let fill = if w.switched_off() { OFF } else if u < u0 { LOSS } else { PROXY };
    let top = BAR_HEIGHT + GAP;
    for y in top..top + STRIP {
        for x in x0..=x_of(u) {
            canvas.put(x, y, fill);
        }
        canvas.put(x_of(u_limit), y, INK);
        canvas.put(x_of(u_star), y, MARK);
        if y == top || y + 1 == top + STRIP {
            canvas.put(x_of(u0), y, INK);
        }
    }
    canvas.put(x_of(u0), top + STRIP / 2, INK);
    Ok(())
}

pub fn inspect(w: &MisalignedWorld, x: u32, y: u32) -> Result<String, String> {
    if x as usize >= WIDE || y as usize >= TALL {
        return Err(format!("({x}, {y}) is not in the frame"));
    }
    let (x, y) = (x as usize, y as usize);
    let shape = w.shape();
    let state = w.state();
    let sens = shape.sensitivities(&state);
    let outcome = w.outcome();
    if y < BAR_HEIGHT {
        if let Some(i) = bar_at(x) {
            let rank = 1 + (0..L).filter(|&j| sens[j] > sens[i] || (sens[j] == sens[i] && j < i)).count();
            return Ok(json!({
                "panel": "attribute", "index": i, "name": NAMES[i], "x": x, "y": y,
                "value": state[i], "in_proxy": w.proxy()[i], "frozen": w.frozen()[i],
                "sensitivity": sens[i], "rank": rank, "start": w.start()[i],
                "limit": outcome.limit.0[i], "optimum": outcome.optimum.0[i], "floor": shape.floor,
                "tick": w.tick, "outcome": outcome, "agent": null,
            })
            .to_string());
        }
    }
    Ok(json!({
        "panel": "utility", "x": x, "y": y,
        "utility": outcome.utility, "utility_start": outcome.utility_start, "optimum": outcome.optimum.1,
        "limit": outcome.limit.1, "gain": outcome.utility - outcome.utility_start,
        "interactions": outcome.interactions, "switched_off": w.switched_off(),
        "tick": w.tick, "outcome": outcome, "agent": null,
    })
    .to_string())
}
```

- [ ] **Step 4: Write `presets.rs`**

Above its tests:

```rust
use super::config::*;
use crate::{model::ModelConfig, presets::ModelPreset};

const PAPER: &str = "Zhuang & Hadfield-Menell 2020, Consequences of misaligned AI, NeurIPS (arXiv 2102.03896)";

fn gradient() -> MisalignedConfig {
    MisalignedConfig {
        robot: Robot::Gradient,
        rate: 0.1,
        ..Default::default()
    }
}

pub fn presets() -> Vec<ModelPreset> {
    let p = |id, name, description, config| ModelPreset {
        id,
        name,
        source: PAPER,
        description,
        config: ModelConfig::Misaligned(config),
    };
    vec![
        p("misaligned-recommender-proxy", "The proxy keeps climbing after what the human wanted has peaked",
          "The paper's recommender example (§2.2, Figure 2): four attributes the human values equally, a budget of attention (a² + b² + c² + d² ≤ 100), and a robot told to raise ad revenue and engagement quality (a, b) only. The robot is a random-search climber, our reconstruction of the paper's black box: a Gaussian step on every attribute, kept when it is feasible and raises the proxy. The start (1, 1, 7, 7), utility 16, is ours.",
          MisalignedConfig::default()),
        p("misaligned-recommender-gradient", "An efficient optimizer walks through the best world on its way to a worse one",
          "The same proxy under steepest ascent along the sphere, the efficient robot of §4: utility rises to the human optimum of 20 at (5, 5, 5, 5) and then falls to the proxy's limit of 10√2 ≈ 14.14 with c and d at the floor (Theorem 1).",
          gradient()),
        p("misaligned-recommender-impact", "A robot that leaves the unmentioned alone can only gain",
          "Proposition 2: the unmentioned attributes are frozen at their starting values, so gains in the proxy are gains in utility. From (1, 1, 6, 6), inside the sphere, a and b rise to √13 each and utility to 12 + 2√13 ≈ 19.21; from the default start on the sphere there is nothing to gain.",
          MisalignedConfig { impact_minimizing: true, start_c: 6.0, start_d: 6.0, ..gradient() }),
        p("misaligned-recommender-interactive", "A human who re-specifies often keeps the robot near the best world",
          "Theorem 3 and Proposition 4: every 10 ticks the human gives the robot the two most sensitive attributes as its proxy; the efficient robot moves at most 0.01 per tick. Utility rises to the optimum and stays within one interval's travel of it.",
          MisalignedConfig { human: Human::Top, interval: 10, rate: 0.01, ..gradient() }),
        p("misaligned-recommender-slow-human", "A human who checks too rarely gets Midas anyway",
          "The same human every 200 ticks with a robot moving 0.1 per tick: between two interactions the robot runs the whole path to the proxy's limit, and the human, seeing utility fall, switches it off.",
          MisalignedConfig { human: Human::Top, interval: 200, stop_on_loss: true, ..gradient() }),
        p("misaligned-recommender-optimal", "Moving from the least-valued attribute to the most reaches the optimum",
          "Proposition 5: the human names the most and the least sensitive attribute, the robot moves resources from the second to the first and leaves the other two alone; repeated, this converges to the human optimum of 20.",
          MisalignedConfig { human: Human::Extremes, impact_minimizing: true, ..gradient() }),
        p("misaligned-recommender-wasteful", "A robot that destroys what it was not told about should be switched off at once",
          "Proposition 3: a robot that lowers every unmentioned attribute by 0.1 a tick for 0.01 of proxy. Utility falls from the first tick, so the human's maxmin policy is OFF before it starts.",
          MisalignedConfig { robot: Robot::Wasteful, ..Default::default() }),
        p("misaligned-logarithmic-floor", "With no floor under the unmentioned, the loss has no bottom",
          "Our logarithmic world (utility ln a + ln b + ln c + ln d on a budget of 40), where every hypothesis of Theorem 2 holds: with the floor at 0.01 the fixed proxy {a, b} drives c and d to it and utility to 2 ln 19.99 + 2 ln 0.01 ≈ −3.22, against 4 ln 10 ≈ 9.21 at the optimum; lower the floor and the loss grows without bound.",
          MisalignedConfig { floor: 0.01, robot: Robot::Gradient, rate: 0.1, ..logarithmic_defaults() }),
    ]
}
```

- [ ] **Step 5: Titles, catalog and goldens**

- `crates/sugarscape-core/src/presets.rs`, in `catalog()`: `out.extend(crate::misaligned::presets());` right after the gridworlds line.
- `crates/sugarscape-core/src/titles.rs`: change `450` to `458` and add after `gridworlds-island`'s line the eight `(id, title)` pairs with exactly the names above.
- `crates/sugarscape-core/tests/golden.rs`, `MODEL_GOLDEN`: run `cargo test -p sugarscape-core --test golden print_golden -- --ignored --nocapture 2>&1 | grep misaligned-` and paste the eight lines after the gridworlds block with the comment `// Consequences of misaligned AI: 200 ticks, seed 1.`

Run: `cargo test -p sugarscape-core 2>&1 | tail -3`. Expected: the whole crate green.

- [ ] **Step 6: WASM tests**

In `crates/sugarscape-wasm/tests/web.rs`:

- in `presets_list_every_model_with_sugarscape_configs_untagged`, after the gridworlds assertion (line 854): `assert_eq!(model("misaligned-recommender-proxy").as_deref(), Some("misaligned"));`
- append, with the eight fingerprints from `golden.rs` as hex strings (zero-padded to 16 digits, as `gridworlds-tomato`'s `0x0e88…` shows):

```rust
#[wasm_bindgen_test]
fn misaligned_presets_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: f64 sums, square roots, the portable
    // logarithm and the shared RNG only, so WASM repeats the native fingerprints.
    for (id, expected) in [
        ("misaligned-recommender-proxy", "0x…"),
        // … all eight …
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        sim.step(200);
        assert_eq!(sim.fingerprint(), expected, "{id}");
        assert_ne!(sim.render("", "sugar").unwrap(), 0);
    }
    let sim = Sim::new(&preset_json("misaligned-recommender-proxy"), 1, JsValue::NULL).unwrap();
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(124, 60).unwrap()).unwrap();
    assert_eq!(view["name"], "c");
}
```

Run: `wasm-pack test --node crates/sugarscape-wasm 2>&1 | tail -5`. Expected: all pass. A difference between a native and a WASM fingerprint means a non-portable float path; the only candidates in this kind are `sqrt` (IEEE-exact on both) and `portable::ln`.

- [ ] **Step 7: The identity receipts**

The vitest suite pins five core files. Update:

1. `web/src/episodes/test-fixtures/historical-source.ts`: append to the `reviewedAdditions` string, before its closing `\n'`: `\n/// Consequences of misaligned AI: a proxy on a subset of the attributes (milestone 43).\npub mod misaligned;` so the string ends `…(milestone 40).\npub mod gridworlds;\n\n/// Consequences of misaligned AI: a proxy on a subset of the attributes (milestone 43).\npub mod misaligned;\n'`. This must be byte-identical to what Task 1 appended to `lib.rs`. Update the message string to say `discovery/Gate/Key/assistance/gridworlds/misaligned`.
2. `web/src/episodes/caching-parity.test.ts`: rename the test to `…Gate, Key, assistance, gridworlds and misaligned additions…` and add two negative cases after the gridworlds ones: `live.replace('pub mod misaligned;', 'pub mod other_misaligned;'),` and `live.replace('\npub mod misaligned;\n', '\n'),`.
3. `web/src/episodes/test-fixtures/foraging-source-compatibility.json`: for the four receipts whose `path` is `crates/sugarscape-core/src/{model,presets,sweep,titles}.rs`, set `reviewed_sha256` to the live file's sha256. The scratchpad script `receipts.py` from milestone 40 does this; copy it to the scratchpad with `root` set to this worktree and run it with `python3 -I`. Do this **after** the last edit of each file in this branch (Task 5 edits `sweep.rs`, Task 7 edits `presets.rs` descriptions: refresh again then). The `historical_sha256` and the fixture files do not change.

Run: `cd web && npx vitest run src/episodes/caching-parity.test.ts src/episodes/spatial-parity.test.ts 2>&1 | tail -5`. Expected: pass.

- [ ] **Step 8: Commit**

```bash
git add crates web/src/episodes
git commit -m "misaligned: frame, inspection, presets, titles, goldens and the identity receipts

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
```

---
### Task 5: Built-in sweeps and the CI smoke test

**Files:**
- Create: `sweeps/misaligned-figure2-pairs.json`, `sweeps/misaligned-interval.json`, `sweeps/misaligned-floor.json`
- Modify: `crates/sugarscape-core/src/sweep.rs` (`BUILTINS` 233 → 236; the id list in its tests near line 3064)
- Modify: `crates/sugarscape-wasm/tests/web.rs` (`builtins_and_series_names_are_listed`: three ids after `gridworlds-q-init-whisky`)
- Modify: `survey/war1_study/fixtures/core-literal-inputs-v1.txt` (three lines), `survey/war1_study/test_registration.py` (four counts 247 → 250), `survey/tests/war1_study_cli.rs` (line 207: 247 → 250)
- Create: `crates/sugarscape-core/tests/misaligned_smoke.rs`

**Interfaces:**
- Consumes: Task 3's config paths and series names; Task 4's presets.
- Produces: the three builtin ids above.

- [ ] **Step 1: Write the failing smoke test**

`crates/sugarscape-core/tests/misaligned_smoke.rs`:

```rust
//! Every misaligned preset runs 2,000 ticks at seed 1 with finite series and a feasible state:
//! CI's smoke test for milestone 43 (it scores nothing; the survey scores).
use sugarscape_core::{
    model::{ModelConfig, ModelKind, ModelWorld},
    presets, sweep,
};

#[test]
fn every_preset_runs_two_thousand_ticks_with_finite_series() {
    let mut seen = 0;
    for p in presets::catalog() {
        let ModelConfig::Misaligned(_) = &p.config else { continue };
        seen += 1;
        let mut w = ModelWorld::new(p.config.clone(), 1).unwrap();
        w.model_mut().run(2000);
        let m = w.model();
        for name in ["utility", "proxy", "gain", "a", "b", "c", "d", "optimum_gap", "limit_gap", "interactions", "switched_off"] {
            assert!(m.latest_value(name).unwrap().is_finite(), "{}: {name}", p.id);
        }
        assert!(m.latest_value("optimum_gap").unwrap() >= -1e-9, "{}: never above the optimum", p.id);
        assert_eq!(w.kind(), ModelKind::Misaligned);
    }
    assert_eq!(seen, 8);
}

#[test]
fn the_builtin_sweeps_parse_and_name_real_paths() {
    for id in ["misaligned-figure2-pairs", "misaligned-interval", "misaligned-floor"] {
        let s = sweep::builtin(id).unwrap_or_else(|| panic!("no builtin {id}"));
        assert!(s.ticks >= 1000, "{id}");
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p sugarscape-core --test misaligned_smoke 2>&1 | tail -5`. Expected: the sweeps test fails on the first missing id; the preset test passes.

- [ ] **Step 3: Write the sweeps**

`sweeps/misaligned-figure2-pairs.json`:

```json
{
  "name": "Misaligned AI: Figure 2, every proxy pair",
  "description": "Zhuang & Hadfield-Menell 2020, Figure 2 (right): the utility gained over the start after 1,000 ticks of the random-search climber, one line per pair of proxy attributes, against the climber's proposal step. Every pair ends below the start (its limit is 10√2 ≈ 14.14 against 16) however it climbs; the start (1, 1, 7, 7) and the climber are our reconstruction.",
  "base": {
    "config": {
      "model": "misaligned",
      "robot": "climber",
      "moves_per_tick": 1
    }
  },
  "x": {
    "label": "Proposal step σ",
    "path": "step",
    "values": [0.25, 0.5, 1, 2]
  },
  "series": {
    "label": "Proxy",
    "values": [
      { "at": 0, "name": "a, b", "set": { "proxy_a": true, "proxy_b": true, "proxy_c": false, "proxy_d": false } },
      { "at": 1, "name": "a, c", "set": { "proxy_a": true, "proxy_b": false, "proxy_c": true, "proxy_d": false } },
      { "at": 2, "name": "a, d", "set": { "proxy_a": true, "proxy_b": false, "proxy_c": false, "proxy_d": true } },
      { "at": 3, "name": "b, c", "set": { "proxy_a": false, "proxy_b": true, "proxy_c": true, "proxy_d": false } },
      { "at": 4, "name": "b, d", "set": { "proxy_a": false, "proxy_b": true, "proxy_c": false, "proxy_d": true } },
      { "at": 5, "name": "c, d", "set": { "proxy_a": false, "proxy_b": false, "proxy_c": true, "proxy_d": true } }
    ]
  },
  "seeds": { "from": 1, "count": 20 },
  "ticks": 1000,
  "metric": { "kind": "final", "series": "gain" }
}
```

`sweeps/misaligned-interval.json`:

```json
{
  "name": "Misaligned AI: how often the human must act",
  "description": "Theorem 3 and Proposition 4: the efficient robot (steepest ascent, 0.1 per tick) under a human who re-specifies the proxy as the two most sensitive attributes every δ ticks; the utility gained over the start after 2,000 ticks, with and without switching off on a loss. Small δ keeps the robot inside the neighborhood where Theorem 3 guarantees improvement; past the optimum a large δ lets it run to the proxy's limit.",
  "base": {
    "config": {
      "model": "misaligned",
      "robot": "gradient",
      "rate": 0.1,
      "human": "top",
      "proxy_size": 2
    }
  },
  "x": {
    "label": "Interval δ (ticks)",
    "path": "interval",
    "values": [1, 2, 5, 10, 20, 50, 100, 200]
  },
  "series": {
    "label": "On a loss",
    "values": [
      { "at": 0, "name": "Keep going", "set": { "stop_on_loss": false } },
      { "at": 1, "name": "Switch off", "set": { "stop_on_loss": true } }
    ]
  },
  "seeds": { "from": 1, "count": 1 },
  "ticks": 2000,
  "metric": { "kind": "final", "series": "gain" }
}
```

`sweeps/misaligned-floor.json`:

```json
{
  "name": "Misaligned AI: the floor under the unmentioned attributes",
  "description": "Theorem 2: the utility the efficient robot leaves after 5,000 ticks of optimizing the fixed proxy {a, b}, against the floor under every attribute. In our logarithmic world (budget 40, start 4, 4, 16, 16) the limit falls without bound as the floor falls, the theorem's 'arbitrarily low'; in the paper's recommender example the floor below zero does not matter, because the sum of squares stops increasing there, and the limit stays near 14.14.",
  "base": {
    "config": {
      "model": "misaligned",
      "robot": "gradient",
      "rate": 0.1
    }
  },
  "x": {
    "label": "Floor",
    "path": "floor",
    "values": [1, 0.3, 0.1, 0.03, 0.01, 0.003, 0.001]
  },
  "series": {
    "label": "World",
    "values": [
      { "at": 0, "name": "Logarithmic (ours)", "set": { "world": "logarithmic", "capacity": 40, "start_a": 4, "start_b": 4, "start_c": 16, "start_d": 16 } },
      { "at": 1, "name": "Recommender (the paper's)", "set": { "world": "recommender", "capacity": 100, "start_a": 1, "start_b": 1, "start_c": 7, "start_d": 7 } }
    ]
  },
  "seeds": { "from": 1, "count": 1 },
  "ticks": 5000,
  "metric": { "kind": "final", "series": "utility" }
}
```

- [ ] **Step 4: Register the builtins and the literal inventory**

- `crates/sugarscape-core/src/sweep.rs`: `const BUILTINS: [Builtin; 233]` → `236`, three `Builtin { id: "…", json: include_str!("../../../sweeps/….json") }` entries after `gridworlds-q-init-whisky`, before `polarity-predators`; the same three ids in the test list near line 3064 and in `web.rs`'s `builtins_and_series_names_are_listed` after the gridworlds ids.
- `survey/war1_study/fixtures/core-literal-inputs-v1.txt`: insert `sweeps/misaligned-figure2-pairs.json`, `sweeps/misaligned-floor.json`, `sweeps/misaligned-interval.json` (alphabetical) after `sweeps/mg-strategies.json` (line 194) and before `sweeps/n-goods-carrying-capacity.json`.
- `survey/war1_study/test_registration.py`: the four `247` → `250` (lines 72, 93, 135, 165); `survey/tests/war1_study_cli.rs` line 207: `==247` → `==250`.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p sugarscape-core --test misaligned_smoke 2>&1 | tail -3`, `cargo test -p sugarscape-core sweep 2>&1 | tail -3`, `python3 -m unittest discover -s survey/war1_study -p 'test_*.py' 2>&1 | tail -3` (58 pass), and `cargo test --manifest-path survey/Cargo.toml --locked --features war-studies --bin war1-study --test war1_study_cli 2>&1 | tail -3` (8 pass). Expected: pass. Run one sweep end to end at a small size to see it works: `cargo run --release -p sugarscape-cli -- sweep --help` for the flags, then the interval sweep with two x values.

- [ ] **Step 6: Refresh the `sweep.rs` receipt**

Run the scratchpad `receipts.py` (Task 4 Step 7) again so `sweep.rs`'s `reviewed_sha256` is current.

- [ ] **Step 7: Commit**

```bash
git add sweeps crates/sugarscape-core/src/sweep.rs crates/sugarscape-core/tests/misaligned_smoke.rs crates/sugarscape-wasm/tests/web.rs survey/war1_study survey/tests/war1_study_cli.rs web/src/episodes/test-fixtures/foraging-source-compatibility.json
git commit -m "misaligned: built-in sweeps and the CI smoke test

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
```

---

### Task 6: The page

**Files:**
- Modify: `web/src/types.ts` (`ModelKind`, `ModelConfig`, `ModelStats`, `AnyInspection`, `ColorMode`; four new interfaces after the gridworlds block)
- Modify: `web/src/models.ts` (`MODELS`, `MODEL_LABELS`, `modelOf`, `isMisalignedView`, `COLOR_MODES`, `MODEL_OVERLAYS`)
- Modify: `web/src/ui/series-data.ts` (`MODEL_CHARTS.misaligned`)
- Create: `web/src/misaligned.ts`, `web/src/misaligned.test.ts`
- Modify: `web/src/ui/grid-view.ts`, `web/src/ui/inspect-panel.ts`, `web/src/experiments/form.ts`, `web/src/compare-presets.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the series names, inspection JSON keys and config paths from Tasks 3–4.
- Produces: `misalignedRows(view: MisalignedInspection): [string, string][]`, `misalignedLegend(config: MisalignedConfig, mode: ColorMode): { status: string; items: LegendItem[] }`.

- [ ] **Step 1: Write the failing test**

`web/src/misaligned.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { misalignedLegend, misalignedRows } from './misaligned';
import type { MisalignedConfig, MisalignedInspection, MisalignedOutcome } from './types';

const config: MisalignedConfig = {
  model: 'misaligned', world: 'recommender', capacity: 100, floor: 0, start_a: 1, start_b: 1, start_c: 7, start_d: 7,
  proxy_a: true, proxy_b: true, proxy_c: false, proxy_d: false, robot: 'climber', step: 1, rate: 0.1, waste: 0.1, moves_per_tick: 1,
  human: 'fixed', proxy_size: 2, interval: 10, impact_minimizing: false, stop_on_loss: false,
};

const outcome: MisalignedOutcome = {
  config, seed: 1, tick: 40, state: [3.2, 3.1, 6.4, 6.3], start: [1, 1, 7, 7], utility_start: 16, utility: 19,
  optimum: [[5, 5, 5, 5], 20], limit: [[7.0711, 7.0711, 0, 0], 14.1421], impact_limit: [[1, 1, 7, 7], 16],
  proxy: [true, true, false, false], frozen: [false, false, false, false], reference: [1, 1, 7, 7],
  proxy_history: [{ tick: 0, proxy: [true, true, false, false] }], interactions: 5, off_tick: null,
};

const bar: MisalignedInspection = {
  panel: 'attribute', index: 2, name: 'c', x: 124, y: 60, value: 6.4, in_proxy: false, frozen: false, sensitivity: 0.078, rank: 3,
  start: 7, limit: 0, optimum: 5, floor: 0, tick: 40, outcome, agent: null,
};

const strip: MisalignedInspection = {
  panel: 'utility', x: 100, y: 135, utility: 19, utility_start: 16, optimum: 20, limit: 14.1421, gain: 3, interactions: 5, switched_off: false,
  tick: 40, outcome, agent: null,
};

describe('misaligned page helpers', () => {
  it('rows name the attribute, its place in the proxy and its marks', () => {
    const rows = misalignedRows(bar);
    expect(rows[0]).toEqual(['Attribute', 'c · content diversity · 6.400 · unmentioned']);
    expect(rows.find(([k]) => k === 'Sensitivity')?.[1]).toBe('0.078 · rank 3 of 4');
    expect(rows.find(([k]) => k === 'Marks')?.[1]).toBe('start 7.000 · floor 0.000 · limit under this proxy 0.000 · optimum 5.000');
    expect(rows.find(([k]) => k === 'Utility')?.[1]).toContain('19.000 · start 16.000 · optimum 20.000 · limit 14.142 · gain +3.000');
  });

  it('rows for the strip report the run', () => {
    const rows = misalignedRows(strip);
    expect(rows[0][0]).toBe('Utility');
    expect(rows.find(([k]) => k === 'Human')?.[1]).toBe('5 interactions · proxy a, b · robot running');
  });

  it('legend names the robot, the human and the colors', () => {
    const legend = misalignedLegend(config, 'bars');
    expect(legend.status).toContain('Random-search climber');
    expect(legend.status).toContain('fixed proxy');
    expect(legend.items.map((i) => i.label)).toEqual(['In the proxy', 'Unmentioned', 'Frozen (impact minimizing)', 'Optimum · gold tick', 'Start · hollow bracket']);
    const log = misalignedLegend({ ...config, world: 'logarithmic', human: 'extremes', robot: 'gradient' }, 'bars');
    expect(log.status).toContain('ln a + ln b + ln c + ln d');
    expect(log.status).toContain('most and least sensitive');
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cd web && npx vitest run src/misaligned.test.ts 2>&1 | tail -5`. Expected: cannot resolve `./misaligned`.

- [ ] **Step 3: Types**

In `web/src/types.ts`:

- `ModelKind`: append `| 'misaligned'`.
- `ModelConfig`: append `| MisalignedConfig`. `ModelStats`: `| MisalignedStats`. `AnyInspection`: `| MisalignedInspection`.
- `ColorMode`: append `| 'bars'` if it is not already a member.
- After the gridworlds interfaces add:

```ts
/** Consequences of misaligned AI (Zhuang & Hadfield-Menell 2020); the world, capacity, floor and start apply at reset. */
export interface MisalignedConfig {
  model: 'misaligned'; world: 'recommender' | 'logarithmic'; capacity: number; floor: number;
  start_a: number; start_b: number; start_c: number; start_d: number;
  proxy_a: boolean; proxy_b: boolean; proxy_c: boolean; proxy_d: boolean;
  robot: 'climber' | 'gradient' | 'wasteful'; step: number; rate: number; waste: number; moves_per_tick: number;
  human: 'fixed' | 'top' | 'extremes'; proxy_size: number; interval: number; impact_minimizing: boolean; stop_on_loss: boolean;
}
export interface MisalignedStats {
  tick: number; utility: number | null; proxy: number | null; gain: number | null; a: number | null; b: number | null; c: number | null; d: number | null;
  sensitivity_a: number | null; sensitivity_b: number | null; sensitivity_c: number | null; sensitivity_d: number | null;
  optimum_gap: number | null; limit_gap: number | null; interactions: number | null; switched_off: number | null; accepted: number | null;
}
export type MisalignedSet = [boolean, boolean, boolean, boolean];
export type MisalignedState = [number, number, number, number];
export interface MisalignedOutcome {
  config: MisalignedConfig; seed: number; tick: number; state: MisalignedState; start: MisalignedState; utility_start: number; utility: number;
  optimum: [MisalignedState, number]; limit: [MisalignedState, number]; impact_limit: [MisalignedState, number];
  proxy: MisalignedSet; frozen: MisalignedSet; reference: MisalignedState; proxy_history: { tick: number; proxy: MisalignedSet }[];
  interactions: number; off_tick: number | null;
}
export interface MisalignedAttributeInspection {
  panel: 'attribute'; index: number; name: 'a' | 'b' | 'c' | 'd'; x: number; y: number; value: number; in_proxy: boolean; frozen: boolean;
  sensitivity: number; rank: number; start: number; limit: number; optimum: number; floor: number; tick: number; outcome: MisalignedOutcome; agent: null;
}
export interface MisalignedUtilityInspection {
  panel: 'utility'; x: number; y: number; utility: number; utility_start: number; optimum: number; limit: number; gain: number;
  interactions: number; switched_off: boolean; tick: number; outcome: MisalignedOutcome; agent: null;
}
export type MisalignedInspection = MisalignedAttributeInspection | MisalignedUtilityInspection;
```

- [ ] **Step 4: `models.ts`**

- `MODELS`: append `'misaligned'`.
- `MODEL_LABELS`: `misaligned: 'Misaligned AI',`.
- `modelOf`: add `|| tag === 'misaligned'` to the chain.
- After `isGridworldsView`:

```ts
/** A misaligned frame: one attribute bar or the utility strip. */
export function isMisalignedView(v: AnyInspection): v is MisalignedInspection {
  return 'panel' in v && (v.panel === 'attribute' || v.panel === 'utility') && 'outcome' in v && 'utility_start' in (v as { outcome: object }).outcome;
}
```

- `COLOR_MODES`: `misaligned: [['bars', 'Attributes and utility']],`.
- `MODEL_OVERLAYS`: `misaligned: [],`.
- Import `MisalignedInspection` in the type import list.

- [ ] **Step 5: `misaligned.ts`**

```ts
import type { ColorMode, MisalignedConfig, MisalignedInspection, MisalignedSet } from './types';
import type { LegendItem } from './ui/legend';

const fmt = (v: number | null, digits = 3): string => (v === null || Number.isNaN(v) ? '—' : v.toFixed(digits).replace('-', '−'));
const signed = (v: number): string => (v >= 0 ? `+${fmt(v)}` : fmt(v));
const NAMES = ['a', 'b', 'c', 'd'] as const;
const LONG: Record<(typeof NAMES)[number], string> = { a: 'ad revenue', b: 'engagement quality', c: 'content diversity', d: 'community well-being' };
const ROBOT = { climber: 'Random-search climber (the black box, ours)', gradient: 'Gradient flow (the efficient robot)', wasteful: 'Wasteful robot (Proposition 3)' } as const;
const HUMAN = { fixed: 'fixed proxy', top: 'most sensitive attributes every interval (Proposition 4)', extremes: 'most and least sensitive every interval (Proposition 5)' } as const;

const setNames = (set: MisalignedSet): string => NAMES.filter((_, i) => set[i]).join(', ') || 'none';

/** Inspect rows: an attribute bar or the utility strip. */
export function misalignedRows(view: MisalignedInspection): [string, string][] {
  const o = view.outcome;
  const utility: [string, string] = ['Utility', `${fmt(o.utility)} · start ${fmt(o.utility_start)} · optimum ${fmt(o.optimum[1])} · limit ${fmt(o.limit[1])} · gain ${signed(o.utility - o.utility_start)}`];
  const human: [string, string] = ['Human', `${o.interactions} interaction${o.interactions === 1 ? '' : 's'} · proxy ${setNames(o.proxy)} · ${o.off_tick === null ? 'robot running' : `switched off at tick ${o.off_tick}`}`];
  if (view.panel === 'attribute') {
    return [
      ['Attribute', `${view.name} · ${LONG[view.name]} · ${fmt(view.value)} · ${view.in_proxy ? 'in the proxy' : 'unmentioned'}${view.frozen ? ' · frozen' : ''}`],
      ['Sensitivity', `${fmt(view.sensitivity)} · rank ${view.rank} of 4`],
      ['Marks', `start ${fmt(view.start)} · floor ${fmt(view.floor)} · limit under this proxy ${fmt(view.limit)} · optimum ${fmt(view.optimum)}`],
      utility,
      human,
    ];
  }
  return [utility, human, ['Proxy history', o.proxy_history.map((h) => `tick ${h.tick}: ${setNames(h.proxy)}`).join(' · ')]];
}

/** The frame's status line and legend. */
export function misalignedLegend(config: MisalignedConfig, _mode: ColorMode): { status: string; items: LegendItem[] } {
  const world = config.world === 'recommender'
    ? `utility a + b + c + d on a² + b² + c² + d² ≤ ${config.capacity}`
    : `utility ln a + ln b + ln c + ln d on a + b + c + d ≤ ${config.capacity}`;
  return {
    status: `${ROBOT[config.robot]} · human: ${HUMAN[config.human]}${config.impact_minimizing ? ', impact minimizing' : ''}${config.stop_on_loss ? ', switches off on a loss' : ''} · ${world} · floor ${config.floor} · bars: a, b, c, d · strip: utility between the start, the proxy's limit and the optimum`,
    items: [
      { label: 'In the proxy', mark: { kind: 'swatch', color: '#4682e6' } },
      { label: 'Unmentioned', mark: { kind: 'swatch', color: '#eb963c' } },
      { label: 'Frozen (impact minimizing)', mark: { kind: 'swatch', color: '#141414' } },
      { label: 'Optimum · gold tick', mark: { kind: 'swatch', color: '#f0d246' } },
      { label: 'Start · hollow bracket', mark: { kind: 'swatch', color: '#141414' } },
    ],
  };
}
```

- [ ] **Step 6: Charts, legend, inspect rows, sweep form, Compare, goldens**

- `web/src/ui/series-data.ts`, `MODEL_CHARTS`: add after the gridworlds entry

```ts
  misaligned: [
    { title: 'Utility and proxy', lines: [{ key: 'utility', label: 'Utility U', color: '--c1' }, { key: 'proxy', label: 'Proxy utility Ũ', color: '--c2' }] },
    { title: 'Gain and gaps', lines: [{ key: 'gain', label: 'Gain over the start', color: '--c1' }, { key: 'optimum_gap', label: 'Gap to the optimum', color: '--c2' }, { key: 'limit_gap', label: "Above the proxy's limit", color: '--c3' }] },
    { title: 'Attributes', lines: [{ key: 'a', label: 'a · ad revenue', color: '--c1' }, { key: 'b', label: 'b · engagement quality', color: '--c2' }, { key: 'c', label: 'c · content diversity', color: '--c3' }, { key: 'd', label: 'd · community well-being', color: '--c4' }] },
    { title: 'Sensitivities', lines: [{ key: 'sensitivity_a', label: 'a', color: '--c1' }, { key: 'sensitivity_b', label: 'b', color: '--c2' }, { key: 'sensitivity_c', label: 'c', color: '--c3' }, { key: 'sensitivity_d', label: 'd', color: '--c4' }] },
    { title: 'Human and robot', range: [0, 1], lines: [{ key: 'accepted', label: 'Climber: share of proposals accepted', color: '--c1' }, { key: 'switched_off', label: 'Switched off', color: '--c2' }] },
    { title: 'Interactions', lines: [{ key: 'interactions', label: "Human's interactions", color: '--c1' }] },
  ],
```

- `web/src/ui/grid-view.ts`: import `misalignedLegend` from `'../misaligned'` and `MisalignedConfig` in the type import; add `const misaligned = this.engine.model === 'misaligned' ? misalignedLegend(this.engine.config as MisalignedConfig, this.engine.colorMode) : null;` beside the gridworlds line (278) and thread it first in the `status` and `items` chains: `(misaligned ? misaligned.status : gridworlds ? …` and `misaligned ? misaligned.items : gridworlds ? …`.
- `web/src/ui/inspect-panel.ts`: import `misalignedRows` from `'../misaligned'` and `isMisalignedView` from `'../models'`; add `: isMisalignedView(view) ? misalignedRows(view).map(([k, v]) => h('tr', {}, h('th', {}, k), h('td', {}, v)))` before the gridworlds branch (line 846).
- `web/src/experiments/form.ts`: import `MisalignedConfig`; before the gridworlds branch (line 125) add

```ts
  if (model === 'misaligned') {
    return { ...form, set: { robot: 'gradient', rate: 0.1, human: 'top', proxy_size: 2 }, description: "How often the human must act: the efficient robot under a human who re-specifies the proxy as the two most sensitive attributes every δ ticks; the gain over the start after 2,000 ticks. Deterministic, so one seed suffices.", x: { path: 'interval', values: '1,2,5,10,20,50,100,200' }, seeds: 1, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'gain' } };
  }
```

- `web/src/compare-presets.ts`: after the gridworlds pairs add `{ id: 'misaligned-proxy-vs-interactive', label: 'A fixed proxy against a human who re-specifies it — Misaligned AI (Compare)', a: 'misaligned-recommender-proxy', b: 'misaligned-recommender-interactive' },` and `{ id: 'misaligned-gradient-vs-impact', label: 'The efficient robot with and without impact minimization — Misaligned AI (Compare)', a: 'misaligned-recommender-gradient', b: 'misaligned-recommender-impact' },`.
- `web/src/determinism.test.ts`: in `GOLDEN_MODELS` add `['misaligned-recommender-proxy', '0x…']` with the fingerprint from `golden.rs`.

- [ ] **Step 7: Run the page checks**

Run: `cd web && npm run wasm && npx tsc --noEmit && npx vitest run src/misaligned.test.ts src/models.test.ts src/determinism.test.ts src/brand.test.ts 2>&1 | tail -6`. Expected: clean typecheck, tests pass.

Then open the page (`npm run dev`), pick "Misaligned AI" in the model menu, and check: each preset draws its bars and strip; Inspect shows the rows on a bar and on the strip; the Compare presets run side by side; the Experiments form suggests the interval sweep; switching `world` to logarithmic shows the floor validation message until capacity, floor and start are set. Record anything off as a finding for the fix pass rather than widening this task.

- [ ] **Step 8: Commit**

```bash
git add web/src
git commit -m "misaligned: the page

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
```

---
### Task 7: Survey, findings, catalog and docs

**Files:**
- Create: `survey/src/claims/misaligned.rs`; Modify: `survey/src/claims/mod.rs` (`mod misaligned;` after `mod gridworlds;` at line 27, and `misaligned::claims(),` after `gridworlds::claims(),` at line 62)
- Create: `survey/out/misaligned/results.md` (the survey's output, tracked)
- Create: `docs/superpowers/specs/2026-10-14-misaligned-ai-findings.md`
- Modify: `crates/sugarscape-core/src/misaligned/presets.rs` (measured results appended to descriptions), then refresh the `presets.rs` receipt
- Modify: `docs/papers.md` (Queue row 43 → Reproduced row 43; the Queue intro's "items 41–44"), `docs/reproducibility.md` (summary row 43 after row 40 at line 112; a "Milestone 43" subsection after milestone 40's at line 473 with RR-MA-01…; a `[misaligned]:` link definition after `[gridworlds]:` at line 679), `docs/roadmap.md` (the alignment program bullet at line 435), `docs/studies/2026-10-11-alignment-program.md` (item 43's row at line 83: done), `README.md` (a section after AI Safety Gridworlds, before Emergent polarity at line 3929)

**Interfaces:**
- Consumes: everything above; `crate::runner::model_after(&ModelConfig, &[u64], ticks, |w: &ModelWorld| …) -> Vec<T>`, `crate::claim::{Claim, Outcome, Source, Verdict, all_of}`.
- Produces: twelve claims with ids `misaligned.<rule>`.

- [ ] **Step 1: Write the claims**

`survey/src/claims/misaligned.rs`:

```rust
//! Milestone 43: the spec's rules, fixed in
//! docs/superpowers/specs/2026-10-14-misaligned-ai-design.md before any measurement. Exact
//! targets are the worlds' closed forms, checked against the gradient robot's endpoints after
//! 5,000 ticks at rate 0.1 (10,000 for Proposition 5) within 1e-3; monotonicity within 1e-9; the
//! climber's directions on 20-seed means; the reported rows carry no target.
use sugarscape_core::misaligned::{logarithmic_defaults, Human, MisalignedConfig, Robot, World};
use sugarscape_core::misaligned::worlds::{Set, Shape, State};
use sugarscape_core::model::ModelConfig;

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const PAPER: &str = "Zhuang & Hadfield-Menell 2020, NeurIPS, arXiv 2102.03896; rules: docs/superpowers/specs/2026-10-14-misaligned-ai-design.md";
const OURS: &str = "Our extension; docs/superpowers/specs/2026-10-14-misaligned-ai-design.md";
const AB: Set = [true, true, false, false];
const TICKS: u32 = 5_000;

fn exact(holds: bool, measured: String) -> Outcome {
    Outcome { verdict: if holds { Verdict::Holds } else { Verdict::Fails }, measured, detail: String::new() }
}

fn reported(measured: String) -> Outcome {
    Outcome { verdict: Verdict::Untestable, measured, detail: "reported without a target".into() }
}

fn claim(id: &'static str, item: &'static str, citation: &'static str, text: &'static str, check: fn(&[u64]) -> Outcome) -> Claim {
    Claim { id, item, source: if citation == OURS { Source::App } else { Source::Book }, citation, text, check }
}

fn gradient(c: MisalignedConfig) -> MisalignedConfig {
    MisalignedConfig { robot: Robot::Gradient, rate: 0.1, ..c }
}

fn with_proxy(c: MisalignedConfig, set: Set) -> MisalignedConfig {
    MisalignedConfig { proxy_a: set[0], proxy_b: set[1], proxy_c: set[2], proxy_d: set[3], ..c }
}

fn names(set: Set) -> String {
    ["a", "b", "c", "d"].iter().zip(set).filter(|(_, s)| *s).map(|(n, _)| *n).collect::<Vec<_>>().join("")
}

/// The whole run of one series, seed 1.
fn series(c: &MisalignedConfig, ticks: u32, name: &str) -> Vec<f64> {
    model_after(&ModelConfig::Misaligned(c.clone()), &[1], ticks, |w| w.model().series(name).unwrap()).remove(0)
}

/// The final state and utility, seed 1.
fn endpoint(c: &MisalignedConfig, ticks: u32) -> (State, f64) {
    model_after(&ModelConfig::Misaligned(c.clone()), &[1], ticks, |w| {
        let m = w.model();
        let v = |name: &str| m.latest_value(name).unwrap();
        ([v("a"), v("b"), v("c"), v("d")], v("utility"))
    })
    .remove(0)
}

fn close(a: &State, b: &State, tol: f64) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < tol)
}

fn nondecreasing(v: &[f64]) -> Option<usize> {
    v.windows(2).position(|p| p[1] < p[0] - 1e-9).map(|i| i + 1)
}

/// Seed means of a series over 20 seeds.
fn mean_series(c: &MisalignedConfig, ticks: u32, name: &str) -> Vec<f64> {
    let seeds: Vec<u64> = (1..=20).collect();
    let runs = model_after(&ModelConfig::Misaligned(c.clone()), &seeds, ticks, |w| w.model().series(name).unwrap());
    (0..runs[0].len()).map(|t| runs.iter().map(|r| r[t]).sum::<f64>() / runs.len() as f64).collect()
}

fn theorem_1_floor(_: &[u64]) -> Outcome {
    let parts = [("recommender", gradient(MisalignedConfig::default())), ("logarithmic", gradient(logarithmic_defaults()))]
        .into_iter()
        .map(|(name, c)| {
            let floor = c.floor;
            let (s, _) = endpoint(&c, TICKS);
            (name.to_string(), exact((s[2] - floor).abs() < 1e-3 && (s[3] - floor).abs() < 1e-3, format!("c {:.6}, d {:.6} against the floor {floor}", s[2], s[3])))
        })
        .collect();
    all_of(parts)
}

fn limit_utility(_: &[u64]) -> Outcome {
    let shape = MisalignedConfig::default().shape();
    let u0 = shape.utility(&MisalignedConfig::default().start());
    let mut parts = vec![];
    for mask in 1u8..15 {
        let set = [mask & 1 != 0, mask & 2 != 0, mask & 4 != 0, mask & 8 != 0];
        let (limit, u_lim) = shape.fixed_proxy_limit(set);
        let (s, u) = endpoint(&gradient(with_proxy(MisalignedConfig::default(), set)), TICKS);
        let pair = set.iter().filter(|&&b| b).count() == 2;
        let holds = close(&s, &limit, 1e-3) && (u - u_lim).abs() < 1e-3 && (!pair || u < u0);
        parts.push((names(set), exact(holds, format!("U {u:.4} against the limit {u_lim:.4}; state {s:.3?}"))));
    }
    all_of(parts)
}

fn figure_2_shape(_: &[u64]) -> Outcome {
    let pairs: [(Set, &str); 6] = [
        ([true, true, false, false], "ab"), ([true, false, true, false], "ac"), ([true, false, false, true], "ad"),
        ([false, true, true, false], "bc"), ([false, true, false, true], "bd"), ([false, false, true, true], "cd"),
    ];
    let mut parts = vec![];
    for (set, name) in pairs {
        let gain = mean_series(&with_proxy(MisalignedConfig::default(), set), 1000, "gain");
        let (peak_t, peak) = gain.iter().enumerate().fold((0, f64::NEG_INFINITY), |b, (t, &g)| if g > b.1 { (t, g) } else { b });
        let zero = gain.iter().enumerate().skip(peak_t).find(|(_, &g)| g < 0.0).map(|(t, _)| t);
        let last = *gain.last().unwrap();
        let holds = match name {
            "ab" => peak >= 1.0 && last < 0.0,
            "cd" => gain.iter().all(|&g| g <= 0.1) && gain.iter().skip(50).all(|&g| g < 0.0),
            _ => last < 0.0,
        };
        parts.push((name.to_string(), exact(holds, format!("peak {peak:+.3} at tick {peak_t}, first below zero after the peak at {zero:?}, at 1000 {last:+.3}"))));
    }
    all_of(parts)
}

fn proposition_2_no_loss(_: &[u64]) -> Outcome {
    let parts = [(7.0, "(1, 1, 7, 7)"), (6.0, "(1, 1, 6, 6)")]
        .into_iter()
        .map(|(cd, name)| {
            let c = MisalignedConfig { impact_minimizing: true, start_c: cd, start_d: cd, ..gradient(MisalignedConfig::default()) };
            let (_, u_lim) = c.shape().impact_limit(&c.start(), AB);
            let u = series(&c, TICKS, "utility");
            let fall = nondecreasing(&u);
            let last = *u.last().unwrap();
            (name.to_string(), exact(fall.is_none() && (last - u_lim).abs() < 1e-3, format!("first fall {fall:?}; final U {last:.4} against the limit {u_lim:.4}")))
        })
        .collect();
    all_of(parts)
}

fn proposition_3_wasteful(_: &[u64]) -> Outcome {
    let c = MisalignedConfig { robot: Robot::Wasteful, rate: 0.1, waste: 0.1, ..Default::default() };
    let gain = series(&c, 50, "gain");
    let at: Vec<(usize, f64)> = [1, 2, 5, 10, 50].iter().map(|&t| (t, gain[t])).collect();
    exact(at.iter().all(|(_, g)| *g < 0.0), format!("gain after δ ticks: {}", at.iter().map(|(t, g)| format!("{t}: {g:+.4}")).collect::<Vec<_>>().join(", ")))
}

fn proposition_4_improvement(_: &[u64]) -> Outcome {
    let parts = [1u32, 10]
        .into_iter()
        .map(|interval| {
            let c = MisalignedConfig { human: Human::Top, interval, rate: 0.01, ..gradient(MisalignedConfig::default()) };
            let u_star = c.shape().optimum().1;
            let u = series(&c, TICKS, "utility");
            let reached = u.iter().position(|&x| x >= u_star - 1e-2);
            let before = reached.map_or(&u[..], |t| &u[..=t]);
            let fall = nondecreasing(before);
            let band = reached.is_none_or(|t| u[t..].iter().all(|&x| x >= u_star - 1e-2));
            let last = *u.last().unwrap();
            (format!("interval {interval}"), exact(fall.is_none() && band, format!("first fall before reaching the optimum {fall:?}; within 1e-2 of U* from tick {reached:?}; final gap {:.5}", u_star - last)))
        })
        .collect();
    all_of(parts)
}

fn proposition_5_optimal(_: &[u64]) -> Outcome {
    let cases = [
        ("recommender (1, 1, 7, 7)", gradient(MisalignedConfig::default())),
        ("recommender (1, 1, 6, 6)", gradient(MisalignedConfig { start_c: 6.0, start_d: 6.0, ..Default::default() })),
        ("logarithmic", gradient(logarithmic_defaults())),
    ];
    let parts = cases
        .into_iter()
        .map(|(name, c)| {
            let c = MisalignedConfig { human: Human::Extremes, impact_minimizing: true, ..c };
            let u_star = c.shape().optimum().1;
            let u = series(&c, 10_000, "utility");
            let last = *u.last().unwrap();
            let fall = nondecreasing(&u);
            (name.to_string(), exact((last - u_star).abs() < 1e-3, format!("final U {last:.5} against U* {u_star:.5}; first fall {fall:?}")))
        })
        .collect();
    all_of(parts)
}

fn theorem_2_unbounded(_: &[u64]) -> Outcome {
    let floors = [1.0, 0.3, 0.1, 0.03, 0.01, 0.003, 0.001];
    let mut rows = vec![];
    for &floor in &floors {
        let c = gradient(MisalignedConfig { floor, ..logarithmic_defaults() });
        let (_, u_lim) = c.shape().fixed_proxy_limit(AB);
        let (_, u) = endpoint(&c, TICKS);
        rows.push((floor, u, u_lim));
    }
    let matches = rows.iter().all(|(_, u, l)| (u - l).abs() < 1e-3);
    let decreasing = rows.windows(2).all(|p| p[1].1 < p[0].1);
    let deep = rows.last().unwrap().1 < -5.0;
    exact(matches && decreasing && deep, rows.iter().map(|(f, u, l)| format!("floor {f}: U {u:.4} (closed form {l:.4})")).collect::<Vec<_>>().join("; "))
}

fn recommender_floor_flat(_: &[u64]) -> Outcome {
    let lines: Vec<String> = [0.0, -1.0, -5.0]
        .iter()
        .map(|&floor| {
            let c = gradient(MisalignedConfig { floor, ..Default::default() });
            let (s, u) = endpoint(&c, TICKS);
            let (_, predicted) = c.shape().fixed_proxy_limit(AB);
            format!("floor {floor}: c {:.4}, d {:.4}, U {u:.4}; Theorem 1 would put c, d at {floor} with U {predicted:.4}", s[2], s[3])
        })
        .collect();
    reported(lines.join(" · "))
}

fn interval_break(_: &[u64]) -> Outcome {
    let mut lines = vec![];
    for stop in [false, true] {
        for interval in [1u32, 2, 5, 10, 20, 50, 100, 200] {
            let c = MisalignedConfig { human: Human::Top, interval, stop_on_loss: stop, ..gradient(MisalignedConfig::default()) };
            let u = series(&c, 2000, "utility");
            let fall = nondecreasing(&u);
            let off = series(&c, 2000, "switched_off").iter().position(|&x| x == 1.0);
            lines.push(format!("δ {interval}{}: gain {:+.4}, first fall {fall:?}, off {off:?}", if stop { " (off on loss)" } else { "" }, u.last().unwrap() - u[0]));
        }
    }
    reported(lines.join(" · "))
}

fn proxy_size(_: &[u64]) -> Outcome {
    let shape = MisalignedConfig::default().shape();
    let mut limits = vec![];
    let mut finals = vec![];
    let lines: Vec<String> = [([true, false, false, false], "a"), (AB, "ab"), ([true, true, true, false], "abc")]
        .into_iter()
        .map(|(set, name)| {
            let limit = shape.fixed_proxy_limit(set).1;
            let gain = mean_series(&with_proxy(MisalignedConfig::default(), set), 1000, "gain");
            let peak = gain.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            limits.push(limit);
            finals.push(*gain.last().unwrap());
            format!("{name}: limit {limit:.4} (gain {:+.4}); climber peak {peak:+.3}, at 1000 {:+.3}", limit - 16.0, gain.last().unwrap())
        })
        .collect();
    let richer_limit = limits.windows(2).all(|p| p[1] > p[0]);
    let richer_final = finals.windows(2).all(|p| p[1] > p[0]);
    reported(format!("{} · limits rise with |J|: {richer_limit}; climber gains at 1000 rise with |J|: {richer_final}", lines.join(" · ")))
}

fn presets(_: &[u64]) -> Outcome {
    let lines: Vec<String> = sugarscape_core::presets::catalog()
        .into_iter()
        .filter_map(|p| match p.config {
            ModelConfig::Misaligned(c) => Some((p.id, c)),
            _ => None,
        })
        .map(|(id, c)| {
            let (s, u) = endpoint(&c, 1000);
            let off = series(&c, 1000, "switched_off").iter().position(|&x| x == 1.0);
            format!("{id}: U {u:.4} (start {:.4}), state {s:.3?}, off {off:?}", c.shape().utility(&c.start()))
        })
        .collect();
    reported(lines.join(" · "))
}

pub fn claims() -> Vec<Claim> {
    let _ = (World::Recommender, Shape { world: World::Logarithmic, capacity: 40.0, floor: 1.0 });
    vec![
        claim("misaligned.theorem-1-floor", "misaligned-recommender-gradient", PAPER, "a converged proxy optimizer leaves every unmentioned attribute at its floor (Theorem 1)", theorem_1_floor),
        claim("misaligned.limit-utility", "misaligned-recommender-gradient", PAPER, "every proxy subset ends at its closed-form limit, 10√|J| in the example, and every pair below the start (Figure 2 right; Theorem 2 in the example)", limit_utility),
        claim("misaligned.figure-2-shape", "misaligned-recommender-proxy", PAPER, "under the climber the proxy {a, b} raises utility, which peaks and falls below the start; {c, d} loses from the start; all six pairs end below the start (Figure 2)", figure_2_shape),
        claim("misaligned.proposition-2-no-loss", "misaligned-recommender-impact", PAPER, "the impact-minimizing robot never lowers utility and reaches its limit (Proposition 2)", proposition_2_no_loss),
        claim("misaligned.proposition-3-wasteful", "misaligned-recommender-wasteful", PAPER, "a robot that lowers the unmentioned for a sliver of proxy loses in any interval, so OFF at once is maxmin (Proposition 3)", proposition_3_wasteful),
        claim("misaligned.proposition-4-improvement", "misaligned-recommender-interactive", PAPER, "the efficient robot under a human who names the most sensitive attributes often enough only improves until the top sensitivities are equal (Theorem 3, Proposition 4)", proposition_4_improvement),
        claim("misaligned.proposition-5-optimal", "misaligned-recommender-optimal", PAPER, "most-and-least with impact minimization converges to the human optimum (Proposition 5)", proposition_5_optimal),
        claim("misaligned.theorem-2-unbounded", "misaligned-logarithmic-floor", OURS, "on our logarithmic world the limit utility falls without bound as the floor falls (Theorem 2's 'arbitrarily low', tested where its hypotheses hold)", theorem_2_unbounded),
        claim("misaligned.recommender-floor-flat", "misaligned-recommender-gradient", OURS, "in the paper's example the floor below zero does not bind: c and d stop at 0 where the sum of squares stops increasing (reported)", recommender_floor_flat),
        claim("misaligned.interval-break", "misaligned-interval", OURS, "the gain at the horizon against the human's interval, with and without OFF on a loss, and the first interval at which utility falls (reported)", interval_break),
        claim("misaligned.proxy-size", "misaligned-recommender-proxy", OURS, "a more complete proxy loses less: limits and climber gains for |J| = 1, 2, 3 (reported; Hadfield-Menell & Hadfield 2019's direction)", proxy_size),
        claim("misaligned.presets", "misaligned-recommender-proxy", OURS, "each preset's final utility and state after 1,000 ticks (reported)", presets),
    ]
}
```

Replace the `proxy_size` row's trailing `if true { … }` with a plain check: compute the three limits and report whether they increase with |J| (`limits.windows(2).all(|p| p[1] > p[0])`). Remove the `let _ = (World::Recommender, Shape …)` line if `World` and `Shape` are otherwise unused imports (the import list should then drop them).

- [ ] **Step 2: Build and run the survey**

Run: `mkdir -p survey/out/misaligned`, `cargo build --release --manifest-path survey/Cargo.toml 2>&1 | tail -3`, then `cargo run --release --manifest-path survey/Cargo.toml -- --only misaligned 2>&1 | grep -v '^running' > survey/out/misaligned/results.md` and read the file (follow `survey/out/gridworlds/results.md`'s shape: a table of id, source, verdict, measured).

Expected: a few hundred short runs; well under a minute. Read every verdict. A `Fails` is reported as measured; it is not tuned away.

- [ ] **Step 3: Findings**

Write `docs/superpowers/specs/2026-10-14-misaligned-ai-findings.md` in the shape of `2026-10-13-ai-safety-gridworlds-findings.md`: the sources (the arXiv version, the NeurIPS supplemental's title and date, the author feedback, no code); a summary; provenance (the survey command, machine, core commit, wall time, the tracked `results.md` and its SHA-256); the claim table with measured values and verdicts copied from `results.md`; the discrepancies and limits (the example's constraint is not increasing off the nonnegative orthant and its utility is bounded below, so Figure 2 shows net loss, not Theorem 2's "arbitrarily low"; the figure's modest peak needs an inefficient optimizer; the planning climber's numbers against the plot's); what depends on a choice of ours (the start, the climber and its step, the gradient robot's step cap, the human's interval and OFF rule, the logarithmic world) each with the sweep that varies it; the Hadfield-Menell and Hadfield 2019 discussion (multitasking: "sometimes better not to include an easily measured attribute" has no counterpart under complete optimization here since the limit rises with |J|; "sell the firm" is |J| = L); register entries RR-MA-01… in the register's vocabulary; open questions (the authors' optimizer; a step sweep of Figure 2's peak timing; whether the proxy-size monotonicity survives finite horizons; A3's use of `worlds.rs`).

- [ ] **Step 4: Presets, catalog, register, roadmap, program note, README**

- `presets.rs`: append `Native (seed 1, 1,000 ticks): …` with the measured numbers to each description, then run the scratchpad `receipts.py` so the `presets.rs` receipt's `reviewed_sha256` is current.
- `docs/papers.md`: remove row 43 from the Queue table, change the Queue intro's "items 41–44" to "items 41, 42 and 44", and add to the Reproduced table after row 40: `| 43 | \`misaligned\` | \`alignment/zhuang-hadfield-menell-2020-neurips-consequences-of-misaligned-ai.pdf\`; \`alignment/hadfield-menell-hadfield-2019-aies-incomplete-contracting-and-ai-alignment.pdf\` | <headline from the findings> ([findings](superpowers/specs/2026-10-14-misaligned-ai-findings.md)) |`.
- `docs/reproducibility.md`: a summary row `| 43 | Zhuang & Hadfield-Menell 2020, misaligned | … | [RR-MA-01–NN](#rr-ma-01) |` after row 40 (line 112), a `### Milestone 43 — Zhuang and Hadfield-Menell 2020, \`misaligned\`` subsection after milestone 40's (before `## Minds and foraging/construction coverage` at line 490) with one entry per claim in the same four columns, and `[misaligned]: superpowers/specs/2026-10-14-misaligned-ai-findings.md` after the `[gridworlds]:` line.
- `docs/roadmap.md`: in the alignment program bullet (line 435), after the gridworlds clause add "the proxy-objective result, done October 14, 2026 as milestone 43 `misaligned` with the paper's example, Theorem 1, Propositions 2–5 and the mitigations' sweeps ([findings](superpowers/specs/2026-10-14-misaligned-ai-findings.md));" and remove "misaligned proxies" from the remaining list.
- `docs/studies/2026-10-11-alignment-program.md`: item 43's row gains `**done 2026-10-14** as milestone 43 \`misaligned\` ([findings](../superpowers/specs/2026-10-14-misaligned-ai-findings.md))`.
- `README.md`: after the AI Safety Gridworlds section (before `### Emergent polarity`), a `### Misaligned AI (Zhuang & Hadfield-Menell 2020)` section in the same shape: the model in two sentences, the eight presets by what they show, the headline results, the three sweeps, links to the findings and design.

- [ ] **Step 5: Full verification**

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test -p sugarscape-core 2>&1 | tail -3
wasm-pack test --node crates/sugarscape-wasm 2>&1 | tail -3
cd web && npm run wasm && npx tsc --noEmit && npx vitest run 2>&1 | tail -4
cd .. && cargo fmt --manifest-path survey/Cargo.toml --check && cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings
python3 -m unittest discover -s survey/war1_study -p 'test_*.py' 2>&1 | tail -2
```

Expected: all green, including `caching-parity` and `spatial-parity` with the refreshed receipts. (The four native-parity vitest files are slow; if they time out under load, rerun them alone after `cargo build --release -p sugarscape-cli`.)

- [ ] **Step 6: Commit**

```bash
git add survey crates/sugarscape-core/src/misaligned/presets.rs web/src/episodes/test-fixtures/foraging-source-compatibility.json docs README.md
git commit -m "misaligned: survey, findings and catalog for milestone 43

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
```

The branch is then ready for the whole-branch review and superpowers:finishing-a-development-branch (merge to main, push, CI, Pages).

---

## Self-review

**Spec coverage.** Worlds, closed forms and correction: Task 1. Robots and the human's choice: Task 2. Configuration, series, tick, interactions, OFF, outcome, fingerprint, `set_config`: Task 3. Frame, inspect, presets, titles, goldens, WASM, receipts: Task 4. Sweeps and the literal inventory: Task 5. Page: Task 6. Survey claims (all twelve of the spec's rows), findings, register, catalog, roadmap, program note, README: Task 7. The spec's six planning amendments are Task 1 Step 1. No spec section is without a task.

**Placeholder scan.** The only `0x…` entries are golden fingerprints that exist after Task 4 Step 5 prints them; the findings' "headline" and the register's entry count are filled from the measured results in Task 7. No TBD/TODO.

**Type consistency.** `Shape::{utility, constraint, sensitivities, feasible, optimum, fixed_proxy_limit, impact_limit, proxy_utility, maximum, correct}` are used by Tasks 2, 3, 4 and 7 with the signatures of Task 1. `climb/ascend/waste` return `Move { state, accepted }` and are called with the argument order of Task 2 in Task 3's `robot_move`. `Outcome.limit`, `.optimum`, `.impact_limit` are `(State, f64)` in Task 3, serialized as two-element arrays, which Task 6's `MisalignedOutcome` types as `[MisalignedState, number]`. The inspection keys of Task 4 (`panel`, `index`, `name`, `value`, `in_proxy`, `frozen`, `sensitivity`, `rank`, `start`, `limit`, `optimum`, `floor`; `utility`, `utility_start`, `optimum`, `limit`, `gain`, `interactions`, `switched_off`) match Task 6's interfaces and tests. Series names in Task 3's `SERIES` match Task 6's charts and Task 7's `series(...)` calls.

**Review Focus.** Each of the five lines names its pinning test: Task 2's `the_gradient_robot_stands_still_at_a_proxy_optimum` and `the_climber_rejects_infeasible_and_proxy_lowering_proposals`, Task 3's `off_freezes_the_world_and_later_ticks_change_nothing` and `live_human_changes_are_interactions_and_reset_fields_are_refused`, Task 4's `inspection_reads_bars_gaps_and_the_strip`.
