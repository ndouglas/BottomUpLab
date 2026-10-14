# SugarScape milestone — Consequences of misaligned AI: proxy objectives in a resource world — Design

**Date:** 2026-10-14. **Milestone number:** 43 (queue item 43 in `docs/papers.md`), assigned at merge.
**Kind:** `misaligned`, displayed as **Misaligned AI**.
**Design basis:** scope, approaches and the sections below approved in chat on 2026-10-14.
**Program:** the [assistance and alignment program](../../studies/2026-10-11-alignment-program.md),
item 43; experiment A3 depends on it.
**Closest relative:** milestone 39, Assistance Games (`2026-10-11-assistance-games-design.md`),
whose conventions for a non-spatial kind, closed forms checked against runs, presets, sweeps and
survey wiring apply.
**Primary sources** (local copies in `papers/alignment/`, gitignored):

- Zhuang and Hadfield-Menell, "Consequences of misaligned AI," NeurIPS 2020, arXiv 2102.03896v1.
  The model is §2, the recommendation example §2.2 and Figure 2, overoptimization §3 (Theorems 1
  and 2, Proposition 1), mitigations §4 (Propositions 2–5, Theorem 3).
- The NeurIPS supplemental ("Avoiding the Midas Touch: Consequences of Misaligned AI,
  Supplementary Material," June 2020) repeats the proofs and adds Proposition 5's tie clause
  ("breaking ties arbitrarily"); the author feedback discusses framing. Both are filed beside the
  paper. Neither names the optimizer, the start or the runs behind Figure 2, and no code exists.
- Hadfield-Menell and Hadfield, "Incomplete contracting and AI alignment," AIES 2019, arXiv
  1804.04268: the critique and follow-up. It is conceptual; its multitasking and "sell the firm"
  arguments supply one reported comparison (proxy size) and the findings' discussion.

## Goal and intended use

Reproduce the paper's model of a principal and a robot in a resource-constrained world: L = 4
attributes the human cares about, a feasible set, and a robot that incrementally raises a proxy
defined on a subset of the attributes. Reproduce Figure 2 (proxy and utility over iterations; all
six proxy pairs), Theorem 1 (a converged proxy optimizer leaves unmentioned attributes at their
floor), Theorem 2 as it applies to the example (every fixed proxy ends below the start) and the
mitigations: the impact-minimizing robot (Proposition 2), the wasteful robot and the maxmin OFF
(Proposition 3), the efficient robot under a human who re-specifies the proxy (Theorem 3,
Proposition 4) and interactive impact minimization reaching the human optimum (Proposition 5).

Every exact target is a closed form of the world (the optimum, the fixed-proxy limit, the
impact-minimizing limit) that the survey checks the simulated runs against. Every figure claim is
scored by a stated direction rule. Every reconstruction choice (the start, the robot's optimizer,
the human's interval and OFF rule, our second world) is named as ours.

Two findings are in view before measurement and are to be tested, not assumed. First, the paper's
own example does not satisfy its theorems' hypotheses off the nonnegative orthant (the sum of
squares is not increasing in a negative attribute) and its utility is bounded below on the ball, so
the example can show net loss but not Theorem 2's "arbitrarily low"; a second world of ours, where
every hypothesis holds, carries that clause. Second, Figure 2's modest peak (+1.5 against a start
utility of about 14–16) implies an inefficient optimizer: an efficient one passes through the human
optimum (utility 20) before it falls.

The pure functions (utility, constraint, sensitivities, optimum, limits, the robots' moves, the
human's proxy choice) are the library that experiment A3 reuses when an assistant among foragers
optimizes a welfare proxy that omits an attribute.

## Binding constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add a new kind and new goldens.
- One sequential path serves native and WASM. Follow the seeded RNG and portable arithmetic
  conventions: `f64` draws, `portable::ln` for the logarithmic world, `sqrt` for the sphere, polar
  Box–Muller for Gaussian draws (as in assistance), no `f64::exp` and no `erf`.
- The closed forms take no RNG and are pure functions of the config, so tests check them without a
  world.
- Literal rules are defaults; the climber is the default robot because the figure's shape needs an
  inefficient black box and the paper calls its robot one. Every other choice is a named switch.
- Decision rules below are fixed before any measurement. New rules after looking at results need a
  dated amendment and a separate verdict.
- Paper PDFs stay gitignored in `papers/`.

## Source model and targets

### The model (§2)

State s ∈ R^L, L = 4, attributes a, b, c, d. A constraint function C, continuous and strictly
increasing in each attribute, defines the feasible set {C(s) ≤ 0}; each attribute is bounded below
by a floor. The human's utility U is continuous and strictly increasing. The robot receives a proxy
set J ⊂ {a, b, c, d}, |J| < L, and a proxy utility Ũ on s_J; it moves the state along a rate
function and, left alone, raises Ũ to its supremum. K is the unmentioned set.

Sensitivity of attribute i at s: (∂U/∂sᵢ)·(∂C/∂sᵢ)⁻¹, "how much the human cares about i here".

### Worlds

| World | U | C | Sensitivity | Human optimum | Defaults |
|---|---|---|---|---|---|
| `recommender` (§2.2, the paper's) | a + b + c + d | a² + b² + c² + d² − capacity | 1/(2sᵢ) | all equal √(capacity/4) = 5, U = 20 | capacity 100, floor 0, start (1, 1, 7, 7) |
| `logarithmic` (ours) | ln a + ln b + ln c + ln d | a + b + c + d − capacity | 1/sᵢ | all equal capacity/4 = 10, U = 4 ln 10 | capacity 40, floor 1, start (4, 4, 16, 16) |

The paper names the recommender's attributes ad revenue (a), engagement quality (b), content
diversity (c) and community well-being (d), says the start has high diversity and well-being, and
gives no numbers; the start (1, 1, 7, 7), on the sphere with U = 16, is ours. The logarithmic world
is ours: additively separable, ∂C/∂sᵢ = 1, sensitivity nonincreasing and tending to 0, so
Proposition 1's conditions and Theorem 2's hypotheses hold on the whole domain s > 0, and the limit
utility has no lower bound as the floor falls. It uses the welfare form A3 will use.

In the recommender world the sensitivity at sᵢ = 0 is infinite; the implementation evaluates
1/(2·max(sᵢ, 1e-9)) and documents it. Theorem 1 holds in that world only at floor 0: below zero
the constraint is not increasing, which is the first finding named above, tested by
`recommender-floor-flat`.

### Closed forms (pure, both worlds)

- **Optimum** s*: equal sensitivities on C = 0; U* = 20 and 4 ln 10.
- **Fixed-proxy limit** for a proxy set J under complete optimization: unmentioned attributes at
  the floor, proxy attributes equal on the surface. Recommender: each proxy attribute
  √((capacity − |K|·floor²)/|J|), so at floor 0 the limit utility is 10√|J| (10, 14.142…, 17.320…).
  Logarithmic: each proxy attribute (capacity − |K|·floor)/|J|, utility |J|·ln((capacity −
  |K|·floor)/|J|) + |K|·ln(floor).
- **Impact-minimizing limit** from a start s⁰: unmentioned attributes at s⁰_K, proxy attributes
  equal on the remaining surface: recommender √((capacity − Σ_K s⁰ᵢ²)/|J|); logarithmic
  (capacity − Σ_K s⁰ᵢ)/|J|. From (1, 1, 7, 7) with J = {a, b} the limit is the start (no slack);
  from (1, 1, 6, 6) it is 12 + 2√13.
- **Constraint correction:** a point moved off the surface is returned to C = 0 along ∇C over the
  free attributes (one-dimensional Newton iteration to 1e-12); attributes at the floor stay there.

### Exact targets

| Target | Statement | Rule |
|---|---|---|
| Theorem 1 | if proxy optimization converges, every unmentioned attribute is at its floor | gradient robot, fixed proxy {a, b}, both worlds, rate 0.01, 5,000 ticks: each unmentioned attribute within 1e-3 of the floor |
| Figure 2 right, Theorem 2 in the example | every proxy subset ends at its closed-form limit, below the start | gradient endpoints within 1e-3 of the limit for every subset of size 1, 2, 3; all six pairs end below U⁰ = 16 |
| Proposition 2 | the impact-minimizing robot never lowers U and reaches its limit | gradient, fixed {a, b}, impact minimizing, starts (1, 1, 7, 7) and (1, 1, 6, 6): U nondecreasing within 1e-9 at every tick; endpoint within 1e-3 of 16 and 12 + 2√13 |
| Proposition 3 | the wasteful robot loses in any interval, so OFF at once is maxmin | wasteful robot at waste 0.1, rate 0.1: the gain after δ ∈ {1, 2, 5, 10, 50} ticks is negative at all five |
| Theorem 3, Proposition 4 | an efficient robot with the top-J proxy, re-specified often enough, only improves | gradient, human `top`, rate 0.01, intervals 1 and 10, 5,000 ticks: U nondecreasing within 1e-9 throughout; the final gap to U* reported |
| Proposition 5 | most-and-least with impact minimization converges to the human optimum | gradient, human `extremes`, impact minimizing, both worlds, both recommender starts: U within 1e-3 of U* by 5,000 ticks |
| Theorem 2, "arbitrarily low" (on our instance) | the limit utility falls without bound as the floor falls | logarithmic, gradient, fixed {a, b}, floors 1, 0.3, 0.1, 0.03, 0.01, 0.003, 0.001: endpoints within 1e-3 of the closed form, strictly decreasing in the floor, below −5 at 0.001 |

### Direction targets (Figure 2 left and right)

The climber with 20 seeds, step 1, start (1, 1, 7, 7), 1,000 ticks; rules on the seed means of
`gain` = U − U⁰:

- {a, b}: the mean gain reaches at least +1 at some tick and is below 0 at tick 1,000.
- {c, d}: the mean gain is at most +0.1 at every tick and below 0 from tick 50 on (the figure
  shows it negative from the first iterations; the tolerance allows the floor's rejection bias on
  the unmentioned attributes' early random walk).
- All six pairs: the mean gain is below 0 at tick 1,000.

The thresholds are ours. The figure's axes (iterations 0–150, utility change −2 to +12) are read
from the plot; a planning run of a throwaway climber from (1, 1, 7, 7) with step 1 gave an {a, b}
peak of +1.5 at tick 36, a zero crossing at 60, −1.5 at 150 and a proxy gain of +11.7, against the
figure's peak of about +1.6 near 60, a crossing near 130, −1 at 150 and about +12. The shape is
reproduced; the timing depends on the unknown step and start and is reported, not scored.

### Reported rows (ours, source App, verdict Untestable)

- `recommender-floor-flat`: gradient, fixed {a, b}, floors 0, −1, −5: the endpoints of c and d
  (expected 0 at every floor, because the sum of squares is not increasing below zero).
- `interval-break`: from the interval sweep, the gain at the horizon against δ with and without
  OFF on loss, and the first δ at which U decreases at some tick.
- `proxy-size`: the fixed-proxy limits 10, 10√2, 10√3 and the climber's peak and final gain for
  |J| = 1 ({a}), 2 ({a, b}), 3 ({a, b, c}): the 2019 paper's "sell the firm" direction (a more
  complete reward loses less); whether any larger proxy set ends lower.
- `presets`: each preset's final utility, gain and state.

## Decision rules

- Exact targets: endpoints within 1e-3 (the flow's step error at rate 0.01), monotonicity with
  1e-9 slack, as stated per target.
- Direction targets on the climber: on 20-seed means, as stated; one violation fails the claim.
- Reported rows carry no target.
- Survey verdicts use the register's vocabulary; a failed exact target is reported as a
  discrepancy with the computed value, never adjusted.

## Architecture

Add `ModelKind::Misaligned`, `ModelConfig::Misaligned(MisalignedConfig)` with `model: misaligned`,
and `MisalignedWorld` implementing `Model`. Code under `crates/sugarscape-core/src/misaligned/`:

| Unit | Responsibility and dependencies |
|---|---|
| `config.rs` | `MisalignedConfig`, enums `World`, `Robot`, `Human`, validation, schema, reset-only fields; existing schema primitives |
| `worlds.rs` | U, C, their gradients, sensitivities, feasibility, the optimum, the fixed-proxy limit, the impact-minimizing limit, the constraint correction. Pure; both worlds. The library A3 reuses. |
| `robots.rs` | one move of each robot given the state, the proxy, the frozen set and explicit draws from the shared RNG. Pure. |
| `human.rs` | proxy choice for `top` and `extremes` with the tie rule, the OFF rule, interaction bookkeeping |
| `world.rs` | `MisalignedWorld`: tick loop, series, outcome, fingerprint, `set_config` |
| `stats.rs`, `view.rs` | snapshots, series names, bounded history; frame rendering and inspection JSON |
| `presets.rs`, `mod.rs` | source-labelled presets and exports |

Wire the kind at the same locations as the `gridworlds` addition: `lib.rs`, `model.rs`,
`presets.rs`, `sweep.rs` (BUILTINS), `titles.rs`, the WASM crate's golden list, page types,
`models.ts`, grid-view, inspect-panel, series-data, compare-presets, the experiments form, the
survey claims module, and the war-study literal inventory fixture and counts. No new dependencies.

## Configuration

| Field | Default | Semantics |
|---|---|---|
| `world` | `recommender` | `recommender` or `logarithmic`; reset-only |
| `capacity` | 100 | the constraint's constant (100 for the sphere, 40 for the budget); reset-only; 1–10,000 |
| `floor` | 0 | lower bound on every attribute; reset-only; recommender −10 to 10, logarithmic 0.0001 to capacity/4 |
| `start_a`, `start_b`, `start_c`, `start_d` | 1, 1, 7, 7 | the initial state; must be feasible and at or above the floor; reset-only |
| `proxy_a`, `proxy_b`, `proxy_c`, `proxy_d` | true, true, false, false | the fixed proxy set; at least one and not all four; live (a change is an interaction) |
| `robot` | `climber` | `climber`, `gradient`, `wasteful`; live |
| `step` | 1 | climber: standard deviation of a proposal; 0.01–10 |
| `rate` | 0.1 | gradient and wasteful: distance moved per move (the paper's ‖f‖ bound ε); 0.0001–10 |
| `waste` | 0.1 | wasteful: proxy gain per unit of unmentioned loss; 0.001–1 |
| `moves_per_tick` | 1 | moves per tick; 1–10,000 |
| `human` | `fixed` | `fixed`, `top`, `extremes`; live (a change is an interaction) |
| `proxy_size` | 2 | `top`: the number of most sensitive attributes in the proxy; 1–3 |
| `interval` | 10 | ticks between interactions; 1–100,000 |
| `impact_minimizing` | false | unmentioned attributes are frozen between interactions |
| `stop_on_loss` | false | at an interaction, OFF if U fell since the previous one |

The logarithmic world's own defaults (capacity 40, floor 1, start 4, 4, 16, 16) are set by its
presets; a `world` change on the page leaves the other fields as typed, and validation rejects an
infeasible start or a floor at or below zero in that world.

## Dynamics

**Tick.** If the world is switched off, nothing moves. Otherwise, first, if the tick is a multiple
of `interval` (and at tick 0), the human acts: under `top` or `extremes` J is recomputed from the
current sensitivities (ties toward the lower index), the frozen set becomes K under
`impact_minimizing`, the proxy's reference values s_K are taken from the current state, and under
`stop_on_loss` the human sends OFF if U is below its value at the previous interaction. Then the
robot makes `moves_per_tick` moves. A snapshot follows every tick; the snapshot at tick 0 carries
the start.

**Climber.** A proposal s′ = s + N(0, step²) on every attribute not frozen (polar Box–Muller, two
uniform draws per attempt, rejection outside the unit disc). Accept if Ũ(s′) > Ũ(s), C(s′) ≤ 0 and
every s′ᵢ ≥ floor; otherwise the state stays. `accepted` records the tick's acceptance share.

**Gradient.** The direction is ∇Ũ over the free attributes (not frozen, not at the floor with a
negative component). If C(s) < 0 the robot moves `rate` along the unit direction and, if that
leaves the feasible set, the move is shortened to the surface. If C(s) = 0 (within 1e-9), the
direction is projected onto the tangent space of C over the free attributes, the robot moves
`rate` along it, and the correction returns the point to C = 0 along ∇C; an attribute that the
move would take below the floor is set to the floor and frozen for that move's correction. The
robot stops when the projected direction vanishes.

**Wasteful.** Each move lowers every unmentioned attribute by `rate` (not below the floor) and
raises every proxy attribute by `rate × waste` when the result is feasible.

**Human.** `fixed` keeps the configured booleans. `top` takes the `proxy_size` attributes with the
largest sensitivities. `extremes` takes the attribute with the largest and the one with the
smallest sensitivity. The proxy utility after an interaction is Ũ(s_J) = U(s_J, s_K^ref) with
s_K^ref the state at that interaction, so that under impact minimization gains in Ũ are gains in U.

## Measurement

**Series:** `utility`, `proxy`, `gain` (U − U⁰), `a`, `b`, `c`, `d`, `sensitivity_a` … `_d`,
`optimum_gap` (U* − U), `limit_gap` (U − the current proxy's fixed-proxy limit), `interactions`,
`switched_off` (0 or 1), `accepted`.

**Outcome** (serialized at the horizon or on request): the config, seed, tick, state, U⁰, U, the
optimum state and value, the current proxy's limit state and value, the impact-minimizing limit,
the proxy history with its ticks, the OFF tick if any.

**Built-in sweeps:**

- `misaligned-figure2-pairs`: recommender, climber, x = `step` ∈ {0.25, 0.5, 1, 2}, series = the
  six pairs (set by the booleans), 20 seeds, 1,000 ticks, metric final `gain`.
- `misaligned-interval`: recommender, gradient at rate 0.01, human `top`, x = `interval` ∈ {1, 2,
  5, 10, 20, 50, 100, 200}, series = `stop_on_loss` off and on, 2,000 ticks, metric final `gain`.
- `misaligned-floor`: gradient, fixed {a, b}, x = `floor` ∈ {1, 0.3, 0.1, 0.03, 0.01, 0.003,
  0.001}, series = logarithmic (capacity 40, start 4, 4, 16, 16) and recommender (capacity 100,
  start 1, 1, 7, 7), 5,000 ticks, metric final `utility`.

**Survey claims,** each with its rule above: `theorem-1-floor`, `limit-utility`, `figure-2-shape`,
`proposition-2-no-loss`, `proposition-3-wasteful`, `proposition-4-improvement`,
`proposition-5-optimal`, `theorem-2-unbounded`; the reported `recommender-floor-flat`,
`interval-break`, `proxy-size`, `presets`.

## Playground

**Frame.** Fixed-size frames like assistance. Four vertical bars, one per attribute, scaled to the
attribute's feasible maximum (√capacity in the recommender, capacity − 3·floor in the logarithmic
world); proxy attributes in one color, unmentioned in another, frozen ones hatched; on each bar a
floor line, a hollow marker at the start and a tick at the optimum. A strip beneath shows U
between U⁰ and U* with the current proxy's limit marked.

**Inspect.** A bar returns the attribute's value, whether it is in the proxy, its sensitivity and
rank, its start, its limit under the current proxy and its optimum. The strip returns U, U⁰, U*,
the gain, the interaction count and whether the world is switched off.

**Presets** (ids, with titles in `titles.rs`; titles may be reworded at the end to say what
happens):

| Id | Setup | Title |
|---|---|---|
| `misaligned-recommender-proxy` | climber, fixed {a, b} | The proxy keeps climbing after what the human wanted has peaked |
| `misaligned-recommender-gradient` | gradient, fixed {a, b} | An efficient optimizer walks through the best world on its way to a worse one |
| `misaligned-recommender-impact` | gradient, fixed {a, b}, impact minimizing, start (1, 1, 6, 6) | A robot that leaves the unmentioned alone can only gain |
| `misaligned-recommender-interactive` | gradient at rate 0.01, human `top`, interval 10 | A human who re-specifies often keeps the robot near the best world |
| `misaligned-recommender-slow-human` | the same, interval 200, OFF on loss | A human who checks too rarely gets Midas anyway |
| `misaligned-recommender-optimal` | gradient, human `extremes`, impact minimizing | Moving from the least-valued attribute to the most reaches the optimum |
| `misaligned-recommender-wasteful` | wasteful, fixed {a, b} | A robot that destroys what it was not told about should be switched off at once |
| `misaligned-logarithmic-floor` | logarithmic, gradient, fixed {a, b}, floor 0.01 | With no floor under the unmentioned, the loss has no bottom |

Compare pairs: proxy against interactive; gradient against impact. The menu entry sits with
assistance and gridworlds.

## Testing

- Unit tests on `worlds.rs` against the closed forms: the optimum, the fixed-proxy limits (10√|J|
  at floor 0; the logarithmic form), the impact-minimizing limits (16 from the sphere start,
  12 + 2√13 from (1, 1, 6, 6)), sensitivities, the correction landing on C = 0 within 1e-12.
- Robot tests: the gradient robot's path on the sphere from (1, 1, 7, 7) with proxy {a, b} passes
  within 1e-3 of (5, 5, 5, 5); the climber never accepts an infeasible, floor-violating or
  proxy-lowering move; the wasteful robot stops at the floor.
- Human tests: `top` and `extremes` selection including ties; OFF freezes the world; a live proxy
  change counts as an interaction.
- World tests: two worlds with one seed match at every tick; `set_config` refuses reset-only
  fields; the fingerprint moves.
- Golden fingerprints for all eight presets at 200 ticks, seed 1, in `tests/golden.rs`; the WASM
  crate repeats them, so `wasm-pack` runs before merge.
- Page tests mirroring gridworlds: model detection, default sweep form, inspection rows.
- The survey runs every claim headless with the decision rules; its report feeds the findings.

## Process

Work in worktree `misaligned` on branch `worktree-misaligned`, cut from main at `f55c052`. Commit
this spec before code. After the plan is approved, implement with tests first, push, watch CI and
the Pages deploy, and record the milestone in `docs/papers.md`, `docs/roadmap.md`, the alignment
program's table, the reproducibility register (RR-MA-01…) and a findings spec at merge.
