# SugarScape milestone — Assistance games: the paperclip game and the off-switch game — Design

**Amended 2026-10-11 (planning):** quadrature, protocol switch, Figure 4's β, pooling, polar Box–Muller, grids, fixed-map fields, uniform-random draws; see the plan's *Spec addenda*.

**Date:** 2026-10-11. **Milestone number:** 39 (queue item 39 in `docs/papers.md`), assigned at merge.
**Kind:** `assistance`, displayed as **Assistance Games**.
**Design basis:** architecture, scope and the sections below approved in chat on 2026-10-11.
**Program:** opens the [assistance and alignment program](../../studies/2026-10-11-alignment-program.md).
**Closest relative:** milestone 35, Q-learning Auctions (`2026-10-02-q-learning-auctions-design.md`),
whose conventions for a non-spatial kind, sequential draws, presets, sweeps and survey wiring apply.
**Primary sources** (local copies in `papers/alignment/`, gitignored):

- Hadfield-Menell, Dragan, Abbeel and Russell, "Cooperative inverse reinforcement learning,"
  NeurIPS 2016, arXiv 1606.03137 (the arXiv version carries the appendix with the proofs). The
  paperclip game is §3.3 and appendix §6, Theorems 2 and 3.
- Hadfield-Menell, Dragan, Abbeel and Russell, "The off-switch game," IJCAI 2017, arXiv
  1611.08219v3. The game is §2, the results §§3–5, Figures 1–4.
- Russell, *Human Compatible* (2019), retells both games in prose; the user's paper copy is the
  second source and a check on whether the retelling matches the papers' numbers.

## Goal and intended use

Reproduce two one-shot assistance games exactly, then show how an equilibrium is established in
the paperclip game by two dynamics that are ours. The paperclip game has closed forms: the expert
rule, the robot's best reply, the posterior means, the teaching thresholds 41/92 and 51/92 and the
optimal joint policy. The off-switch game has theorems about the robot's incentive Δ to wait for the
human, a decomposition under a Gaussian belief and a Boltzmann human, and two figures. Every exact
target must match to the digit; every figure claim is scored by a stated direction rule; every
dynamics outcome is reported without a target because it is our extension.

Success is a faithful, inspectable experiment with fixed decision rules and a truthful account of
what depends on a reconstruction choice. The deferred CIRL gridworld experiment (maximum-entropy
IRL, 500 sampled reward vectors, Figure 2 of the 2016 paper) is a separate future item.

## Binding constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add a new kind and new goldens.
- One sequential path serves native and WASM. Follow the seeded RNG and portable arithmetic
  conventions: `f64` draws, `u32` ranges, `portable::exp_neg` on nonpositive arguments and the
  identity `1/(1+e^x) = e^{-x}/(1+e^{-x})` for the positive branch of the logistic; no `f64::exp`
  on positive arguments, no `erf`. Quadrature is the composite trapezoid rule with step min(σ, β)/`quadrature_density` over μ ± 9σ, in plain arithmetic and `portable::exp_neg`; β = 0 uses closed forms through a portable normal c.d.f. built from the error function's power series. Gaussian draws use the polar Box–Muller transform (`sqrt` and `portable::ln`).
- The analytic units take no RNG and are pure functions of the config, so tests check them against
  the papers' numbers without a world.
- Literal rules are defaults. Every reconstruction choice (the Figure 4 population prior, noise and
  action-set construction; fictitious play's rule; tie-breaking) is named in config or docs as ours.
- Decision rules below are fixed before any measurement. New rules after looking at results need a
  dated amendment and a separate verdict.
- Paper PDFs stay gitignored in `papers/`.

## Source models and targets

### The paperclip game (CIRL 2016, §3.3; appendix §6)

Two rounds. In round 0 the human H acts, in round 1 the robot R acts; state is (paperclips,
staples, round). H's actions are (0,2), (1,1), (2,0): two items in any split. R's actions are
(0,90), (50,50), (90,0). The type θ is uniform on [0, 1] and known to H only; R's prior is uniform.
Reward per action is θ·p + (1−θ)·q and both players receive the sum over the two rounds.

Exact targets:

| Target | Statement | Where |
|---|---|---|
| Expert rule δ^E | (0,2) if θ < 1/2, (1,1) if θ = 1/2, (2,0) if θ > 1/2 | appendix eq. (2) |
| R's best reply to δ^E | (0,2)→(0,90), (1,1)→(50,50), (2,0)→(90,0) | appendix eq. (3) |
| Posterior under δ^E after (0,2) | uniform on [0, 1/2), R acts as θ = 1/4 | appendix |
| Posterior under the three-way rule with cuts 1/3, 2/3 after (0,2) | uniform on [0, 1/3), R acts as θ = 1/6 | §3.3 text |
| H's best reply to br(δ^E) | (0,2) if θ < 41/92, (1,1) if 41/92 ≤ θ ≤ 51/92, (2,0) if θ > 51/92 | appendix, proof of Thm 3 |
| Theorem 3 | br(br(δ^E)) ≠ δ^E; at θ = 0.49 H makes (1,1) | §3.3 |
| Optimal joint policy | enumerate R's 27 maps from H actions to deployments; for each, H's pointwise best reply; the maximizer of expected total reward equals the 41/92, 51/92 pair | our computation; the paper says "computed exhaustively" |

The thresholds are rational because payoffs are linear in θ: under R's best reply to δ^E, H's
options at θ pay 92(1−θ), 51 and 92θ, whose crossings are 41/92 and 51/92. The implementation
computes them as fractions and compares exactly.

Our extensions, labeled: the set of all pure mutual best responses, and whether a pooling pair (H always (1,1), R always (50,50), off-path deployments resolved by the prior) survives as a mutual best response. Planning note: H's best reply to any constant map is the expert rule, so it does not; the enumeration records the set; and which pair iterated best response and fictitious play reach from each
named start.

### The off-switch game (IJCAI 2017)

R moves first and chooses `a` (act, utility U_a), `w(a)` (wait for H), or `s` (switch off,
utility 0). After `w(a)`, H chooses `s` or `¬s`; `¬s` yields U_a. U_a is drawn from a prior and R's
belief B^R starts equal to it. The rational H allows `a` iff U_a ≥ 0. R's incentive to wait is
Δ = E[U_a·π^H(U_a)] − max{E[U_a], 0}, where π^H is the probability H lets `a` execute.

Targets:

| Target | Statement | Rule |
|---|---|---|
| Theorem 1 | rational H: Δ ≥ 0 for every prior; Δ = 0 iff the prior gives no weight to U_a < 0 or none to U_a > 0 | exact on a grid of Gaussian and two-point priors |
| Corollary 1 | point-mass belief at U_a: Δ ≤ 0 for any H, with equality only for the rational H | exact |
| Theorem 2 | Gaussian belief N(μ, σ²), Boltzmann H with π^H = 1/(1 + exp(−U_a/β)): Δ = σ²·E[π̇^H] − |μ|·Pr(C), with Pr(C) from eq. (6) | the direct quadrature of Δ and the decomposition agree within 1e-9 at the configured `quadrature_density` |
| Theorem 2(2) | w(a) optimal iff |μ|·Pr(C)/σ² < E[π̇^H] | exact given the above |
| Figure 3 | over σ and β at μ ∈ {−1/4, 0, 1/4}: Δ is lowest at low σ and high β, highest at high σ and low β; at μ = 0 the sign of Δ is nonnegative everywhere; for μ < 0 the region with Δ ≤ 0 is where R switches off, for μ > 0 where R acts | direction claims over the sweep's full grid |
| Figure 4 | value V of the game against the standard deviation R uses, when R gets a noisy report of U_a: V peaks at the true posterior standard deviation; Δ rises with σ; with more actions, a given Δ costs more value. Under a rational H, Theorem 1 makes R wait at every σ, so V is flat; the figure is reconstructed with a Boltzmann H at β = 0.5 (ours) and the rational case is reported. | direction claims; the setup is reconstructed |

Figure 4's reconstruction, named as ours: the population prior is N(0, 1); H's report is U_a plus
N(0, `observation_noise`²) noise; R's belief is Gaussian with the Bayesian posterior mean and a
standard deviation set by the sweep; with `actions` = k, R draws k candidate U values from the prior,
receives a noisy report of each, and plays the off-switch game on the best of them by its belief.

## Decision rules

- Exact targets: agree within 1e-9, or exactly for rational quantities.
- Direction claims on exact quantities (Figure 3): the stated ordering holds over the sweep's
  entire configured range; one violation fails the claim.
- Direction claims on sampled quantities (Figure 4, estimated by Monte Carlo at 100,000 episodes
  per sweep point with a fixed seed): the ordering holds on the means, and a violation counts only
  when it exceeds two standard errors of the difference.
- Dynamics: report the pair reached, the round it stabilized, and whether it equals the paper's
  equilibrium, for each named start, 20 seeds, both modes. No target.
- Survey verdicts use the register's vocabulary; a failed exact target is reported as a
  discrepancy with the computed value, never adjusted.

## Architecture

Add `ModelKind::Assistance`, `ModelConfig::Assistance(AssistanceConfig)` with `model: assistance`,
and `AssistanceWorld` implementing `Model`. Code under `crates/sugarscape-core/src/assistance/`:

| Unit | Responsibility and dependencies |
|---|---|
| `config.rs` | `AssistanceConfig`, enums, validation, schema, `set_path`; existing schema primitives |
| `paperclips.rs` | Payoffs; H rules as threshold triples or per-bin action tables; R maps; posterior under a rule; best replies both ways; enumeration of the optimal joint policy and of all pure mutual best responses; rational arithmetic for thresholds. Pure. |
| `offswitch.rs` | Trapezoid quadrature moments; Δ by direct quadrature and by Theorem 2; Pr(C); E[π̇]; the Figure 3 grid; the Figure 4 value curve under the named reconstruction. Pure. |
| `dynamics.rs` | Episode sampling for both games; iterated best response; fictitious play. Consumes explicit draws from the shared RNG. |
| `world.rs` | `AssistanceWorld`: tick loop, mode dispatch, outcome struct, series, fingerprint, `set_config` with reset-only fields refused |
| `stats.rs`, `view.rs` | Snapshots, series names, bounded history; frame rendering and inspection JSON |
| `presets.rs`, `mod.rs` | Source-labelled presets and exports |

Reuse existing RNG, portable math, schema, canvas, stats and experiment infrastructure. Wire the
kind into model parsing and dispatch, schema, preset titles, export, CLI, WASM, page types,
controls, charts, Inspect, Compare, Experiments, built-in sweeps and the standalone survey, at the
same locations as the `auctions` addition. No new dependencies.

## Configuration

| Field | Default | Semantics |
|---|---|---|
| `game` | `paperclips` | `paperclips` or `off_switch`; reset-only |
| `mode` | `iterated` | paperclips only: `static`, `iterated`, `learning`; reset-only |
| `human_rule` | `expert` | static mode's H rule, or the start for dynamics: `expert`, `equilibrium` (41/92, 51/92), `pooling` (always (1,1)), `uniform_random` (each bin's action drawn once at reset). `human_rule` `uniform_random` draws its table at reset in every paperclips world |
| `robot_map` | `best_response` | static: best reply to `human_rule`, or `fixed`, the map in `fixed_0`, `fixed_1`, `fixed_2` (deployment index 0–2 for H's actions in order) |
| `theta_bins` | 92 | θ discretization for tables and fictitious play; 92 makes the paper's thresholds bin edges; reset-only |
| `episodes_per_tick` | 1000 | sampled games per tick |
| `respond_every` | 1 | iterated mode: ticks between best-response rounds |
| `fictitious_weight` | 1 | learning mode: weight of each new observation in the frequency tables; 1 is classic fictitious play |
| `mu`, `sigma`, `beta` | 0, 1, 0 | off_switch: belief mean, belief standard deviation, H's temperature; β = 0 is the rational H |
| `actions` | 1 | off_switch, Figure 4: candidate actions R chooses among |
| `observation_noise` | 1 | off_switch, Figure 4: standard deviation of H's noisy report; our reconstruction |
| `protocol` | `prior` | off_switch: `prior` (the paper's game, U_a drawn from the belief) or `report` (Figure 4's reconstruction) |
| `quadrature_density` | 8 | points per smallest scale in the trapezoid rule; 4–64 |

`sigma` > 0, `beta` ≥ 0, `theta_bins` ≥ 3, `episodes_per_tick` ≥ 1. All other fields may change
live; a live change of `mu`, `sigma` or `beta` recomputes the exact values and the frame.

## Dynamics

**Episodes.** Each tick samples `episodes_per_tick` games with the shared RNG. Paperclips: draw θ
uniform; H acts by its current rule (the bin containing θ); R deploys by its current map; record
both rounds' reward, H's action, and whether R's deployment was θ-optimal. Off-switch: draw U_a
from N(μ, σ²) by Box–Muller on two `f64` draws; R picks the option with the highest expected value
under its belief (exact values, not sampled); if it waits, H presses with probability
1 − π^H(U_a; β), the rational H pressing iff U_a < 0; record the outcome and the realized utility.
Empirical Δ is the mean over episodes of U_a·[H allows] minus the better of the mean U_a and 0,
evaluating every branch on each draw.

**Iterated best response (paperclips).** Every `respond_every` ticks, one round in the paper's
order: H's rule becomes the pointwise best reply to R's map over θ-bins (ties toward the smaller
action index); then R's map becomes the best reply to H's new rule, using the posterior each H
action induces and deploying for its mean, with actions H never chooses resolved by the prior.
Rounds since the pair last changed and the pair itself are series and outcome fields.

**Fictitious play (paperclips, our rule, stated as such).** Each player keeps smoothed frequency
tables of the other's play: H, for each of its actions, the frequency of R's deployments; R, for
each θ-bin, the frequency of H's actions. θ is revealed to R after deployment, as the realized
reward reveals it in the paper's deployment phase; this is a modeling choice and is documented.
After each tick's episodes, both players best-respond to the tables, each new observation entering
with weight `fictitious_weight`. The start is `human_rule` for H and the uniform map for R.
Common-payoff games are potential games, for which fictitious play converges (Monderer and
Shapley 1996); which equilibrium it reaches is the question.

## Measurement

**Series, paperclips:** `reward` (mean per episode), `regret` (against the θ-optimal deployment),
`threshold_low`, `threshold_high` (H's rule as thresholds, or the nearest thresholds to a table),
`map_0`, `map_1`, `map_2` (R's deployments by H action), `stable_rounds`, `distance` (0 when the
pair equals the paper's equilibrium, else the number of bins and map entries that differ).

**Series, off-switch:** `delta_exact`, `delta_empirical`, `prob_correct` (Pr(C)), `rationality`
(E[π̇]), `value_act`, `value_wait`, `value_off`, `share_wait`, `share_act`, `share_off`, `value` (mean realized utility per episode).

**Outcome** (serialized at the horizon or on request): the config, seed, the exact targets as
computed, the current pair and its history of changes, the full equilibrium set, and for off-switch
the Figure 3 grid at the current μ.

**Built-in sweeps:** `assistance-figure3-neg`, `assistance-figure3-zero`, `assistance-figure3-pos`
(σ × β grids, statistic `delta_exact`); `assistance-figure4` (σ, statistics value and Δ, series by
`actions` ∈ {1, 4, 8}); `assistance-equilibria` (the four starts × two modes, 20 seeds, statistic
`distance` at the horizon). The Figure 3 grid is σ 0.05–1.00 step 0.05 (20 values) and β 0–4 step 0.2 (21 values); the sweeps' β series are {0, 0.1, 0.2, 0.4, 0.6, 0.8, 1, 1.5, 2, 3, 4}. `assistance-figure4` sweeps σ ∈ {0.1, 0.2, …, 1.0, 1.2, 1.5} with β = 0.5, `observation_noise` = 1, series `actions` ∈ {1, 4, 8}, one seed, 100 ticks of 1,000 episodes, and metrics the window means of `value` and `delta_exact`.

**Survey claims,** each with its rule: `expert-threshold`, `robot-best-reply`, `posterior-quarter`,
`posterior-sixth`, `teaching-thresholds`, `theorem-3`, `optimal-joint`; `theorem-1`, `corollary-1`,
`theorem-2-agree`, `theorem-2-condition`, `figure-3-gradient`, `figure-3-zero-mean`,
`figure-3-sign-regions`, `figure-4-peak`, `figure-4-delta`, `figure-4-actions`; and the recorded
`equilibria-iterated`, `equilibria-learning` outcomes.

## Playground

**Frame.** Fixed-size frames like auctions. Paperclips: three bands. Top, the θ line colored by
H's current action with the paper's thresholds as ticks. Middle, R's map as three cells colored by
deployment. Bottom, R's posterior over θ-bins after the latest observation. In learning mode the
bands show the smoothed frequencies. Off-switch: the Figure 3 surface at the current μ, Δ over σ
and β on a fixed grid with the zero boundary drawn and a marker at the configured (σ, β), recomputed
only when μ, the grid or the quadrature change.

**Inspect.** A θ-bin on the paperclip frame returns H's action, each option's reward at that θ,
R's eventual deployment and the regret. A cell on the off-switch surface returns Δ, its two terms
and Pr(C).

**Presets** (ids, with titles in `titles.rs`):

| Id | Setup | Title |
|---|---|---|
| `assistance-paperclips-expert` | static, expert rule, R's best reply | The human demonstrates and the robot copies, and at 0.49 that goes wrong |
| `assistance-paperclips-teaching` | static, equilibrium rule | The human makes one of each to teach the robot |
| `assistance-paperclips-iterated` | iterated from expert | Best replies, turn by turn, settle on a teaching rule |
| `assistance-paperclips-learning` | fictitious play from uniform random | Two learners find an equilibrium, not always the paper's |
| `assistance-offswitch-rational` | β = 0, μ = 1/4, σ = 1 | An uncertain robot lets itself be switched off |
| `assistance-offswitch-toddler` | β = 4, μ = 1/4, σ = 0.1 | A confident robot facing an erratic human bypasses the switch |

Compare pairs expert against teaching. The menu entry sits with auctions and collusion.

## Testing

- Unit tests on `paperclips.rs` and `offswitch.rs` against the papers' numbers: thresholds as
  exact rationals, posterior means, Theorem 3, the enumerated optimum, Δ's two routes agreeing,
  Theorem 1 on a prior grid, Corollary 1, and the quadrature against a closed form it can check
  (the Gaussian mean and second moment).
- Golden fingerprints for all six presets at 200 ticks, seed 1, in `tests/golden.rs`; the WASM crate
  repeats them, so `wasm-pack` runs before merge.
- Determinism: two worlds with one seed match at every tick; native and WASM agree.
- Page tests mirroring auctions: model detection, default sweep form, inspection rows.
- The survey runs every claim headless with the decision rules; its report feeds the findings.

## Process

Work in a worktree on branch `assistance`. Commit this spec before code. After the plan is
approved, implement with tests first, push, watch CI and the Pages deploy, and record the milestone
in `docs/papers.md`, `docs/roadmap.md` and the reproducibility register at merge.
