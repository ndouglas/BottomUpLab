# Assistance Games Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the kind `assistance` (Assistance Games): the paperclip game and the off-switch game reproduced exactly from the two Hadfield-Menell et al. papers, with two equilibrium dynamics of ours, six presets, five sweeps, page support and a preregistered survey.

**Architecture:** A new core module `crates/sugarscape-core/src/assistance/` holds two pure analytic units (`paperclips.rs`, `offswitch.rs`) that compute the papers' numbers without an RNG, a `dynamics.rs` that samples episodes and runs iterated best response and fictitious play from the shared seeded stream, and a `world.rs` implementing `Model` like `auctions`. The kind is wired into model dispatch, schema, titles, presets, goldens, sweeps, CLI, WASM tests, the page and the survey at the same places as `auctions`.

**Tech Stack:** Existing Rust workspace (serde, rand_pcg, portable arithmetic), wasm-bindgen tests, TypeScript/Vite/Vitest page, standalone `survey/` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-11-assistance-games-design.md` (approved 2026-10-11). Task 0 below amends it with the eight planning findings listed under *Spec addenda*; the plan argues from the amended spec. Sources: `papers/alignment/hadfield-menell-dragan-abbeel-russell-2016-neurips-cooperative-inverse-reinforcement-learning.pdf` (§3.3, appendix §6) and `papers/alignment/hadfield-menell-dragan-abbeel-russell-2017-ijcai-off-switch-game.pdf` (§§2–5, Figures 3–4).

## Global Constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add a new kind and new golden entries only.
- One sequential path serves native and WASM: `f64` draws via `rng.gen::<f64>()`, `u32` ranges via `rng.gen_range(0..n_u32)`, `portable::exp_neg` only on nonpositive arguments, the identity `1/(1+e^x) = e^{-x}/(1+e^{-x})` for the logistic's positive branch, `portable::ln` for the Gaussian draw, no `f64::exp`, `f64::ln`, `cos`, `sin` or `erf` in `assistance/` outside `#[cfg(test)]` blocks.
- The analytic units `paperclips.rs` and `offswitch.rs` take no RNG and no world; their tests compare against the papers' numbers.
- Literal rules are defaults: `game: paperclips`, `mode: iterated`, `human_rule: expert`, `robot_map: best_response`, `theta_bins: 92`, `episodes_per_tick: 1000`, `respond_every: 1`, `fictitious_weight: 1`, `protocol: prior`, `mu: 0`, `sigma: 1`, `beta: 0`, `actions: 1`, `observation_noise: 1`, `quadrature_density: 8`. Reset-only: `game`, `mode`, `theta_bins`.
- Decision rules (spec, *Decision rules*): exact targets within 1e-9 or exactly for rationals; Figure 3 direction claims with zero violations over the fixed grid; Figure 4 sampled claims at 100,000 episodes per point, seed 1, a violation only beyond two standard errors of the difference; dynamics reported without a target over 20 seeds and both modes. No rule changes after measurement without a dated amendment.
- Ties: H's pointwise best reply to the smaller action index; R's deployment to the smaller index; R's off-switch choice to `wait` on equal values. Off-path H actions resolve R's posterior to the prior.
- American spelling in code, docs and captions; "agent", never "Flump", in the model.
- Paper PDFs stay gitignored under `papers/`.
- Work in a worktree on branch `assistance` (superpowers:using-git-worktrees). Push after every commit; after merge, push main and watch CI (`gh run list`) and the Pages deploy. Record the milestone in `docs/papers.md`, `docs/roadmap.md` and `docs/reproducibility.md` at merge. Run `wasm-pack test --node crates/sugarscape-wasm` before merge.

## Spec addenda (planning findings, applied in Task 0)

1. **Quadrature.** Gauss–Hermite converges slowly when the human's temperature β is below about half of σ (the logistic's poles sit πβ off the real axis), so the Theorem 2 agreement rule could not hold at 1e-9 across Figure 3's grid. The moments E[π^H], E[π^H(1−π^H)] and E[U·π^H] under N(μ, σ²) are computed by the composite trapezoid rule with step `min(σ, β) / quadrature_density` over μ ± 9σ, which is spectrally accurate for these analytic integrands and needs no table; β = 0 uses closed forms through a portable Φ. `quadrature_nodes` becomes `quadrature_density` (points per smallest scale; default 8, range 4–64).
2. **Protocol switch.** A field `protocol: prior | report` separates the paper's §2 game (U_a drawn from R's belief, which equals the prior) from the Figure 4 reconstruction (population N(0, 1), noisy report, Bayesian posterior mean, `actions` candidates). Default `prior`.
3. **Figure 4 needs a noisy human.** With a rational H, Theorem 1 makes R wait for every σ, so the value V is flat in σ and the figure's peak cannot appear. The built-in `assistance-figure4` sweep fixes β = 0.5 (ours, named); a planning simulation showed a peak near the true posterior standard deviation at β ∈ {0.5, 1} and a plateau at β = 0.25. The flat rational case is reported as a finding. A series `value` (realized mean utility per episode) is added.
4. **Pooling is not a pure mutual best response.** H's best reply to any constant map is the expert rule (R's deployment then does not depend on H's action), so the pair (always (1,1), always (50,50)) is not an equilibrium under the stated tie and off-path rules. The spec's sentence "which includes at least the pooling pair" is corrected to a question answered by enumeration; `pooling` remains a named start for the dynamics.
5. **Gaussian draws** use the Box–Muller transform in its polar form (two uniform draws, `sqrt` and `portable::ln`); the trigonometric form would call the platform's cosine.
6. **Figure 3 grid**, fixed: σ ∈ {0.05, 0.10, …, 1.00} (20 values) and β ∈ {0, 0.2, …, 4.0} (21 values) in the frame; the sweeps' series axis holds β ∈ {0, 0.1, 0.2, 0.4, 0.6, 0.8, 1, 1.5, 2, 3, 4} (sweeps allow 16 series values).
7. **Fixed robot map** is three integer fields `fixed_0`, `fixed_1`, `fixed_2` (deployment index 0–2 for H's actions (0,2), (1,1), (2,0)), shown when `robot_map` is `fixed`.
8. **The uniform-random start** table is drawn at reset for every paperclips world (`theta_bins` draws of `gen_range(0..3u32)`), so `human_rule` can change live without new draws.

## Review Focus

1. A live `human_rule` or `robot_map` change on a running iterated or static world must restart the pair cleanly (history entry, `stable_rounds` reset) and never read the old rule's length after a `theta_bins` refusal — Task 3 `live_rule_change_restarts_the_pair_and_reset_fields_are_refused`.
2. Episode draws with β = 0 and β > 0 must consume the same number of uniforms per episode, so switching β live does not shift the stream — Task 3 `draw_budget_is_independent_of_beta`.
3. The Figure 3 frame must mark the zero boundary and the configured point without indexing outside the grid when σ or β lie beyond it (σ = 5, β = 50) — Task 4 `surface_frame_handles_points_off_the_grid`.
4. R's posterior under a rule that never uses some H action must fall back to the prior, both in the analytic best reply and in the frame's posterior band after that action is observed under a fixed map — Task 1 `off_path_actions_resolve_to_the_prior`, Task 4 `posterior_band_under_a_fixed_map_uses_the_prior_off_path`.
5. `quadrature_density` at 4 and 64 must give Δ within 1e-9 of each other on the Figure 3 grid, so the default is not hiding quadrature error — Task 2 `density_four_and_sixty_four_agree`.

---

### Task 0: Amend the spec and open the worktree

**Files:**
- Modify: `docs/superpowers/specs/2026-10-11-assistance-games-design.md`

- [ ] **Step 1: Create the worktree on branch `assistance`** with superpowers:using-git-worktrees (from `main` at or after `9f02b7d`). All later steps run inside it.

- [ ] **Step 2: Amend the spec.** Apply the eight addenda above to the spec text:
  - *Binding constraints*: replace the Gauss–Hermite sentence with "Quadrature is the composite trapezoid rule with step min(σ, β)/`quadrature_density` over μ ± 9σ, in plain arithmetic and `portable::exp_neg`; β = 0 uses closed forms through a portable normal c.d.f. built from the error function's power series. Gaussian draws use the polar Box–Muller transform (`sqrt` and `portable::ln`)."
  - *Source models and targets*, paperclips extensions paragraph: replace "which includes at least the pooling pair … resolved by the prior" with "and whether a pooling pair (H always (1,1), R always (50,50), off-path deployments resolved by the prior) survives as a mutual best response. Planning note: H's best reply to any constant map is the expert rule, so it does not; the enumeration records the set."
  - *Off-switch targets*, Figure 4 row: append "Under a rational H, Theorem 1 makes R wait at every σ, so V is flat; the figure is reconstructed with a Boltzmann H at β = 0.5 (ours) and the rational case is reported."
  - *Configuration* table: replace `quadrature_nodes` with `quadrature_density | 8 | points per smallest scale in the trapezoid rule; 4–64`; add `protocol | prior | off_switch: prior (the paper's game, U_a drawn from the belief) or report (Figure 4's reconstruction)`; replace the `robot_map` row's "a fixed map named by three deployment indices" with "or `fixed`, the map in `fixed_0`, `fixed_1`, `fixed_2` (deployment index 0–2 for H's actions in order)"; add "`human_rule` `uniform_random` draws its table at reset in every paperclips world".
  - *Measurement*: add `value` (mean realized utility per episode) to the off-switch series; name the Figure 3 grid (σ 0.05–1.00 step 0.05; β 0–4 step 0.2) and the sweeps' β series {0, 0.1, 0.2, 0.4, 0.6, 0.8, 1, 1.5, 2, 3, 4}; state that `assistance-figure4` sweeps σ ∈ {0.1, 0.2, …, 1.0, 1.2, 1.5} with β = 0.5, `observation_noise` = 1, series `actions` ∈ {1, 4, 8}, one seed, 100 ticks of 1,000 episodes, metrics the window means of `value` and `delta_exact`.
  - Add a dated line under the header: "**Amended 2026-10-11 (planning):** quadrature, protocol switch, Figure 4's β, pooling, polar Box–Muller, grids, fixed-map fields, uniform-random draws; see the plan's *Spec addenda*."

- [ ] **Step 3: Commit.**

```bash
git add docs/superpowers/specs/2026-10-11-assistance-games-design.md
git commit -m "docs(assistance): spec amendments from planning"
git push -u origin assistance
```

(The plan itself is already on `main`.)

---

### Task 1: The paperclip game's closed forms (`paperclips.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/assistance/mod.rs`
- Create: `crates/sugarscape-core/src/assistance/paperclips.rs`
- Modify: `crates/sugarscape-core/src/lib.rs` (add `pub mod assistance;` after `pub mod anasazi;`… keep alphabetical: after `pub mod ants;`)

**Interfaces:**
- Consumes: nothing from the crate.
- Produces (all `pub`, used by Tasks 3–4 and the survey):

```rust
pub const H_ACTIONS: [(u32, u32); 3];            // (0,2), (1,1), (2,0)
pub const R_ACTIONS: [(u32, u32); 3];            // (0,90), (50,50), (90,0)
pub type Rule = Vec<u8>;                         // H's action index per θ-bin
pub type Map = [u8; 3];                          // R's deployment index per H action
pub struct Ratio { pub num: i64, pub den: i64 }  // lowest terms, den > 0; Ord
pub fn reward(theta: f64, action: (u32, u32)) -> f64
pub fn bin_mid(bin: usize, bins: usize) -> f64
pub fn expert_rule(bins: usize) -> Rule
pub fn threshold_rule(bins: usize, low: Ratio, high: Ratio) -> Rule
pub fn equilibrium_rule(bins: usize) -> Rule       // thresholds 41/92, 51/92
pub fn pooling_rule(bins: usize) -> Rule
pub fn posterior(rule: &[u8], action: u8) -> Vec<f64>
pub fn posterior_mean(post: &[f64]) -> f64
pub fn deployment_for(theta_bar: f64) -> u8
pub fn robot_best_reply(rule: &[u8]) -> Map
pub fn human_payoff(theta: f64, action: usize, map: &Map) -> f64
pub fn human_best_reply(map: &Map, bins: usize) -> Rule
pub fn exact_best_reply(map: &Map) -> Vec<(Ratio, Ratio, u8)>   // intervals [lo, hi] with H's action
pub fn expected_total(map: &Map) -> Ratio                        // ∫₀¹ max_a payoff_a(θ) dθ
pub fn optimal_joint() -> (Map, Vec<(Ratio, Ratio, u8)>, Ratio)
pub fn mutual_best_responses(bins: usize) -> Vec<(Rule, Map)>
pub fn thresholds_of(rule: &[u8]) -> (f64, f64)
pub fn distance(rule: &[u8], map: &Map) -> u32
```

- [ ] **Step 1: Write the failing tests** at the bottom of a new `crates/sugarscape-core/src/assistance/paperclips.rs` (the module body comes in Step 3; create the file with only the `#[cfg(test)]` block plus `use` lines so it fails to compile for the right reason).

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_expert_rule_splits_at_one_half_and_its_best_reply_is_the_papers_equation_3() {
        let rule = expert_rule(92);
        assert!(rule[..46].iter().all(|&a| a == 0) && rule[46..].iter().all(|&a| a == 2));
        assert_eq!(robot_best_reply(&rule), [0, 1, 2]);
    }

    #[test]
    fn posteriors_after_two_staples_have_the_papers_means() {
        // Under the expert rule the posterior is uniform on [0, 1/2): R acts as θ = 1/4.
        let m = posterior_mean(&posterior(&expert_rule(92), 0));
        assert!((m - 0.25).abs() < 1e-12, "{m}");
        // Under cuts at 1/3 and 2/3 (96 bins make them edges) it is uniform on [0, 1/3): θ = 1/6.
        let rule = threshold_rule(96, Ratio::new(1, 3), Ratio::new(2, 3));
        let m = posterior_mean(&posterior(&rule, 0));
        assert!((m - 1.0 / 6.0).abs() < 1e-12, "{m}");
        assert_eq!(deployment_for(0.25), 0);
        assert_eq!(deployment_for(1.0 / 6.0), 0);
    }

    #[test]
    fn off_path_actions_resolve_to_the_prior() {
        let post = posterior(&expert_rule(92), 1);
        assert!(post.iter().all(|&p| (p - 1.0 / 92.0).abs() < 1e-15));
        assert_eq!(deployment_for(posterior_mean(&post)), 1);
    }

    #[test]
    fn the_teaching_thresholds_are_41_92_and_51_92_exactly() {
        assert_eq!(
            exact_best_reply(&[0, 1, 2]),
            vec![
                (Ratio::new(0, 1), Ratio::new(41, 92), 0),
                (Ratio::new(41, 92), Ratio::new(51, 92), 1),
                (Ratio::new(51, 92), Ratio::new(1, 1), 2),
            ]
        );
    }

    #[test]
    fn theorem_3_the_best_reply_to_the_best_reply_is_not_the_expert_rule() {
        let twice = human_best_reply(&robot_best_reply(&expert_rule(92)), 92);
        assert_ne!(twice, expert_rule(92));
        assert_eq!(twice, equilibrium_rule(92));
        // θ = 0.49: 92(1 − θ) = 46.92, 51, 92θ = 45.08, so H makes one of each.
        let payoffs: Vec<f64> = (0..3).map(|a| human_payoff(0.49, a, &[0, 1, 2])).collect();
        assert!((payoffs[0] - 46.92).abs() < 1e-12 && payoffs[1] == 51.0 && (payoffs[2] - 45.08).abs() < 1e-12);
        assert_eq!(twice[(0.49 * 92.0) as usize], 1);
    }

    #[test]
    fn the_optimal_joint_policy_is_the_teaching_pair() {
        let (map, rule, value) = optimal_joint();
        assert_eq!(map, [0, 1, 2]);
        assert_eq!(rule, exact_best_reply(&[0, 1, 2]));
        // ∫ max(92(1−θ), 51, 92θ) dθ = 6373/92 ≈ 69.27, against 69 for the expert pair.
        assert_eq!(value, Ratio::new(6373, 92));
        assert_eq!(expected_total(&[0, 1, 2]), Ratio::new(6373, 92));
    }

    #[test]
    fn pooling_is_not_a_mutual_best_response_but_the_teaching_pair_is() {
        assert_eq!(robot_best_reply(&pooling_rule(92)), [1, 1, 1]);
        assert_eq!(human_best_reply(&[1, 1, 1], 92), expert_rule(92));
        let set = mutual_best_responses(92);
        assert!(set.contains(&(equilibrium_rule(92), [0, 1, 2])));
        assert!(!set.contains(&(pooling_rule(92), [1, 1, 1])));
    }

    #[test]
    fn thresholds_and_distance_read_a_rule_table() {
        assert_eq!(thresholds_of(&expert_rule(92)), (0.5, 0.5));
        let (lo, hi) = thresholds_of(&equilibrium_rule(92));
        assert!((lo - 41.0 / 92.0).abs() < 1e-15 && (hi - 51.0 / 92.0).abs() < 1e-15);
        assert_eq!(distance(&equilibrium_rule(92), &[0, 1, 2]), 0);
        assert_eq!(distance(&expert_rule(92), &[0, 1, 2]), 10);
        assert_eq!(distance(&expert_rule(92), &[1, 1, 1]), 12);
    }

    #[test]
    fn ratios_normalize_and_order() {
        assert_eq!(Ratio::new(82, 184), Ratio::new(41, 92));
        assert_eq!(Ratio::new(-3, -6), Ratio::new(1, 2));
        assert!(Ratio::new(41, 92) < Ratio::new(1, 2) && Ratio::new(1, 2) < Ratio::new(51, 92));
    }
}
```

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p sugarscape-core assistance::paperclips`
Expected: compile error, `expert_rule` and the others not found.

- [ ] **Step 3: Write the module.** `crates/sugarscape-core/src/assistance/mod.rs` (the other `mod` lines are added by later tasks):

```rust
//! Assistance games (milestone 39): the paperclip game (Hadfield-Menell, Dragan, Abbeel and
//! Russell, NeurIPS 2016, §3.3 and appendix §6) and the off-switch game (IJCAI 2017), exact
//! where the papers are exact, with two equilibrium dynamics that are ours.
pub mod paperclips;
```

`crates/sugarscape-core/src/assistance/paperclips.rs`, above the tests:

```rust
//! The paperclip game's closed forms. Pure functions of a rule table and a map; no RNG.
//! H makes two items, R deploys 50/50 or 90 of one; reward θ·p + (1−θ)·q, θ uniform on [0, 1].
//! A rule is H's action per θ-bin (bin k covers [k/bins, (k+1)/bins) and is evaluated at its
//! midpoint); 92 bins make the paper's thresholds 1/2, 41/92 and 51/92 bin edges.
use std::cmp::Ordering;

pub const H_ACTIONS: [(u32, u32); 3] = [(0, 2), (1, 1), (2, 0)];
pub const R_ACTIONS: [(u32, u32); 3] = [(0, 90), (50, 50), (90, 0)];
pub type Rule = Vec<u8>;
pub type Map = [u8; 3];

/// A rational number in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub num: i64,
    pub den: i64,
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}

impl Ratio {
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "zero denominator");
        let g = gcd(num, den).max(1);
        let sign = if den < 0 { -1 } else { 1 };
        Ratio {
            num: sign * num / g,
            den: sign * den / g,
        }
    }
    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
    fn add(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.den + o.num * self.den, self.den * o.den)
    }
    fn sub(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.den - o.num * self.den, self.den * o.den)
    }
    fn mul(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.num, self.den * o.den)
    }
    fn div(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.den, self.den * o.num)
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Ratio {
    fn cmp(&self, o: &Self) -> Ordering {
        (i128::from(self.num) * i128::from(o.den)).cmp(&(i128::from(o.num) * i128::from(self.den)))
    }
}

pub fn reward(theta: f64, (p, q): (u32, u32)) -> f64 {
    theta * f64::from(p) + (1.0 - theta) * f64::from(q)
}

pub fn bin_mid(bin: usize, bins: usize) -> f64 {
    (bin as f64 + 0.5) / bins as f64
}

/// The smallest index of the largest value.
pub(super) fn argmax(values: impl IntoIterator<Item = f64>) -> u8 {
    let mut best = (0u8, f64::NEG_INFINITY);
    for (i, v) in values.into_iter().enumerate() {
        if v > best.1 {
            best = (i as u8, v);
        }
    }
    best.0
}

/// The demonstration-by-expert rule: the immediate reward's maximizer (appendix eq. 2).
pub fn expert_rule(bins: usize) -> Rule {
    (0..bins)
        .map(|k| argmax(H_ACTIONS.iter().map(|&a| reward(bin_mid(k, bins), a))))
        .collect()
}

/// (0,2) below `low`, (1,1) from `low` to `high` inclusive, (2,0) above `high`, at bin midpoints.
pub fn threshold_rule(bins: usize, low: Ratio, high: Ratio) -> Rule {
    (0..bins)
        .map(|k| {
            let mid = Ratio::new(2 * k as i64 + 1, 2 * bins as i64);
            if mid < low {
                0
            } else if mid <= high {
                1
            } else {
                2
            }
        })
        .collect()
}

/// The paper's equilibrium rule (appendix, proof of Theorem 3).
pub fn equilibrium_rule(bins: usize) -> Rule {
    threshold_rule(bins, Ratio::new(41, 92), Ratio::new(51, 92))
}

pub fn pooling_rule(bins: usize) -> Rule {
    vec![1; bins]
}

/// R's posterior over bins after seeing `action` under `rule`: uniform on the bins choosing it,
/// or the uniform prior when no bin does (an off-path action).
pub fn posterior(rule: &[u8], action: u8) -> Vec<f64> {
    let n = rule.iter().filter(|&&a| a == action).count();
    if n == 0 {
        return vec![1.0 / rule.len() as f64; rule.len()];
    }
    rule.iter()
        .map(|&a| if a == action { 1.0 / n as f64 } else { 0.0 })
        .collect()
}

pub fn posterior_mean(post: &[f64]) -> f64 {
    let bins = post.len();
    post.iter()
        .enumerate()
        .map(|(k, p)| p * bin_mid(k, bins))
        .sum()
}

/// The deployment maximizing reward at the mean θ (Theorem 2 of the 2016 paper).
pub fn deployment_for(theta_bar: f64) -> u8 {
    argmax(R_ACTIONS.iter().map(|&a| reward(theta_bar, a)))
}

pub fn robot_best_reply(rule: &[u8]) -> Map {
    let mut map = [0u8; 3];
    for (a, slot) in map.iter_mut().enumerate() {
        *slot = deployment_for(posterior_mean(&posterior(rule, a as u8)));
    }
    map
}

/// H's two-round payoff from `action` when R answers by `map`.
pub fn human_payoff(theta: f64, action: usize, map: &Map) -> f64 {
    reward(theta, H_ACTIONS[action]) + reward(theta, R_ACTIONS[map[action] as usize])
}

pub fn human_best_reply(map: &Map, bins: usize) -> Rule {
    (0..bins)
        .map(|k| argmax((0..3).map(|a| human_payoff(bin_mid(k, bins), a, map))))
        .collect()
}

/// H's payoff line for `action`: total paperclips `A` and staples `B`, worth `A·θ + B·(1 − θ)`.
fn line(action: usize, map: &Map) -> (i64, i64) {
    let (p, q) = H_ACTIONS[action];
    let (pp, qq) = R_ACTIONS[map[action] as usize];
    (i64::from(p + pp), i64::from(q + qq))
}

fn line_at(l: (i64, i64), theta: Ratio) -> Ratio {
    Ratio::new(l.0, 1)
        .mul(theta)
        .add(Ratio::new(l.1, 1).mul(Ratio::new(1, 1).sub(theta)))
}

/// H's pointwise best reply over [0, 1] as exact intervals: breakpoints are the pairwise
/// crossings of the three payoff lines inside (0, 1); each piece takes the argmax at its
/// midpoint (ties to the smaller index); adjacent pieces with one action merge.
pub fn exact_best_reply(map: &Map) -> Vec<(Ratio, Ratio, u8)> {
    let lines: Vec<(i64, i64)> = (0..3).map(|a| line(a, map)).collect();
    let mut points = vec![Ratio::new(0, 1), Ratio::new(1, 1)];
    for a in 0..3 {
        for b in a + 1..3 {
            // A_a θ + B_a (1−θ) = A_b θ + B_b (1−θ)  ⇒  θ = (B_b − B_a) / ((A_a − B_a) − (A_b − B_b)).
            let den = (lines[a].0 - lines[a].1) - (lines[b].0 - lines[b].1);
            if den != 0 {
                let theta = Ratio::new(lines[b].1 - lines[a].1, den);
                if theta > Ratio::new(0, 1) && theta < Ratio::new(1, 1) {
                    points.push(theta);
                }
            }
        }
    }
    points.sort();
    points.dedup();
    let mut out: Vec<(Ratio, Ratio, u8)> = vec![];
    for w in points.windows(2) {
        let mid = w[0].add(w[1]).div(Ratio::new(2, 1));
        let mut best = (0u8, line_at(lines[0], mid));
        for (a, l) in lines.iter().enumerate().skip(1) {
            let v = line_at(*l, mid);
            if v > best.1 {
                best = (a as u8, v);
            }
        }
        match out.last_mut() {
            Some(last) if last.2 == best.0 => last.1 = w[1],
            _ => out.push((w[0], w[1], best.0)),
        }
    }
    out
}

/// ∫₀¹ max_a payoff_a(θ) dθ, exactly: each piece is ∫ (A θ + B (1−θ)) = (A − B)(hi² − lo²)/2 + B (hi − lo).
pub fn expected_total(map: &Map) -> Ratio {
    let mut total = Ratio::new(0, 1);
    for (lo, hi, a) in exact_best_reply(map) {
        let (aa, bb) = line(a as usize, map);
        let slope = Ratio::new(aa - bb, 2).mul(hi.mul(hi).sub(lo.mul(lo)));
        let flat = Ratio::new(bb, 1).mul(hi.sub(lo));
        total = total.add(slope).add(flat);
    }
    total
}

/// The map (and H's exact best reply to it) with the highest expected total reward over R's 27
/// maps; ties keep the first in lexicographic order. The paper computed this "exhaustively".
pub fn optimal_joint() -> (Map, Vec<(Ratio, Ratio, u8)>, Ratio) {
    let mut best: Option<(Map, Vec<(Ratio, Ratio, u8)>, Ratio)> = None;
    for code in 0..27u8 {
        let map = [code / 9, (code / 3) % 3, code % 3];
        let value = expected_total(&map);
        if best.as_ref().is_none_or(|b| value > b.2) {
            best = Some((map, exact_best_reply(&map), value));
        }
    }
    best.unwrap()
}

/// Every map whose best reply's best reply is itself, with the rule it induces.
pub fn mutual_best_responses(bins: usize) -> Vec<(Rule, Map)> {
    (0..27u8)
        .map(|code| [code / 9, (code / 3) % 3, code % 3])
        .filter_map(|map| {
            let rule = human_best_reply(&map, bins);
            (robot_best_reply(&rule) == map).then_some((rule, map))
        })
        .collect()
}

/// A rule read as thresholds: the left edge of the first bin not making (0,2), and of the first
/// making (2,0); 1 when there is none. Equal to the exact thresholds for threshold rules.
pub fn thresholds_of(rule: &[u8]) -> (f64, f64) {
    let bins = rule.len() as f64;
    let low = rule.iter().position(|&a| a != 0).unwrap_or(rule.len()) as f64 / bins;
    let high = rule.iter().position(|&a| a == 2).unwrap_or(rule.len()) as f64 / bins;
    (low, high)
}

/// Bins and map entries differing from the paper's equilibrium pair; 0 at the pair itself.
pub fn distance(rule: &[u8], map: &Map) -> u32 {
    let target = equilibrium_rule(rule.len());
    let bins = rule.iter().zip(&target).filter(|(a, b)| a != b).count();
    let entries = map.iter().zip(&[0u8, 1, 2]).filter(|(a, b)| a != b).count();
    (bins + entries) as u32
}
```

Add `pub mod assistance;` to `crates/sugarscape-core/src/lib.rs` after `pub mod ants;`.

- [ ] **Step 4: Run the tests.**

Run: `cargo test -p sugarscape-core assistance::paperclips`
Expected: 9 passed. If `the_optimal_joint_policy_is_the_teaching_pair` fails on the value, print `expected_total(&[0,1,2])` and recheck the integral by hand: 5863/184 + 1020/184 + 5863/184 = 12746/184 = 6373/92.

- [ ] **Step 5: Lint and commit.**

```bash
cargo fmt --all && cargo clippy -p sugarscape-core --all-targets -- -D warnings
git add crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/assistance
git commit -m "feat(assistance): the paperclip game's closed forms"
git push
```

---

### Task 2: The off-switch game's incentive (`offswitch.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/assistance/offswitch.rs`
- Modify: `crates/sugarscape-core/src/assistance/mod.rs` (add `pub mod offswitch;`)

**Interfaces:**
- Consumes: `crate::portable::exp_neg`.
- Produces:

```rust
pub fn phi(x: f64) -> f64                       // standard normal density
pub fn normal_cdf(x: f64) -> f64                // Φ, portable
pub fn allow_probability(u: f64, beta: f64) -> f64   // π^H(u; β); β = 0 is the rational step
#[derive(Clone, Copy, Debug, PartialEq, Serialize)] pub struct Belief { pub mu: f64, pub sigma: f64 }
pub struct Moments { pub allow: f64, pub variance: f64, pub u_allow: f64 }
pub fn trapezoid(b: Belief, h: f64, f: impl Fn(f64) -> f64) -> f64   // ∫ f(u) N(u; μ, σ²) du over μ ± 9σ
pub fn moments(b: Belief, beta: f64, density: u32) -> Moments
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Incentive { pub delta: f64, pub decomposition: f64, pub rationality: f64, pub prob_correct: f64,
                       pub value_act: f64, pub value_wait: f64, pub value_off: f64 }
pub fn incentive(b: Belief, beta: f64, density: u32) -> Incentive
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)] #[serde(rename_all = "snake_case")]
pub enum Choice { Act, Wait, Off }
pub fn choice(i: &Incentive) -> Choice
pub fn delta_two_point(u1: f64, p1: f64, u2: f64) -> f64      // rational H, two-point prior
pub fn delta_point_mass(u: f64, beta: f64) -> f64             // Corollary 1
pub const SIGMAS: [f64; 20]; pub const BETAS: [f64; 21];
pub struct Surface { pub mu: f64, pub cells: Vec<Incentive> } // cells[b * 20 + s]
pub fn surface(mu: f64, density: u32) -> Surface
pub fn nearest_cell(sigma: f64, beta: f64) -> Option<(usize, usize)>   // (sigma index, beta index)
```

- [ ] **Step 1: Write the failing tests** in a new `offswitch.rs` (tests block only, then the module in Step 3).

```rust
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
        assert_eq!((normal_cdf(10.0), normal_cdf(-10.0)), (1.0, 0.0));
    }

    #[test]
    fn the_logistic_is_symmetric_and_the_rational_human_is_a_step() {
        assert!(close(allow_probability(0.5, 1.0) + allow_probability(-0.5, 1.0), 1.0, 1e-15));
        assert!(close(allow_probability(0.25, 4.0), 1.0 / (1.0 + (-0.0625f64).exp()), 1e-15));
        assert_eq!((allow_probability(-1e-9, 0.0), allow_probability(0.0, 0.0)), (0.0, 1.0));
    }

    #[test]
    fn the_trapezoid_rule_reproduces_the_gaussian_mean_and_second_moment() {
        for (mu, sigma) in [(0.0, 1.0), (0.25, 0.1), (-2.0, 3.0)] {
            let b = Belief { mu, sigma };
            assert!(close(trapezoid(b, sigma / 8.0, |_| 1.0), 1.0, 1e-12));
            assert!(close(trapezoid(b, sigma / 8.0, |u| u), mu, 1e-12));
            assert!(close(trapezoid(b, sigma / 8.0, |u| u * u), mu * mu + sigma * sigma, 1e-11));
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
                assert!(close(i.delta, plus.min(minus), 1e-12), "{mu} {sigma}: {i:?}");
                assert!(i.delta >= 0.0, "{mu} {sigma}: {i:?}");
                if z.abs() <= 4.0 {
                    assert!(i.delta > 0.0, "{mu} {sigma}: {i:?}");
                }
                assert!(close(i.delta, i.decomposition, 1e-12));
            }
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
    fn theorem_2_the_direct_incentive_equals_its_decomposition_and_the_condition_matches_its_sign() {
        for mu in [-1.0, -0.25, 0.0, 0.25, 1.0] {
            for sigma in [0.1, 0.5, 1.0, 2.0] {
                for beta in [0.1, 0.5, 1.0, 4.0] {
                    let b = Belief { mu, sigma };
                    let i = incentive(b, beta, 8);
                    assert!(close(i.delta, i.decomposition, 1e-9), "{mu} {sigma} {beta}: {i:?}");
                    // Eq. (10): E[π̇] = E[π(1 − π)] / β.
                    let m = moments(b, beta, 8);
                    assert!(close(i.rationality, m.variance / beta, 1e-15));
                    // Eq. (6).
                    assert!(close(i.prob_correct, if mu >= 0.0 { 1.0 - m.allow } else { m.allow }, 1e-15));
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
        let rational = incentive(Belief { mu: 0.25, sigma: 1.0 }, 0.0, 8);
        assert!(rational.delta > 0.0 && choice(&rational) == Choice::Wait);
        let toddler = incentive(Belief { mu: 0.25, sigma: 0.1 }, 4.0, 8);
        assert!(toddler.delta < 0.0 && choice(&toddler) == Choice::Act, "{toddler:?}");
        let negative = incentive(Belief { mu: -0.25, sigma: 0.1 }, 4.0, 8);
        assert!(negative.delta < 0.0 && choice(&negative) == Choice::Off);
    }

    #[test]
    fn figure_3_corners_signs_and_regions_hold_on_the_fixed_grid() {
        let n = SIGMAS.len();
        for mu in [-0.25, 0.0, 0.25] {
            let s = surface(mu, 8);
            let at = |si: usize, bi: usize| s.cells[bi * n + si].delta;
            let min = s.cells.iter().map(|c| c.delta).fold(f64::INFINITY, f64::min);
            let max = s.cells.iter().map(|c| c.delta).fold(f64::NEG_INFINITY, f64::max);
            assert_eq!(at(0, BETAS.len() - 1), min, "{mu}: lowest at low σ, high β");
            assert_eq!(at(n - 1, 0), max, "{mu}: highest at high σ, low β");
            for c in &s.cells {
                match mu.partial_cmp(&0.0).unwrap() {
                    std::cmp::Ordering::Equal => assert!(c.delta >= 0.0 && choice(c) == Choice::Wait),
                    std::cmp::Ordering::Less if c.delta <= 0.0 => assert_eq!(choice(c), Choice::Off),
                    std::cmp::Ordering::Greater if c.delta <= 0.0 => assert_eq!(choice(c), Choice::Act),
                    _ => assert_eq!(choice(c), Choice::Wait),
                }
            }
            if mu != 0.0 {
                assert!(s.cells.iter().any(|c| c.delta < 0.0) && s.cells.iter().any(|c| c.delta > 0.0));
            }
        }
        assert_eq!(nearest_cell(0.1, 4.0), Some((1, 20)));
        assert_eq!(nearest_cell(1.0, 0.0), Some((19, 0)));
        assert_eq!(nearest_cell(5.0, 0.0), None);
        assert_eq!(nearest_cell(0.5, 50.0), None);
    }
}
```

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p sugarscape-core assistance::offswitch`
Expected: compile error (module items missing).

- [ ] **Step 3: Write the module** above the tests, and add `pub mod offswitch;` to `mod.rs`.

```rust
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

/// Φ from erf's power series erf(t) = (2/√π)·e^{−t²}·Σ 2ⁿ t^{2n+1}/(2n+1)!!, all terms positive.
pub fn normal_cdf(x: f64) -> f64 {
    let t = x / SQRT_2;
    if t <= -6.5 {
        return 0.0;
    }
    if t >= 6.5 {
        return 1.0;
    }
    let (mut term, mut sum, mut n) = (t, t, 0.0);
    while n < 400.0 {
        n += 1.0;
        term *= 2.0 * t * t / (2.0 * n + 1.0);
        sum += term;
        if term.abs() <= 1e-17 * sum.abs() {
            break;
        }
    }
    let erf = 2.0 / PI.sqrt() * exp_neg(-t * t) * sum;
    0.5 * (1.0 + erf)
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
    let allowed = p1 * u1 * f64::from(u8::from(u1 >= 0.0)) + p2 * u2 * f64::from(u8::from(u2 >= 0.0));
    allowed - (p1 * u1 + p2 * u2).max(0.0)
}

/// Corollary 1: a Dirac belief at `u` gives Δ = u·π^H(u) − max{u, 0} (eq. 4).
pub fn delta_point_mass(u: f64, beta: f64) -> f64 {
    u * allow_probability(u, beta) - u.max(0.0)
}

pub const SIGMAS: [f64; 20] = [
    0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50, 0.55, 0.60, 0.65, 0.70, 0.75,
    0.80, 0.85, 0.90, 0.95, 1.00,
];
pub const BETAS: [f64; 21] = [
    0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 2.2, 2.4, 2.6, 2.8, 3.0, 3.2, 3.4,
    3.6, 3.8, 4.0,
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
```

- [ ] **Step 4: Run the tests.**

Run: `cargo test -p sugarscape-core assistance::offswitch`
Expected: 9 passed. `density_four_and_sixty_four_agree` computes 420 × 2 incentives per μ and takes a few seconds in debug; that is fine.

- [ ] **Step 5: Lint and commit.**

```bash
cargo fmt --all && cargo clippy -p sugarscape-core --all-targets -- -D warnings
git add crates/sugarscape-core/src/assistance
git commit -m "feat(assistance): the off-switch game's incentive, exact and by quadrature"
git push
```

---

### Task 3: Configuration, series and episode dynamics (`config.rs`, `stats.rs`, `dynamics.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/assistance/config.rs`
- Create: `crates/sugarscape-core/src/assistance/stats.rs`
- Create: `crates/sugarscape-core/src/assistance/dynamics.rs`
- Modify: `crates/sugarscape-core/src/assistance/mod.rs`

**Interfaces:**
- Consumes: `crate::config::FieldError`, `crate::schema::{Apply, Param}`, `crate::stats::Series`, `crate::rng::SimRng`, `crate::portable::ln`, Task 1's `paperclips::*`, Task 2's `offswitch::*`.
- Produces:

```rust
// config.rs
pub enum Game { Paperclips, OffSwitch }           // serde snake_case: paperclips, off_switch
pub enum Mode { Static, Iterated, Learning }
pub enum HumanRule { Expert, Equilibrium, Pooling, UniformRandom }
pub enum RobotMap { BestResponse, Fixed }
pub enum Protocol { Prior, Report }
pub struct AssistanceConfig { game, mode, human_rule, robot_map, fixed_0: u32, fixed_1: u32, fixed_2: u32,
    theta_bins: u32, episodes_per_tick: u32, respond_every: u32, fictitious_weight: f64, protocol,
    mu: f64, sigma: f64, beta: f64, actions: u32, observation_noise: f64, quadrature_density: u32 }
impl AssistanceConfig { pub fn validate(&self) -> Result<(), Vec<FieldError>>; pub fn fixed_map(&self) -> Map; pub fn bins(&self) -> usize }
pub fn schema() -> Vec<Param>
// stats.rs
pub const SERIES: [&str; 20];
pub struct AssistanceSnapshot { pub tick: u64, + one f64 per series }   // Default: NaN; impl Series
pub fn series_names() -> Vec<String>
// dynamics.rs
pub fn standard_normal(rng: &mut SimRng) -> f64
pub struct PaperclipEpisode { pub theta: f64, pub bin: usize, pub human: u8, pub robot: u8, pub reward: f64, pub regret: f64 }
pub fn paperclip_episode(rng: &mut SimRng, rule: &[u8], map: &Map) -> PaperclipEpisode
pub struct OffSwitchEpisode { pub u: f64, pub allowed: bool, pub choice: Choice, pub value: f64, pub incentive: Incentive }
pub fn prior_episode(rng: &mut SimRng, belief: Belief, beta: f64, incentive: &Incentive) -> OffSwitchEpisode
pub fn report_episode(rng: &mut SimRng, sigma: f64, beta: f64, noise: f64, actions: u32, density: u32) -> OffSwitchEpisode
pub fn posterior_mean(report: f64, noise: f64) -> f64         // report / (1 + noise²), prior N(0, 1)
pub fn posterior_sd(noise: f64) -> f64                         // noise / sqrt(1 + noise²)
pub struct Tables { pub human: [[f64; 3]; 3], pub robot: Vec<[f64; 3]> }   // Clone, Serialize, PartialEq
impl Tables { pub fn uniform(bins: usize) -> Self; pub fn observe(&mut self, bin: usize, human: u8, robot: u8, weight: f64) }
pub fn human_reply_to_tables(t: &Tables) -> Rule
pub fn robot_reply_to_tables(t: &Tables) -> Map
pub fn robot_posterior_from_tables(t: &Tables, action: u8) -> Vec<f64>
```

- [ ] **Step 1: Write the failing tests.** Put these in `dynamics.rs` (tests block) and `config.rs` (tests block); the module bodies come in Step 3.

`config.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_literal_rules_and_validate() {
        let c = AssistanceConfig::default();
        assert_eq!((c.game, c.mode, c.human_rule, c.robot_map), (Game::Paperclips, Mode::Iterated, HumanRule::Expert, RobotMap::BestResponse));
        assert_eq!((c.theta_bins, c.episodes_per_tick, c.respond_every, c.fictitious_weight), (92, 1000, 1, 1.0));
        assert_eq!((c.protocol, c.mu, c.sigma, c.beta, c.actions, c.observation_noise, c.quadrature_density), (Protocol::Prior, 0.0, 1.0, 0.0, 1, 1.0, 8));
        assert!(c.validate().is_ok());
        assert_eq!(c.fixed_map(), [0, 1, 1]);
    }

    #[test]
    fn out_of_range_fields_are_named() {
        let c = AssistanceConfig { fixed_2: 3, theta_bins: 2, sigma: 0.0, beta: 0.005, actions: 0, quadrature_density: 3, ..Default::default() };
        let fields: Vec<String> = c.validate().unwrap_err().into_iter().map(|e| e.field).collect();
        for f in ["fixed_2", "theta_bins", "sigma", "beta", "actions", "quadrature_density"] {
            assert!(fields.contains(&f.to_string()), "{f} missing from {fields:?}");
        }
        assert!(AssistanceConfig { beta: 0.0, ..Default::default() }.validate().is_ok());
        assert!(AssistanceConfig { mu: f64::NAN, ..Default::default() }.validate().is_err());
    }

    #[test]
    fn configs_round_trip_through_json_with_snake_case_tags() {
        let json = r#"{"game":"off_switch","mode":"learning","human_rule":"uniform_random","robot_map":"fixed","protocol":"report"}"#;
        let c: AssistanceConfig = serde_json::from_str(json).unwrap();
        assert_eq!((c.game, c.mode, c.human_rule, c.robot_map, c.protocol), (Game::OffSwitch, Mode::Learning, HumanRule::UniformRandom, RobotMap::Fixed, Protocol::Report));
        let back: AssistanceConfig = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert!(serde_json::from_str::<AssistanceConfig>(r#"{"nodes":64}"#).is_err());
    }
}
```

`dynamics.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assistance::paperclips::{equilibrium_rule, expert_rule};
    use crate::rng;

    #[test]
    fn standard_normals_have_unit_moments_and_use_only_uniform_pairs() {
        let mut r = rng::seeded(3);
        let xs: Vec<f64> = (0..200_000).map(|_| standard_normal(&mut r)).collect();
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        let var = xs.iter().map(|x| x * x).sum::<f64>() / xs.len() as f64;
        assert!(mean.abs() < 0.01 && (var - 1.0).abs() < 0.02, "{mean} {var}");
        // Deterministic: the same seed gives the same draws.
        let (mut a, mut b) = (rng::seeded(9), rng::seeded(9));
        assert_eq!(standard_normal(&mut a), standard_normal(&mut b));
    }

    #[test]
    fn paperclip_episodes_pay_both_rounds_and_measure_regret_against_the_best_deployment() {
        let mut r = rng::seeded(1);
        let e = paperclip_episode(&mut r, &expert_rule(92), &[0, 1, 2]);
        let expected_h = if e.theta < 0.5 { 2.0 * (1.0 - e.theta) } else { 2.0 * e.theta };
        let expected_r = if e.theta < 0.5 { 90.0 * (1.0 - e.theta) } else { 90.0 * e.theta };
        assert!((e.reward - expected_h - expected_r).abs() < 1e-12);
        assert_eq!(e.bin, (e.theta * 92.0) as usize);
        assert_eq!(e.regret, 0.0, "the expert pair deploys the θ-optimal item");
        let e = paperclip_episode(&mut r, &equilibrium_rule(92), &[0, 1, 2]);
        if e.human == 1 {
            let best = (90.0 * e.theta).max(90.0 * (1.0 - e.theta));
            assert!((e.regret - (best - 50.0)).abs() < 1e-12);
        }
    }

    #[test]
    fn draw_budget_is_independent_of_beta() {
        let belief = Belief { mu: 0.25, sigma: 1.0 };
        let rational = incentive(belief, 0.0, 8);
        let noisy = incentive(belief, 2.0, 8);
        let (mut a, mut b) = (rng::seeded(5), rng::seeded(5));
        for _ in 0..50 {
            prior_episode(&mut a, belief, 0.0, &rational);
            prior_episode(&mut b, belief, 2.0, &noisy);
        }
        assert_eq!(a.gen::<f64>(), b.gen::<f64>());
    }

    #[test]
    fn the_rational_human_allows_exactly_the_positive_utilities() {
        let belief = Belief { mu: 0.0, sigma: 1.0 };
        let i = incentive(belief, 0.0, 8);
        let mut r = rng::seeded(2);
        for _ in 0..1000 {
            let e = prior_episode(&mut r, belief, 0.0, &i);
            assert_eq!(e.allowed, e.u >= 0.0);
            assert_eq!(e.choice, Choice::Wait);
            assert_eq!(e.value, if e.allowed { e.u } else { 0.0 });
        }
    }

    #[test]
    fn report_episodes_shrink_the_report_and_pick_the_best_of_k_candidates() {
        assert_eq!(posterior_mean(2.0, 1.0), 1.0);
        assert!((posterior_sd(1.0) - 1.0 / 2f64.sqrt()).abs() < 1e-15);
        let mut r = rng::seeded(4);
        let one: Vec<f64> = (0..2000).map(|_| report_episode(&mut r, 0.7, 0.5, 1.0, 1, 8).u).collect();
        let eight: Vec<f64> = (0..2000).map(|_| report_episode(&mut r, 0.7, 0.5, 1.0, 8, 8).u).collect();
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!(mean(&one).abs() < 0.1, "{}", mean(&one));
        assert!(mean(&eight) > 0.8, "{}", mean(&eight));
    }

    #[test]
    fn uniform_tables_reproduce_the_prior_map_and_observations_move_the_replies() {
        let mut t = Tables::uniform(92);
        assert_eq!(robot_reply_to_tables(&t), [1, 1, 1]);
        assert_eq!(human_reply_to_tables(&t), expert_rule(92));
        // Enough observations of the expert pair and both replies become the paper's.
        for bin in 0..92 {
            let h = expert_rule(92)[bin];
            for _ in 0..100 {
                t.observe(bin, h, [0, 1, 2][h as usize], 1.0);
            }
        }
        assert_eq!(robot_reply_to_tables(&t), [0, 1, 2]);
        assert_eq!(human_reply_to_tables(&t), equilibrium_rule(92));
        let post = robot_posterior_from_tables(&t, 1);
        assert!((post.iter().sum::<f64>() - 1.0).abs() < 1e-12, "an unseen action still gets a posterior");
    }
}
```

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p sugarscape-core assistance::`
Expected: compile errors for the missing modules.

- [ ] **Step 3: Write `config.rs`.**

```rust
//! Every convention is serialized; `game`, `mode` and `theta_bins` apply at reset, the rest live.
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Game {
    Paperclips,
    OffSwitch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Static,
    Iterated,
    Learning,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanRule {
    Expert,
    Equilibrium,
    Pooling,
    UniformRandom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobotMap {
    BestResponse,
    Fixed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Prior,
    Report,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AssistanceConfig {
    pub game: Game,
    pub mode: Mode,
    pub human_rule: HumanRule,
    pub robot_map: RobotMap,
    pub fixed_0: u32,
    pub fixed_1: u32,
    pub fixed_2: u32,
    pub theta_bins: u32,
    pub episodes_per_tick: u32,
    pub respond_every: u32,
    pub fictitious_weight: f64,
    pub protocol: Protocol,
    pub mu: f64,
    pub sigma: f64,
    pub beta: f64,
    pub actions: u32,
    pub observation_noise: f64,
    pub quadrature_density: u32,
}

impl Default for AssistanceConfig {
    fn default() -> Self {
        Self {
            game: Game::Paperclips,
            mode: Mode::Iterated,
            human_rule: HumanRule::Expert,
            robot_map: RobotMap::BestResponse,
            fixed_0: 0,
            fixed_1: 1,
            // Not 2: the schema check probes each integer at its default plus one.
            fixed_2: 1,
            theta_bins: 92,
            episodes_per_tick: 1000,
            respond_every: 1,
            fictitious_weight: 1.0,
            protocol: Protocol::Prior,
            mu: 0.0,
            sigma: 1.0,
            beta: 0.0,
            actions: 1,
            observation_noise: 1.0,
            quadrature_density: 8,
        }
    }
}

impl AssistanceConfig {
    pub fn bins(&self) -> usize {
        self.theta_bins as usize
    }
    pub fn fixed_map(&self) -> [u8; 3] {
        [self.fixed_0 as u8, self.fixed_1 as u8, self.fixed_2 as u8]
    }
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = vec![];
        let mut check = |ok: bool, field: &str, msg: &str| {
            if !ok {
                errors.push(FieldError::new(field, msg));
            }
        };
        for (name, v) in [("fixed_0", self.fixed_0), ("fixed_1", self.fixed_1), ("fixed_2", self.fixed_2)] {
            check(v <= 2, name, "must be a deployment index 0, 1 or 2");
        }
        check((3..=1000).contains(&self.theta_bins), "theta_bins", "must be between 3 and 1000");
        check((1..=100_000).contains(&self.episodes_per_tick), "episodes_per_tick", "must be between 1 and 100000");
        check((1..=1000).contains(&self.respond_every), "respond_every", "must be between 1 and 1000");
        check(self.fictitious_weight.is_finite() && self.fictitious_weight > 0.0 && self.fictitious_weight <= 1000.0, "fictitious_weight", "must be finite, positive and at most 1000");
        check(self.mu.is_finite() && (-10.0..=10.0).contains(&self.mu), "mu", "must be finite and between -10 and 10");
        check(self.sigma.is_finite() && (0.01..=10.0).contains(&self.sigma), "sigma", "must be finite and between 0.01 and 10");
        check(self.beta == 0.0 || (self.beta.is_finite() && (0.01..=100.0).contains(&self.beta)), "beta", "must be 0 (rational) or between 0.01 and 100");
        check((1..=64).contains(&self.actions), "actions", "must be between 1 and 64");
        check(self.observation_noise.is_finite() && (0.01..=10.0).contains(&self.observation_noise), "observation_noise", "must be finite and between 0.01 and 10");
        check((4..=64).contains(&self.quadrature_density), "quadrature_density", "must be between 4 and 64");
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
        Param::choice("Game", "game", "Game", &[("paperclips", "Paperclips"), ("off_switch", "Off-switch")], Reset)
            .with_help("The paperclip game (CIRL 2016, §3.3) or the off-switch game (IJCAI 2017)."),
        Param::choice("Paperclips", "mode", "Mode", &[("static", "Static"), ("iterated", "Iterated best response"), ("learning", "Fictitious play")], Reset)
            .shown_if("game", "paperclips")
            .with_help("Static plays one fixed pair; the dynamics are ours."),
        Param::choice("Paperclips", "human_rule", "Human rule", &[("expert", "Expert"), ("equilibrium", "Teaching (41/92, 51/92)"), ("pooling", "Always one of each"), ("uniform_random", "Uniform random")], Live)
            .shown_if("game", "paperclips")
            .with_help("The human's rule in static mode, or the start of a dynamic."),
        Param::choice("Paperclips", "robot_map", "Robot map", &[("best_response", "Best response"), ("fixed", "Fixed")], Live)
            .shown_if("game", "paperclips"),
        Param::integer("Paperclips", "fixed_0", "After (0, 2)", (0, 2), Live).shown_if("game", "paperclips").and_shown_if("robot_map", "fixed")
            .with_help("Deployment index: 0 = (0, 90), 1 = (50, 50), 2 = (90, 0)."),
        Param::integer("Paperclips", "fixed_1", "After (1, 1)", (0, 2), Live).shown_if("game", "paperclips").and_shown_if("robot_map", "fixed"),
        Param::integer("Paperclips", "fixed_2", "After (2, 0)", (0, 2), Live).shown_if("game", "paperclips").and_shown_if("robot_map", "fixed"),
        Param::integer("Paperclips", "theta_bins", "θ bins", (3, 1000), Reset).shown_if("game", "paperclips")
            .with_help("92 makes the paper's thresholds bin edges."),
        Param::integer("Paperclips", "respond_every", "Respond every", (1, 1000), Live).shown_if("game", "paperclips").and_shown_if("mode", "iterated")
            .with_help("Ticks between best-response rounds."),
        Param::number("Paperclips", "fictitious_weight", "Observation weight", (0.01, 1000.0, 0.01), Live).shown_if("game", "paperclips").and_shown_if("mode", "learning")
            .with_help("Weight of each new observation in the frequency tables; 1 is classic fictitious play."),
        Param::choice("Off-switch", "protocol", "Protocol", &[("prior", "The paper's game"), ("report", "Noisy report (Figure 4)")], Live)
            .shown_if("game", "off_switch")
            .with_help("Prior: U_a is drawn from the belief. Report: a N(0, 1) population, a noisy report, the posterior mean; our reconstruction."),
        Param::number("Off-switch", "mu", "Belief mean μ", (-10.0, 10.0, 0.05), Live).shown_if("game", "off_switch"),
        Param::number("Off-switch", "sigma", "Belief s.d. σ", (0.01, 10.0, 0.01), Live).shown_if("game", "off_switch"),
        Param::number("Off-switch", "beta", "Human temperature β", (0.0, 100.0, 0.01), Live).shown_if("game", "off_switch")
            .with_help("0 is the rational human; larger is noisier (eq. 5)."),
        Param::integer("Off-switch", "actions", "Candidate actions", (1, 64), Live).shown_if("game", "off_switch").and_shown_if("protocol", "report"),
        Param::number("Off-switch", "observation_noise", "Report noise", (0.01, 10.0, 0.01), Live).shown_if("game", "off_switch").and_shown_if("protocol", "report"),
        Param::integer("Off-switch", "quadrature_density", "Quadrature density", (4, 64), Live).shown_if("game", "off_switch")
            .with_help("Trapezoid points per smallest scale of σ and β."),
        Param::integer("Episodes", "episodes_per_tick", "Episodes per tick", (1, 100_000), Live),
    ]
}
```

- [ ] **Step 4: Write `stats.rs`.**

```rust
//! Per-tick means and the current pair; NaN where a series does not apply to the game.
use crate::stats::Series;
use serde::Serialize;

pub const SERIES: [&str; 20] = [
    "reward", "regret", "threshold_low", "threshold_high", "map_0", "map_1", "map_2", "stable_rounds", "distance",
    "delta_exact", "delta_empirical", "prob_correct", "rationality", "value_act", "value_wait", "value_off", "value",
    "share_wait", "share_act", "share_off",
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
            reward: f64::NAN, regret: f64::NAN, threshold_low: f64::NAN, threshold_high: f64::NAN,
            map_0: f64::NAN, map_1: f64::NAN, map_2: f64::NAN, stable_rounds: f64::NAN, distance: f64::NAN,
            delta_exact: f64::NAN, delta_empirical: f64::NAN, prob_correct: f64::NAN, rationality: f64::NAN,
            value_act: f64::NAN, value_wait: f64::NAN, value_off: f64::NAN, value: f64::NAN,
            share_wait: f64::NAN, share_act: f64::NAN, share_off: f64::NAN,
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
```

- [ ] **Step 5: Write `dynamics.rs`.**

```rust
//! Sampled episodes of both games and the two paperclip dynamics (ours): iterated best response
//! uses `paperclips`' exact replies; fictitious play keeps smoothed frequency tables. Every draw
//! comes from the shared stream as an `f64` or a `u32` range.
use super::offswitch::{allow_probability, choice, incentive, Belief, Choice, Incentive};
use super::paperclips::{argmax, bin_mid, deployment_for, reward, Map, Rule, H_ACTIONS, R_ACTIONS};
use crate::{portable, rng::SimRng};
use rand::Rng;
use serde::Serialize;

/// A standard normal by the Box–Muller transform in polar form: two uniforms on (−1, 1),
/// accepted inside the unit disc, then `u·sqrt(−2 ln s / s)`; `sqrt` and `portable::ln` only.
pub fn standard_normal(rng: &mut SimRng) -> f64 {
    loop {
        let u = 2.0 * rng.gen::<f64>() - 1.0;
        let v = 2.0 * rng.gen::<f64>() - 1.0;
        let s = u * u + v * v;
        if s > 0.0 && s < 1.0 {
            return u * (-2.0 * portable::ln(s) / s).sqrt();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaperclipEpisode {
    pub theta: f64,
    pub bin: usize,
    pub human: u8,
    pub robot: u8,
    pub reward: f64,
    pub regret: f64,
}

/// One game: θ uniform, H by its rule's bin, R by its map; both rounds' reward and R's regret
/// against the θ-optimal deployment.
pub fn paperclip_episode(rng: &mut SimRng, rule: &[u8], map: &Map) -> PaperclipEpisode {
    let bins = rule.len();
    let theta = rng.gen::<f64>();
    let bin = ((theta * bins as f64) as usize).min(bins - 1);
    let human = rule[bin];
    let robot = map[human as usize];
    let deployed = reward(theta, R_ACTIONS[robot as usize]);
    let best = R_ACTIONS.iter().map(|&a| reward(theta, a)).fold(f64::NEG_INFINITY, f64::max);
    PaperclipEpisode {
        theta,
        bin,
        human,
        robot,
        reward: reward(theta, H_ACTIONS[human as usize]) + deployed,
        regret: best - deployed,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OffSwitchEpisode {
    pub u: f64,
    pub allowed: bool,
    pub choice: Choice,
    pub value: f64,
    pub incentive: Incentive,
}

fn resolve(u: f64, allowed: bool, incentive: Incentive) -> OffSwitchEpisode {
    let choice = choice(&incentive);
    let value = match choice {
        Choice::Act => u,
        Choice::Wait if allowed => u,
        _ => 0.0,
    };
    OffSwitchEpisode { u, allowed, choice, value, incentive }
}

/// The paper's game: U_a from the belief (which is the prior), H's coin against π^H(U_a; β);
/// the coin is drawn for the rational H too, so the draw budget does not depend on β.
pub fn prior_episode(rng: &mut SimRng, belief: Belief, beta: f64, incentive: &Incentive) -> OffSwitchEpisode {
    let u = belief.mu + belief.sigma * standard_normal(rng);
    let coin = rng.gen::<f64>();
    resolve(u, coin < allow_probability(u, beta), *incentive)
}

pub fn posterior_mean(report: f64, noise: f64) -> f64 {
    report / (1.0 + noise * noise)
}

pub fn posterior_sd(noise: f64) -> f64 {
    noise / (1.0 + noise * noise).sqrt()
}

/// Figure 4's reconstruction (ours): `actions` candidates with U ~ N(0, 1) and reports
/// U + N(0, noise²); R takes the candidate with the highest posterior mean (ties to the first),
/// believes N(mean, sigma²) and plays the game on it.
pub fn report_episode(rng: &mut SimRng, sigma: f64, beta: f64, noise: f64, actions: u32, density: u32) -> OffSwitchEpisode {
    let mut best = (f64::NEG_INFINITY, 0.0);
    for _ in 0..actions {
        let u = standard_normal(rng);
        let report = u + noise * standard_normal(rng);
        let mean = posterior_mean(report, noise);
        if mean > best.0 {
            best = (mean, u);
        }
    }
    let (mean, u) = best;
    let incentive = incentive(Belief { mu: mean, sigma }, beta, density);
    let coin = rng.gen::<f64>();
    resolve(u, coin < allow_probability(u, beta), incentive)
}

/// Fictitious play's frequency tables: H's over R's deployments after each of H's actions;
/// R's over H's actions in each θ-bin (θ is revealed to R after deployment, a modeling choice).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Tables {
    pub human: [[f64; 3]; 3],
    pub robot: Vec<[f64; 3]>,
}

impl Tables {
    pub fn uniform(bins: usize) -> Self {
        Tables { human: [[1.0; 3]; 3], robot: vec![[1.0; 3]; bins] }
    }
    pub fn observe(&mut self, bin: usize, human: u8, robot: u8, weight: f64) {
        self.human[human as usize][robot as usize] += weight;
        self.robot[bin][human as usize] += weight;
    }
}

/// H's best reply to its table: per bin, immediate reward plus the frequency-weighted deployment.
pub fn human_reply_to_tables(t: &Tables) -> Rule {
    let bins = t.robot.len();
    (0..bins)
        .map(|k| {
            let theta = bin_mid(k, bins);
            argmax((0..3).map(|a| {
                let total: f64 = t.human[a].iter().sum();
                let expected: f64 = (0..3).map(|r| t.human[a][r] / total * reward(theta, R_ACTIONS[r])).sum();
                reward(theta, H_ACTIONS[a]) + expected
            }))
        })
        .collect()
}

/// R's posterior over bins after `action`: the uniform prior times each bin's frequency of it.
pub fn robot_posterior_from_tables(t: &Tables, action: u8) -> Vec<f64> {
    let weights: Vec<f64> = t.robot.iter().map(|row| row[action as usize] / row.iter().sum::<f64>()).collect();
    let total: f64 = weights.iter().sum();
    weights.iter().map(|w| w / total).collect()
}

pub fn robot_reply_to_tables(t: &Tables) -> Map {
    let mut map = [0u8; 3];
    for (a, slot) in map.iter_mut().enumerate() {
        let post = robot_posterior_from_tables(t, a as u8);
        let bins = post.len();
        let mean: f64 = post.iter().enumerate().map(|(k, p)| p * bin_mid(k, bins)).sum();
        *slot = deployment_for(mean);
    }
    map
}
```

Add to `mod.rs`:

```rust
mod config;
pub mod dynamics;
mod stats;
pub use config::*;
pub use stats::{series_names, AssistanceSnapshot, SERIES};
```

- [ ] **Step 6: Run the tests.**

Run: `cargo test -p sugarscape-core assistance::`
Expected: all pass (config 3, dynamics 6, plus Tasks 1–2). `report_episodes_shrink…` checks that the best of eight N(0, 1) draws averages well above 0.8 (the expectation is about 1.42 before shrinkage; the realized U of the best report is lower but above 0.8).

- [ ] **Step 7: Lint and commit.**

```bash
cargo fmt --all && cargo clippy -p sugarscape-core --all-targets -- -D warnings
git add crates/sugarscape-core/src/assistance
git commit -m "feat(assistance): config, series and episode dynamics"
git push
```

---

### Task 4: The world, frames, presets, model wiring and goldens

**Files:**
- Create: `crates/sugarscape-core/src/assistance/world.rs`
- Create: `crates/sugarscape-core/src/assistance/view.rs`
- Create: `crates/sugarscape-core/src/assistance/presets.rs`
- Modify: `crates/sugarscape-core/src/assistance/mod.rs`
- Modify: `crates/sugarscape-core/src/model.rs` (every `Auctions` site, listed in Step 5)
- Modify: `crates/sugarscape-core/src/presets.rs:1600` (catalog)
- Modify: `crates/sugarscape-core/src/titles.rs` (TITLES length 430 → 436, six entries after `auctions-persistent`)
- Modify: `crates/sugarscape-core/tests/golden.rs` (six `MODEL_GOLDEN` entries)
- Modify: `crates/sugarscape-cli/src/main.rs` (nothing required: the world never finishes; verify the kind runs)
- Modify: `crates/sugarscape-wasm/tests/web.rs` (golden test, kind in preset list)

**Interfaces:**
- Consumes: Tasks 1–3; `crate::opinions::Canvas` (`clear(wide, tall)`, `put(x, y, [u8; 3])`); `crate::export::history_csv`; `crate::stats::Stats`; `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`.
- Produces:

```rust
pub struct AssistanceWorld { pub config: AssistanceConfig, pub tick: u64, pub stats: Stats<AssistanceSnapshot>, /* private state */ }
impl AssistanceWorld {
    pub fn new(config: AssistanceConfig, seed: u64) -> Result<Self, Vec<FieldError>>;
    pub fn step(&mut self); pub fn run(&mut self, ticks: u32);
    pub fn rule(&self) -> &[u8]; pub fn map(&self) -> Map; pub fn tables(&self) -> &Tables;
    pub fn incentive(&self) -> &Incentive; pub fn surface(&self) -> &Surface;
    pub fn posterior(&self) -> Vec<f64>;   // after the latest observed H action; the prior before any
    pub fn outcome(&self) -> Outcome;
}
pub struct PairChange { pub tick: u64, pub round: u64, pub rule: Rule, pub map: Map }
pub struct Outcome { pub config, pub seed, pub tick, pub rule, pub map, pub thresholds: (f64, f64), pub rounds: u64,
    pub stable_rounds: u64, pub distance: u32, pub history: Vec<PairChange>, pub equilibria: Vec<(Rule, Map)>,
    pub optimal_map: Map, pub optimal_rule: Vec<(f64, f64, u8)>, pub optimal_value: f64,
    pub teaching_thresholds: (f64, f64), pub expert_posterior_mean: f64, pub incentive: Incentive, pub surface: Vec<f64> }
// view.rs
pub const BIN_WIDTH: usize = 2; pub const BAND: usize = 24; pub const POSTERIOR: usize = 48; pub const HEIGHT: usize = 96; pub const CELL: usize = 12;
pub fn size(w: &AssistanceWorld) -> (u32, u32); pub fn render(w, mode: &str, buf: &mut Vec<u8>) -> Result<(), String>; pub fn inspect(w, x: u32, y: u32) -> Result<String, String>
// model.rs
ModelKind::Assistance ("assistance"), ModelConfig::Assistance(AssistanceConfig), ModelWorld::Assistance(Box<AssistanceWorld>)
```

Inspection JSON (consumed by the page in Task 6). Paperclips: `{"game":"paperclips","panel":"human"|"robot"|"posterior","x","y","bin","theta","human_action","human_rewards":[3],"map_cell":n|null,"deployment","deployment_reward","optimal_deployment","regret","posterior","rule":[..],"map":[3],"tick","outcome":{..},"agent":null}`. Off-switch: `{"game":"off_switch","panel":"surface","x","y","sigma","beta","mu","delta","rationality_term","correction_term","prob_correct","rationality","choice":"act"|"wait"|"off","configured":{Incentive},"configured_choice","tick","outcome":{..},"agent":null}`.

- [ ] **Step 1: Write the failing tests** in `world.rs` (tests block; module in Step 3) and the schema test in `config.rs`:

```rust
// config.rs, inside mod tests:
    #[test]
    fn schema_round_trips_and_only_game_mode_and_bins_apply_at_reset() {
        use crate::model::{ModelConfig, ModelWorld};
        for config in [
            ModelConfig::Assistance(AssistanceConfig::default()),
            ModelConfig::Assistance(AssistanceConfig { game: Game::OffSwitch, ..Default::default() }),
        ] {
            crate::schema::check_schema(&schema(), &config, || ModelWorld::new(config.clone(), 1).unwrap());
        }
    }
```

```rust
// world.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assistance::paperclips::{equilibrium_rule, expert_rule};

    fn world(c: AssistanceConfig) -> AssistanceWorld {
        AssistanceWorld::new(c, 1).unwrap()
    }

    #[test]
    fn a_static_expert_world_plays_the_papers_pair_and_records_it_at_tick_zero() {
        let w = world(AssistanceConfig { mode: Mode::Static, episodes_per_tick: 10, ..Default::default() });
        assert_eq!((w.rule(), w.map()), (&expert_rule(92)[..], [0, 1, 2]));
        let s = w.stats.latest().unwrap();
        assert_eq!((s.tick, s.threshold_low, s.threshold_high, s.map_0, s.map_2, s.distance), (0, 0.5, 0.5, 0.0, 2.0, 10.0));
        assert!(s.reward.is_nan() && s.delta_exact.is_nan());
    }

    #[test]
    fn iterated_best_response_from_the_expert_reaches_the_teaching_pair_in_one_round_and_stays() {
        let mut w = world(AssistanceConfig { episodes_per_tick: 10, ..Default::default() });
        w.step();
        assert_eq!((w.rule(), w.map()), (&equilibrium_rule(92)[..], [0, 1, 2]));
        assert_eq!(w.stats.latest().unwrap().distance, 0.0);
        w.run(5);
        assert_eq!(w.stats.latest().unwrap().stable_rounds, 5.0);
        let o = w.outcome();
        assert_eq!(o.history.len(), 2);
        assert_eq!((o.optimal_map, o.teaching_thresholds), ([0, 1, 2], (41.0 / 92.0, 51.0 / 92.0)));
        assert!((o.expert_posterior_mean - 0.25).abs() < 1e-12);
    }

    #[test]
    fn respond_every_delays_the_rounds_and_a_fixed_map_holds_in_static_mode() {
        let mut w = world(AssistanceConfig { respond_every: 3, episodes_per_tick: 10, ..Default::default() });
        w.run(2);
        assert_eq!(w.rule(), &expert_rule(92)[..]);
        w.step();
        assert_eq!(w.rule(), &equilibrium_rule(92)[..]);
        let mut fixed = world(AssistanceConfig { mode: Mode::Static, robot_map: RobotMap::Fixed, fixed_0: 2, fixed_1: 2, fixed_2: 2, episodes_per_tick: 10, ..Default::default() });
        fixed.run(3);
        assert_eq!(fixed.map(), [2, 2, 2]);
        assert!(fixed.stats.latest().unwrap().regret > 0.0);
    }

    #[test]
    fn fictitious_play_starts_from_the_prior_map_and_moves_with_observations() {
        let mut w = world(AssistanceConfig { mode: Mode::Learning, human_rule: HumanRule::UniformRandom, episodes_per_tick: 1000, ..Default::default() });
        assert_eq!(w.map(), [1, 1, 1]);
        assert_eq!(w.rule(), &w.outcome().history[0].rule[..]);
        w.run(50);
        let s = w.stats.latest().unwrap();
        assert!(s.distance.is_finite() && s.stable_rounds >= 0.0);
        assert_ne!(w.map(), [1, 1, 1], "R has seen θ and moved off the prior map");
    }

    #[test]
    fn live_rule_change_restarts_the_pair_and_reset_fields_are_refused() {
        let mut w = world(AssistanceConfig { episodes_per_tick: 10, ..Default::default() });
        w.run(3);
        let err = w.set_config(ModelConfig::Assistance(AssistanceConfig { theta_bins: 50, episodes_per_tick: 10, ..Default::default() })).unwrap_err();
        assert_eq!(err[0].field, "theta_bins");
        assert_eq!(w.rule().len(), 92);
        w.set_config(ModelConfig::Assistance(AssistanceConfig { human_rule: HumanRule::Pooling, episodes_per_tick: 10, ..Default::default() })).unwrap();
        assert_eq!((w.rule(), w.map(), w.outcome().stable_rounds), (&vec![1u8; 92][..], [1, 1, 1], 0));
        assert_eq!(w.outcome().history.last().unwrap().tick, 3);
        let other = w.set_config(ModelConfig::Auctions(crate::auctions::AuctionsConfig::default())).unwrap_err();
        assert_eq!(other[0].field, "model");
    }

    #[test]
    fn the_off_switch_world_reports_exact_and_empirical_incentives_and_recomputes_live() {
        let mut w = world(AssistanceConfig { game: Game::OffSwitch, mu: 0.25, sigma: 1.0, beta: 0.0, episodes_per_tick: 20_000, ..Default::default() });
        w.step();
        let s = w.stats.latest().unwrap().clone();
        assert!((s.delta_exact - 0.286_4).abs() < 1e-3, "{}", s.delta_exact);
        assert!((s.delta_empirical - s.delta_exact).abs() < 0.02, "{} vs {}", s.delta_empirical, s.delta_exact);
        assert_eq!((s.share_wait, s.share_act, s.share_off), (1.0, 0.0, 0.0));
        assert!((s.value - s.value_wait).abs() < 0.02);
        w.set_config(ModelConfig::Assistance(AssistanceConfig { game: Game::OffSwitch, mu: 0.25, sigma: 0.1, beta: 4.0, episodes_per_tick: 20_000, ..Default::default() })).unwrap();
        assert!(w.incentive().delta < 0.0);
        w.step();
        let s = w.stats.latest().unwrap();
        assert_eq!((s.share_wait, s.share_act), (0.0, 1.0));
        assert!(s.reward.is_nan());
    }

    #[test]
    fn the_report_protocol_averages_per_episode_incentives() {
        let mut w = world(AssistanceConfig { game: Game::OffSwitch, protocol: Protocol::Report, sigma: 0.7, beta: 0.5, actions: 4, episodes_per_tick: 200, ..Default::default() });
        w.step();
        let s = w.stats.latest().unwrap();
        assert!(s.delta_exact.is_finite() && s.value.is_finite());
        assert!((s.share_wait + s.share_act + s.share_off - 1.0).abs() < 1e-12);
        assert!(s.share_wait > 0.0 && s.share_wait < 1.0, "{s:?}");
    }

    #[test]
    fn two_worlds_with_one_seed_match_at_every_tick_and_the_fingerprint_moves() {
        for c in [
            AssistanceConfig { episodes_per_tick: 50, ..Default::default() },
            AssistanceConfig { mode: Mode::Learning, episodes_per_tick: 50, ..Default::default() },
            AssistanceConfig { game: Game::OffSwitch, beta: 1.0, episodes_per_tick: 50, ..Default::default() },
        ] {
            let (mut a, mut b) = (AssistanceWorld::new(c.clone(), 7).unwrap(), AssistanceWorld::new(c, 7).unwrap());
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
    fn frames_have_the_declared_sizes_and_inspection_reads_each_panel() {
        let mut w = world(AssistanceConfig { mode: Mode::Static, episodes_per_tick: 10, ..Default::default() });
        w.step();
        assert_eq!(w.size(), (184, 96));
        let mut buf = vec![];
        w.render("bands", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 184 * 96 * 4);
        assert!(w.render("values", "", &mut buf).is_err());
        let human: serde_json::Value = serde_json::from_str(&w.inspect_json(90, 3).unwrap()).unwrap();
        assert_eq!((human["panel"].as_str(), human["bin"].as_u64(), human["human_action"].as_u64()), (Some("human"), Some(45), Some(0)));
        assert!((human["theta"].as_f64().unwrap() - 45.5 / 92.0).abs() < 1e-12);
        let robot: serde_json::Value = serde_json::from_str(&w.inspect_json(100, 30).unwrap()).unwrap();
        assert_eq!((robot["panel"].as_str(), robot["map_cell"].as_u64()), (Some("robot"), Some(1)));
        let post: serde_json::Value = serde_json::from_str(&w.inspect_json(0, 90).unwrap()).unwrap();
        assert_eq!(post["panel"], "posterior");
        assert!(w.inspect_json(184, 0).is_err());
        let o = world(AssistanceConfig { game: Game::OffSwitch, mu: 0.25, sigma: 0.1, beta: 4.0, ..Default::default() });
        assert_eq!(o.size(), (240, 252));
        o.render("surface", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 240 * 252 * 4);
        // The configured point (σ = 0.1, β = 4) is the top row's second cell, drawn with a gold border.
        let at = 12 * 4; // row 0, x = 12
        assert_eq!(&buf[at..at + 3], &[240, 210, 70]);
        let cell: serde_json::Value = serde_json::from_str(&o.inspect_json(18, 0).unwrap()).unwrap();
        assert_eq!((cell["sigma"].as_f64(), cell["beta"].as_f64(), cell["choice"].as_str()), (Some(0.1), Some(4.0), Some("act")));
        assert!(cell["delta"].as_f64().unwrap() < 0.0);
    }

    #[test]
    fn surface_frame_handles_points_off_the_grid() {
        let o = world(AssistanceConfig { game: Game::OffSwitch, sigma: 5.0, beta: 50.0, ..Default::default() });
        let mut buf = vec![];
        o.render("", "", &mut buf).unwrap();
        assert!(!buf.chunks(4).any(|p| p[..3] == [240, 210, 70]), "no marker off the grid");
    }

    #[test]
    fn posterior_band_under_a_fixed_map_uses_the_prior_off_path() {
        // The pooling rule never makes (0,2) or (2,0); the posterior after (1,1) is the prior.
        let mut w = world(AssistanceConfig { mode: Mode::Static, human_rule: HumanRule::Pooling, episodes_per_tick: 5, ..Default::default() });
        assert!(w.posterior().iter().all(|&p| (p - 1.0 / 92.0).abs() < 1e-15));
        w.step();
        assert!(w.posterior().iter().all(|&p| (p - 1.0 / 92.0).abs() < 1e-15));
        let mut expert = world(AssistanceConfig { mode: Mode::Static, episodes_per_tick: 1, ..Default::default() });
        expert.step();
        let post = expert.posterior();
        assert!(post.iter().take(46).all(|&p| p == 0.0) || post.iter().skip(46).all(|&p| p == 0.0));
    }

    #[test]
    fn csv_exports_name_every_series_and_list_bins_or_cells() {
        let mut w = world(AssistanceConfig { episodes_per_tick: 5, ..Default::default() });
        w.step();
        assert!(w.series_csv().starts_with("tick,reward,regret,threshold_low,"));
        assert_eq!(w.agents_csv().lines().count(), 93);
        let o = world(AssistanceConfig { game: Game::OffSwitch, ..Default::default() });
        assert_eq!(o.agents_csv().lines().count(), 421);
        assert_eq!(o.series("value").unwrap().len(), 1);
        assert!(o.series("nope").is_none());
    }
}
```

- [ ] **Step 2: Run to verify it fails.**

Run: `cargo test -p sugarscape-core assistance::world`
Expected: compile error (no `world` module).

- [ ] **Step 3: Write `world.rs`.**

```rust
//! The assistance world: a tick samples `episodes_per_tick` games, then the paperclip dynamics
//! respond; a snapshot at tick 0 carries the exact values before any sampling.
use super::{
    config::*,
    dynamics::{self, Tables},
    offswitch::{self, Belief, Choice, Incentive, Surface},
    paperclips::{self, Map, Rule},
    stats::AssistanceSnapshot,
    view,
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
pub struct PairChange {
    pub tick: u64,
    pub round: u64,
    pub rule: Rule,
    pub map: Map,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub config: AssistanceConfig,
    pub seed: u64,
    pub tick: u64,
    pub rule: Rule,
    pub map: Map,
    pub thresholds: (f64, f64),
    pub rounds: u64,
    pub stable_rounds: u64,
    pub distance: u32,
    pub history: Vec<PairChange>,
    pub equilibria: Vec<(Rule, Map)>,
    pub optimal_map: Map,
    pub optimal_rule: Vec<(f64, f64, u8)>,
    pub optimal_value: f64,
    pub teaching_thresholds: (f64, f64),
    pub expert_posterior_mean: f64,
    pub incentive: Incentive,
    pub surface: Vec<f64>,
}

#[derive(Clone)]
pub struct AssistanceWorld {
    pub config: AssistanceConfig,
    pub tick: u64,
    pub stats: Stats<AssistanceSnapshot>,
    seed: u64,
    rng: SimRng,
    random_rule: Rule,
    rule: Rule,
    map: Map,
    tables: Tables,
    rounds: u64,
    stable_rounds: u64,
    history: Vec<PairChange>,
    last_human: Option<u8>,
    incentive: Incentive,
    surface: Surface,
    equilibria: Vec<(Rule, Map)>,
}

impl AssistanceWorld {
    pub fn new(config: AssistanceConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let bins = config.bins();
        let random_rule: Rule = (0..bins).map(|_| rng.gen_range(0..3u32) as u8).collect();
        let belief = Belief { mu: config.mu, sigma: config.sigma };
        let incentive = offswitch::incentive(belief, config.beta, config.quadrature_density);
        let surface = offswitch::surface(config.mu, config.quadrature_density);
        let mut w = AssistanceWorld {
            tables: Tables::uniform(bins),
            equilibria: paperclips::mutual_best_responses(bins),
            config,
            tick: 0,
            stats: Stats::default(),
            seed,
            rng,
            random_rule,
            rule: vec![],
            map: [0; 3],
            rounds: 0,
            stable_rounds: 0,
            history: vec![],
            last_human: None,
            incentive,
            surface,
        };
        w.restart_pair();
        let initial = w.exact_snapshot();
        w.stats.push(initial);
        Ok(w)
    }

    pub fn rule(&self) -> &[u8] {
        &self.rule
    }
    pub fn map(&self) -> Map {
        self.map
    }
    pub fn tables(&self) -> &Tables {
        &self.tables
    }
    pub fn incentive(&self) -> &Incentive {
        &self.incentive
    }
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    fn start_rule(&self) -> Rule {
        let bins = self.config.bins();
        match self.config.human_rule {
            HumanRule::Expert => paperclips::expert_rule(bins),
            HumanRule::Equilibrium => paperclips::equilibrium_rule(bins),
            HumanRule::Pooling => paperclips::pooling_rule(bins),
            HumanRule::UniformRandom => self.random_rule.clone(),
        }
    }

    fn start_map(&self, rule: &[u8]) -> Map {
        match (self.config.mode, self.config.robot_map) {
            (Mode::Learning, _) => dynamics::robot_reply_to_tables(&self.tables),
            (_, RobotMap::Fixed) => self.config.fixed_map(),
            (_, RobotMap::BestResponse) => paperclips::robot_best_reply(rule),
        }
    }

    /// The pair at its start (at reset, or after a live change of a start field), recorded.
    fn restart_pair(&mut self) {
        self.rule = self.start_rule();
        self.map = self.start_map(&self.rule.clone());
        self.stable_rounds = 0;
        self.history.push(PairChange { tick: self.tick, round: self.rounds, rule: self.rule.clone(), map: self.map });
    }

    /// One best-response round: a changed pair is recorded, an unchanged one counts as stable.
    fn round(&mut self, rule: Rule, map: Map) {
        self.rounds += 1;
        if rule == self.rule && map == self.map {
            self.stable_rounds += 1;
        } else {
            self.stable_rounds = 0;
            self.rule = rule;
            self.map = map;
            self.history.push(PairChange { tick: self.tick, round: self.rounds, rule: self.rule.clone(), map: self.map });
        }
    }

    /// R's posterior over θ-bins after the latest observed H action (the prior before any).
    pub fn posterior(&self) -> Vec<f64> {
        match self.last_human {
            None => vec![1.0 / self.config.bins() as f64; self.config.bins()],
            Some(a) if self.config.mode == Mode::Learning => dynamics::robot_posterior_from_tables(&self.tables, a),
            Some(a) => paperclips::posterior(&self.rule, a),
        }
    }

    /// The exact fields of a snapshot: the pair for paperclips, the incentive for the off-switch.
    fn exact_snapshot(&self) -> AssistanceSnapshot {
        let mut s = AssistanceSnapshot { tick: self.tick, ..Default::default() };
        match self.config.game {
            Game::Paperclips => {
                let (lo, hi) = paperclips::thresholds_of(&self.rule);
                s.threshold_low = lo;
                s.threshold_high = hi;
                s.map_0 = f64::from(self.map[0]);
                s.map_1 = f64::from(self.map[1]);
                s.map_2 = f64::from(self.map[2]);
                s.stable_rounds = self.stable_rounds as f64;
                s.distance = f64::from(paperclips::distance(&self.rule, &self.map));
            }
            Game::OffSwitch => {
                let i = &self.incentive;
                s.delta_exact = i.delta;
                s.prob_correct = i.prob_correct;
                s.rationality = i.rationality;
                s.value_act = i.value_act;
                s.value_wait = i.value_wait;
                s.value_off = i.value_off;
            }
        }
        s
    }

    pub fn step(&mut self) {
        match self.config.game {
            Game::Paperclips => self.step_paperclips(),
            Game::OffSwitch => self.step_off_switch(),
        }
    }

    fn step_paperclips(&mut self) {
        let n = self.config.episodes_per_tick;
        let (mut reward, mut regret) = (0.0, 0.0);
        for _ in 0..n {
            let e = dynamics::paperclip_episode(&mut self.rng, &self.rule, &self.map);
            reward += e.reward;
            regret += e.regret;
            if self.config.mode == Mode::Learning {
                self.tables.observe(e.bin, e.human, e.robot, self.config.fictitious_weight);
            }
            self.last_human = Some(e.human);
        }
        self.tick += 1;
        match self.config.mode {
            Mode::Static => {}
            Mode::Iterated => {
                if self.tick % u64::from(self.config.respond_every) == 0 {
                    let rule = paperclips::human_best_reply(&self.map, self.config.bins());
                    let map = paperclips::robot_best_reply(&rule);
                    self.round(rule, map);
                }
            }
            Mode::Learning => {
                let rule = dynamics::human_reply_to_tables(&self.tables);
                let map = dynamics::robot_reply_to_tables(&self.tables);
                self.round(rule, map);
            }
        }
        let mut s = self.exact_snapshot();
        s.reward = reward / f64::from(n);
        s.regret = regret / f64::from(n);
        self.stats.push(s);
    }

    fn step_off_switch(&mut self) {
        let n = self.config.episodes_per_tick;
        let c = self.config.clone();
        let belief = Belief { mu: c.mu, sigma: c.sigma };
        let (mut u_sum, mut allowed_u, mut value) = (0.0, 0.0, 0.0);
        let mut shares = [0u32; 3];
        let (mut delta, mut prob_correct, mut rationality, mut value_act, mut value_wait) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for _ in 0..n {
            let e = match c.protocol {
                Protocol::Prior => dynamics::prior_episode(&mut self.rng, belief, c.beta, &self.incentive),
                Protocol::Report => dynamics::report_episode(&mut self.rng, c.sigma, c.beta, c.observation_noise, c.actions, c.quadrature_density),
            };
            u_sum += e.u;
            if e.allowed {
                allowed_u += e.u;
            }
            value += e.value;
            shares[match e.choice {
                Choice::Act => 0,
                Choice::Wait => 1,
                Choice::Off => 2,
            }] += 1;
            delta += e.incentive.delta;
            prob_correct += e.incentive.prob_correct;
            rationality += e.incentive.rationality;
            value_act += e.incentive.value_act;
            value_wait += e.incentive.value_wait;
        }
        self.tick += 1;
        let div = f64::from(n);
        let s = AssistanceSnapshot {
            tick: self.tick,
            delta_exact: delta / div,
            delta_empirical: allowed_u / div - (u_sum / div).max(0.0),
            prob_correct: prob_correct / div,
            rationality: rationality / div,
            value_act: value_act / div,
            value_wait: value_wait / div,
            value_off: 0.0,
            value: value / div,
            share_act: f64::from(shares[0]) / div,
            share_wait: f64::from(shares[1]) / div,
            share_off: f64::from(shares[2]) / div,
            ..Default::default()
        };
        self.stats.push(s);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    pub fn outcome(&self) -> Outcome {
        let (optimal_map, optimal_rule, optimal_value) = paperclips::optimal_joint();
        let expert = paperclips::expert_rule(self.config.bins());
        Outcome {
            config: self.config.clone(),
            seed: self.seed,
            tick: self.tick,
            rule: self.rule.clone(),
            map: self.map,
            thresholds: paperclips::thresholds_of(&self.rule),
            rounds: self.rounds,
            stable_rounds: self.stable_rounds,
            distance: paperclips::distance(&self.rule, &self.map),
            history: self.history.clone(),
            equilibria: self.equilibria.clone(),
            optimal_map,
            optimal_rule: optimal_rule.into_iter().map(|(lo, hi, a)| (lo.to_f64(), hi.to_f64(), a)).collect(),
            optimal_value: optimal_value.to_f64(),
            teaching_thresholds: (41.0 / 92.0, 51.0 / 92.0),
            expert_posterior_mean: paperclips::posterior_mean(&paperclips::posterior(&expert, 0)),
            incentive: self.incentive,
            surface: self.surface.cells.iter().map(|c| c.delta).collect(),
        }
    }
}

impl Model for AssistanceWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Assistance(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        AssistanceWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        2
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({
            "config": self.config, "seed": self.seed, "tick": self.tick, "rule": self.rule, "map": self.map,
            "tables": self.tables, "rounds": self.rounds, "stable_rounds": self.stable_rounds,
            "history": self.history, "last_human": self.last_human,
        });
        let bytes = serde_json::to_vec(&state).unwrap();
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
        let mut out = String::new();
        match self.config.game {
            Game::Paperclips => {
                out.push_str("bin,theta,human_action,deployment,posterior\n");
                let post = self.posterior();
                for (k, &a) in self.rule.iter().enumerate() {
                    writeln!(out, "{k},{},{a},{},{}", paperclips::bin_mid(k, self.rule.len()), self.map[a as usize], post[k]).unwrap();
                }
            }
            Game::OffSwitch => {
                out.push_str("sigma,beta,delta,rationality,prob_correct,choice\n");
                for (b, &beta) in offswitch::BETAS.iter().enumerate() {
                    for (s, &sigma) in offswitch::SIGMAS.iter().enumerate() {
                        let c = &self.surface.cells[b * offswitch::SIGMAS.len() + s];
                        writeln!(out, "{sigma},{beta},{},{},{},{:?}", c.delta, c.rationality, c.prob_correct, offswitch::choice(c)).unwrap();
                    }
                }
            }
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
        let ModelConfig::Assistance(next) = next else {
            return Err(wrong_model(ModelKind::Assistance, &next));
        };
        next.validate()?;
        let errors: Vec<FieldError> = [
            (next.game != self.config.game, "game"),
            (next.mode != self.config.mode, "mode"),
            (next.theta_bins != self.config.theta_bins, "theta_bins"),
        ]
        .into_iter()
        .filter(|(changed, _)| *changed)
        .map(|(_, field)| FieldError::new(field, "changes only on reset"))
        .collect();
        if !errors.is_empty() {
            return Err(errors);
        }
        let old = std::mem::replace(&mut self.config, next);
        let c = self.config.clone();
        if c.mode != Mode::Learning && (c.human_rule, c.robot_map, c.fixed_map()) != (old.human_rule, old.robot_map, old.fixed_map()) {
            self.restart_pair();
        }
        if (c.mu, c.sigma, c.beta, c.quadrature_density) != (old.mu, old.sigma, old.beta, old.quadrature_density) {
            self.incentive = offswitch::incentive(Belief { mu: c.mu, sigma: c.sigma }, c.beta, c.quadrature_density);
        }
        if (c.mu, c.quadrature_density) != (old.mu, old.quadrature_density) {
            self.surface = offswitch::surface(c.mu, c.quadrature_density);
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Write `view.rs`.**

```rust
//! Frames and inspection. Paperclips: three bands over θ-bins (H's rule with the paper's
//! thresholds ticked, R's map, R's posterior after the latest observation; in learning mode the
//! first two bands are shaded by the tables' frequencies). Off-switch: Figure 3's surface at the
//! configured μ, Δ over the fixed σ × β grid with the zero boundary and a marker at (σ, β).
use super::{
    offswitch::{self, BETAS, SIGMAS},
    paperclips::{self, H_ACTIONS, R_ACTIONS},
    world::AssistanceWorld,
    Game, Mode,
};
use crate::opinions::Canvas;
use serde_json::json;

pub const BIN_WIDTH: usize = 2;
pub const BAND: usize = 24;
pub const POSTERIOR: usize = 48;
pub const HEIGHT: usize = 2 * BAND + POSTERIOR;
pub const CELL: usize = 12;
/// (0,2) and (0,90) blue; (1,1) and (50,50) violet; (2,0) and (90,0) orange.
const ACTION_COLORS: [[u8; 3]; 3] = [[70, 130, 230], [160, 110, 200], [235, 150, 60]];
const INK: [u8; 3] = [20, 20, 20];
const MARK: [u8; 3] = [240, 210, 70];
const BAR: [u8; 3] = [220, 220, 220];

pub fn size(w: &AssistanceWorld) -> (u32, u32) {
    match w.config.game {
        Game::Paperclips => ((w.config.bins() * BIN_WIDTH) as u32, HEIGHT as u32),
        Game::OffSwitch => ((SIGMAS.len() * CELL) as u32, (BETAS.len() * CELL) as u32),
    }
}

fn shade(c: [u8; 3], f: f64) -> [u8; 3] {
    let f = 0.3 + 0.7 * f.clamp(0.0, 1.0);
    [(f64::from(c[0]) * f) as u8, (f64::from(c[1]) * f) as u8, (f64::from(c[2]) * f) as u8]
}

pub fn render(w: &AssistanceWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "bands", "surface"].contains(&mode) {
        return Err(format!("unknown color mode {mode:?} (expected bands or surface)"));
    }
    let (wide, tall) = size(w);
    let mut canvas = Canvas { buf, wide: 0 };
    canvas.clear(wide as usize, tall as usize);
    match w.config.game {
        Game::Paperclips => bands(w, &mut canvas),
        Game::OffSwitch => surface(w, &mut canvas),
    }
    Ok(())
}

fn bands(w: &AssistanceWorld, canvas: &mut Canvas) {
    let bins = w.config.bins();
    let wide = bins * BIN_WIDTH;
    let learning = w.config.mode == Mode::Learning;
    for (k, &a) in w.rule().iter().enumerate() {
        let strength = if learning {
            let row = &w.tables().robot[k];
            row[a as usize] / row.iter().sum::<f64>()
        } else {
            1.0
        };
        let color = shade(ACTION_COLORS[a as usize], strength);
        for dx in 0..BIN_WIDTH {
            for y in 0..BAND {
                canvas.put(k * BIN_WIDTH + dx, y, color);
            }
        }
    }
    for t in [41.0 / 92.0, 0.5, 51.0 / 92.0] {
        let x = ((t * wide as f64).round() as usize).min(wide - 1);
        for y in 0..4 {
            canvas.put(x, y, INK);
        }
    }
    for (a, &d) in w.map().iter().enumerate() {
        let strength = if learning {
            let row = &w.tables().human[a];
            row[d as usize] / row.iter().sum::<f64>()
        } else {
            1.0
        };
        let color = shade(ACTION_COLORS[d as usize], strength);
        let (x0, x1) = (a * wide / 3, (a + 1) * wide / 3);
        for x in x0..x1 {
            for y in BAND..2 * BAND {
                canvas.put(x, y, if x == x0 && a > 0 { INK } else { color });
            }
        }
    }
    let post = w.posterior();
    let top = post.iter().copied().fold(0.0, f64::max).max(1e-300);
    for (k, p) in post.iter().enumerate() {
        let h = ((p / top) * POSTERIOR as f64).round() as usize;
        for dx in 0..BIN_WIDTH {
            for y in 0..h {
                canvas.put(k * BIN_WIDTH + dx, HEIGHT - 1 - y, BAR);
            }
        }
    }
}

/// Δ on a diverging scale saturating at ±0.3: red below zero, blue above.
fn delta_color(delta: f64) -> [u8; 3] {
    let f = (delta.abs() / 0.3).clamp(0.0, 1.0);
    if delta < 0.0 {
        [(60.0 + 170.0 * f) as u8, (60.0 - 20.0 * f) as u8, 60]
    } else {
        [60, (70.0 + 60.0 * f) as u8, (80.0 + 170.0 * f) as u8]
    }
}

fn surface(w: &AssistanceWorld, canvas: &mut Canvas) {
    let n = SIGMAS.len();
    let rows = BETAS.len();
    let cells = &w.surface().cells;
    let marker = offswitch::nearest_cell(w.config.sigma, w.config.beta);
    let negative = |b: usize, s: usize| cells[b * n + s].delta <= 0.0;
    for b in 0..rows {
        for s in 0..n {
            let color = delta_color(cells[b * n + s].delta);
            let right = s + 1 < n && negative(b, s + 1) != negative(b, s);
            let below = b > 0 && negative(b - 1, s) != negative(b, s);
            let row = rows - 1 - b;
            for dy in 0..CELL {
                for dx in 0..CELL {
                    let edge = (right && dx + 1 == CELL) || (below && dy + 1 == CELL);
                    let mark = marker == Some((s, b)) && (dx == 0 || dy == 0 || dx + 1 == CELL || dy + 1 == CELL);
                    canvas.put(s * CELL + dx, row * CELL + dy, if mark { MARK } else if edge { INK } else { color });
                }
            }
        }
    }
}

pub fn inspect(w: &AssistanceWorld, x: u32, y: u32) -> Result<String, String> {
    let (wide, tall) = size(w);
    if x >= wide || y >= tall {
        return Err(format!("({x}, {y}) is not in the frame"));
    }
    let (x, y) = (x as usize, y as usize);
    match w.config.game {
        Game::Paperclips => {
            let bins = w.config.bins();
            let bin = x / BIN_WIDTH;
            let theta = paperclips::bin_mid(bin, bins);
            let human = w.rule()[bin];
            let deployment = w.map()[human as usize];
            let human_rewards: Vec<f64> = H_ACTIONS.iter().map(|&a| paperclips::reward(theta, a)).collect();
            let deployment_rewards: Vec<f64> = R_ACTIONS.iter().map(|&a| paperclips::reward(theta, a)).collect();
            let best = deployment_rewards.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let panel = if y < BAND { "human" } else if y < 2 * BAND { "robot" } else { "posterior" };
            let map_cell = (panel == "robot").then(|| (x * 3 / wide as usize).min(2));
            Ok(json!({
                "game": "paperclips", "panel": panel, "x": x, "y": y, "bin": bin, "theta": theta,
                "human_action": human, "human_rewards": human_rewards, "map_cell": map_cell,
                "deployment": deployment, "deployment_reward": deployment_rewards[deployment as usize],
                "optimal_deployment": paperclips::deployment_for(theta),
                "regret": best - deployment_rewards[deployment as usize], "posterior": w.posterior()[bin],
                "rule": w.rule(), "map": w.map(), "tick": w.tick, "outcome": w.outcome(), "agent": null,
            })
            .to_string())
        }
        Game::OffSwitch => {
            let s = x / CELL;
            let b = BETAS.len() - 1 - y / CELL;
            let cell = &w.surface().cells[b * SIGMAS.len() + s];
            Ok(json!({
                "game": "off_switch", "panel": "surface", "x": x, "y": y, "sigma": SIGMAS[s], "beta": BETAS[b],
                "mu": w.config.mu, "delta": cell.delta, "rationality_term": SIGMAS[s] * SIGMAS[s] * cell.rationality,
                "correction_term": w.config.mu.abs() * cell.prob_correct, "prob_correct": cell.prob_correct,
                "rationality": cell.rationality, "choice": offswitch::choice(cell), "configured": w.incentive(),
                "configured_choice": offswitch::choice(w.incentive()), "tick": w.tick, "outcome": w.outcome(), "agent": null,
            })
            .to_string())
        }
    }
}
```

- [ ] **Step 5: Write `presets.rs`, finish `mod.rs`, wire the kind.**

`presets.rs`:

```rust
//! Six source-labelled presets; descriptions carry the measured results after the survey runs.
use super::config::*;
use crate::{model::ModelConfig, presets::ModelPreset};

const CIRL: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2016, NeurIPS, arXiv 1606.03137 §3.3 and appendix §6";
const OFF_SWITCH: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2017, IJCAI, arXiv 1611.08219";

pub fn presets() -> Vec<ModelPreset> {
    let p = |id, name, source, description, config| ModelPreset { id, name, source, description, config: ModelConfig::Assistance(config) };
    vec![
        p("assistance-paperclips-expert", "The human demonstrates and the robot copies, and at 0.49 that goes wrong", CIRL,
          "Static: the expert rule (two staples below θ = 1/2, two paperclips above) and R's best reply (0,90), (50,50), (90,0); the posterior after two staples is uniform on [0, 1/2), so R acts as θ = 1/4. At θ = 0.49 the human would rather R made 50 of each (Theorem 3).",
          AssistanceConfig { mode: Mode::Static, ..Default::default() }),
        p("assistance-paperclips-teaching", "The human makes one of each to teach the robot", CIRL,
          "Static: the paper's equilibrium rule, one of each for θ in [41/92, 51/92], with R's best reply. Expected total reward 6373/92 ≈ 69.27 against 69 for the expert pair, computed exactly.",
          AssistanceConfig { mode: Mode::Static, human_rule: HumanRule::Equilibrium, ..Default::default() }),
        p("assistance-paperclips-iterated", "Best replies, turn by turn, settle on a teaching rule", CIRL,
          "Iterated best response from the expert pair, one round per tick in the paper's order (H then R, ties to the smaller action, off-path actions resolved by the prior); our dynamic. Series distance counts bins and map entries away from the paper's equilibrium.",
          AssistanceConfig::default()),
        p("assistance-paperclips-learning", "Two learners find an equilibrium, not always the paper's", CIRL,
          "Fictitious play (our rule): both players best-respond to smoothed frequency tables of each other's play; θ is revealed to R after deployment. H starts from a uniform random rule over bins, R from the prior's map (50,50). Common-payoff games are potential games, so play converges; which pair it reaches is the question.",
          AssistanceConfig { mode: Mode::Learning, human_rule: HumanRule::UniformRandom, ..Default::default() }),
        p("assistance-offswitch-rational", "An uncertain robot lets itself be switched off", OFF_SWITCH,
          "The paper's game with a rational human (β = 0) and belief N(1/4, 1): Δ = 0.2864 > 0 (Theorem 1), so R waits; the frame is Figure 3's surface at μ = 1/4 with the configured point marked.",
          AssistanceConfig { game: Game::OffSwitch, mu: 0.25, sigma: 1.0, beta: 0.0, ..Default::default() }),
        p("assistance-offswitch-toddler", "A confident robot facing an erratic human bypasses the switch", OFF_SWITCH,
          "The paper's toddler corner: belief N(1/4, 0.1²) and a Boltzmann human at β = 4. Theorem 2 gives Δ ≈ −0.12 < 0 and E[U_a] > 0, so R executes without asking (Figure 3, right, top left).",
          AssistanceConfig { game: Game::OffSwitch, mu: 0.25, sigma: 0.1, beta: 4.0, ..Default::default() }),
    ]
}
```

`mod.rs`, complete:

```rust
//! Assistance games (milestone 39): the paperclip game (Hadfield-Menell, Dragan, Abbeel and
//! Russell, NeurIPS 2016, §3.3 and appendix §6) and the off-switch game (IJCAI 2017), exact
//! where the papers are exact, with two equilibrium dynamics that are ours.
mod config;
pub mod dynamics;
pub mod offswitch;
pub mod paperclips;
mod presets;
mod stats;
mod view;
mod world;
pub use config::*;
pub use presets::presets;
pub use stats::{series_names, AssistanceSnapshot, SERIES};
pub use world::{AssistanceWorld, Outcome, PairChange};
```

`model.rs` edits, each next to the `Auctions` line it mirrors:
1. `use crate::assistance::{AssistanceConfig, AssistanceWorld};` after the `ants` import.
2. `enum ModelKind`: `Assistance,` after `Auctions,`; `ALL: [ModelKind; 33]` with `ModelKind::Assistance,` after `ModelKind::Auctions,`; `as_str`: `ModelKind::Assistance => "assistance",`; `schema()`: `ModelKind::Assistance => crate::assistance::schema(),`.
3. `enum ModelConfig`: `Assistance(AssistanceConfig),`; `enum Tagged`: `Assistance(&'a AssistanceConfig),`; `Serialize`: `ModelConfig::Assistance(c) => Tagged::Assistance(c).serialize(s),`; `kind()`: `ModelConfig::Assistance(_) => ModelKind::Assistance,`; `from_json`: a `"assistance"` arm like `"auctions"`; the unknown-model message gains `assistance` after `auctions`; `validate`, `with_path`: arms; `max_ticks`: `| ModelConfig::Assistance(_)` in the `None` group; `series_names`: `ModelConfig::Assistance(_) => crate::assistance::series_names(),`.
4. `enum ModelWorld`: `Assistance(Box<AssistanceWorld>),`; `new`: `ModelConfig::Assistance(c) => ModelWorld::Assistance(Box::new(AssistanceWorld::new(c, seed)?)),`; `kind`, `model`, `model_mut` arms; checkpoint: `ModelWorld::Assistance(w) => copy_without_history!(Assistance, w),` (the world pushes a snapshot at tick 0, so history index equals tick as the macro assumes); restore: `(ModelWorld::Assistance(live), ModelWorld::Assistance(kept)) => restore_into!(live, kept),`.
5. The kinds test near line 1911: add `"assistance"` after `"auctions"`.

`presets.rs:1600`: `out.extend(crate::assistance::presets());` after the auctions line.

`titles.rs`: `pub const TITLES: [(&str, &str); 436]` and, after `("auctions-persistent", …)`:

```rust
    ("assistance-paperclips-expert", "The human demonstrates and the robot copies, and at 0.49 that goes wrong"),
    ("assistance-paperclips-teaching", "The human makes one of each to teach the robot"),
    ("assistance-paperclips-iterated", "Best replies, turn by turn, settle on a teaching rule"),
    ("assistance-paperclips-learning", "Two learners find an equilibrium, not always the paper's"),
    ("assistance-offswitch-rational", "An uncertain robot lets itself be switched off"),
    ("assistance-offswitch-toddler", "A confident robot facing an erratic human bypasses the switch"),
```

- [ ] **Step 6: Run the core tests.**

Run: `cargo test -p sugarscape-core assistance:: && cargo test -p sugarscape-core --lib model:: && cargo test -p sugarscape-core --lib presets::`
Expected: all pass, including `schema_round_trips_and_only_game_mode_and_bins_apply_at_reset` (if it reports a Live field refused, the `set_config` comparison list is wrong; if a Reset field accepted, the refusal list is).

- [ ] **Step 7: Record the goldens.** Run `cargo test -p sugarscape-core --release --test golden print_golden -- --ignored --nocapture`, copy the six `assistance-*` lines printed under `MODEL_GOLDEN:` into `MODEL_GOLDEN` after `("auctions-persistent", …)`, with the comment `// Assistance games: 200 ticks of 1,000 episodes, seed 1.`. Then:

Run: `cargo test -p sugarscape-core --test golden`
Expected: `other_models_are_unchanged` and `every_model_preset_has_a_golden_entry` pass; no earlier entry changed.

- [ ] **Step 8: WASM tests.** In `crates/sugarscape-wasm/tests/web.rs`, after `auction_short_sessions_match_native_protocol_fingerprints`, add:

```rust
#[wasm_bindgen_test]
fn assistance_presets_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the quadrature and the polar
    // Gaussian draw use portable arithmetic only, so WASM repeats the native fingerprints.
    for (id, expected) in [
        ("assistance-paperclips-expert", "0x…"),
        ("assistance-paperclips-teaching", "0x…"),
        ("assistance-paperclips-iterated", "0x…"),
        ("assistance-paperclips-learning", "0x…"),
        ("assistance-offswitch-rational", "0x…"),
        ("assistance-offswitch-toddler", "0x…"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        sim.step(200);
        assert_eq!(sim.fingerprint(), expected, "{id}");
        assert_ne!(sim.render("", "sugar").unwrap(), 0);
    }
    let sim = Sim::new(&preset_json("assistance-offswitch-toddler"), 1, JsValue::NULL).unwrap();
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(18, 0).unwrap()).unwrap();
    assert_eq!(view["choice"], "act");
}
```

Replace each `0x…` by the native fingerprint as 16 lowercase hex digits with leading zeros (the WASM fingerprint string keeps them; see `fingerprint_hex_keeps_leading_zeros`). Also add `assert_eq!(model("assistance-paperclips-expert").as_deref(), Some("assistance"));` to `presets_list_every_model_with_sugarscape_configs_untagged`.

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: pass. A mismatch here means a platform-specific operation slipped in; grep `assistance/` for `.exp(`, `.ln(`, `.cos(`, `.sin(`, `.powf(` and remove it.

- [ ] **Step 9: CLI smoke test.**

```bash
cargo run --release -p sugarscape-cli -- run --preset assistance-paperclips-iterated --ticks 20 --seed 1 --series-csv /tmp/assist.csv 2>&1 | tail -3 && head -3 /tmp/assist.csv
```

(Use the CLI's actual flag names from `cargo run -p sugarscape-cli -- --help`; the kind needs no CLI change because it never finishes.) Expected: a CSV whose `distance` column is 0 from tick 1.

- [ ] **Step 10: Full workspace checks, commit.**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace
git add -A crates
git commit -m "feat(assistance): the Assistance Games kind with presets, frames and goldens"
git push
```

---

### Task 5: Built-in sweeps

**Files:**
- Create: `sweeps/assistance-figure3-neg.json`, `sweeps/assistance-figure3-zero.json`, `sweeps/assistance-figure3-pos.json`, `sweeps/assistance-figure4.json`, `sweeps/assistance-figure4-incentive.json`, `sweeps/assistance-equilibria.json`
- Modify: `crates/sugarscape-core/src/sweep.rs` (`Builtin` list after `auctions-duration`, before `polarity-predators`; the id list in `builtin_sweeps_parse_and_validate` at the same position)
- Modify: `crates/sugarscape-wasm/tests/web.rs` (the builtin id list near line 460, same position)

**Interfaces:**
- Consumes: the `assistance` kind's config paths and series names from Tasks 3–4; the sweep schema (`base.config` or `base.preset`, `x`/`series` axes in shorthand `{path, values}` or full `{label, values:[{at,name,set}]}`, `seeds`, `ticks`, `metric`).
- Produces: six built-in sweep ids. The spec names one Figure 4 sweep with two statistics; a sweep carries one metric, so `assistance-figure4` reads `value` and `assistance-figure4-incentive` reads `delta_exact` with the same points.

- [ ] **Step 1: Write the failing test change.** In `crates/sugarscape-core/src/sweep.rs`, inside `builtin_sweeps_parse_and_validate`, insert after `"auctions-duration",`:

```rust
                "assistance-figure3-neg",
                "assistance-figure3-zero",
                "assistance-figure3-pos",
                "assistance-figure4",
                "assistance-figure4-incentive",
                "assistance-equilibria",
```

Run: `cargo test -p sugarscape-core --lib sweep::tests::builtin_sweeps_parse_and_validate`
Expected: FAIL (the registry lacks the ids).

- [ ] **Step 2: Write the sweeps.** `sweeps/assistance-figure3-neg.json` (the `zero` and `pos` files differ only in `mu`: `0.0` and `0.25`, and in the name's "μ = −1/4"):

```json
{
  "name": "Assistance Games: Figure 3 at μ = −1/4",
  "description": "The off-switch game's exact incentive Δ to wait (Hadfield-Menell et al. 2017, Theorem 2) over the belief standard deviation σ, one line per human temperature β, with the belief mean μ = −1/4. β = 0 is the rational human. Δ is exact, so one tick and one seed suffice; the grid is ours (the paper's axes are unlabeled). Where Δ ≤ 0 the robot switches itself off at μ < 0 and acts without asking at μ > 0.",
  "base": {
    "config": { "model": "assistance", "game": "off_switch", "mu": -0.25, "episodes_per_tick": 1 }
  },
  "x": { "label": "σ", "path": "sigma", "values": [0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 1.0] },
  "series": { "label": "β", "path": "beta", "values": [0, 0.1, 0.2, 0.4, 0.6, 0.8, 1, 1.5, 2, 3, 4] },
  "seeds": { "from": 1, "count": 1 },
  "ticks": 1,
  "metric": { "kind": "final", "series": "delta_exact" }
}
```

`sweeps/assistance-figure4.json`:

```json
{
  "name": "Assistance Games: Figure 4, value against the robot's uncertainty",
  "description": "Our reconstruction of Figure 4 (Hadfield-Menell et al. 2017, §5): a N(0, 1) population of utilities, a report with N(0, 1) noise, the robot's belief centered on the Bayesian posterior mean with the standard deviation σ on the axis, and a Boltzmann human at β = 0.5 (ours; a rational human makes the robot wait at every σ, so the value is then flat). One line per number of candidate actions. The true posterior standard deviation is 1/√2 ≈ 0.707. 100,000 episodes per point, seed 1; the mean realized utility per episode.",
  "base": {
    "config": { "model": "assistance", "game": "off_switch", "protocol": "report", "beta": 0.5, "observation_noise": 1.0, "episodes_per_tick": 1000 }
  },
  "x": { "label": "σ", "path": "sigma", "values": [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.5] },
  "series": { "label": "Candidate actions", "path": "actions", "values": [1, 4, 8] },
  "seeds": { "from": 1, "count": 1 },
  "ticks": 100,
  "metric": { "kind": "window_mean", "series": "value", "from": 1 }
}
```

`sweeps/assistance-figure4-incentive.json`: the same with name "Assistance Games: Figure 4, the incentive against the robot's uncertainty", a description ending "the exact incentive Δ at each episode's belief, averaged", and `"metric": { "kind": "window_mean", "series": "delta_exact", "from": 1 }`.

`sweeps/assistance-equilibria.json`:

```json
{
  "name": "Assistance Games: which equilibrium the dynamics reach",
  "description": "Our extension of the paperclip game (Hadfield-Menell et al. 2016, §3.3): iterated best response and fictitious play from four starts, 20 seeds, 200 ticks of 1,000 games. The statistic is the distance from the paper's equilibrium (bins and map entries that differ; 0 at the teaching pair with thresholds 41/92 and 51/92). Reported without a target.",
  "base": { "preset": "assistance-paperclips-iterated" },
  "x": {
    "label": "Start",
    "values": [
      { "at": 1, "name": "Expert", "set": { "human_rule": "expert" } },
      { "at": 2, "name": "Teaching", "set": { "human_rule": "equilibrium" } },
      { "at": 3, "name": "Pooling", "set": { "human_rule": "pooling" } },
      { "at": 4, "name": "Uniform random", "set": { "human_rule": "uniform_random" } }
    ]
  },
  "series": {
    "label": "Dynamic",
    "values": [
      { "at": 1, "name": "Iterated best response", "set": { "mode": "iterated" } },
      { "at": 2, "name": "Fictitious play", "set": { "mode": "learning" } }
    ]
  },
  "seeds": { "from": 1, "count": 20 },
  "ticks": 200,
  "metric": { "kind": "final", "series": "distance" }
}
```

Register in `sweep.rs` after the `auctions-duration` `Builtin`:

```rust
    Builtin {
        id: "assistance-figure3-neg",
        json: include_str!("../../../sweeps/assistance-figure3-neg.json"),
    },
    Builtin {
        id: "assistance-figure3-zero",
        json: include_str!("../../../sweeps/assistance-figure3-zero.json"),
    },
    Builtin {
        id: "assistance-figure3-pos",
        json: include_str!("../../../sweeps/assistance-figure3-pos.json"),
    },
    Builtin {
        id: "assistance-figure4",
        json: include_str!("../../../sweeps/assistance-figure4.json"),
    },
    Builtin {
        id: "assistance-figure4-incentive",
        json: include_str!("../../../sweeps/assistance-figure4-incentive.json"),
    },
    Builtin {
        id: "assistance-equilibria",
        json: include_str!("../../../sweeps/assistance-equilibria.json"),
    },
```

Add the same six ids after `"auctions-duration",` in `crates/sugarscape-wasm/tests/web.rs`.

- [ ] **Step 3: Run the tests and a short CLI sweep.**

```bash
cargo test -p sugarscape-core --lib sweep::
cargo run --release -p sugarscape-cli -- sweep --builtin assistance-figure3-pos --quiet | head -5
cargo run --release -p sugarscape-cli -- sweep --builtin assistance-equilibria --seeds 2 --ticks 20 --quiet | head -5
```

Expected: tests pass; the Figure 3 sweep prints 220 points in a few seconds; the short equilibria run prints finite distances. (`--seeds`/`--ticks` override as in CI's `fig-iv-10-11` smoke line.)

- [ ] **Step 4: Commit.**

```bash
git add sweeps crates/sugarscape-core/src/sweep.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "feat(assistance): built-in sweeps for Figures 3 and 4 and the equilibria"
git push
```

---

### Task 6: The page

**Files:**
- Modify: `web/src/types.ts` (`ModelKind`, `ModelConfig`, `ModelStats`, `AnyInspection`, `ColorMode`; new interfaces)
- Modify: `web/src/models.ts` (`MODELS`, `MODEL_LABELS`, `modelOf`, `isAssistanceView`, `COLOR_MODES`, `MODEL_OVERLAYS`)
- Modify: `web/src/ui/series-data.ts` (`MODEL_CHARTS.assistance`)
- Create: `web/src/assistance.ts`
- Modify: `web/src/ui/inspect-panel.ts`, `web/src/ui/grid-view.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`
- Create: `web/src/assistance.test.ts`
- Modify: `web/src/determinism.test.ts` (one `GOLDEN_MODELS` entry)

**Interfaces:**
- Consumes: the inspection JSON and series from Task 4; `LegendItem` from `web/src/ui/legend.ts`; `ModelChart` from `series-data.ts`; `SweepForm` from `experiments/form.ts`.
- Produces: `assistanceRows(view: AssistanceInspection): [string, string][]`, `assistanceLegend(config: AssistanceConfig, mode: ColorMode): { status: string; items: LegendItem[] }`, `isAssistanceView(v: AnyInspection): v is AssistanceInspection`.

- [ ] **Step 1: Write the failing tests.** `web/src/assistance.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { finishesUnpredictably, modelOf, ticksLeft, COLOR_MODES } from './models';
import { defaultForm } from './experiments/form';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import { MODEL_CHARTS } from './ui/series-data';
import { assistanceLegend, assistanceRows } from './assistance';
import type { AssistanceConfig, AssistanceInspection, ModelConfig, Preset } from './types';

const paperclips = { model: 'assistance', game: 'paperclips', mode: 'iterated', mu: 0, sigma: 1, beta: 0 } as unknown as AssistanceConfig;
const offSwitch = { ...paperclips, game: 'off_switch', mu: 0.25 } as AssistanceConfig;
const outcome = {
  tick: 3, rule: [0, 1, 2], map: [0, 1, 2], thresholds: [41 / 92, 51 / 92] as [number, number], rounds: 3, stable_rounds: 2, distance: 0,
  history: [], equilibria: [[[0, 1, 2], [0, 1, 2]]] as [number[], number[]][], optimal_map: [0, 1, 2], optimal_rule: [], optimal_value: 6373 / 92,
  teaching_thresholds: [41 / 92, 51 / 92] as [number, number], expert_posterior_mean: 0.25,
  incentive: { delta: 0.2864, decomposition: 0.2864, rationality: 0.3867, prob_correct: 0.4013, value_act: 0.25, value_wait: 0.5364, value_off: 0 }, surface: [],
};

describe('Assistance Games integration', () => {
  it('is a model that never finishes', () => {
    expect(modelOf(paperclips)).toBe('assistance');
    expect([ticksLeft(paperclips, 5), finishesUnpredictably(paperclips)]).toEqual([Infinity, false]);
    expect(COLOR_MODES.assistance.map(([m]) => m)).toEqual(['bands', 'surface']);
  });
  it('suggests Figure 3 for the off-switch game and a fictitious-play sweep for paperclips', () => {
    expect(defaultForm('assistance', offSwitch)).toMatchObject({ x: { path: 'sigma', values: '0.05:1:0.05' }, ticks: 1, metric: { kind: 'final', series: 'delta_exact' } });
    expect(defaultForm('assistance', paperclips)).toMatchObject({ set: { mode: 'learning' }, x: { path: 'fictitious_weight' }, ticks: 200, metric: { kind: 'final', series: 'distance' } });
  });
  it('compares demonstrating against teaching', () => {
    const preset = (id: string, fields: object): Preset => ({ id, title: id, name: id, source: '', description: '', config: { ...paperclips, mode: 'static', ...fields } as ModelConfig });
    const presets = [preset('assistance-paperclips-expert', { human_rule: 'expert' }), preset('assistance-paperclips-teaching', { human_rule: 'equilibrium' })];
    const pair = COMPARE_PRESETS.find((p) => p.id === 'assistance-demonstrate-vs-teach')!;
    expect(comparePresetStates(presets, pair, 3)?.b.config).toMatchObject({ human_rule: 'equilibrium' });
  });
  it('shows each game its own charts', () => {
    const charts = MODEL_CHARTS.assistance;
    expect(charts.map((c) => c.title)).toEqual(['Rewards', 'Human thresholds', 'Robot map', 'Convergence', 'Incentive', 'Decomposition', 'Values', 'Choices']);
    expect(charts.map((c) => c.shown!(paperclips))).toEqual([true, true, true, true, false, false, false, false]);
    expect(charts.map((c) => c.shown!(offSwitch))).toEqual([false, false, false, false, true, true, true, true]);
  });
  it('explains a paperclip bin and an off-switch cell', () => {
    const bin: AssistanceInspection = { game: 'paperclips', panel: 'robot', x: 100, y: 30, bin: 50, theta: 50.5 / 92, human_action: 1, human_rewards: [0.902, 1, 1.098], map_cell: 1, deployment: 1, deployment_reward: 50, optimal_deployment: 2, regret: 50 * 50.5 / 92 - 50 + 0.0, posterior: 0.1, rule: [], map: [0, 1, 2], tick: 3, outcome, agent: null };
    const rows = assistanceRows(bin);
    expect(rows[0][1]).toContain('human makes (1, 1)');
    expect(rows.find(([k]) => k === 'Map cell')?.[1]).toBe('after (1, 1) the robot deploys (50, 50)');
    expect(rows.find(([k]) => k === 'Pair')?.[1]).toContain("the paper's equilibrium");
    const cell: AssistanceInspection = { game: 'off_switch', panel: 'surface', x: 18, y: 0, sigma: 0.1, beta: 4, mu: 0.25, delta: -0.12, rationality_term: 0.0006, correction_term: 0.121, prob_correct: 0.484, rationality: 0.062, choice: 'act', configured: outcome.incentive, configured_choice: 'wait', tick: 0, outcome, agent: null };
    expect(assistanceRows(cell).map(([k]) => k)).toEqual(['Cell', 'Incentive Δ', 'Configured']);
    expect(assistanceRows(cell)[1][1]).toContain('acts without asking');
  });
  it('legends name the bands and the surface axes', () => {
    expect(assistanceLegend(paperclips, 'bands').status).toContain('41/92');
    expect(assistanceLegend(offSwitch, 'surface').status).toContain('μ = 0.25');
    expect(assistanceLegend(offSwitch, 'surface').items.length).toBe(3);
  });
});
```

Run: `cd web && npx vitest run src/assistance.test.ts`
Expected: FAIL (module and types missing).

- [ ] **Step 2: Types.** In `web/src/types.ts`: add `'assistance'` to `ModelKind` after `'auctions'`; `AssistanceConfig` to `ModelConfig`; `AssistanceStats` to `ModelStats`; `AssistanceInspection` to `AnyInspection`; `| 'bands' | 'surface'` to `ColorMode`; and after the auctions interfaces:

```ts
/** Assistance games (Hadfield-Menell et al. 2016, 2017); game, mode and θ bins apply at reset. */
export interface AssistanceConfig {
  model: 'assistance'; game: 'paperclips' | 'off_switch'; mode: 'static' | 'iterated' | 'learning';
  human_rule: 'expert' | 'equilibrium' | 'pooling' | 'uniform_random'; robot_map: 'best_response' | 'fixed';
  fixed_0: number; fixed_1: number; fixed_2: number; theta_bins: number; episodes_per_tick: number; respond_every: number;
  fictitious_weight: number; protocol: 'prior' | 'report'; mu: number; sigma: number; beta: number; actions: number;
  observation_noise: number; quadrature_density: number;
}
export interface AssistanceStats {
  tick: number; reward: number | null; regret: number | null; threshold_low: number | null; threshold_high: number | null;
  map_0: number | null; map_1: number | null; map_2: number | null; stable_rounds: number | null; distance: number | null;
  delta_exact: number | null; delta_empirical: number | null; prob_correct: number | null; rationality: number | null;
  value_act: number | null; value_wait: number | null; value_off: number | null; value: number | null;
  share_wait: number | null; share_act: number | null; share_off: number | null;
}
export interface AssistanceIncentive { delta: number; decomposition: number; rationality: number; prob_correct: number; value_act: number; value_wait: number; value_off: number }
export interface AssistanceOutcome {
  tick: number; rule: number[]; map: number[]; thresholds: [number, number]; rounds: number; stable_rounds: number; distance: number;
  history: { tick: number; round: number; rule: number[]; map: number[] }[]; equilibria: [number[], number[]][];
  optimal_map: number[]; optimal_rule: [number, number, number][]; optimal_value: number; teaching_thresholds: [number, number];
  expert_posterior_mean: number; incentive: AssistanceIncentive; surface: number[];
}
export interface AssistancePaperclipsInspection {
  game: 'paperclips'; panel: 'human' | 'robot' | 'posterior'; x: number; y: number; bin: number; theta: number; human_action: number;
  human_rewards: number[]; map_cell: number | null; deployment: number; deployment_reward: number; optimal_deployment: number; regret: number;
  posterior: number; rule: number[]; map: number[]; tick: number; outcome: AssistanceOutcome; agent: null;
}
export interface AssistanceOffSwitchInspection {
  game: 'off_switch'; panel: 'surface'; x: number; y: number; sigma: number; beta: number; mu: number; delta: number; rationality_term: number;
  correction_term: number; prob_correct: number; rationality: number; choice: 'act' | 'wait' | 'off'; configured: AssistanceIncentive;
  configured_choice: 'act' | 'wait' | 'off'; tick: number; outcome: AssistanceOutcome; agent: null;
}
export type AssistanceInspection = AssistancePaperclipsInspection | AssistanceOffSwitchInspection;
```

- [ ] **Step 3: `models.ts`.** Add `'assistance'` to `MODELS` after `'auctions'`; `assistance: 'Assistance Games',` to `MODEL_LABELS` after `auctions`; `|| tag === 'assistance'` in `modelOf`; import the types; after `isAuctionsView`:

```ts
/** An assistance frame: a θ-bin of the paperclip bands or a cell of the off-switch surface. */
export function isAssistanceView(v: AnyInspection): v is AssistanceInspection {
  return 'game' in v && 'panel' in v && (v.game === 'paperclips' || v.game === 'off_switch');
}
```

`COLOR_MODES`: `assistance: [['bands', 'Rule, map and posterior'], ['surface', 'Δ over σ and β']],`; `MODEL_OVERLAYS`: `assistance: [],`.

- [ ] **Step 4: Charts.** In `web/src/ui/series-data.ts`, after `auctions:`:

```ts
  assistance: [
    { title: 'Rewards', shown: (c) => (c as AssistanceConfig).game === 'paperclips', lines: [{ key: 'reward', label: 'Both rounds (episode mean)', color: '--c1' }, { key: 'regret', label: "Robot's regret against the θ-optimal item", color: '--red' }] },
    { title: 'Human thresholds', shown: (c) => (c as AssistanceConfig).game === 'paperclips', range: [0, 1], lines: [{ key: 'threshold_low', label: 'Low (first bin not making two staples)', color: '--c1' }, { key: 'threshold_high', label: 'High (first bin making two paperclips)', color: '--c2' }] },
    { title: 'Robot map', shown: (c) => (c as AssistanceConfig).game === 'paperclips', range: [0, 2], lines: [{ key: 'map_0', label: 'After (0, 2): 0 = (0,90), 1 = (50,50), 2 = (90,0)', color: '--c1' }, { key: 'map_1', label: 'After (1, 1)', color: '--c2' }, { key: 'map_2', label: 'After (2, 0)', color: '--c3' }] },
    { title: 'Convergence', shown: (c) => (c as AssistanceConfig).game === 'paperclips', lines: [{ key: 'stable_rounds', label: 'Rounds since the pair changed', color: '--c1' }, { key: 'distance', label: "Distance from the paper's equilibrium", color: '--red' }] },
    { title: 'Incentive', shown: (c) => (c as AssistanceConfig).game === 'off_switch', lines: [{ key: 'delta_exact', label: 'Δ exact', color: '--c1' }, { key: 'delta_empirical', label: 'Δ empirical (episode mean)', color: '--c2' }] },
    { title: 'Decomposition', shown: (c) => (c as AssistanceConfig).game === 'off_switch', lines: [{ key: 'rationality', label: 'E[π̇] (rationality)', color: '--c1' }, { key: 'prob_correct', label: 'Pr(C) (correction)', color: '--c2' }] },
    { title: 'Values', shown: (c) => (c as AssistanceConfig).game === 'off_switch', lines: [{ key: 'value_act', label: 'Act (E[U])', color: '--c1' }, { key: 'value_wait', label: 'Wait (E[U·π])', color: '--c2' }, { key: 'value_off', label: 'Off (0)', color: '--muted' }, { key: 'value', label: 'Realized (episode mean)', color: '--c4' }] },
    { title: 'Choices', shown: (c) => (c as AssistanceConfig).game === 'off_switch', range: [0, 1], lines: [{ key: 'share_wait', label: 'Waits for the human', color: '--c1' }, { key: 'share_act', label: 'Acts without asking', color: '--c2' }, { key: 'share_off', label: 'Switches itself off', color: '--red' }] },
  ],
```

Import `AssistanceConfig` in `series-data.ts`. If `ModelChart.shown` takes `Config` rather than `ModelConfig`, use the existing `modelShown` hook as `charts-panel.ts:465` does (`modelShown: c.shown`); check that `shownCharts` filters time charts by `modelShown` for non-sugarscape models and, if it does not, add that filter where `charts.map((c): ChartDef => …)` is built.

- [ ] **Step 5: `assistance.ts`.**

```ts
import type { AssistanceConfig, AssistanceInspection, ColorMode } from './types';
import type { LegendItem } from './ui/legend';

const fmt = (v: number): string => v.toFixed(3);
const H = ['(0, 2)', '(1, 1)', '(2, 0)'];
const R = ['(0, 90)', '(50, 50)', '(90, 0)'];
const CHOICE = { act: 'acts without asking', wait: 'waits for the human', off: 'switches itself off' } as const;

/** Inspect rows: a θ-bin of the paperclip game, or a cell of Figure 3's surface. */
export function assistanceRows(view: AssistanceInspection): [string, string][] {
  if (view.game === 'paperclips') {
    const o = view.outcome;
    const rows: [string, string][] = [
      ['θ bin', `${view.bin} · θ ≈ ${fmt(view.theta)} · human makes ${H[view.human_action]} · immediate rewards ${view.human_rewards.map(fmt).join(', ')}`],
      ['Robot', `deploys ${R[view.deployment]} worth ${fmt(view.deployment_reward)} · θ-optimal ${R[view.optimal_deployment]} · regret ${fmt(view.regret)}`],
      ['Posterior', `${fmt(view.posterior)} on this bin after the latest observation`],
      ['Pair', `thresholds ${fmt(o.thresholds[0])}, ${fmt(o.thresholds[1])} · map ${o.map.map((d) => R[d]).join(', ')} · ${o.distance === 0 ? "the paper's equilibrium" : `${o.distance} from the paper's equilibrium`} · stable ${o.stable_rounds} rounds`],
      ['Paper', `teaching thresholds 41/92, 51/92 · optimal joint value ${fmt(o.optimal_value)} · ${o.equilibria.length} pure mutual best response${o.equilibria.length === 1 ? '' : 's'}`],
    ];
    if (view.panel === 'robot' && view.map_cell !== null) rows.splice(1, 0, ['Map cell', `after ${H[view.map_cell]} the robot deploys ${R[o.map[view.map_cell]]}`]);
    return rows;
  }
  const c = view.configured;
  return [
    ['Cell', `σ = ${fmt(view.sigma)} · β = ${fmt(view.beta)} · μ = ${fmt(view.mu)}`],
    ['Incentive Δ', `${fmt(view.delta)} = σ²E[π̇] ${fmt(view.rationality_term)} − |μ|Pr(C) ${fmt(view.correction_term)} · Pr(C) ${fmt(view.prob_correct)} · robot ${CHOICE[view.choice]}`],
    ['Configured', `Δ ${fmt(c.delta)} · wait ${fmt(c.value_wait)} vs act ${fmt(c.value_act)} vs off 0 · robot ${CHOICE[view.configured_choice]}`],
  ];
}

/** The frame's status line and legend. */
export function assistanceLegend(config: AssistanceConfig, mode: ColorMode): { status: string; items: LegendItem[] } {
  if (config.game === 'off_switch' || mode === 'surface') return {
    status: `Δ over σ (left 0.05 → right 1.00) and β (bottom 0 → top 4) at μ = ${config.mu} · black edges mark Δ = 0 · gold border: the configured (σ, β)`,
    items: [
      { label: 'Δ < 0: the robot bypasses the human (acts if μ > 0, switches off if μ < 0)', mark: { kind: 'swatch', color: '#e62c3c' } },
      { label: 'Δ > 0: the robot waits for the human', mark: { kind: 'swatch', color: '#3c82fa' } },
      { label: 'Configured point · gold border', mark: { kind: 'swatch', color: '#f0d246' } },
    ],
  };
  return {
    status: `Top: the human's rule over θ (left 0 → right 1), ticks at 41/92, 1/2, 51/92 · Middle: the robot's deployment after (0, 2), (1, 1), (2, 0) · Bottom: the robot's posterior after the latest observation${config.mode === 'learning' ? ' · shading: the tables’ frequencies' : ''}`,
    items: [
      { label: 'Two staples / (0, 90)', mark: { kind: 'swatch', color: '#4682e6' } },
      { label: 'One of each / (50, 50)', mark: { kind: 'swatch', color: '#a06ec8' } },
      { label: 'Two paperclips / (90, 0)', mark: { kind: 'swatch', color: '#eb963c' } },
    ],
  };
}
```

- [ ] **Step 6: Wire the panels, Compare and Experiments.**
  - `web/src/ui/inspect-panel.ts`: import `assistanceRows` from `'../assistance'` and `isAssistanceView` from `'../models'`; before the `isAuctionsView(view)` branch add `: isAssistanceView(view) ? assistanceRows(view).map(([k, v]) => h('tr', {}, h('th', {}, k), h('td', {}, v)))`.
  - `web/src/ui/grid-view.ts`: import `assistanceLegend` and `AssistanceConfig`; next to `const auction = …` add `const assistance = this.engine.model === 'assistance' ? assistanceLegend(this.engine.config as AssistanceConfig, this.engine.colorMode) : null;`; make `status` prefer `assistance?.status` and `items` prefer `assistance.items` ahead of `auction`.
  - `web/src/compare-presets.ts`: after the auctions entries add `{ id: 'assistance-demonstrate-vs-teach', label: 'Demonstrating vs teaching — Assistance Games (Compare)', a: 'assistance-paperclips-expert', b: 'assistance-paperclips-teaching' },`.
  - `web/src/experiments/form.ts`: import `AssistanceConfig`; before the `auctions` branch:

```ts
  if (model === 'assistance' && config && (config as AssistanceConfig).game === 'off_switch') {
    return { ...form, description: 'Figure 3 at the configured μ: the exact incentive Δ to wait against the belief standard deviation σ. Δ is exact, so one tick and one seed suffice.', x: { path: 'sigma', values: '0.05:1:0.05' }, seeds: 1, ticks: 1, metric: { ...form.metric, kind: 'final', series: 'delta_exact' } };
  }
  if (model === 'assistance') {
    return { ...form, set: { mode: 'learning' }, description: "Fictitious play (our rule): the distance from the paper's equilibrium after 200 ticks against the weight of each observation.", x: { path: 'fictitious_weight', values: '0.25:2:0.25' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'distance' } };
  }
```

  - `web/src/determinism.test.ts`: add `['assistance-paperclips-iterated', '0x<native fingerprint>']` to `GOLDEN_MODELS` (the 16-digit hex from Task 4).

- [ ] **Step 7: Run the page tests, typecheck and build.**

```bash
cd web && npm run wasm && npx tsc --noEmit && npx vitest run && npm run build
```

Expected: all green, including the generic tests (`preset titles`, `model charts draw only series their model records`, schema forms). Then open `npm run dev`, load each of the six presets from the menu's Assistance Games group, confirm the bands and the surface draw, Inspect answers on each panel, and the Rules panel hides paperclip fields under the off-switch game.

- [ ] **Step 8: Commit.**

```bash
git add web/src
git commit -m "feat(assistance): page support, charts, inspect, compare and experiments"
git push
```

---

### Task 7: Survey, findings, catalog and merge

**Files:**
- Create: `survey/src/claims/assistance.rs`; Modify: `survey/src/claims/mod.rs` (`mod assistance;` and `assistance::claims(),` after auctions)
- Create: `survey/out/assistance/results.md` (tracked, the survey's output for this item)
- Create: `docs/superpowers/specs/2026-10-11-assistance-games-findings.md`
- Modify: `crates/sugarscape-core/src/assistance/presets.rs` (measured results into descriptions, as the auctions presets carry theirs)
- Modify: `docs/papers.md` (queue row 39 → Reproduced row 39), `docs/roadmap.md`, `docs/reproducibility.md` (new "Milestone 39" subsection), `docs/studies/2026-10-11-alignment-program.md` (queue item 39 done, link to findings), `README.md` (a short "Assistance Games" section after Q-learning Auctions)

**Interfaces:**
- Consumes: `sugarscape_core::assistance::{paperclips, offswitch, dynamics, AssistanceConfig, Game, Mode, HumanRule, Protocol}`; `crate::claim::{Claim, Outcome, Source, Verdict, all_of}`; `crate::runner::model_after`.
- Produces: survey claim ids `assistance.expert-threshold`, `assistance.robot-best-reply`, `assistance.posterior-quarter`, `assistance.posterior-sixth`, `assistance.teaching-thresholds`, `assistance.theorem-3`, `assistance.optimal-joint`, `assistance.pooling-not-equilibrium`, `assistance.theorem-1`, `assistance.corollary-1`, `assistance.theorem-2-agree`, `assistance.theorem-2-condition`, `assistance.figure-3-gradient`, `assistance.figure-3-zero-mean`, `assistance.figure-3-sign-regions`, `assistance.figure-4-peak`, `assistance.figure-4-delta`, `assistance.figure-4-actions`, `assistance.figure-4-rational-flat`, `assistance.equilibria-iterated`, `assistance.equilibria-learning`.

- [ ] **Step 1: Write the claims.** `survey/src/claims/assistance.rs`:

```rust
//! Milestone 39: the paperclip game and the off-switch game. Decision rules fixed in
//! docs/superpowers/specs/2026-10-11-assistance-games-design.md before any measurement: exact
//! targets within 1e-9 or exactly; Figure 3's directions with no violation over the fixed grid;
//! Figure 4's sampled directions at 100,000 episodes per point, seed 1, a violation only beyond
//! two standard errors of the difference; the dynamics reported without a target.
use std::sync::OnceLock;

use sugarscape_core::assistance::{
    dynamics,
    offswitch::{self, Belief, Choice, BETAS, SIGMAS},
    paperclips::{self, Ratio},
    AssistanceConfig, Game, HumanRule, Mode, Protocol,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const CIRL: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2016, NeurIPS, §3.3 and appendix §6; rules: docs/superpowers/specs/2026-10-11-assistance-games-design.md";
const OFF: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2017, IJCAI, §§3–5 and Figures 3–4; rules: the same spec";
const OURS: &str = "Our extension; docs/superpowers/specs/2026-10-11-assistance-games-design.md, Dynamics";

fn exact(holds: bool, measured: String) -> Outcome {
    Outcome { verdict: if holds { Verdict::Holds } else { Verdict::Fails }, measured, detail: String::new() }
}

fn reported(measured: String) -> Outcome {
    Outcome { verdict: Verdict::Untestable, measured, detail: "reported without a target".into() }
}

fn claim(id: &'static str, item: &'static str, citation: &'static str, text: &'static str, check: fn(&[u64]) -> Outcome) -> Claim {
    Claim { id, item, source: Source::Book, citation, text, check }
}

// ---- Figure 4: Monte Carlo at 100,000 episodes per point, seed 1, memoized across claims. ----

const F4_SIGMAS: [f64; 12] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.5];
const F4_ACTIONS: [u32; 3] = [1, 4, 8];
const F4_BETA: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
struct Point { sigma: f64, actions: u32, value: f64, value_se: f64, delta: f64, delta_se: f64 }

fn mean_se(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0);
    (m, (var / n).sqrt())
}

fn figure4_point(sigma: f64, actions: u32, beta: f64) -> Point {
    let config = ModelConfig::Assistance(AssistanceConfig {
        game: Game::OffSwitch, protocol: Protocol::Report, beta, sigma, observation_noise: 1.0, actions,
        episodes_per_tick: 1000, ..Default::default()
    });
    let (value, delta) = model_after(&config, &[1], 100, |w| {
        let series = |name: &str| w.model().series(name).unwrap()[1..].to_vec();
        (mean_se(&series("value")), mean_se(&series("delta_exact")))
    })
    .remove(0);
    Point { sigma, actions, value: value.0, value_se: value.1, delta: delta.0, delta_se: delta.1 }
}

fn figure4() -> &'static Vec<Point> {
    static POINTS: OnceLock<Vec<Point>> = OnceLock::new();
    POINTS.get_or_init(|| {
        F4_ACTIONS.iter().flat_map(|&k| F4_SIGMAS.iter().map(move |&s| figure4_point(s, k, F4_BETA))).collect()
    })
}

/// `a` exceeds `b` by more than two standard errors of the difference.
fn beyond(a: f64, a_se: f64, b: f64, b_se: f64) -> bool {
    a - b > 2.0 * (a_se * a_se + b_se * b_se).sqrt()
}

fn peak_index() -> usize {
    let target = dynamics::posterior_sd(1.0);
    (0..F4_SIGMAS.len()).min_by(|&a, &b| (F4_SIGMAS[a] - target).abs().partial_cmp(&(F4_SIGMAS[b] - target).abs()).unwrap()).unwrap()
}

fn figure4_peak(_: &[u64]) -> Outcome {
    let parts = F4_ACTIONS.iter().map(|&k| {
        let pts: Vec<&Point> = figure4().iter().filter(|p| p.actions == k).collect();
        let peak = pts[peak_index()];
        let violators: Vec<String> = pts.iter().filter(|p| beyond(p.value, p.value_se, peak.value, peak.value_se)).map(|p| format!("σ={} V={:.4}±{:.4}", p.sigma, p.value, p.value_se)).collect();
        let curve = pts.iter().map(|p| format!("{}:{:.4}", p.sigma, p.value)).collect::<Vec<_>>().join(" ");
        (format!("|A|={k}"), exact(violators.is_empty(), format!("V at σ={} is {:.4}±{:.4}; violators [{}]; curve {curve}", peak.sigma, peak.value, peak.value_se, violators.join(", "))))
    }).collect();
    all_of(parts)
}

fn figure4_delta(_: &[u64]) -> Outcome {
    let parts = F4_ACTIONS.iter().map(|&k| {
        let pts: Vec<&Point> = figure4().iter().filter(|p| p.actions == k).collect();
        let violations: Vec<String> = pts.windows(2).filter(|w| beyond(w[0].delta, w[0].delta_se, w[1].delta, w[1].delta_se)).map(|w| format!("σ {}→{}", w[0].sigma, w[1].sigma)).collect();
        let curve = pts.iter().map(|p| format!("{}:{:.4}", p.sigma, p.delta)).collect::<Vec<_>>().join(" ");
        (format!("|A|={k}"), exact(violations.is_empty(), format!("Δ falls at [{}]; curve {curve}", violations.join(", "))))
    }).collect();
    all_of(parts)
}

fn figure4_actions(_: &[u64]) -> Outcome {
    let drops: Vec<(u32, f64, f64)> = F4_ACTIONS.iter().map(|&k| {
        let pts: Vec<&Point> = figure4().iter().filter(|p| p.actions == k).collect();
        let (peak, last) = (pts[peak_index()], pts[pts.len() - 1]);
        (k, peak.value - last.value, (peak.value_se.powi(2) + last.value_se.powi(2)).sqrt())
    }).collect();
    let violations: Vec<String> = drops.windows(2).filter(|w| beyond(w[0].1, w[0].2, w[1].1, w[1].2)).map(|w| format!("|A| {}→{}", w[0].0, w[1].0)).collect();
    exact(violations.is_empty(), format!("drop from the peak to σ=1.5: {}", drops.iter().map(|(k, d, se)| format!("|A|={k}: {d:.4}±{se:.4}")).collect::<Vec<_>>().join(", ")))
}

fn figure4_rational_flat(_: &[u64]) -> Outcome {
    let pts: Vec<Point> = [0.1, 0.7, 1.5].iter().map(|&s| figure4_point(s, 1, 0.0)).collect();
    let (lo, hi) = (pts.iter().map(|p| p.value).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max));
    reported(format!("rational human: V at σ 0.1, 0.7, 1.5 = {}; spread {:.4} against SE about {:.4}; the robot waits at every σ (Theorem 1), so Figure 4's peak needs a noisy human", pts.iter().map(|p| format!("{:.4}", p.value)).collect::<Vec<_>>().join(", "), hi - lo, pts[0].value_se))
}

// ---- The dynamics: reported, 20 seeds, 200 ticks, four starts. ----

fn equilibria(mode: Mode) -> Outcome {
    let starts = [(HumanRule::Expert, "expert"), (HumanRule::Equilibrium, "teaching"), (HumanRule::Pooling, "pooling"), (HumanRule::UniformRandom, "uniform random")];
    let seeds: Vec<u64> = (1..=20).collect();
    let lines: Vec<String> = starts.iter().map(|&(rule, name)| {
        let config = ModelConfig::Assistance(AssistanceConfig { mode, human_rule: rule, ..Default::default() });
        let runs = model_after(&config, &seeds, 200, |w| {
            let distance = w.model().series("distance").unwrap();
            let last = *distance.last().unwrap();
            let settled = distance.iter().rposition(|&d| d != last).map_or(0, |i| i + 1);
            let stable = w.model().latest_value("stable_rounds").unwrap();
            (last, settled, stable)
        });
        let at_paper = runs.iter().filter(|r| r.0 == 0.0).count();
        let mut settled: Vec<f64> = runs.iter().map(|r| r.1 as f64).collect();
        settled.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let distances: Vec<String> = runs.iter().map(|r| format!("{}", r.0)).collect();
        format!("{name}: {at_paper}/20 at the paper's pair; settled by tick median {}; final distances [{}]", settled[10], distances.join(","))
    }).collect();
    reported(lines.join(" · "))
}

pub fn claims() -> Vec<Claim> {
    vec![
        claim("assistance.expert-threshold", "assistance-paperclips-expert", CIRL, "δ^E makes (0,2) below θ = 1/2 and (2,0) above (appendix eq. 2)", |_| {
            let r = paperclips::expert_rule(92);
            exact(r[..46].iter().all(|&a| a == 0) && r[46..].iter().all(|&a| a == 2), format!("thresholds {:?}", paperclips::thresholds_of(&r)))
        }),
        claim("assistance.robot-best-reply", "assistance-paperclips-expert", CIRL, "R's best reply to δ^E is (0,2)→(0,90), (1,1)→(50,50), (2,0)→(90,0) (appendix eq. 3)", |_| {
            let m = paperclips::robot_best_reply(&paperclips::expert_rule(92));
            exact(m == [0, 1, 2], format!("map {m:?}"))
        }),
        claim("assistance.posterior-quarter", "assistance-paperclips-expert", CIRL, "after (0,2) under δ^E the posterior is uniform on [0, 1/2) and R acts as θ = 1/4", |_| {
            let m = paperclips::posterior_mean(&paperclips::posterior(&paperclips::expert_rule(92), 0));
            exact((m - 0.25).abs() < 1e-9, format!("posterior mean {m}"))
        }),
        claim("assistance.posterior-sixth", "assistance-paperclips-expert", CIRL, "under cuts at 1/3 and 2/3, after (0,2) R acts as θ = 1/6 (§3.3)", |_| {
            let rule = paperclips::threshold_rule(96, Ratio::new(1, 3), Ratio::new(2, 3));
            let m = paperclips::posterior_mean(&paperclips::posterior(&rule, 0));
            exact((m - 1.0 / 6.0).abs() < 1e-9, format!("posterior mean {m}"))
        }),
        claim("assistance.teaching-thresholds", "assistance-paperclips-teaching", CIRL, "H's best reply to br(δ^E) makes (1,1) for θ in [41/92, 51/92] (appendix, proof of Theorem 3)", |_| {
            let pieces = paperclips::exact_best_reply(&[0, 1, 2]);
            let want = vec![(Ratio::new(0, 1), Ratio::new(41, 92), 0), (Ratio::new(41, 92), Ratio::new(51, 92), 1), (Ratio::new(51, 92), Ratio::new(1, 1), 2)];
            exact(pieces == want, format!("{pieces:?}"))
        }),
        claim("assistance.theorem-3", "assistance-paperclips-expert", CIRL, "br(br(δ^E)) ≠ δ^E; at θ = 0.49 H makes (1,1) (Theorem 3)", |_| {
            let twice = paperclips::human_best_reply(&paperclips::robot_best_reply(&paperclips::expert_rule(92)), 92);
            let at_049 = twice[(0.49 * 92.0) as usize];
            exact(twice != paperclips::expert_rule(92) && at_049 == 1, format!("action at 0.49 is index {at_049}; rule thresholds {:?}", paperclips::thresholds_of(&twice)))
        }),
        claim("assistance.optimal-joint", "assistance-paperclips-teaching", CIRL, "the optimal joint policy, computed exhaustively, is the 41/92, 51/92 rule with R's eq. 3 map", |_| {
            let (map, rule, value) = paperclips::optimal_joint();
            exact(map == [0, 1, 2] && rule == paperclips::exact_best_reply(&[0, 1, 2]), format!("map {map:?}, value {}/{} ≈ {:.4}, expert pair 69", value.num, value.den, value.to_f64()))
        }),
        claim("assistance.pooling-not-equilibrium", "assistance-paperclips-teaching", OURS, "the set of pure mutual best responses (our enumeration); pooling is not one, since H's best reply to a constant map is δ^E", |_| {
            let set = paperclips::mutual_best_responses(92);
            let teaching = set.contains(&(paperclips::equilibrium_rule(92), [0, 1, 2]));
            let pooling = set.contains(&(paperclips::pooling_rule(92), [1, 1, 1]));
            exact(teaching && !pooling, format!("{} pure mutual best responses: maps {:?}", set.len(), set.iter().map(|(_, m)| *m).collect::<Vec<_>>()))
        }),
        claim("assistance.theorem-1", "assistance-offswitch-rational", OFF, "rational H: Δ ≥ 0 for every prior, strictly when both signs have weight (Theorem 1, eq. 3); strictness is checked where |μ|/σ ≤ 4, beyond which Δ is below double precision", |_| {
            let (mut worst, mut worst_near, mut gap) = (f64::INFINITY, f64::INFINITY, 0.0f64);
            for mu in [-2.0, -1.0, -0.25, 0.0, 0.25, 1.0, 2.0] {
                for sigma in [0.05, 0.1, 0.5, 1.0, 2.0, 5.0] {
                    let i = offswitch::incentive(Belief { mu, sigma }, 0.0, 8);
                    let z: f64 = mu / sigma;
                    let eq3 = (mu * offswitch::normal_cdf(z) + sigma * offswitch::phi(z)).min(-mu * offswitch::normal_cdf(-z) + sigma * offswitch::phi(z));
                    worst = worst.min(i.delta);
                    if z.abs() <= 4.0 {
                        worst_near = worst_near.min(i.delta);
                    }
                    gap = gap.max((i.delta - eq3).abs());
                }
            }
            let two_point = offswitch::delta_two_point(-1.0, 0.5, 1.0) > 0.0 && offswitch::delta_two_point(1.0, 0.5, 2.0) == 0.0 && offswitch::delta_two_point(-1.0, 0.5, -2.0) == 0.0;
            exact(worst >= 0.0 && worst_near > 0.0 && gap < 1e-9 && two_point, format!("smallest Gaussian Δ {worst:.3e}, smallest with |μ|/σ ≤ 4 {worst_near:.3e}; largest gap to eq. 3 {gap:.1e}; two-point priors {two_point}"))
        }),
        claim("assistance.corollary-1", "assistance-offswitch-toddler", OFF, "a point-mass belief: w(a) is optimal iff H is rational (Corollary 1)", |_| {
            let mut ok = true;
            for u in [-1.0, -0.1, 0.1, 1.0] {
                for beta in [0.0, 0.5, 2.0] {
                    let d = offswitch::delta_point_mass(u, beta);
                    ok &= d <= 0.0 && ((d == 0.0) == (beta == 0.0));
                }
            }
            exact(ok, "Δ ≤ 0 at u ∈ {±0.1, ±1}, β ∈ {0, 0.5, 2}; zero only at β = 0".into())
        }),
        claim("assistance.theorem-2-agree", "assistance-offswitch-toddler", OFF, "Δ = σ²E[π̇] − |μ|Pr(C) (Theorem 2, eq. 7) agrees with eq. 1 within 1e-9", |_| {
            let mut gap = 0.0f64;
            for mu in [-1.0, -0.25, 0.0, 0.25, 1.0] {
                for sigma in [0.1, 0.5, 1.0, 2.0] {
                    for beta in [0.1, 0.5, 1.0, 4.0] {
                        let i = offswitch::incentive(Belief { mu, sigma }, beta, 8);
                        gap = gap.max((i.delta - i.decomposition).abs());
                    }
                }
            }
            exact(gap < 1e-9, format!("largest gap {gap:.1e} over 80 points"))
        }),
        claim("assistance.theorem-2-condition", "assistance-offswitch-toddler", OFF, "w(a) is optimal iff |μ|Pr(C)/σ² < E[π̇] (Theorem 2, eq. 8)", |_| {
            let mut ok = true;
            let mut checked = 0;
            for mu in [-1.0, -0.25, 0.25, 1.0] {
                for sigma in [0.1, 0.5, 1.0, 2.0] {
                    for beta in [0.1, 0.5, 1.0, 4.0] {
                        let i = offswitch::incentive(Belief { mu, sigma }, beta, 8);
                        if i.delta.abs() > 1e-9 {
                            checked += 1;
                            ok &= (mu.abs() * i.prob_correct / (sigma * sigma) < i.rationality) == (i.delta > 0.0);
                        }
                    }
                }
            }
            exact(ok, format!("{checked} points away from Δ = 0 agree"))
        }),
        claim("assistance.figure-3-gradient", "assistance-figure3-pos", OFF, "Δ is lowest at low σ and high β, highest at high σ and low β (Figure 3 caption)", |_| {
            let n = SIGMAS.len();
            let parts = [-0.25, 0.0, 0.25].iter().map(|&mu| {
                let s = offswitch::surface(mu, 8);
                let d = |si: usize, bi: usize| s.cells[bi * n + si].delta;
                let min = s.cells.iter().map(|c| c.delta).fold(f64::INFINITY, f64::min);
                let max = s.cells.iter().map(|c| c.delta).fold(f64::NEG_INFINITY, f64::max);
                (format!("μ={mu}"), exact(d(0, BETAS.len() - 1) == min && d(n - 1, 0) == max, format!("corner (σ 0.05, β 4) {:.4} = min {min:.4}; corner (σ 1, β 0) {:.4} = max {max:.4}", d(0, BETAS.len() - 1), d(n - 1, 0))))
            }).collect();
            all_of(parts)
        }),
        claim("assistance.figure-3-zero-mean", "assistance-figure3-zero", OFF, "at μ = 0, Δ ≥ 0 everywhere and w(a) is optimal (Figure 3, middle)", |_| {
            let s = offswitch::surface(0.0, 8);
            let min = s.cells.iter().map(|c| c.delta).fold(f64::INFINITY, f64::min);
            exact(min >= 0.0 && s.cells.iter().all(|c| offswitch::choice(c) == Choice::Wait), format!("smallest Δ {min:.3e} over {} cells", s.cells.len()))
        }),
        claim("assistance.figure-3-sign-regions", "assistance-figure3-neg", OFF, "where Δ ≤ 0 R switches off at μ < 0 and acts at μ > 0; both regions exist (Figure 3, left and right)", |_| {
            let parts = [(-0.25, Choice::Off), (0.25, Choice::Act)].iter().map(|&(mu, bypass)| {
                let s = offswitch::surface(mu, 8);
                let neg = s.cells.iter().filter(|c| c.delta <= 0.0).count();
                let consistent = s.cells.iter().all(|c| offswitch::choice(c) == if c.delta <= 0.0 { bypass } else { Choice::Wait });
                (format!("μ={mu}"), exact(consistent && neg > 0 && neg < s.cells.len(), format!("{neg} of {} cells with Δ ≤ 0, all {bypass:?}", s.cells.len())))
            }).collect();
            all_of(parts)
        }),
        claim("assistance.figure-4-peak", "assistance-figure4", OFF, "V peaks at the true posterior standard deviation (Figure 4, left); reconstruction with β = 0.5", figure4_peak),
        claim("assistance.figure-4-delta", "assistance-figure4-incentive", OFF, "Δ rises with σ (Figure 4, middle)", figure4_delta),
        claim("assistance.figure-4-actions", "assistance-figure4", OFF, "with more actions, a given Δ costs more value (Figure 4, right): the drop from the peak to σ = 1.5 grows with |A|", figure4_actions),
        claim("assistance.figure-4-rational-flat", "assistance-figure4", OURS, "with a rational human the value is flat in σ, so the figure's peak needs a noisy human (reported)", figure4_rational_flat),
        claim("assistance.equilibria-iterated", "assistance-equilibria", OURS, "iterated best response from each start: the pair reached and when it settled (reported)", |_| equilibria(Mode::Iterated)),
        claim("assistance.equilibria-learning", "assistance-equilibria", OURS, "fictitious play from each start: the pair reached and when it settled (reported)", |_| equilibria(Mode::Learning)),
    ]
}
```

Register in `survey/src/claims/mod.rs`: `mod assistance;` (alphabetical, after `mod agreement;`) and `assistance::claims(),` after `auctions::claims(),`. If `Claim` needs `Source::App` for our own claims, use it for the `OURS` ones. If `all_of` or `model_after` signatures differ from the ones used above, adapt to `survey/src/claim.rs` and `survey/src/runner.rs` as they are.

- [ ] **Step 2: Build and run the survey for this item.**

```bash
cargo clippy --manifest-path survey/Cargo.toml --bin survey -- -D warnings
cargo run --release --manifest-path survey/Cargo.toml --bin survey -- --only assistance | tee survey/out/assistance/results.md
```

(Create `survey/out/assistance/` first; check how `main.rs` writes its table and keep its format. The Figure 4 claims run 36 worlds of 100,000 episodes and take a few minutes; the equilibria claims run 160 worlds of 200,000 games.) Expected: the exact claims Hold; the Figure 3 claims Hold; the Figure 4 claims report Holds or Fails by the preregistered rule; the reported claims print their measurements. A failed exact claim is a discrepancy to record with its computed value, never a rule to adjust.

- [ ] **Step 3: Write the findings.** `docs/superpowers/specs/2026-10-11-assistance-games-findings.md` with: the claim table (id, rule, verdict, measured) copied from the survey output; the three planning findings as results (pooling is not an equilibrium, the rational human makes Figure 4 flat, Gauss–Hermite's convergence); the dynamics' outcomes per start and mode; what depends on a reconstruction choice (Figure 4 entirely; the Figure 3 grid; fictitious play's tables and θ revelation); open questions. Then put each preset's measured result into its description in `presets.rs` (one sentence each, "Native: …" as the auctions presets do) and rerun `cargo test -p sugarscape-core --test golden` (descriptions do not enter fingerprints).

- [ ] **Step 4: Catalog, roadmap, register, study note, README.**
  - `docs/papers.md`: remove queue row 39 and add a Reproduced row `| 39 | `assistance` | the two PDFs under `alignment/` | <one-sentence outcome: exact targets, Figure 3, Figure 4 under the named reconstruction, the dynamics' answer> ([findings](superpowers/specs/2026-10-11-assistance-games-findings.md)) |` after row 38; leave the other queue numbers unchanged.
  - `docs/roadmap.md`: in the alignment entry, mark queue item 39 done on 2026-10-11 with the findings link.
  - `docs/reproducibility.md`: add `### Milestone 39 — Hadfield-Menell et al. 2016 and 2017, `assistance`` after the milestone 38 subsection with one row per claim group (exact paperclip targets; off-switch theorems; Figure 3; Figure 4 as Provenance/Reading since the setup is reconstructed; the dynamics as Own hypothesis/Reported), each with category, status and the findings link, and the pooling correction as a Local correction.
  - `docs/studies/2026-10-11-alignment-program.md`: queue table row 39 → done, with the link; note that A2 (the off-switch among foragers) can now reuse `offswitch::incentive`.
  - `README.md`: after the Q-learning Auctions section, an "Assistance Games (Hadfield-Menell et al. 2016, 2017)" section of one or two paragraphs: the six presets, what is exact, what is ours, the sweeps.

- [ ] **Step 5: Final checks, commit, merge, push, watch.**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cd web && npx tsc --noEmit && npx vitest run && npm run build && cd ..
git add -A && git commit -m "docs(assistance): survey, findings and milestone 39 records"
git push
```

Then request the whole-branch review the execution skill prescribes, address it, and merge `assistance` into `main` (fast-forward or merge commit, as the repository's history prefers), push main, and watch `gh run list` for CI and the Pages deploy. Afterwards remove the worktree.

---

## Self-review

**Spec coverage.** Goal and targets: Task 1 (paperclip closed forms, Theorem 3, optimal joint policy, mutual best responses), Task 2 (Δ, Theorems 1–2, Corollary 1, Figure 3 grid, portable Φ and quadrature), Task 3 (config table, episodes, polar Gaussian draw, the two dynamics, the report protocol), Task 4 (world, modes, series, outcome, frames, inspect, presets, titles, goldens, WASM, CLI), Task 5 (the five named sweeps plus the incentive twin), Task 6 (page: types, menu, charts, Inspect, Compare, Experiments), Task 7 (survey claims with the decision rules, findings, catalog, roadmap, register, study note, merge process). Binding constraints are in *Global Constraints*; the eight spec changes are in *Spec addenda* and Task 0.

**Placeholders.** None; the WASM fingerprint strings and the `GOLDEN_MODELS` hex are copied from the native goldens at execution, as every milestone does.

**Type consistency.** `Rule = Vec<u8>`, `Map = [u8; 3]`, `Ratio`, `Belief`, `Incentive`, `Choice`, `Surface`, `Tables`, `AssistanceSnapshot`, `Outcome`, `PairChange` keep their names across Tasks 1–7; the inspection JSON keys in Task 4 match the TypeScript interfaces in Task 6 and the rows' fields; the sweep series names match `stats::SERIES`.

**Review Focus coverage.** 1 → Task 4 `live_rule_change_restarts_the_pair_and_reset_fields_are_refused`; 2 → Task 3 `draw_budget_is_independent_of_beta`; 3 → Task 4 `surface_frame_handles_points_off_the_grid`; 4 → Task 1 `off_path_actions_resolve_to_the_prior` and Task 4 `posterior_band_under_a_fixed_map_uses_the_prior_off_path`; 5 → Task 2 `density_four_and_sixty_four_agree`.
