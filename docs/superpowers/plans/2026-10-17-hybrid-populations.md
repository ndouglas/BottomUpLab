# Hybrid Human–Machine Populations (milestone 42) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A new model kind `hybrid` (displayed "Hybrid Populations") with four worlds on one engine — Booker et al.'s Samaritan and discriminatory bots, Guo et al.'s probabilistic bots on the (T, S) plane, Si et al.'s cooperative bots in three strategy frameworks, and Shirado and Christakis's noisy bots in a network coloring game — their exact references, a statistical dock against Si et al.'s published C++, and the spec's decision rules run in the survey.

**Architecture:** `crates/sugarscape-core/src/hybrid/` holds pure exact references (`booker.rs`, `guo.rs`, `coloring.rs`, with shared numerics in `exact.rs`), the shared `Population` and Fermi rule (`engine.rs`), lattices and placement (`graph.rs`), each world's update rule beside its references, a `Runner` that steps any world without history (what the survey drives) and the `HybridWorld` the page drives (`world.rs`). Registration follows milestone 43 (`misaligned`) site for site. The survey gains `survey/src/claims/hybrid.rs`; Si et al.'s C++ is patched minimally, compiled and run in a sandbox by `survey/hybrid_si/dock.py`, and its per-run outputs become tracked receipts.

**Tech Stack:** Rust 2021 (`sugarscape-core`, `serde`, `serde_json`, `rand` through `crate::rng::SimRng`), wasm-bindgen through the existing `Sim`, Vite + TypeScript page, vitest, the standalone `survey` crate, Python 3 (standard library only) for the docking harness, Apple clang for the reference build.

**Spec:** `docs/superpowers/specs/2026-10-17-hybrid-populations-design.md` (read it first; Task 1 amends it).

## Global Constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add a new kind and new goldens.
- One sequential path serves native and WASM: seeded RNG (`crate::rng::seeded`, `SimRng`), `portable::exp_neg` for every exponential (through `exact::logistic`, `exact::tanh_half`, `exact::exp`; the argument handed to `exp_neg` is never positive), `portable::ln` for every logarithm, `f64::sqrt` allowed; no `f64::exp`, `ln`, `powf` or `tanh` anywhere in this kind (tests compare against literal constants).
- NaNs that reach a fingerprint are canonicalized to `f64::NAN` first (native and wasm32 can differ in the sign bit of a computed NaN).
- Each world's defaults are its paper's literal setup. Where sources disagree the default follows the spec's priority (Booker, Guo: equations and pseudocode over prose; Si: code over prose) and the other reading is a named switch.
- Decision rules are the spec's (as amended in Task 1), fixed before any measurement; the survey never adjusts a failed rule. Unit tests check identities and paper-supplied numbers only; they never assert a scored claim's verdict.
- Bots are "bots", people are "humans" (Si's "ordinary players" only when quoting Si); never "Flump". American spelling (neighbor, color, center, behavior).
- `pub mod hybrid;` goes at the **end** of `crates/sugarscape-core/src/lib.rs` (after `pub mod negotiable;`). The vitest identity fixtures pin `lib.rs` (`reviewedAdditions`) and `model.rs`, `presets.rs`, `sweep.rs`, `titles.rs` by sha256 (receipts); `crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json` pins `lib.rs` by bytes and sha256 in two bindings. Refresh receipts after the last `cargo fmt` of each pinned file.
- Root `Cargo.toml` and `Cargo.lock` must not change. No new dependencies anywhere.
- Paper PDFs and downloaded code stay gitignored in `papers/`. Si et al.'s C++ is copied, patched and compiled only under the session scratchpad, run only under `sandbox-exec` with network denied, and never from inside `papers/`. No other downloaded code is executed.
- Never use bare `git stash`. The worktree refuses `git -C` and complex compound shell lines: write multi-step shell to a script in the scratchpad and run `bash <script>`.
- Commits end with `Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR`. Push after every commit; after a push to `main`, check CI (`gh run list --limit 3`) and the Pages deploy.
- Run every command from the worktree `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/misaligned` (branch `hybrid`; spec commits `8bcfad3`, `a50ccb9`, `ffc4a69`). The session scratchpad is `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-SugarScape/1082ac4c-3302-454b-aa9f-77a920225c83/scratchpad`, written `$SCRATCH` below.

## Review Focus

1. An isolated node (an Erdős–Rényi graph of mean degree 4 has about 1.8% of them) drawn as focal must end the update without a panic, and its payoff under `accumulate: mean` must read 0, not NaN; pinned in Task 1's `isolated_nodes_have_no_partner_and_mean_payoffs_of_zero` and Task 4's `guo_isolated_humans_do_nothing`.
2. A Bernoulli bot draw that leaves no humans (small N, ρ near 1), or no bots, must run and report NaN cooperation (shown as "—"), never panic or break native–WASM fingerprint agreement; pinned in Task 1's `a_population_without_humans_reports_nan` and Task 7's `a_world_without_humans_runs_and_fingerprints`.
3. Booker's exact chain at strong selection (β up to 100, M up to 100) must stay finite in every output (the weight and both fixation probabilities); pinned in Task 2's `the_chain_is_finite_at_strong_selection`.
4. Inspect on a padding cell past the last node of the square layout must return `panel: "empty"`, on a node well-formed JSON with no NaN, and outside the frame an error; pinned in Task 8's `inspection_reads_nodes_padding_and_refuses_outside`.
5. A coloring session that starts properly colored, is solved, or reaches its time limit must stay finished: later ticks change nothing and `finished()` holds; pinned in Task 6's `a_finished_session_stays_finished` and Task 7's `a_finished_coloring_world_holds`.

---

## File map

| Path | Responsibility |
|---|---|
| `crates/sugarscape-core/src/hybrid/mod.rs` | Module exports |
| `…/hybrid/exact.rs` | `logistic`, `tanh_half`, `exp`, `log_sum_exp`, `ln_factorial`, `Root`, `roots` |
| `…/hybrid/graph.rs` | `Neighborhood`, `Placement`, `lattice`, `erdos_renyi`, `place`, `bernoulli` |
| `…/hybrid/engine.rs` | `Accumulate`, `Population`, `fermi`, `accumulate` |
| `…/hybrid/booker.rs` | `BotKind`, `Donation`, `Chain`, `Replicator`, the α_c forms; `Rules`, `play`, `payoff`, `init`, `micro` |
| `…/hybrid/guo.rs` | `Plane`, `Phase`, `phase_numeric`, `master_roots`; `BotAction`, `Rules`, `init`, `payoff`, `payoff_stored`, `micro_network`, `expected`, `micro_well_mixed` |
| `…/hybrid/si.rs` | `Framework`, `MixedDraw`, `Division`, `Rules`, `pay`, `init`, `payoff_graph`, `payoff_stored`, `payoff_well_mixed`, `round` |
| `…/hybrid/coloring.rs` | `colorings`, `greedy`, `errant`, `Rules`, `Session`, `conflicts_of`, `convergence_steps`, `wait` |
| `…/hybrid/config.rs` | `HybridConfig`, `World`, `Structure`, `Bots`, `StepUnit`, the four `*_defaults()`, validation, schema |
| `…/hybrid/stats.rs` | `HybridSnapshot`, `SERIES`, `series_names` |
| `…/hybrid/world.rs` | `Body`, `Runner`, `HybridWorld` and its `Model` impl |
| `…/hybrid/view.rs` | Layout, frame, inspection JSON |
| `…/hybrid/presets.rs` | Eight presets |
| `crates/sugarscape-core/tests/hybrid_smoke.rs` | Every preset runs 200 ticks with sane series; the built-in sweeps parse |
| `sweeps/hybrid-{booker-lattice-alpha,guo-phi-networks,si-lattice-r}.json` | Built-in sweeps |
| `web/src/hybrid.ts`, `web/src/hybrid.test.ts` | Rules-panel narrowing, legend, Inspect rows |
| `survey/hybrid_si/dock.py`, `survey/hybrid_si/test_dock.py` | Patch, compile and run Si et al.'s C++ in a sandbox; per-run receipts |
| `survey/out/hybrid/si-reference/*.csv`, `…/receipts.json` | The reference runs (tracked) |
| `survey/src/claims/hybrid.rs`, `survey/out/hybrid/results.md` | The spec's rules as survey claims, and the tracked output |
| `docs/superpowers/specs/2026-10-17-hybrid-populations-findings.md` | Findings |

Registration sites (each listed again in its task): `lib.rs`, `browser_experiments/fixtures/key-registration-compatibility.json`, `web/src/episodes/test-fixtures/historical-source.ts`, `web/src/episodes/caching-parity.test.ts` (Task 1); `model.rs`, 17 sites (Task 7); the `presets.rs` catalog, `titles.rs`, `tests/golden.rs`, `crates/sugarscape-wasm/tests/web.rs`, `web/src/episodes/test-fixtures/foraging-source-compatibility.json` (Task 8); `sweep.rs`, `survey/war1_study/fixtures/core-literal-inputs-v1.txt` and its counts (Task 9); `web/src/{types,models,compare-presets,determinism.test,schema-form}.ts`, `web/src/ui/{series-data,grid-view,inspect-panel,schema-panel}.ts`, `web/src/experiments/form.ts` (Task 10); `survey/src/claims/mod.rs` (Task 12).

---

### Task 1: Spec amendments, shared numerics, structures and the population (`exact.rs`, `graph.rs`, `engine.rs`)

**Files:**
- Modify: `docs/superpowers/specs/2026-10-17-hybrid-populations-design.md`
- Create: `crates/sugarscape-core/src/hybrid/{mod.rs,exact.rs,graph.rs,engine.rs}`
- Modify: `crates/sugarscape-core/src/lib.rs` (append), `crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json`, `web/src/episodes/test-fixtures/historical-source.ts`, `web/src/episodes/caching-parity.test.ts`

**Interfaces:**
- Produces:
  - `exact::{logistic(f64) -> f64, tanh_half(f64) -> f64, exp(f64) -> f64, log_sum_exp(&[f64]) -> f64, ln_factorial(u32) -> f64, Root { x: f64, stable: bool }, PROBE: f64, roots(impl Fn(f64) -> f64, lo: f64, hi: f64, cells: usize) -> Vec<Root>}`
  - `graph::{Neighborhood::{VonNeumann, Moore}, Placement::{Random, Hubs, Leaves}}` (serde snake_case), `lattice(side: usize, Neighborhood, periodic: bool) -> Vec<Vec<u32>>`, `erdos_renyi(n: usize, mean_degree: f64, &mut SimRng) -> Vec<Vec<u32>>`, `place(degrees: &[usize], count: usize, Placement, &mut SimRng) -> Vec<bool>`, `bernoulli(n: usize, p: f64, &mut SimRng) -> Vec<bool>`
  - `engine::{Accumulate::{Sum, Mean}}` (serde snake_case), `Population { graph, nodes, bot, humans, strategy, action, cooperating, strategy_sum, human_strategy_sum }` with `new(Graph, Vec<bool>)`, `well_mixed`, `neighbors(i) -> &[u32]`, `degree(i)`, `set_strategy(i, f64)`, `set_action(i, bool)`, `set_pure(i, bool)`, `human_cooperation() -> f64`, `bot_cooperation() -> f64`, `random_node(&mut SimRng) -> usize`, `random_human(&mut SimRng) -> Option<usize>`, `partner(i, &mut SimRng) -> Option<usize>`; `fermi(beta, focal, model) -> f64`; `accumulate(total, degree, Accumulate) -> f64`

- [ ] **Step 1: Amend the spec**

Append this section to the end of the spec, verbatim:

```markdown
## Amendments (2026-10-20, planning, before any measurement)

Planning read the published code and pseudocode line by line and settled what the design left
open. Each item names its default and, where there is one, its named switch.

1. **Si et al.'s well-mixed payoff drops the S and P terms.** In every well-mixed discrete and
   mixed file (Figs. 1A, 1C, 2A, 2C, 6 Discrete, 6 Mixed) the payoff is
   `(double) c / nn * M[a][0] + (nn - c) / nn * M[a][1]` with `int c, nn`: the second share is
   integer division, 0 unless no other player cooperates. A cooperator earns c/(N − 1), a
   defector (1 + r)·c/(N − 1); S = −r and P = 0 never enter. Every such file subtracts the focal
   from the cooperator count when the focal cooperates, so the files agree on self-exclusion.
   The continuous files and every lattice file compute the stated game. Default: the code
   (`division: integer`, for docking). Named switch: `division: float` (the stated Eq. 1).
   **S2 is judged twice**, each with its own verdict: as the code runs it, and on the stated game.
2. **Si et al.'s deposit does not compile as published.** Three files begin with a stray `c` or
   `cc` before `#include` (Fig. 3B, Fig. 3C, Fig. 5 continuous), and the well-mixed files
   initialize a variable-length array (`double record_fc[loop]={0}` with `int loop`), which clang
   rejects. The docking build removes the stray characters and makes `loop` a `const int`; it
   also pins each file's parameter loops to one point, sets the seed and splits each point's 100
   runs into four processes of 25 (seeds 4p + 1 … 4p + 4 for point p). Every patch is recorded
   with the source and patched hashes. Per-run values are read from each run's stdout line.
3. **Si et al.'s well-mixed mixed update** (Figs. 1C, 2C, 6 Mixed) draws a role model for every
   picked node, lets an ordinary player imitate, then redraws the picked node's action from its
   strategy, bots included. The lattice mixed payoff redraws both players' actions for every
   game. Both are the defaults; `mixed_draw: per_round` draws every action once per round.
4. **Guo et al.'s Algorithm II** sets a bot chosen as focal to C with probability φ and to D
   otherwise (`bot_action: redraw`, the default); `frozen` keeps the initial draw;
   `per_interaction` draws a fresh action for every game and for every imitation of a bot, on
   graphs only.
5. **Guo et al.'s Algorithm I** fixes no run length. Ours, for G3: bots keep the action drawn at
   the start (the pseudocode draws once and never chooses a bot to learn); 10⁶ comparisons; the
   mean human cooperation over the last 5 × 10⁵, read after every comparison; 20 runs; humans
   start cooperating with p = 0.5, and a cell with two stable roots is run twice, from p = 0 and
   from p = 1, each compared with the stable root on its side. N = 1,000 with
   round(1,000·y/(1 + y)) bots.
6. **G2 with several stable roots** compares highest with highest and lowest with lowest.
7. **B3, operational.** On Eq. 18 (and separately Eq. 20), let x̄(b) be the largest stable fixed
   point and b*(β_H) the smallest b in {1.1, 1.11, …, 3} with x̄ ≥ 0.5 (∞ if none). Statement 1
   ("lower β cooperates at lower b") holds if b*(0.1) < b*(0.5) at α = 0.1, β_AI = 1 (the
   Fig. 3 pair). Statement 2 ("x = 0 is a fixed point at α = 0.5") holds if |ẋ(0)| < 1e-12 at
   α = 0.5, β_H = 0.5, β_AI = 1 for every b on that grid. The claim holds if both hold.
8. **Compute.** A pilot times one run of every simulated protocol. A scored row whose projected
   wall time exceeds 8 hours on the survey machine has its run count cut to the largest count that
   fits (at least 10) and is labeled "reduced"; sizes, lengths and windows never change. An
   evaluation row that would exceed 8 hours is reported as not run, with its projection.
9. **Graphs per run.** A new graph per run (Guo, Si, coloring). Booker's Barabási–Albert protocol:
   ten networks (`graph_seed` 1–10) × 20 runs each. Booker's lattice bot count is
   round(α·2,500).
10. **B5's `sweep_steps` row** is judged by B5's rule as its own variant row.
11. **Bootstrap (G4, G5):** 2,000 resamples of runs within each φ, survey seed 42, percentile 95%.
12. **Coloring.** A Barabási–Albert graph grown from a triangle with two links per node is
    2-degenerate, so it is always 3-colorable: "by construction" holds and the rejection step
    never fires (kept as an assertion). The solution count is exact backtracking in arrival order
    (deletion–contraction would branch 2³⁷ times on 37 edges); deletion–contraction's closed forms
    (paths, cycles, complete graphs, wheels) are its tests. Each bot's first decision falls at a
    uniform offset in [0, 1.5 s) (ours). An errant human move picks uniformly among the colors
    that would raise the node's conflicts, and keeps its color if none would (ours). Extended Data
    Fig. 2's recolored node is drawn uniformly from all nodes (literal); a repeat that has not
    converged after 10⁷ steps is censored and reported.
13. **Live changes.** Game parameters (β, the payoff accumulation, b, c, T, S, φ, r, the
    bot-action, mixed-draw and division switches, and the coloring rates) apply to the running
    world; structure, population, framework, θ, the step unit and the time limit apply on reset,
    which rebuilds the world from the current seed.
14. **View.** One cell per node, so bots are drawn in their own shades instead of outlined.
```

Then make three replacements (each `old` text occurs once): in **World `coloring`**, "kept only if 3-colorable (rejection, ours; the paper says "by construction")" → "always 3-colorable (amendment 12)", and "Exact: the chromatic polynomial at 3 by deletion–contraction (20 nodes is small)." → "Exact: the number of proper 3-colorings, by backtracking (amendment 12)."; in **Configuration**, "Live changes rebuild the world from the current seed." → "Live changes follow amendment 13."

- [ ] **Step 2: Write the failing tests**

Create `crates/sugarscape-core/src/hybrid/mod.rs`:

```rust
//! Hybrid human–machine populations (milestone 42): fixed-behavior bots among imitating humans.
//! Four worlds share one engine: Booker et al.'s donation game with Samaritan or discriminatory
//! bots, Guo et al.'s (T, S) plane with probabilistic bots, Si et al.'s r-family Prisoner's
//! Dilemma in three strategy frameworks, and Shirado and Christakis's network coloring game with
//! noisy bots. Exact references are pure functions of the config.
pub mod engine;
pub mod exact;
pub mod graph;
```

Append to `crates/sugarscape-core/src/lib.rs`, after the final `pub mod negotiable;` line, so the file ends exactly with:

```rust
pub mod negotiable;

/// Hybrid human–machine populations: bots as a second species (milestone 42).
pub mod hybrid;
```

Create `crates/sugarscape-core/src/hybrid/exact.rs` with only its doc comment and tests for now:

```rust
//! Numerics shared by the exact references: the logistic function, tanh(x/2) and eˣ from the
//! portable exponential, log-sum-exp, ln n!, and bracketing root scans that read the flow's
//! direction on each side of a root.

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
    fn roots_find_crossings_and_read_stability() {
        // ẋ = x(1 − x)(x − 0.3): 0 and 1 stable, 0.3 unstable.
        let r = roots(|x| x * (1.0 - x) * (x - 0.3), 0.0, 1.0, 1000);
        assert_eq!(r.len(), 3, "{r:?}");
        assert!(r[0].x.abs() < 1e-15 && (r[1].x - 0.3).abs() < 1e-15 && (r[2].x - 1.0).abs() < 1e-15);
        assert_eq!(r.iter().map(|r| r.stable).collect::<Vec<_>>(), vec![true, false, true]);
        // A crossing between samples is bisected to adjacent floats.
        let one = roots(|x| 0.123_456_789 - x, 0.0, 1.0, 10);
        assert_eq!(one.len(), 1);
        assert!((one[0].x - 0.123_456_789).abs() < 1e-15 && one[0].stable);
    }
}
```

Create `crates/sugarscape-core/src/hybrid/graph.rs` with only its doc comment and tests:

```rust
//! The structures the hybrid worlds use, on top of `crate::graph`: square lattices (von Neumann
//! or Moore, periodic or open), Erdős–Rényi with a target mean degree, and bot placement (random,
//! hubs or leaves, or one Bernoulli draw per node as in Si et al.'s code).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::barabasi_albert, rng::seeded};

    #[test]
    fn periodic_lattices_have_full_degree_and_wrap() {
        let vn = lattice(5, Neighborhood::VonNeumann, true);
        assert!(vn.iter().all(|n| n.len() == 4));
        assert_eq!(vn[0], vec![20, 5, 4, 1], "up wraps to the bottom row, left to the last column");
        let moore = lattice(5, Neighborhood::Moore, true);
        assert!(moore.iter().all(|n| n.len() == 8));
        assert_eq!(&moore[0][4..], &[24, 21, 9, 6]);
        for (i, ns) in moore.iter().enumerate() {
            for &j in ns {
                assert!(moore[j as usize].contains(&(i as u32)), "symmetric at {i}–{j}");
            }
        }
    }

    #[test]
    fn open_lattices_lose_the_edge_links() {
        let g = lattice(50, Neighborhood::VonNeumann, false);
        let degrees: Vec<usize> = g.iter().map(Vec::len).collect();
        assert_eq!((degrees[0], degrees[1], degrees[51]), (2, 3, 4));
        assert_eq!(degrees.iter().sum::<usize>(), 2 * 2 * 50 * 49);
    }

    #[test]
    fn random_graphs_have_the_target_mean_degree() {
        let mut rng = seeded(1);
        let er = erdos_renyi(10_000, 4.0, &mut rng);
        let mean = er.iter().map(Vec::len).sum::<usize>() as f64 / 10_000.0;
        assert!((mean - 4.0).abs() < 0.1, "{mean}");
        let ba = barabasi_albert(1000, 2, &mut rng);
        assert_eq!(ba.iter().map(Vec::len).sum::<usize>() / 2, 3 + 2 * 997, "a triangle, then two links per node");
    }

    #[test]
    fn placement_orders_by_degree_with_ties_by_index() {
        let degrees = [2, 5, 5, 1, 3, 1];
        let mut rng = seeded(1);
        assert_eq!(place(&degrees, 3, Placement::Hubs, &mut rng), vec![false, true, true, false, true, false]);
        assert_eq!(place(&degrees, 3, Placement::Leaves, &mut rng), vec![true, false, false, true, false, true]);
        assert_eq!(place(&degrees, 4, Placement::Random, &mut rng).iter().filter(|&&b| b).count(), 4);
        assert_eq!(place(&degrees, 9, Placement::Random, &mut rng), vec![true; 6], "at most every node");
        let b = bernoulli(100_000, 0.3, &mut seeded(2));
        let share = b.iter().filter(|&&x| x).count() as f64 / 1e5;
        assert!((share - 0.3).abs() < 0.01, "{share}");
    }
}
```

Create `crates/sugarscape-core/src/hybrid/engine.rs` with only its doc comment and tests:

```rust
//! The population every hybrid world but coloring shares: a graph (empty when well-mixed), which
//! nodes are bots, each node's strategy (its probability of cooperating; 0 or 1 in a pure
//! framework) and current action, with the counts the well-mixed payoffs read; the Fermi rule;
//! role-model draws.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::seeded;

    fn ring4() -> Graph {
        Graph::from_lists(vec![vec![1, 3], vec![0, 2], vec![1, 3], vec![2, 0]])
    }

    #[test]
    fn counts_follow_every_change() {
        let mut p = Population::new(ring4(), vec![true, false, false, false]);
        assert_eq!(p.humans, vec![1, 2, 3]);
        p.set_pure(0, true);
        p.set_pure(1, true);
        p.set_strategy(2, 0.5);
        assert_eq!((p.cooperating, p.strategy_sum, p.human_strategy_sum), (2, 2.5, 1.5));
        assert_eq!((p.human_cooperation(), p.bot_cooperation()), (0.5, 1.0));
        p.set_pure(1, false);
        assert_eq!((p.cooperating, p.human_strategy_sum), (1, 0.5));
        p.set_action(1, false);
        assert_eq!(p.cooperating, 1, "setting an action to itself changes nothing");
    }

    #[test]
    fn isolated_nodes_have_no_partner_and_mean_payoffs_of_zero() {
        let p = Population::new(Graph::from_lists(vec![vec![1], vec![0], vec![]]), vec![false; 3]);
        let mut rng = seeded(1);
        assert_eq!(p.partner(2, &mut rng), None);
        assert_eq!(p.partner(0, &mut rng), Some(1));
        assert_eq!(accumulate(0.0, 0, Accumulate::Mean), 0.0);
        assert_eq!(accumulate(6.0, 3, Accumulate::Mean), 2.0);
        assert_eq!(accumulate(6.0, 3, Accumulate::Sum), 6.0);
    }

    #[test]
    fn well_mixed_partners_are_any_other_node() {
        let p = Population::new(Graph::default(), vec![false; 5]);
        assert!(p.well_mixed() && p.degree(0) == 4 && p.neighbors(0).is_empty());
        let mut rng = seeded(3);
        let mut seen = [0; 5];
        for _ in 0..10_000 {
            seen[p.partner(2, &mut rng).unwrap()] += 1;
        }
        assert_eq!(seen[2], 0);
        assert!(seen.iter().enumerate().all(|(i, &n)| i == 2 || (2300..2700).contains(&n)), "{seen:?}");
    }

    #[test]
    fn a_population_without_humans_reports_nan() {
        let p = Population::new(ring4(), vec![true; 4]);
        assert!(p.human_cooperation().is_nan());
        assert_eq!(p.random_human(&mut seeded(1)), None);
        let q = Population::new(ring4(), vec![false; 4]);
        assert!(q.bot_cooperation().is_nan());
    }

    #[test]
    fn fermi_is_one_half_at_equal_payoffs_and_saturates() {
        assert_eq!(fermi(10.0, 1.0, 1.0), 0.5);
        assert!(fermi(10.0, 0.0, 1.0) > 0.9999 && fermi(10.0, 1.0, 0.0) < 1e-4);
        assert_eq!(fermi(0.0, 5.0, -5.0), 0.5);
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: compile errors naming `logistic`, `roots`, `lattice`, `Population` and the rest (nothing is defined yet).

- [ ] **Step 4: Write `exact.rs`**

Above the tests module:

```rust
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
/// cannot shrink. A root is stable when the flow points toward it on every side it has inside the
/// interval (f > 0 just left of it, f < 0 just right), read at ±`PROBE`.
pub fn roots(f: impl Fn(f64) -> f64, lo: f64, hi: f64, cells: usize) -> Vec<Root> {
    let at = |k: usize| lo + (hi - lo) * k as f64 / cells as f64;
    let mut xs = vec![];
    let (mut px, mut pf) = (at(0), f(at(0)));
    if pf == 0.0 {
        xs.push(px);
    }
    for k in 1..=cells {
        let x = at(k);
        let fx = f(x);
        if fx == 0.0 {
            xs.push(x);
        } else if pf * fx < 0.0 {
            let (mut a, mut b) = (px, x);
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
                if (fm < 0.0) == (pf < 0.0) {
                    a = m;
                } else {
                    b = m;
                }
            }
            xs.push(0.5 * (a + b));
        }
        (px, pf) = (x, fx);
    }
    xs.into_iter()
        .map(|x| {
            let left = (x - PROBE >= lo).then(|| f(x - PROBE) > 0.0);
            let right = (x + PROBE <= hi).then(|| f(x + PROBE) < 0.0);
            let stable =
                (left.is_some() || right.is_some()) && left.unwrap_or(true) && right.unwrap_or(true);
            Root { x, stable }
        })
        .collect()
}
```

- [ ] **Step 5: Write `graph.rs`**

Above the tests module:

```rust
use crate::{graph::gnp_sparse, rng::SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    VonNeumann,
    Moore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    Random,
    Hubs,
    Leaves,
}

/// A side × side lattice, node i at (i mod side, i div side). Neighbors in Si et al.'s order:
/// up, down, left, right, then (Moore) up-left, up-right, down-left, down-right. Open boundaries
/// drop links that would leave the square; a side of at least 3 keeps periodic neighbors distinct.
pub fn lattice(side: usize, hood: Neighborhood, periodic: bool) -> Vec<Vec<u32>> {
    let offsets: &[(i64, i64)] = match hood {
        Neighborhood::VonNeumann => &[(0, -1), (0, 1), (-1, 0), (1, 0)],
        Neighborhood::Moore => &[
            (0, -1),
            (0, 1),
            (-1, 0),
            (1, 0),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ],
    };
    let s = side as i64;
    (0..side * side)
        .map(|i| {
            let (x, y) = ((i % side) as i64, (i / side) as i64);
            offsets
                .iter()
                .filter_map(|&(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    if periodic {
                        Some((nx.rem_euclid(s) + ny.rem_euclid(s) * s) as u32)
                    } else if (0..s).contains(&nx) && (0..s).contains(&ny) {
                        Some((nx + ny * s) as u32)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

/// Erdős–Rényi G(n, p) with p = mean_degree / (n − 1), drawn in time proportional to its links.
pub fn erdos_renyi(n: usize, mean_degree: f64, rng: &mut SimRng) -> Vec<Vec<u32>> {
    gnp_sparse(n, (mean_degree / (n - 1) as f64).min(1.0), rng)
}

/// `count` bots as a flag per node. Hubs: highest degree first; leaves: lowest first; ties to the
/// lower index. Random: a uniform sample without replacement (a partial Fisher–Yates shuffle).
pub fn place(degrees: &[usize], count: usize, placement: Placement, rng: &mut SimRng) -> Vec<bool> {
    let n = degrees.len();
    let count = count.min(n);
    let mut order: Vec<usize> = (0..n).collect();
    match placement {
        Placement::Random => {
            for k in 0..count {
                let j = k + rng.gen_range(0..(n - k) as u32) as usize;
                order.swap(k, j);
            }
        }
        Placement::Hubs => order.sort_by(|&a, &b| degrees[b].cmp(&degrees[a]).then(a.cmp(&b))),
        Placement::Leaves => order.sort_by(|&a, &b| degrees[a].cmp(&degrees[b]).then(a.cmp(&b))),
    }
    let mut bot = vec![false; n];
    for &i in &order[..count] {
        bot[i] = true;
    }
    bot
}

/// Each node a bot with probability p: one uniform draw per node in index order (Si et al.'s
/// `randf() < rho`).
pub fn bernoulli(n: usize, p: f64, rng: &mut SimRng) -> Vec<bool> {
    (0..n).map(|_| rng.gen::<f64>() < p).collect()
}
```

- [ ] **Step 6: Write `engine.rs`**

Above the tests module:

```rust
use super::exact::logistic;
use crate::{graph::Graph, rng::SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// How a node's games add up: summed over its neighbors (all three papers) or averaged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Accumulate {
    Sum,
    Mean,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Population {
    /// Neighbor lists; empty (`is_complete`) when well-mixed.
    pub graph: Graph,
    pub nodes: usize,
    pub bot: Vec<bool>,
    /// The human nodes, in index order.
    pub humans: Vec<u32>,
    /// Each node's probability of cooperating (0 or 1 in a pure framework).
    pub strategy: Vec<f64>,
    /// Each node's current action, true = cooperate.
    pub action: Vec<bool>,
    /// Nodes acting C now.
    pub cooperating: usize,
    /// Σ strategy over all nodes, and over the humans.
    pub strategy_sum: f64,
    pub human_strategy_sum: f64,
}

impl Population {
    /// Every node defecting with strategy 0.
    pub fn new(graph: Graph, bot: Vec<bool>) -> Self {
        let nodes = bot.len();
        let humans = (0..nodes as u32).filter(|&i| !bot[i as usize]).collect();
        Population {
            graph,
            nodes,
            bot,
            humans,
            strategy: vec![0.0; nodes],
            action: vec![false; nodes],
            cooperating: 0,
            strategy_sum: 0.0,
            human_strategy_sum: 0.0,
        }
    }

    pub fn well_mixed(&self) -> bool {
        self.graph.is_complete()
    }

    /// Node i's neighbors (empty when well-mixed).
    pub fn neighbors(&self, i: usize) -> &[u32] {
        if self.well_mixed() {
            &[]
        } else {
            self.graph.of(i)
        }
    }

    /// How many games node i plays: its neighbors, or every other node when well-mixed.
    pub fn degree(&self, i: usize) -> usize {
        if self.well_mixed() {
            self.nodes - 1
        } else {
            self.graph.of(i).len()
        }
    }

    pub fn set_strategy(&mut self, i: usize, s: f64) {
        let d = s - self.strategy[i];
        self.strategy_sum += d;
        if !self.bot[i] {
            self.human_strategy_sum += d;
        }
        self.strategy[i] = s;
    }

    pub fn set_action(&mut self, i: usize, cooperate: bool) {
        if self.action[i] != cooperate {
            if cooperate {
                self.cooperating += 1;
            } else {
                self.cooperating -= 1;
            }
            self.action[i] = cooperate;
        }
    }

    /// A pure strategy and its action together.
    pub fn set_pure(&mut self, i: usize, cooperate: bool) {
        self.set_strategy(i, if cooperate { 1.0 } else { 0.0 });
        self.set_action(i, cooperate);
    }

    /// The humans' mean strategy, their cooperation (NaN without humans).
    pub fn human_cooperation(&self) -> f64 {
        if self.humans.is_empty() {
            f64::NAN
        } else {
            self.human_strategy_sum / self.humans.len() as f64
        }
    }

    /// The share of bots acting C now (NaN without bots).
    pub fn bot_cooperation(&self) -> f64 {
        let bots = self.nodes - self.humans.len();
        if bots == 0 {
            return f64::NAN;
        }
        let c = (0..self.nodes).filter(|&i| self.bot[i] && self.action[i]).count();
        c as f64 / bots as f64
    }

    pub fn random_node(&self, rng: &mut SimRng) -> usize {
        rng.gen_range(0..self.nodes as u32) as usize
    }

    /// A uniformly random human (None without humans).
    pub fn random_human(&self, rng: &mut SimRng) -> Option<usize> {
        if self.humans.is_empty() {
            return None;
        }
        Some(self.humans[rng.gen_range(0..self.humans.len() as u32) as usize] as usize)
    }

    /// The role model for node i: a uniformly random neighbor (None when isolated), or, when
    /// well-mixed, a uniformly random node redrawn until it is not i (Si et al.'s code).
    pub fn partner(&self, i: usize, rng: &mut SimRng) -> Option<usize> {
        if self.well_mixed() {
            assert!(self.nodes >= 2, "a well-mixed population needs two nodes");
            loop {
                let j = self.random_node(rng);
                if j != i {
                    return Some(j);
                }
            }
        }
        let ns = self.graph.of(i);
        if ns.is_empty() {
            return None;
        }
        Some(ns[rng.gen_range(0..ns.len() as u32) as usize] as usize)
    }
}

/// The probability that a focal earning `focal` adopts a model earning `model`:
/// 1 / (1 + e^{−β(π_model − π_focal)}).
pub fn fermi(beta: f64, focal: f64, model: f64) -> f64 {
    logistic(beta * (model - focal))
}

/// A summed payoff, or its mean over `degree` games (0 for an isolated node).
pub fn accumulate(total: f64, degree: usize, how: Accumulate) -> f64 {
    match how {
        Accumulate::Sum => total,
        Accumulate::Mean if degree == 0 => 0.0,
        Accumulate::Mean => total / degree as f64,
    }
}
```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: 11 passed.

- [ ] **Step 8: Re-record `lib.rs` in the key-registration fixture and the vitest identity fixture**

Write `$SCRATCH/m42/rekey.py` and run it with `python3 -I $SCRATCH/m42/rekey.py`:

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

Then:

1. `web/src/episodes/test-fixtures/historical-source.ts`: in the `reviewedAdditions` string, replace its ending `\npub mod negotiable;\n'` with `\npub mod negotiable;\n\n/// Hybrid human–machine populations: bots as a second species (milestone 42).\npub mod hybrid;\n'` (byte-identical to Step 2's append), and in the message string on line 29 replace `misaligned/negotiable` with `misaligned/negotiable/hybrid`.
2. `web/src/episodes/caching-parity.test.ts`: rename the test on line 164 to `'authenticates the exact combined war, discovery, Gate, Key, assistance, gridworlds, misaligned, negotiable and hybrid additions against the frozen historical lib'`, and after the two negotiable tamper cases (lines 185–186) add `live.replace('pub mod hybrid;', 'pub mod other_hybrid;'),` and `live.replace('\npub mod hybrid;\n', '\n'),`.

Run: `cargo test -p sugarscape-core browser_experiments 2>&1 | tail -3` (expected: pass) and `cd web && npx vitest run src/episodes/caching-parity.test.ts 2>&1 | tail -4` (expected: pass; a failure names the byte where `reviewedAdditions` and `lib.rs` differ).

- [ ] **Step 9: Commit**

```bash
git add docs/superpowers/specs/2026-10-17-hybrid-populations-design.md crates/sugarscape-core/src/hybrid crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json web/src/episodes
git commit -m "hybrid: spec amendments, shared numerics, structures and the population

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push -u origin hybrid
```

---

### Task 2: Booker's exact references (`booker.rs`, first part)

**Files:**
- Create: `crates/sugarscape-core/src/hybrid/booker.rs`
- Modify: `crates/sugarscape-core/src/hybrid/mod.rs` (`pub mod booker;`)

**Interfaces:**
- Consumes: Task 1's `exact::{exp, ln_factorial, log_sum_exp, logistic, roots, tanh_half, Root}`.
- Produces: `BotKind::{Samaritan, Discriminatory, Malicious}` (serde snake_case); `Donation { b, c }` with `r()`, `s()`, `t()`, `p()`, `payoff(me: bool, other: bool) -> f64`; `Chain { n: u32, m: u32, game: Donation, kind: BotKind, beta: f64 }` with `pi_c(k)`, `pi_d(k)`, `delta(k)`, `t_plus(k)`, `t_minus(k)`, `ln_ratio()`, `all_c_weight()`, `fixation() -> (f64, f64)` (ρ_{D,C}, ρ_{C,D}), `f_closed()`, `f_direct()`, `ln_g_closed()`, `ln_g_direct()`, `ln_g_d_printed()`; `Replicator { game, alpha, beta_h, beta_ai, kind }` with `delta(x) -> Option<f64>`, `rate(x) -> Option<f64>`, `fixed_points() -> Option<Vec<Root>>`, `top_stable() -> Option<f64>`; `alpha_c_general(beta_h, beta_ai, t_minus_r)`, `alpha_c_printed(beta, t_minus_r)`, `alpha_c_corrected(beta, t_minus_r)`, `alpha_c_scan(Donation, beta_h, beta_ai)`.

Notation: N humans, M bots, k cooperating humans. Every agent plays every other once; the focal is a uniform human, the role model uniform among all N + M agents (Eq 6). For k = 1 … N − 1 both sampling factors are positive, so every logarithm below is of a positive normal number.

- [ ] **Step 1: Write the failing tests**

Create `booker.rs` with only its doc comment and tests, and add `pub mod booker;` to `mod.rs` (alphabetical, first):

```rust
//! Booker et al. (arXiv 2306.17747v2): the donation game T = b, R = b − c, P = 0, S = −c with
//! Samaritan (always C), discriminatory (mirroring the partner) or malicious (always D) bots. The
//! finite well-mixed chain (Eqs 1–15) and the replicator equations (Eqs 16–20) are exact
//! references; the network dynamics follow in this file's second part.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portable::ln;

    const GAME: Donation = Donation { b: 2.0, c: 1.0 };
    const KINDS: [BotKind; 3] = [BotKind::Samaritan, BotKind::Discriminatory, BotKind::Malicious];

    fn chain(m: u32, kind: BotKind, beta: f64) -> Chain {
        Chain { n: 100, m, game: GAME, kind, beta }
    }

    #[test]
    fn the_donation_game_has_the_papers_entries() {
        assert_eq!((GAME.r(), GAME.s(), GAME.t(), GAME.p()), (1.0, -1.0, 2.0, 0.0));
        assert_eq!((GAME.payoff(true, true), GAME.payoff(true, false)), (1.0, -1.0));
        assert_eq!((GAME.payoff(false, true), GAME.payoff(false, false)), (2.0, 0.0));
    }

    #[test]
    fn closed_forms_match_direct_sums() {
        for kind in KINDS {
            for m in [0, 1, 3, 50, 100] {
                for b in [1.1, 2.0, 3.0] {
                    let c = Chain { n: 100, m, game: Donation { b, c: 1.0 }, kind, beta: 1.0 };
                    let (fc, fd) = (c.f_closed(), c.f_direct());
                    assert!((fc - fd).abs() < 1e-9 * fd.abs().max(1.0), "F {kind:?} M {m} b {b}: {fc} vs {fd}");
                    let (gc, gd) = (c.ln_g_closed(), c.ln_g_direct());
                    assert!((gc - gd).abs() < 1e-9 * gd.abs().max(1.0), "ln G {kind:?} M {m}: {gc} vs {gd}");
                }
            }
        }
    }

    #[test]
    fn the_printed_g_d_is_off_by_a_factor_m_plus_one() {
        // N = 10, M = 3: (N − 1)! M! / (N + M − 1)! = 9! 3! / 12! = 1/220; Eq 14 prints 9! 4! / 12!.
        let c = Chain { n: 10, m: 3, game: GAME, kind: BotKind::Malicious, beta: 1.0 };
        assert!((exp(c.ln_g_direct()) - 1.0 / 220.0).abs() < 1e-15);
        assert!((c.ln_g_d_printed() - c.ln_g_direct() - ln(4.0)).abs() < 1e-12);
    }

    #[test]
    fn transition_rates_follow_eq_6() {
        let c = chain(10, BotKind::Samaritan, 1.0);
        let k = 30;
        // T⁺ = ((N − k)/N)((k + M)/(N + M)) / (1 + e^{−βΔ}); T⁻ = (k/N)((N − k)/(N + M)) / (1 + e^{βΔ}).
        let d = c.delta(k);
        let up = (70.0 / 100.0) * (40.0 / 110.0) * logistic(d);
        let down = (30.0 / 100.0) * (70.0 / 110.0) * logistic(-d);
        assert!((c.t_plus(k) - up).abs() < 1e-15 && (c.t_minus(k) - down).abs() < 1e-15);
        let ir = chain(10, BotKind::Discriminatory, 1.0);
        assert!((ir.t_plus(k) - (70.0 / 100.0) * (30.0 / 110.0) * logistic(ir.delta(k))).abs() < 1e-15, "bots are null draws");
        let bad = chain(10, BotKind::Malicious, 1.0);
        assert!((bad.t_minus(k) - (30.0 / 100.0) * (80.0 / 110.0) * logistic(-bad.delta(k))).abs() < 1e-15, "bots are D role models");
    }

    #[test]
    fn the_stationary_weight_agrees_with_the_fixation_probabilities() {
        for (m, kind, beta) in [(5, BotKind::Samaritan, 0.1), (57, BotKind::Samaritan, 1.0), (80, BotKind::Discriminatory, 2.5), (20, BotKind::Malicious, 0.5)] {
            let c = chain(m, kind, beta);
            let (dc, cd) = c.fixation();
            assert!((dc / (dc + cd) - c.all_c_weight()).abs() < 1e-12, "{kind:?} {m}");
            assert!((0.0..=1.0).contains(&dc) && (0.0..=1.0).contains(&cd));
            let expect = c.ln_g_direct() + beta * c.f_direct();
            assert!((c.ln_ratio() - expect).abs() < 1e-9 * expect.abs().max(1.0), "ln r = ln G + βF");
        }
        assert_eq!(chain(0, BotKind::Samaritan, 1.0).all_c_weight(), chain(0, BotKind::Discriminatory, 1.0).all_c_weight(), "no bots, no difference");
    }

    #[test]
    fn the_chain_is_finite_at_strong_selection() {
        for beta in [10.0, 100.0] {
            for m in [0, 1, 100] {
                for kind in KINDS {
                    let c = chain(m, kind, beta);
                    let (dc, cd) = c.fixation();
                    assert!(c.all_c_weight().is_finite() && dc.is_finite() && cd.is_finite(), "{kind:?} M {m} β {beta}");
                }
            }
        }
    }

    #[test]
    fn eq_18_has_a_constant_payoff_gap_and_never_fixes_zero() {
        for b in [1.1, 2.0, 3.0] {
            let r = Replicator { game: Donation { b, c: 1.0 }, alpha: 0.1, beta_h: 0.1, beta_ai: 2.0, kind: BotKind::Samaritan };
            for x in [0.0, 0.3, 1.0] {
                assert!((r.delta(x).unwrap() + 1.0).abs() < 1e-15, "Δf = −c at x {x}, b {b}");
            }
            assert!(r.rate(0.0).unwrap() > 0.0);
        }
        let bad = Replicator { game: GAME, alpha: 0.1, beta_h: 1.0, beta_ai: 1.0, kind: BotKind::Malicious };
        assert_eq!((bad.rate(0.5), bad.fixed_points()), (None, None), "the paper has no malicious equation");
    }

    #[test]
    fn the_interior_root_of_eq_18_is_where_h_vanishes() {
        let r = Replicator { game: GAME, alpha: 0.1, beta_h: 1.0, beta_ai: 1.0, kind: BotKind::Samaritan };
        // h(x) = x(1 − α) tanh(−βc/2) + α logistic(−βc) is linear in x.
        let want = 0.1 * logistic(-1.0) / (0.9 * tanh_half(1.0));
        let pts = r.fixed_points().unwrap();
        assert!(pts.iter().any(|p| (p.x - want).abs() < 1e-12 && p.stable), "{pts:?}");
        assert!(pts.iter().any(|p| p.x == 1.0 && !p.stable));
        assert!((r.top_stable().unwrap() - want).abs() < 1e-12);
    }

    #[test]
    fn eq_20_fixes_both_ends() {
        let r = Replicator { game: Donation { b: 3.0, c: 1.0 }, alpha: 0.5, beta_h: 0.5, beta_ai: 1.0, kind: BotKind::Discriminatory };
        assert_eq!((r.rate(0.0), r.rate(1.0)), (Some(0.0), Some(0.0)));
        // Δf = −c(1 − α) + α(b − c) = 0.5 > 0: cooperation takes over.
        assert!((r.delta(0.4).unwrap() - 0.5).abs() < 1e-15);
        assert_eq!(r.top_stable(), Some(1.0));
    }

    #[test]
    fn alpha_c_reduces_to_one_minus_e_to_the_minus_beta_c() {
        for beta in [0.1, 1.0, 2.5] {
            let general = alpha_c_general(beta, beta, 1.0);
            assert!((general - alpha_c_corrected(beta, 1.0)).abs() < 1e-12, "β {beta}");
            assert!((alpha_c_scan(GAME, beta, beta) - general).abs() < 1e-9, "β {beta}");
        }
        assert!((alpha_c_corrected(1.0, 1.0) - 0.632_120_558_828_557_7).abs() < 1e-15);
        assert!((alpha_c_printed(1.0, 1.0) - 0.268_941_421_369_995_1).abs() < 1e-15);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid::booker 2>&1 | tail -5`. Expected: compile errors (`Donation`, `Chain`, `Replicator` undefined).

- [ ] **Step 3: Write the references**

Above the tests module:

```rust
use super::exact::{exp, ln_factorial, log_sum_exp, logistic, roots, tanh_half, Root};
use crate::portable::ln;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotKind {
    Samaritan,
    Discriminatory,
    Malicious,
}

/// The donation game.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Donation {
    pub b: f64,
    pub c: f64,
}

impl Donation {
    pub fn r(&self) -> f64 {
        self.b - self.c
    }
    pub fn s(&self) -> f64 {
        -self.c
    }
    pub fn t(&self) -> f64 {
        self.b
    }
    pub fn p(&self) -> f64 {
        0.0
    }
    /// The row player's payoff (true = cooperate).
    pub fn payoff(&self, me: bool, other: bool) -> f64 {
        match (me, other) {
            (true, true) => self.r(),
            (true, false) => self.s(),
            (false, true) => self.t(),
            (false, false) => self.p(),
        }
    }
}

/// Booker's finite well-mixed chain: `n` humans, `m` bots of one kind, Fermi intensity `beta`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chain {
    pub n: u32,
    pub m: u32,
    pub game: Donation,
    pub kind: BotKind,
    pub beta: f64,
}

impl Chain {
    /// (d_C, d_D, d_IR) of Eqs 1–5.
    fn d(&self) -> (f64, f64, f64) {
        match self.kind {
            BotKind::Samaritan => (1.0, 0.0, 0.0),
            BotKind::Malicious => (0.0, 1.0, 0.0),
            BotKind::Discriminatory => (0.0, 0.0, 1.0),
        }
    }

    /// Π_C(k), a cooperating human's mean payoff with k cooperating humans (Eqs 1, 4, 5).
    pub fn pi_c(&self, k: u32) -> f64 {
        let (n, m, k) = (f64::from(self.n), f64::from(self.m), f64::from(k));
        let g = &self.game;
        let (dc, dd, di) = self.d();
        ((k - 1.0) * g.r() + (n - k) * g.s() + m * (dc * g.r() + dd * g.s() + di * g.r()))
            / (n + m - 1.0)
    }

    /// Π_D(k).
    pub fn pi_d(&self, k: u32) -> f64 {
        let (n, m, k) = (f64::from(self.n), f64::from(self.m), f64::from(k));
        let g = &self.game;
        let (dc, dd, di) = self.d();
        (k * g.t() + (n - k - 1.0) * g.p() + m * (dc * g.t() + dd * g.p() + di * g.p()))
            / (n + m - 1.0)
    }

    pub fn delta(&self, k: u32) -> f64 {
        self.pi_c(k) - self.pi_d(k)
    }

    /// The logarithms of T⁺(k)'s and T⁻(k)'s sampling factors (Eq 6 without the Fermi terms):
    /// a C role model is a cooperating human or (Samaritan) a bot, a D role model a defecting
    /// human or (malicious) a bot; a discriminatory bot is a null draw.
    fn ln_sampling(&self, k: u32) -> (f64, f64) {
        let (n, m, k) = (f64::from(self.n), f64::from(self.m), f64::from(k));
        let (dc, _, di) = self.d();
        let up = ((n - k) / n) * ((k + (1.0 - di) * dc * m) / (n + m));
        let down = (k / n) * ((n - k + (1.0 - di) * (1.0 - dc) * m) / (n + m));
        (ln(up), ln(down))
    }

    /// T⁺(k), the probability that one more human cooperates after one step (Eq 6).
    pub fn t_plus(&self, k: u32) -> f64 {
        exp(self.ln_sampling(k).0) * logistic(self.beta * self.delta(k))
    }

    /// T⁻(k).
    pub fn t_minus(&self, k: u32) -> f64 {
        exp(self.ln_sampling(k).1) * logistic(-self.beta * self.delta(k))
    }

    /// ln(T⁺(k)/T⁻(k)) = ln(up/down) + βΔf(k): the Fermi terms' ratio is exactly e^{βΔf}.
    fn ln_step(&self, k: u32) -> f64 {
        let (up, down) = self.ln_sampling(k);
        up - down + self.beta * self.delta(k)
    }

    /// ln r with r = ∏_{k=1}^{N−1} T⁺(k)/T⁻(k) = ρ_{D,C}/ρ_{C,D}.
    pub fn ln_ratio(&self) -> f64 {
        (1..self.n).map(|k| self.ln_step(k)).sum()
    }

    /// The stationary weight on all-C, r/(1 + r), Fig. 1's "frequency of cooperation".
    pub fn all_c_weight(&self) -> f64 {
        logistic(self.ln_ratio())
    }

    /// (ρ_{D,C}, ρ_{C,D}) by Eq 7, in log space: ρ_{D,C} = 1/(1 + Σ_i ∏_{j≤i} T⁻/T⁺) and
    /// ρ_{C,D} = ρ_{D,C}/r.
    pub fn fixation(&self) -> (f64, f64) {
        let mut terms = vec![0.0];
        let mut acc = 0.0;
        for i in 1..self.n {
            acc -= self.ln_step(i);
            terms.push(acc);
        }
        let ln_dc = -log_sum_exp(&terms);
        (exp(ln_dc), exp(ln_dc - self.ln_ratio()))
    }

    /// F (Eqs 9–11) in closed form.
    pub fn f_closed(&self) -> f64 {
        let (n, m) = (f64::from(self.n), f64::from(self.m));
        let g = &self.game;
        let x = match self.kind {
            BotKind::Samaritan => g.r() - g.t(),
            BotKind::Malicious => g.s() - g.p(),
            BotKind::Discriminatory => g.r() - g.p(),
        };
        (n - 1.0) / (n + m - 1.0)
            * ((g.p() - g.r())
                + n * (g.s() - g.p())
                + m * x
                + (g.r() + g.p() - g.t() - g.s()) * n / 2.0)
    }

    /// F = Σ_{k=1}^{N−1} Δf(k), summed directly.
    pub fn f_direct(&self) -> f64 {
        (1..self.n).map(|k| self.delta(k)).sum()
    }

    /// ln G, G the product of the sampling ratios, directly.
    pub fn ln_g_direct(&self) -> f64 {
        (1..self.n)
            .map(|k| {
                let (up, down) = self.ln_sampling(k);
                up - down
            })
            .sum()
    }

    /// ln G from Eqs 13 and 15 and the corrected Eq 14, G_D = (N − 1)! M! / (N + M − 1)!.
    pub fn ln_g_closed(&self) -> f64 {
        let (n1, m) = (self.n - 1, self.m);
        match self.kind {
            BotKind::Samaritan => ln_factorial(n1 + m) - ln_factorial(n1) - ln_factorial(m),
            BotKind::Malicious => ln_factorial(n1) + ln_factorial(m) - ln_factorial(n1 + m),
            BotKind::Discriminatory => 0.0,
        }
    }

    /// ln of Eq 14 as printed: (N − 1)! (M + 1)! / (N + M − 1)!.
    pub fn ln_g_d_printed(&self) -> f64 {
        ln_factorial(self.n - 1) + ln_factorial(self.m + 1) - ln_factorial(self.n - 1 + self.m)
    }
}

/// Booker's infinite-population equations: x the cooperating share of humans, α the bots' share
/// of everyone, β_H and β_AI the intensities of copying humans and bots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Replicator {
    pub game: Donation,
    pub alpha: f64,
    pub beta_h: f64,
    pub beta_ai: f64,
    pub kind: BotKind,
}

impl Replicator {
    /// Δf(x) for Eq 18 (Samaritan) or Eq 20 (discriminatory); None for malicious bots, for which
    /// the paper gives no equation.
    pub fn delta(&self, x: f64) -> Option<f64> {
        let (g, a) = (&self.game, self.alpha);
        match self.kind {
            BotKind::Samaritan => Some(
                (g.r() - g.t()) * ((1.0 - a) * x + a) + (g.s() - g.p()) * (1.0 - a) * (1.0 - x),
            ),
            BotKind::Discriminatory => Some(
                (g.r() - g.t()) * (1.0 - a) * x
                    + (g.s() - g.p()) * (1.0 - a) * (1.0 - x)
                    + a * (g.r() - g.p()),
            ),
            BotKind::Malicious => None,
        }
    }

    /// ẋ: Eq 18, (1 − x)[x(1 − α) tanh(β_H Δf/2) + α/(1 + e^{−β_AI Δf})], or Eq 20,
    /// (1 − x) x (1 − α) tanh(β_H Δf/2).
    pub fn rate(&self, x: f64) -> Option<f64> {
        let d = self.delta(x)?;
        let a = self.alpha;
        Some(match self.kind {
            BotKind::Samaritan => {
                (1.0 - x) * (x * (1.0 - a) * tanh_half(self.beta_h * d) + a * logistic(self.beta_ai * d))
            }
            _ => (1.0 - x) * x * (1.0 - a) * tanh_half(self.beta_h * d),
        })
    }

    /// The fixed points on [0, 1], 10,000 cells.
    pub fn fixed_points(&self) -> Option<Vec<Root>> {
        self.delta(0.0)?;
        Some(roots(|x| self.rate(x).unwrap(), 0.0, 1.0, 10_000))
    }

    /// The largest stable fixed point.
    pub fn top_stable(&self) -> Option<f64> {
        Some(
            self.fixed_points()?
                .iter()
                .filter(|r| r.stable)
                .map(|r| r.x)
                .fold(f64::NAN, f64::max),
        )
    }
}

/// The α below which Eq 18 has an interior root (h(1) < 0), by the paper's general formula:
/// 1 / (1 + 1 / [(1 + e^{β_AI(T − R)}) tanh(β_H(T − R)/2)]).
pub fn alpha_c_general(beta_h: f64, beta_ai: f64, t_minus_r: f64) -> f64 {
    1.0 / (1.0 + 1.0 / ((1.0 + exp(beta_ai * t_minus_r)) * tanh_half(beta_h * t_minus_r)))
}

/// The paper's simplification for β_H = β_AI = β: 1/(1 + e^{β(T − R)}).
pub fn alpha_c_printed(beta: f64, t_minus_r: f64) -> f64 {
    logistic(-beta * t_minus_r)
}

/// What the general formula reduces to for β_H = β_AI = β, since (1 + eᵃ) tanh(a/2) = eᵃ − 1:
/// 1 − e^{−β(T − R)}.
pub fn alpha_c_corrected(beta: f64, t_minus_r: f64) -> f64 {
    1.0 - exp(-beta * t_minus_r)
}

/// α_c by a root scan of h(1; α) = (1 − α) tanh(β_H Δ/2) + α/(1 + e^{−β_AI Δ}) with Δ = R − T.
pub fn alpha_c_scan(game: Donation, beta_h: f64, beta_ai: f64) -> f64 {
    let d = game.r() - game.t();
    let h = |a: f64| (1.0 - a) * tanh_half(beta_h * d) + a * logistic(beta_ai * d);
    roots(h, 0.0, 1.0, 10_000)[0].x
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid::booker 2>&1 | tail -5`. Expected: 10 passed. If `closed_forms_match_direct_sums` fails only for `Discriminatory` at `ln G` with a difference of order 1e-14, the tolerance is right and the direct sum's rounding is the cause (both should be 0); widen nothing else.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/hybrid
git commit -m "hybrid: Booker's chain, closed forms and replicator equations

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 3: Guo's exact references (`guo.rs`, first part)

**Files:**
- Create: `crates/sugarscape-core/src/hybrid/guo.rs`
- Modify: `crates/sugarscape-core/src/hybrid/mod.rs` (`pub mod guo;`)

**Interfaces:**
- Consumes: Task 1's `exact::{logistic, roots, Root}`.
- Produces: `Plane { t, s }` (R = 1, P = 0) with `payoff(me, other)`, `rho_hat()`, `z(rho, y, phi)`, `gap(rho, y, phi)`, `replicator(rho, y, phi)`, `master(rho, y, phi, k)`, `snowdrift()`, `stag_hunt()`, `phase_closed(y, phi) -> Phase`, `boundary_distance(y, phi)`, `y_min()`; `Phase::{FullC, FullD, Coexistence, Bistable}` (serde snake_case); `phase_numeric(&Plane, y, phi) -> Option<Phase>`; `master_roots(&Plane, y, phi, k) -> Vec<Root>`.

y is the ratio of bots to humans (Table 1), z = (ρ + yφ)/(1 + y) the cooperating share of everyone.

- [ ] **Step 1: Write the failing tests**

Create `guo.rs` with only its doc comment and tests, and add `pub mod guo;` to `mod.rs`:

```rust
//! Guo et al. (iScience 2023; arXiv 2307.01123 and its supplement's source): games with R = 1,
//! P = 0 on the (T, S) plane, bots that cooperate with probability φ, and y bots per human. The
//! replicator dynamics (Eqs 3–5, Theorems 1–3) and the pairwise-comparison master equation
//! (Eqs 6–9) are exact references; Algorithms I and II follow in this file's second part.

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
                let pi_c = (rho + (1.0 - rho) * p.s) / (1.0 + y) + y * (phi + (1.0 - phi) * p.s) / (1.0 + y);
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
        assert!(r.iter().any(|r| r.stable && (r.x - want).abs() < 1e-12), "{r:?} vs {want}");
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
            assert_eq!(phase_numeric(&plane, y, phi), Some(phase), "{plane:?} y {y} φ {phi}");
            assert!(plane.boundary_distance(y, phi) > 1e-3);
        }
    }

    #[test]
    fn the_master_equation_runs_from_the_replicator_root_to_phi() {
        let (y, phi) = (0.5, 0.6);
        let want = SD.rho_hat() + y * (SD.rho_hat() - phi);
        let strong = master_roots(&SD, y, phi, 1e4);
        assert!(strong.iter().any(|r| r.stable && (r.x - want).abs() < 1e-3), "{strong:?}");
        // As K → 0 the comparison is a coin toss and humans drift to the bots' φ.
        let weak = master_roots(&SD, y, phi, 1e-9);
        assert!(weak.iter().any(|r| r.stable && (r.x - phi).abs() < 1e-6), "{weak:?}");
        assert!(SD.master(0.0, y, phi, 10.0) > 0.0 && SD.master(1.0, y, phi, 10.0) < 0.0, "bots keep both ends open");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid::guo 2>&1 | tail -5`. Expected: compile errors (`Plane` undefined).

- [ ] **Step 3: Write the references**

Above the tests module:

```rust
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
        (y * phi / (1.0 + y) - h).abs().min(((1.0 + y * phi) / (1.0 + y) - h).abs())
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid::guo 2>&1 | tail -5`. Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/hybrid
git commit -m "hybrid: Guo's replicator phases and master equation

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 4: Booker's and Guo's population dynamics (`booker.rs`, `guo.rs`, second parts)

**Files:**
- Modify: `crates/sugarscape-core/src/hybrid/booker.rs`, `crates/sugarscape-core/src/hybrid/guo.rs`

**Interfaces:**
- Consumes: Task 1's `engine::{accumulate, fermi, Accumulate, Population}`; Task 2's `Donation`, `BotKind`; Task 3's `Plane`.
- Produces:
  - `booker::Rules { game: Donation, kind: BotKind, beta: f64, accumulate: Accumulate }`, `booker::play(&Population, BotKind, i, j) -> (bool, bool)`, `booker::payoff(&Rules, &Population, i) -> f64`, `booker::init(&mut Population, BotKind, p: f64, &mut SimRng)`, `booker::micro(&Rules, &mut Population, &mut SimRng)`
  - `guo::BotAction::{Redraw, Frozen, PerInteraction}` (serde snake_case), `guo::Rules { plane: Plane, phi: f64, beta: f64, bot_action: BotAction, accumulate: Accumulate }`, `guo::init(&mut Population, phi, p, &mut SimRng)`, `guo::payoff(&Rules, &Population, i, &mut SimRng) -> f64`, `guo::payoff_stored(&Rules, &Population, i) -> f64`, `guo::micro_network(&Rules, &mut Population, &mut SimRng)`, `guo::expected(&Plane, cooperate: bool, c: usize, others: usize) -> f64`, `guo::micro_well_mixed(&Rules, &mut Population, &mut SimRng)`

- [ ] **Step 1: Write the failing tests**

Add to `booker.rs`'s tests module (and `use crate::{graph::Graph, rng::seeded};` at its top):

```rust
    fn rules(kind: BotKind, beta: f64) -> Rules {
        Rules { game: GAME, kind, beta, accumulate: Accumulate::Sum }
    }

    #[test]
    fn discriminatory_bots_mirror_and_are_never_copied() {
        // A path: human 0 — bot 1 — human 2.
        let g = Graph::from_lists(vec![vec![1], vec![0, 2], vec![1]]);
        let mut pop = Population::new(g, vec![false, true, false]);
        pop.set_pure(0, true);
        pop.set_pure(2, false);
        let r = rules(BotKind::Discriminatory, 100.0);
        assert_eq!(play(&pop, BotKind::Discriminatory, 0, 1), (true, true));
        assert_eq!(payoff(&r, &pop, 0), 1.0, "C against a mirror: R = b − c");
        assert_eq!(payoff(&r, &pop, 2), 0.0, "D against a mirror: P");
        let mut rng = seeded(1);
        for _ in 0..1000 {
            micro(&r, &mut pop, &mut rng);
        }
        assert_eq!((pop.action[0], pop.action[2]), (true, false), "their only neighbor is never a role model");
    }

    #[test]
    fn samaritan_bots_are_role_models_and_payoffs_are_sums() {
        // A star: the Samaritan bot 0 at the center of four defecting humans.
        let g = Graph::from_lists(vec![vec![1, 2, 3, 4], vec![0], vec![0], vec![0], vec![0]]);
        let mut pop = Population::new(g, vec![true, false, false, false, false]);
        init(&mut pop, BotKind::Samaritan, 0.0, &mut seeded(2));
        assert_eq!((pop.cooperating, pop.action[0]), (1, true), "p = 0: only the bot cooperates");
        let r = rules(BotKind::Samaritan, 1.0);
        assert_eq!(payoff(&r, &pop, 0), -4.0, "the Samaritan pays c to each of four defectors");
        assert_eq!(payoff(&r, &pop, 1), 2.0, "a defector earns b from the Samaritan");
        assert_eq!(payoff(&Rules { accumulate: Accumulate::Mean, ..r }, &pop, 0), -1.0);
        // Each update: a defector copies the Samaritan with probability 1/(1 + e^{6}).
        let mut rng = seeded(3);
        let trials = 40_000;
        let mut copied = 0;
        for _ in 0..trials {
            let mut q = pop.clone();
            micro(&r, &mut q, &mut rng);
            copied += q.cooperating - pop.cooperating;
        }
        let p = logistic(-6.0);
        let share = copied as f64 / f64::from(trials);
        assert!((share - p).abs() < 4.0 * (p / f64::from(trials)).sqrt(), "{share} vs {p}");
    }

    #[test]
    fn malicious_bots_defect_and_humans_start_at_one_half() {
        let g = Graph::from_lists(vec![vec![1], vec![0]]);
        let mut pop = Population::new(g, vec![true, false]);
        init(&mut pop, BotKind::Malicious, 1.0, &mut seeded(4));
        assert_eq!(pop.action, vec![false, true]);
        let n = 100_000;
        let mut big = Population::new(Graph::from_lists(vec![vec![]; n]), vec![false; n]);
        init(&mut big, BotKind::Samaritan, 0.5, &mut seeded(5));
        assert!((big.human_cooperation() - 0.5).abs() < 0.01);
    }
```

Add to `guo.rs`'s tests module (and `use crate::{graph::Graph, rng::seeded};`):

```rust
    fn rules(phi: f64, bot_action: BotAction) -> Rules {
        Rules { plane: PD, phi, beta: 10.0, bot_action, accumulate: Accumulate::Sum }
    }

    #[test]
    fn a_focal_bot_redraws_under_redraw_and_keeps_its_action_when_frozen() {
        let mut pop = Population::new(Graph::from_lists(vec![vec![1], vec![0]]), vec![true, true]);
        let redraw = rules(0.3, BotAction::Redraw);
        let mut rng = seeded(1);
        let mut c = 0;
        for _ in 0..20_000 {
            micro_network(&redraw, &mut pop, &mut rng);
            c += pop.cooperating;
        }
        let share = c as f64 / 40_000.0;
        assert!((share - 0.3).abs() < 0.02, "{share}");
        let before = pop.action.clone();
        for _ in 0..100 {
            micro_network(&rules(0.3, BotAction::Frozen), &mut pop, &mut rng);
        }
        assert_eq!(pop.action, before);
    }

    #[test]
    fn per_interaction_bots_draw_a_fresh_action_for_every_game() {
        // A human with 1,000 bot neighbors, all storing D.
        let n = 1001;
        let mut lists = vec![(1..n as u32).collect::<Vec<_>>()];
        lists.extend((1..n).map(|_| vec![0]));
        let pop = Population::new(Graph::from_lists(lists), (0..n).map(|i| i > 0).collect());
        let fresh = rules(0.5, BotAction::PerInteraction);
        let pay = payoff(&fresh, &pop, 0, &mut seeded(4));
        assert!((pay / 1000.0 - 0.65).abs() < 0.08, "a defector earns T = 1.3 from about half: {pay}");
        assert_eq!(payoff_stored(&fresh, &pop, 0), 0.0, "stored D everywhere: P");
        assert_eq!(payoff(&rules(0.5, BotAction::Redraw), &pop, 0, &mut seeded(4)), 0.0);
    }

    #[test]
    fn guo_isolated_humans_do_nothing() {
        let g = Graph::from_lists(vec![vec![1], vec![0], vec![]]);
        let mut pop = Population::new(g, vec![false, false, false]);
        pop.set_pure(2, true);
        let mean = Rules { accumulate: Accumulate::Mean, ..rules(0.5, BotAction::Redraw) };
        assert_eq!(payoff_stored(&mean, &pop, 2), 0.0);
        let mut rng = seeded(6);
        for _ in 0..1000 {
            micro_network(&mean, &mut pop, &mut rng);
        }
        assert!(pop.action[2], "an isolated human has no role model");
    }

    #[test]
    fn algorithm_i_uses_expected_payoffs_against_everyone_else() {
        // Ten players, four cooperating: a cooperator meets 3 C and 6 D, a defector 4 C and 5 D.
        assert!((expected(&SD, true, 3, 9) - (3.0 + 6.0 * 0.5) / 9.0).abs() < 1e-15);
        assert!((expected(&SD, false, 4, 9) - 4.0 * 1.2 / 9.0).abs() < 1e-15);
        let mut pop = Population::new(Graph::default(), vec![false; 10]);
        for i in 0..10 {
            pop.set_pure(i, i < 4);
        }
        let coin = Rules { plane: SD, phi: 0.5, beta: 0.0, bot_action: BotAction::Frozen, accumulate: Accumulate::Sum };
        // β = 0: a comparison with a player acting otherwise flips the human half the time.
        let mut rng = seeded(5);
        let trials = 40_000;
        let mut flips = 0;
        for _ in 0..trials {
            let mut q = pop.clone();
            micro_well_mixed(&coin, &mut q, &mut rng);
            flips += usize::from(q.action != pop.action);
        }
        // P(the pair differs) = 2 · (4 · 6)/(10 · 9).
        let want = 0.5 * 2.0 * 24.0 / 90.0;
        assert!((flips as f64 / f64::from(trials) - want).abs() < 0.015, "{flips}");
    }

    #[test]
    fn init_draws_bots_at_phi_and_humans_at_p() {
        let n = 100_000;
        let mut pop = Population::new(Graph::default(), (0..n).map(|i| i % 2 == 0).collect());
        init(&mut pop, 0.9, 0.5, &mut seeded(7));
        assert!((pop.bot_cooperation() - 0.9).abs() < 0.01 && (pop.human_cooperation() - 0.5).abs() < 0.01);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: compile errors (`Rules`, `micro`, `micro_network` undefined).

- [ ] **Step 3: Write Booker's dynamics**

In `booker.rs`, extend the imports at the top to:

```rust
use super::engine::{accumulate, fermi, Accumulate, Population};
use super::exact::{exp, ln_factorial, log_sum_exp, logistic, roots, tanh_half, Root};
use crate::{portable::ln, rng::SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};
```

and add above the tests module:

```rust
/// What the network dynamics read from the config.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub game: Donation,
    pub kind: BotKind,
    pub beta: f64,
    pub accumulate: Accumulate,
}

/// The actions nodes i and j play against each other: a discriminatory bot mirrors its partner
/// (perfect intention recognition); two discriminatory bots cooperate (ours; never needed, since
/// they are never role models).
pub fn play(pop: &Population, kind: BotKind, i: usize, j: usize) -> (bool, bool) {
    let mirror = |k: usize| pop.bot[k] && kind == BotKind::Discriminatory;
    match (mirror(i), mirror(j)) {
        (false, false) => (pop.action[i], pop.action[j]),
        (false, true) => (pop.action[i], pop.action[i]),
        (true, false) => (pop.action[j], pop.action[j]),
        (true, true) => (true, true),
    }
}

/// Node i's payoff: the donation game against every neighbor, summed (Booker's "sum of
/// payoffs") or averaged.
pub fn payoff(rules: &Rules, pop: &Population, i: usize) -> f64 {
    let total: f64 = pop
        .neighbors(i)
        .iter()
        .map(|&j| {
            let (a, b) = play(pop, rules.kind, i, j as usize);
            rules.game.payoff(a, b)
        })
        .sum();
    accumulate(total, pop.degree(i), rules.accumulate)
}

/// The initial state, in index order: each human cooperates with probability `p` (the paper's
/// ½); Samaritan bots cooperate, malicious bots defect, discriminatory bots store C (they mirror
/// when they play).
pub fn init(pop: &mut Population, kind: BotKind, p: f64, rng: &mut SimRng) {
    for i in 0..pop.nodes {
        let c = if pop.bot[i] {
            kind != BotKind::Malicious
        } else {
            rng.gen::<f64>() < p
        };
        pop.set_pure(i, c);
    }
}

/// One update, the paper's step: a uniformly random human picks a uniformly random neighbor as
/// its role model and adopts the model's action with the Fermi probability. A discriminatory bot
/// is never a role model (in Eq 6 it only enlarges the sampling denominator), so drawing one ends
/// the update; so does an isolated human.
pub fn micro(rules: &Rules, pop: &mut Population, rng: &mut SimRng) {
    let Some(i) = pop.random_human(rng) else {
        return;
    };
    let Some(j) = pop.partner(i, rng) else {
        return;
    };
    if pop.bot[j] && rules.kind == BotKind::Discriminatory {
        return;
    }
    let p = fermi(rules.beta, payoff(rules, pop, i), payoff(rules, pop, j));
    if rng.gen::<f64>() < p {
        let a = pop.action[j];
        pop.set_pure(i, a);
    }
}
```

- [ ] **Step 4: Write Guo's dynamics**

In `guo.rs`, extend the imports at the top to:

```rust
use super::engine::{accumulate, fermi, Accumulate, Population};
use super::exact::{logistic, roots, Root};
use crate::rng::SimRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
```

and add above the tests module:

```rust
/// How a bot's action is drawn on a graph (amendment 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotAction {
    /// Algorithm II: a bot chosen as focal plays C with probability φ from then on.
    Redraw,
    /// The initial draw, kept.
    Frozen,
    /// A fresh draw for every game and every imitation.
    PerInteraction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub plane: Plane,
    pub phi: f64,
    pub beta: f64,
    pub bot_action: BotAction,
    pub accumulate: Accumulate,
}

/// In index order: a bot cooperates with probability φ, a human with probability `p` (½ in
/// Algorithm II, the initial frequency p in Algorithm I).
pub fn init(pop: &mut Population, phi: f64, p: f64, rng: &mut SimRng) {
    for i in 0..pop.nodes {
        let c = rng.gen::<f64>() < if pop.bot[i] { phi } else { p };
        pop.set_pure(i, c);
    }
}

fn act(rules: &Rules, pop: &Population, k: usize, rng: &mut SimRng) -> bool {
    if pop.bot[k] && rules.bot_action == BotAction::PerInteraction {
        rng.gen::<f64>() < rules.phi
    } else {
        pop.action[k]
    }
}

/// Node i's payoff on a graph (Algorithm II): the game against every neighbor, summed or
/// averaged; under `per_interaction` every bot's action in every game is a fresh draw.
pub fn payoff(rules: &Rules, pop: &Population, i: usize, rng: &mut SimRng) -> f64 {
    let mut total = 0.0;
    for &j in pop.neighbors(i) {
        let a = act(rules, pop, i, rng);
        let b = act(rules, pop, j as usize, rng);
        total += rules.plane.payoff(a, b);
    }
    accumulate(total, pop.degree(i), rules.accumulate)
}

/// Node i's payoff from the stored actions (inspection): on a graph against its neighbors,
/// well-mixed against everyone else.
pub fn payoff_stored(rules: &Rules, pop: &Population, i: usize) -> f64 {
    if pop.well_mixed() {
        let a = pop.action[i];
        return expected(&rules.plane, a, pop.cooperating - usize::from(a), pop.nodes - 1);
    }
    let total: f64 = pop
        .neighbors(i)
        .iter()
        .map(|&j| rules.plane.payoff(pop.action[i], pop.action[j as usize]))
        .sum();
    accumulate(total, pop.degree(i), rules.accumulate)
}

/// Algorithm II's inner step: a uniformly random node; a bot redraws its action under `redraw`
/// and does nothing otherwise; a human plays its neighbors, picks a uniformly random neighbor
/// and adopts its action with the Fermi probability (copying a `per_interaction` bot adopts a
/// fresh draw). An isolated human does nothing.
pub fn micro_network(rules: &Rules, pop: &mut Population, rng: &mut SimRng) {
    let i = pop.random_node(rng);
    if pop.bot[i] {
        if rules.bot_action == BotAction::Redraw {
            let c = rng.gen::<f64>() < rules.phi;
            pop.set_pure(i, c);
        }
        return;
    }
    let Some(j) = pop.partner(i, rng) else {
        return;
    };
    let p = fermi(rules.beta, payoff(rules, pop, i, rng), payoff(rules, pop, j, rng));
    if rng.gen::<f64>() < p {
        let a = act(rules, pop, j, rng);
        pop.set_pure(i, a);
    }
}

/// The expected payoff of an action against the other `others` players, `c` of them
/// cooperating.
pub fn expected(plane: &Plane, cooperate: bool, c: usize, others: usize) -> f64 {
    let (c, o) = (c as f64, others as f64);
    (c * plane.payoff(cooperate, true) + (o - c) * plane.payoff(cooperate, false)) / o
}

/// Algorithm I's step: a uniformly random human and a uniformly random other player; if their
/// actions differ the human adopts the other's with the Fermi probability, both payoffs expected
/// against everyone else. Bots keep the action drawn at the start (amendment 5).
pub fn micro_well_mixed(rules: &Rules, pop: &mut Population, rng: &mut SimRng) {
    let Some(i) = pop.random_human(rng) else {
        return;
    };
    let j = pop.partner(i, rng).expect("a well-mixed population has two players");
    let (a, b) = (pop.action[i], pop.action[j]);
    if a == b {
        return;
    }
    let others = pop.nodes - 1;
    let pi = expected(&rules.plane, a, pop.cooperating - usize::from(a), others);
    let pj = expected(&rules.plane, b, pop.cooperating - usize::from(b), others);
    if rng.gen::<f64>() < fermi(rules.beta, pi, pj) {
        pop.set_pure(i, b);
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: 33 passed (11 + 13 Booker + 9 Guo).

- [ ] **Step 6: Commit**

```bash
git add crates/sugarscape-core/src/hybrid
git commit -m "hybrid: Booker's and Guo's population dynamics

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 5: Si et al.'s three frameworks as the code runs them (`si.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/hybrid/si.rs`
- Modify: `crates/sugarscape-core/src/hybrid/mod.rs` (`pub mod si;`)

**Interfaces:**
- Consumes: Task 1's `engine::{accumulate, fermi, Accumulate, Population}`.
- Produces: `Framework::{Discrete, Continuous, Mixed}`, `MixedDraw::{PerInteraction, PerRound}`, `Division::{Integer, Float}` (serde snake_case); `Rules { r, framework, mixed_draw, division, beta, accumulate }`; `pay(r, me: bool, other: bool) -> f64`; `init(&mut Population, Framework, theta, &mut SimRng)`; `payoff_graph(&Rules, &mut Population, i, &mut SimRng) -> f64`; `payoff_stored(&Rules, &Population, i) -> f64`; `payoff_well_mixed(&Rules, &Population, i) -> f64`; `round(&Rules, &mut Population, &mut SimRng)`.

The code's references: lattice `cal_payoff`, `learn_strategy`, `round_game` (Figs. 3A–C); well-mixed `cal_payoff`, `learn_strategy`, `main_process` (Figs. 1A–C). The code's Fermi probability `1/(1 + exp((π_x − π_n)/κ))` with κ = 0.1 on the lattice and `1/(1 + exp((π_x − π_n)·κ))` with κ = 10 well-mixed are both β = 10 here.

- [ ] **Step 1: Write the failing tests**

Create `si.rs` with only its doc comment and tests, and add `pub mod si;` to `mod.rs`:

```rust
//! Si, He, Shen and Tanimoto (J. R. Soc. Interface 2025; code Zenodo 10.5281/zenodo.13957161):
//! the Prisoner's Dilemma R = 1, S = −r, T = 1 + r, P = 0 with bots of strategy θ in three
//! frameworks, on the Moore lattice and well-mixed, as the published code runs it. Defaults are
//! the code's: well-mixed discrete and mixed payoffs drop the S and P terms (`division: integer`;
//! `float` is the stated game), and mixed actions are drawn for every game on the lattice and
//! whenever a node is picked well-mixed (`mixed_draw: per_round` draws once per round, Fig. A1).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::Graph, hybrid::graph::{lattice, Neighborhood}, rng::seeded};

    const R: f64 = 0.1;

    fn rules(framework: Framework) -> Rules {
        Rules { r: R, framework, mixed_draw: MixedDraw::PerInteraction, division: Division::Integer, beta: 10.0, accumulate: Accumulate::Sum }
    }

    #[test]
    fn eq_2_is_the_expectation_of_eq_1() {
        for (si, sj) in [(0.4, 0.9), (0.0, 1.0), (0.25, 0.25)] {
            let mut e = 0.0;
            for (a, pa) in [(true, si), (false, 1.0 - si)] {
                for (b, pb) in [(true, sj), (false, 1.0 - sj)] {
                    e += pa * pb * pay(R, a, b);
                }
            }
            assert!((e - (-R * si + (1.0 + R) * sj)).abs() < 1e-15, "{si} {sj}");
        }
    }

    #[test]
    fn figure_a1_payoffs() {
        // Fig. A1: a defecting bot with three cooperating neighbors earns 3(1 + r); a defecting
        // human with two cooperating bot neighbors 2(1 + r), with four 4(1 + r).
        // Nodes: 0 the left bot (D); 1 the right human (D); 2–4 cooperators around the bot;
        // 5–8 cooperating bots around the human; 9 a defector beside the left bot.
        let lists = vec![vec![2, 3, 4, 9], vec![5, 6, 7, 8], vec![0], vec![0], vec![0], vec![1], vec![1], vec![1], vec![1], vec![0]];
        let bot = vec![true, false, false, false, false, true, true, true, true, false];
        let mut pop = Population::new(Graph::from_lists(lists), bot);
        for (i, c) in [false, false, true, true, true, true, true, false, false, false].into_iter().enumerate() {
            pop.set_pure(i, c);
        }
        let per_round = Rules { mixed_draw: MixedDraw::PerRound, ..rules(Framework::Mixed) };
        assert!((payoff_stored(&per_round, &pop, 0) - 3.0 * (1.0 + R)).abs() < 1e-15);
        assert!((payoff_stored(&per_round, &pop, 1) - 2.0 * (1.0 + R)).abs() < 1e-15, "panel a");
        pop.set_pure(7, true);
        pop.set_pure(8, true);
        assert!((payoff_stored(&per_round, &pop, 1) - 4.0 * (1.0 + R)).abs() < 1e-15, "panel b");
    }

    #[test]
    fn a_theta_one_mixed_bot_plays_as_a_cooperating_discrete_bot() {
        let g = Graph::from_lists(lattice(5, Neighborhood::Moore, true));
        let bot: Vec<bool> = (0..25).map(|i| i % 3 == 0).collect();
        let mut mixed = Population::new(g.clone(), bot.clone());
        let mut discrete = Population::new(g, bot);
        for i in 0..25 {
            let c = mixed.bot[i] || i % 2 == 0;
            mixed.set_pure(i, c);
            discrete.set_pure(i, c);
        }
        let mut rng = seeded(1);
        for i in 0..25 {
            let m = payoff_graph(&rules(Framework::Mixed), &mut mixed, i, &mut rng);
            let d = payoff_graph(&rules(Framework::Discrete), &mut discrete, i, &mut rng);
            assert_eq!(m, d, "node {i}: strategies of 0 and 1 draw deterministically");
        }
    }

    #[test]
    fn the_codes_well_mixed_payoff_drops_s_and_p() {
        // N = 11: ten others, four of them cooperating.
        let mut pop = Population::new(Graph::default(), vec![false; 11]);
        for i in 0..5 {
            pop.set_pure(i, true);
        }
        let int = rules(Framework::Discrete);
        let float = Rules { division: Division::Float, ..int };
        assert!((payoff_well_mixed(&int, &pop, 0) - 0.4).abs() < 1e-15, "C: (4/10)·1 + 0·(−r)");
        assert!((payoff_well_mixed(&float, &pop, 0) - (0.4 - 0.6 * R)).abs() < 1e-15, "C: the stated game");
        assert!((payoff_well_mixed(&int, &pop, 10) - 0.5 * (1.0 + R)).abs() < 1e-15, "D: (5/10)(1 + r)");
        // Nobody else cooperating: the integer share is 10/10 = 1.
        let mut lone = Population::new(Graph::default(), vec![false; 11]);
        lone.set_pure(0, true);
        assert!((payoff_well_mixed(&int, &lone, 0) + R).abs() < 1e-15, "C among defectors: S = −r");
        assert_eq!(payoff_well_mixed(&int, &lone, 1), 0.1 * (1.0 + R), "D: (1/10)(1 + r)");
        let mut cont = Population::new(Graph::default(), vec![false; 5]);
        for (i, s) in [0.2, 0.4, 0.6, 0.8, 1.0].into_iter().enumerate() {
            cont.set_strategy(i, s);
        }
        let want = -R * 0.2 + (1.0 + R) * (0.4 + 0.6 + 0.8 + 1.0) / 4.0;
        assert!((payoff_well_mixed(&rules(Framework::Continuous), &cont, 0) - want).abs() < 1e-15);
    }

    #[test]
    fn lattice_bots_never_change_and_well_mixed_mixed_bots_keep_redrawing() {
        let g = Graph::from_lists(lattice(10, Neighborhood::Moore, true));
        let bot: Vec<bool> = (0..100).map(|i| i % 4 == 0).collect();
        let mut pop = Population::new(g, bot.clone());
        let mut rng = seeded(2);
        init(&mut pop, Framework::Continuous, 0.9, &mut rng);
        let before: Vec<f64> = (0..100).filter(|&i| bot[i]).map(|i| pop.strategy[i]).collect();
        for _ in 0..20 {
            round(&rules(Framework::Continuous), &mut pop, &mut rng);
        }
        let after: Vec<f64> = (0..100).filter(|&i| bot[i]).map(|i| pop.strategy[i]).collect();
        assert_eq!(before, after);
        assert!(before.iter().all(|&s| s == 0.9));
        let n = 2000;
        let mut wm = Population::new(Graph::default(), (0..n).map(|i| i % 2 == 0).collect());
        init(&mut wm, Framework::Mixed, 0.9, &mut rng);
        let mut share = 0.0;
        for _ in 0..50 {
            round(&rules(Framework::Mixed), &mut wm, &mut rng);
            share += wm.bot_cooperation();
        }
        assert!((share / 50.0 - 0.9).abs() < 0.02, "{share}");
    }

    #[test]
    fn init_follows_the_code() {
        let n = 100_000;
        let bot: Vec<bool> = (0..n).map(|i| i % 10 == 0).collect();
        let mut d = Population::new(Graph::default(), bot.clone());
        init(&mut d, Framework::Discrete, 0.9, &mut seeded(3));
        assert!((d.bot_cooperation() - 0.9).abs() < 0.03 && (d.human_cooperation() - 0.5).abs() < 0.01);
        assert!(d.strategy.iter().all(|&s| s == 0.0 || s == 1.0));
        let mut c = Population::new(Graph::default(), bot);
        init(&mut c, Framework::Continuous, 0.9, &mut seeded(3));
        assert!((c.human_cooperation() - 0.5).abs() < 0.01, "uniform strategies");
        assert!((0..n).filter(|&i| c.bot[i]).all(|i| c.strategy[i] == 0.9));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid::si 2>&1 | tail -5`. Expected: compile errors (`Rules`, `pay` undefined).

- [ ] **Step 3: Write `si.rs`**

Above the tests module:

```rust
use super::engine::{accumulate, fermi, Accumulate, Population};
use crate::rng::SimRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    Discrete,
    Continuous,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MixedDraw {
    /// The code: on a graph both players draw for every game; well-mixed, a picked node redraws.
    PerInteraction,
    /// Fig. A1: every node draws once at the start of each round.
    PerRound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Division {
    /// The code: `(nn - c) / nn` in integers.
    Integer,
    /// The stated game (Eq. 1).
    Float,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub r: f64,
    pub framework: Framework,
    pub mixed_draw: MixedDraw,
    pub division: Division,
    pub beta: f64,
    pub accumulate: Accumulate,
}

/// Eq. 1: R = 1, S = −r, T = 1 + r, P = 0 (true = cooperate).
pub fn pay(r: f64, me: bool, other: bool) -> f64 {
    match (me, other) {
        (true, true) => 1.0,
        (true, false) => -r,
        (false, true) => 1.0 + r,
        (false, false) => 0.0,
    }
}

/// The code's initial state, in index order. Discrete: a bot cooperates with probability θ, drawn
/// once; a human cooperates if `randi(2)` is 0. Continuous and mixed: a bot's strategy is θ, a
/// human's uniform on [0, 1); each node's action is drawn from its strategy.
pub fn init(pop: &mut Population, framework: Framework, theta: f64, rng: &mut SimRng) {
    for i in 0..pop.nodes {
        if framework == Framework::Discrete {
            let c = if pop.bot[i] {
                rng.gen::<f64>() < theta
            } else {
                rng.gen_range(0..2u32) == 0
            };
            pop.set_pure(i, c);
        } else {
            let s = if pop.bot[i] { theta } else { rng.gen::<f64>() };
            pop.set_strategy(i, s);
            let c = rng.gen::<f64>() < s;
            pop.set_action(i, c);
        }
    }
}

fn draw(pop: &Population, k: usize, rng: &mut SimRng) -> bool {
    rng.gen::<f64>() < pop.strategy[k]
}

/// Node i's payoff on a graph, as the lattice code's `cal_payoff`: summed over its neighbors
/// (averaged under `accumulate: mean`). Continuous: −r·sᵢ + (1 + r)·sⱼ per neighbor. Mixed per
/// interaction: the focal and the neighbor each draw an action for every game, and the drawn
/// actions are stored as the code stores them; otherwise the stored actions.
pub fn payoff_graph(rules: &Rules, pop: &mut Population, i: usize, rng: &mut SimRng) -> f64 {
    let fresh = rules.framework == Framework::Mixed && rules.mixed_draw == MixedDraw::PerInteraction;
    let degree = pop.neighbors(i).len();
    let mut total = 0.0;
    for k in 0..degree {
        let j = pop.neighbors(i)[k] as usize;
        total += match rules.framework {
            Framework::Continuous => -rules.r * pop.strategy[i] + (1.0 + rules.r) * pop.strategy[j],
            _ if fresh => {
                let a = draw(pop, i, rng);
                pop.set_action(i, a);
                let b = draw(pop, j, rng);
                pop.set_action(j, b);
                pay(rules.r, a, b)
            }
            _ => pay(rules.r, pop.action[i], pop.action[j]),
        };
    }
    accumulate(total, degree, rules.accumulate)
}

/// Node i's payoff from the stored actions and strategies (inspection, and Fig. A1): on a graph
/// as `payoff_graph` without fresh draws, well-mixed as `payoff_well_mixed`.
pub fn payoff_stored(rules: &Rules, pop: &Population, i: usize) -> f64 {
    if pop.well_mixed() {
        return payoff_well_mixed(rules, pop, i);
    }
    let total: f64 = pop
        .neighbors(i)
        .iter()
        .map(|&j| match rules.framework {
            Framework::Continuous => -rules.r * pop.strategy[i] + (1.0 + rules.r) * pop.strategy[j as usize],
            _ => pay(rules.r, pop.action[i], pop.action[j as usize]),
        })
        .sum();
    accumulate(total, pop.degree(i), rules.accumulate)
}

/// Node i's well-mixed payoff, from counts as the code computes it. Discrete and mixed: against
/// the N − 1 others, c of them cooperating (the focal excluded), (c/(N − 1))·M[a][C] plus
/// ((N − 1 − c)/(N − 1))·M[a][D], the second share in integers under `division: integer` (so 0
/// unless c = 0) and in floating point under `float`. Continuous: −r·sᵢ + (1 + r)·(the others'
/// mean strategy).
pub fn payoff_well_mixed(rules: &Rules, pop: &Population, i: usize) -> f64 {
    let nn = pop.nodes - 1;
    if rules.framework == Framework::Continuous {
        return -rules.r * pop.strategy[i]
            + (1.0 + rules.r) * (pop.strategy_sum - pop.strategy[i]) / nn as f64;
    }
    let a = pop.action[i];
    let c = pop.cooperating - usize::from(a);
    let defect_share = match rules.division {
        Division::Integer => ((nn - c) / nn) as f64,
        Division::Float => (nn - c) as f64 / nn as f64,
    };
    c as f64 / nn as f64 * pay(rules.r, a, true) + defect_share * pay(rules.r, a, false)
}

/// The focal adopts the model's strategy (discrete: its action).
fn adopt(rules: &Rules, pop: &mut Population, i: usize, j: usize) {
    if rules.framework == Framework::Discrete {
        let a = pop.action[j];
        pop.set_pure(i, a);
    } else {
        let s = pop.strategy[j];
        pop.set_strategy(i, s);
    }
}

/// One round: under `per_round` (mixed) every node first draws its action; then N focal picks
/// with replacement. On a graph (the lattice code's `round_game`) only ordinary players act: a
/// uniformly random neighbor is the role model, both payoffs are computed afresh, and the focal
/// adopts with the Fermi probability. Well-mixed discrete and continuous (`main_process`): only
/// ordinary players act, against a uniformly random other. Well-mixed mixed (Fig. 1C's
/// `learn_strategy`): every picked node draws a role model; an ordinary player may adopt; then
/// the picked node, bot or not, draws its action from its strategy (not under `per_round`).
pub fn round(rules: &Rules, pop: &mut Population, rng: &mut SimRng) {
    let mixed = rules.framework == Framework::Mixed;
    if mixed && rules.mixed_draw == MixedDraw::PerRound {
        for k in 0..pop.nodes {
            let a = draw(pop, k, rng);
            pop.set_action(k, a);
        }
    }
    for _ in 0..pop.nodes {
        let c = pop.random_node(rng);
        if pop.well_mixed() {
            if !mixed && pop.bot[c] {
                continue;
            }
            let j = pop.partner(c, rng).expect("two players");
            if !pop.bot[c] {
                let p = fermi(rules.beta, payoff_well_mixed(rules, pop, c), payoff_well_mixed(rules, pop, j));
                if rng.gen::<f64>() < p {
                    adopt(rules, pop, c, j);
                }
            }
            if mixed && rules.mixed_draw == MixedDraw::PerInteraction {
                let a = draw(pop, c, rng);
                pop.set_action(c, a);
            }
        } else if !pop.bot[c] {
            let Some(j) = pop.partner(c, rng) else {
                continue;
            };
            let pc = payoff_graph(rules, pop, c, rng);
            let pj = payoff_graph(rules, pop, j, rng);
            if rng.gen::<f64>() < fermi(rules.beta, pc, pj) {
                adopt(rules, pop, c, j);
            }
        }
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid::si 2>&1 | tail -5`. Expected: 6 passed. Then `cargo clippy -p sugarscape-core --all-targets -- -D warnings 2>&1 | tail -5`; if it flags `needless_range_loop` on `payoff_graph`'s index loop, keep the loop (its body mutates `pop`, so it cannot hold an iterator over `pop.neighbors(i)`) and put `#[allow(clippy::needless_range_loop)]` on the function with that reason as a comment.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/hybrid
git commit -m "hybrid: Si et al.'s three frameworks as the published code runs them

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 6: Shirado and Christakis's coloring game (`coloring.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/hybrid/coloring.rs`
- Modify: `crates/sugarscape-core/src/hybrid/mod.rs` (`pub mod coloring;`)

**Interfaces:**
- Consumes: `crate::graph::barabasi_albert`, `crate::portable::ln`.
- Produces: `COLORS: usize = 3`; `colorings(&[Vec<u32>], k: u8) -> u64`; `counts(&[Vec<u32>], &[u8], v) -> [usize; 3]`; `conflicts_of(&[Vec<u32>], &[u8]) -> usize`; `greedy([usize; 3], own: u8, &mut SimRng) -> u8`; `errant([usize; 3], own: u8, &mut SimRng) -> u8`; `wait(mean, &mut SimRng) -> f64`; `Rules { noise, errant, reaction, bot_interval, time_limit }`; `Session { adj, bot, color, next, time, conflicts, solved_at: Option<f64>, decisions: u64, rules }` with `new(Vec<Vec<u32>>, Vec<bool>, Rules, &mut SimRng)`, `finished()`, `advance_to(t, &mut SimRng)`, `play(&mut SimRng) -> Option<f64>`; `convergence_steps(&[Vec<u32>], start: &[u8], cap: u64, &mut SimRng) -> Option<u64>`.

The game: three colors; each player sees its own and its neighbors' colors; the session is solved when no edge joins two equal colors, and fails at the time limit (300 s). The bot (the paper's) decides every 1.5 s: with probability `noise` a uniform color among all three, otherwise the least-common rule. The human (our reconstruction, amendment 12): decisions after exponential waits of mean `reaction` (1.59 s), an errant move with probability `errant`, otherwise the least-common rule.

- [ ] **Step 1: Write the failing tests**

Create `coloring.rs` with only its doc comment and tests, and add `pub mod coloring;` to `mod.rs`:

```rust
//! Shirado and Christakis (Nature 545:370, 2017): a network 3-coloring game with noisy bots. The
//! game, the bot, the Barabási–Albert graph, the solution count and Extended Data Fig. 2's
//! statistic are the paper's; the human players are our labeled reconstruction (greedy
//! responders with exponential decision times and an errant-move rate).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::barabasi_albert, rng::seeded};

    fn path(n: usize) -> Vec<Vec<u32>> {
        (0..n)
            .map(|i| [i.checked_sub(1), (i + 1 < n).then_some(i + 1)].into_iter().flatten().map(|j| j as u32).collect())
            .collect()
    }
    fn cycle(n: usize) -> Vec<Vec<u32>> {
        (0..n).map(|i| vec![((i + n - 1) % n) as u32, ((i + 1) % n) as u32]).collect()
    }
    fn complete(n: usize) -> Vec<Vec<u32>> {
        (0..n).map(|i| (0..n as u32).filter(|&j| j as usize != i).collect()).collect()
    }
    fn wheel(n: usize) -> Vec<Vec<u32>> {
        // A hub (node n) joined to every node of an n-cycle.
        let mut g = cycle(n);
        for ns in g.iter_mut() {
            ns.push(n as u32);
        }
        g.push((0..n as u32).collect());
        g
    }
    const RULES: Rules = Rules { noise: 0.1, errant: 0.041, reaction: 1.59, bot_interval: 1.5, time_limit: 300.0 };

    #[test]
    fn colorings_match_the_deletion_contraction_closed_forms() {
        for n in 1..=8u32 {
            assert_eq!(colorings(&path(n as usize), 3), 3 * 2u64.pow(n - 1), "path {n}");
        }
        for n in 3..=8u32 {
            let want = 2u64.pow(n) as i64 + if n % 2 == 0 { 2 } else { -2 };
            assert_eq!(colorings(&cycle(n as usize), 3) as i64, want, "cycle {n}");
            assert_eq!(colorings(&wheel(n as usize), 3), if n % 2 == 0 { 6 } else { 0 }, "wheel {n}");
        }
        assert_eq!((colorings(&complete(3), 3), colorings(&complete(4), 3)), (6, 0));
    }

    #[test]
    fn barabasi_albert_graphs_from_a_triangle_are_always_three_colorable() {
        let mut rng = seeded(1);
        for _ in 0..50 {
            let n = colorings(&barabasi_albert(20, 2, &mut rng), 3);
            assert!((6..=6 * 2u64.pow(17)).contains(&n), "{n}");
        }
    }

    #[test]
    fn the_least_common_rule_keeps_a_least_common_color_and_breaks_ties_at_random() {
        let mut rng = seeded(2);
        assert_eq!(greedy([0, 1, 1], 0, &mut rng), 0);
        assert_eq!(greedy([2, 1, 1], 1, &mut rng), 1, "tied for least common: keep");
        assert_eq!(greedy([2, 0, 1], 0, &mut rng), 1);
        let mut seen = [0; 3];
        for _ in 0..10_000 {
            seen[greedy([1, 0, 0], 0, &mut rng) as usize] += 1;
        }
        assert_eq!(seen[0], 0);
        assert!((4700..5300).contains(&seen[1]), "{seen:?}");
    }

    #[test]
    fn an_errant_move_raises_conflicts_when_it_can() {
        let mut rng = seeded(3);
        for _ in 0..100 {
            assert!([1, 2].contains(&errant([0, 1, 2], 0, &mut rng)));
            assert_eq!(errant([0, 1, 2], 2, &mut rng), 2, "nothing is worse than the worst");
        }
        assert_eq!(errant([1, 1, 1], 0, &mut rng), 0);
    }

    #[test]
    fn waits_have_the_given_mean() {
        let mut rng = seeded(4);
        let mean = (0..100_000).map(|_| wait(1.59, &mut rng)).sum::<f64>() / 1e5;
        assert!((mean - 1.59).abs() < 0.02, "{mean}");
    }

    #[test]
    fn sessions_count_conflicts_and_bots_decide_on_their_clock() {
        let mut rng = seeded(5);
        let adj = barabasi_albert(20, 2, &mut rng);
        let mut bot = vec![false; 20];
        bot[0] = true;
        let mut s = Session::new(adj.clone(), bot, RULES, &mut rng);
        assert_eq!(s.conflicts, conflicts_of(&s.adj, &s.color));
        assert!(s.next[0] < 1.5, "a bot's first decision falls in its first interval");
        let first = s.next[0];
        while s.time < first && !s.finished() {
            let t = s.time + 0.01;
            s.advance_to(t, &mut rng);
        }
        if !s.finished() {
            assert!((s.next[0] - (first + 1.5)).abs() < 1e-12, "then every 1.5 s");
        }
        assert_eq!(s.conflicts, conflicts_of(&s.adj, &s.color));
    }

    #[test]
    fn a_finished_session_stays_finished() {
        let mut rng = seeded(6);
        // No edges: properly colored at the start.
        let mut empty = Session::new(vec![vec![]; 3], vec![false; 3], RULES, &mut rng);
        assert_eq!(empty.solved_at, Some(0.0));
        empty.advance_to(100.0, &mut rng);
        assert_eq!((empty.time, empty.decisions), (0.0, 0));
        // K4 cannot be 3-colored: the session runs to the time limit and holds there.
        let mut k4 = Session::new(complete(4), vec![false; 4], Rules { time_limit: 30.0, ..RULES }, &mut rng);
        assert_eq!(k4.play(&mut rng), None);
        assert!(k4.finished() && k4.time == 30.0);
        let (decisions, color) = (k4.decisions, k4.color.clone());
        k4.advance_to(1e9, &mut rng);
        assert_eq!((k4.decisions, k4.color.clone(), k4.time), (decisions, color, 30.0));
        // A solvable session stops at its solve time.
        let mut p = Session::new(path(6), vec![false; 6], RULES, &mut rng);
        let at = p.play(&mut rng).expect("a path solves within 300 s");
        assert!(p.conflicts == 0 && p.time == at && p.finished());
        p.advance_to(1e9, &mut rng);
        assert_eq!(p.time, at);
    }

    #[test]
    fn the_extended_data_fig_2_recolorer_converges_or_is_censored() {
        let mut rng = seeded(7);
        let steps = convergence_steps(&path(5), &[0, 0, 0, 0, 0], 1_000_000, &mut rng);
        assert!(steps.is_some_and(|s| s > 0));
        assert_eq!(convergence_steps(&path(3), &[0, 1, 0], 10, &mut rng), Some(0), "already proper");
        assert_eq!(convergence_steps(&complete(4), &[0, 1, 2, 0], 1000, &mut rng), None);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid::coloring 2>&1 | tail -5`. Expected: compile errors (`colorings`, `Session` undefined).

- [ ] **Step 3: Write `coloring.rs`**

Above the tests module:

```rust
use crate::{portable::ln, rng::SimRng};
use rand::Rng;

pub const COLORS: usize = 3;

/// The number of proper colorings with `k` colors, by backtracking in index order. Exact on any
/// graph; fast on a Barabási–Albert graph grown with two links per node, where every node after
/// the triangle has exactly two earlier neighbors (so at most two branches each).
pub fn colorings(adj: &[Vec<u32>], k: u8) -> u64 {
    fn go(adj: &[Vec<u32>], k: u8, v: usize, color: &mut [u8]) -> u64 {
        if v == adj.len() {
            return 1;
        }
        let mut total = 0;
        for c in 0..k {
            if adj[v].iter().all(|&u| u as usize >= v || color[u as usize] != c) {
                color[v] = c;
                total += go(adj, k, v + 1, color);
            }
        }
        total
    }
    go(adj, k, 0, &mut vec![0; adj.len()])
}

/// How many of node v's neighbors hold each color.
pub fn counts(adj: &[Vec<u32>], color: &[u8], v: usize) -> [usize; COLORS] {
    let mut c = [0; COLORS];
    for &u in &adj[v] {
        c[color[u as usize] as usize] += 1;
    }
    c
}

/// Edges joining two equal colors.
pub fn conflicts_of(adj: &[Vec<u32>], color: &[u8]) -> usize {
    adj.iter()
        .enumerate()
        .map(|(v, ns)| ns.iter().filter(|&&u| (u as usize) > v && color[u as usize] == color[v]).count())
        .sum()
}

/// The least-common rule: if the node's color is not among the least common around it, a
/// uniformly random least-common color (ties at random, ours); otherwise its own.
pub fn greedy(counts: [usize; COLORS], own: u8, rng: &mut SimRng) -> u8 {
    let min = *counts.iter().min().unwrap();
    if counts[own as usize] == min {
        return own;
    }
    let least: Vec<u8> = (0..COLORS as u8).filter(|&c| counts[c as usize] == min).collect();
    least[rng.gen_range(0..least.len() as u32) as usize]
}

/// An errant move (our reconstruction): a uniformly random color that would raise the node's
/// conflicts; its own color if none would.
pub fn errant(counts: [usize; COLORS], own: u8, rng: &mut SimRng) -> u8 {
    let worse: Vec<u8> = (0..COLORS as u8).filter(|&c| counts[c as usize] > counts[own as usize]).collect();
    if worse.is_empty() {
        own
    } else {
        worse[rng.gen_range(0..worse.len() as u32) as usize]
    }
}

/// An exponential wait of the given mean, −mean·ln(1 − u) with the portable logarithm.
pub fn wait(mean: f64, rng: &mut SimRng) -> f64 {
    -mean * ln(1.0 - rng.gen::<f64>())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub noise: f64,
    pub errant: f64,
    pub reaction: f64,
    pub bot_interval: f64,
    pub time_limit: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub adj: Vec<Vec<u32>>,
    pub bot: Vec<bool>,
    pub color: Vec<u8>,
    /// Each node's next decision time (s).
    pub next: Vec<f64>,
    pub time: f64,
    pub conflicts: usize,
    pub solved_at: Option<f64>,
    pub decisions: u64,
    pub rules: Rules,
}

impl Session {
    /// Colors uniform at random (ours, amendment 12); each human's first decision after an
    /// exponential wait, each bot's at a uniform offset in [0, bot_interval).
    pub fn new(adj: Vec<Vec<u32>>, bot: Vec<bool>, rules: Rules, rng: &mut SimRng) -> Self {
        let n = adj.len();
        let color: Vec<u8> = (0..n).map(|_| rng.gen_range(0..COLORS as u32) as u8).collect();
        let next = (0..n)
            .map(|i| {
                if bot[i] {
                    rng.gen::<f64>() * rules.bot_interval
                } else {
                    wait(rules.reaction, rng)
                }
            })
            .collect();
        let conflicts = conflicts_of(&adj, &color);
        Session {
            adj,
            bot,
            color,
            next,
            time: 0.0,
            conflicts,
            solved_at: (conflicts == 0).then_some(0.0),
            decisions: 0,
            rules,
        }
    }

    pub fn finished(&self) -> bool {
        self.solved_at.is_some() || self.time >= self.rules.time_limit
    }

    /// Runs the decisions due by `t` (at most the time limit) in time order, ties to the lower
    /// index, stopping at the first proper coloring.
    pub fn advance_to(&mut self, t: f64, rng: &mut SimRng) {
        if self.finished() {
            return;
        }
        let until = t.min(self.rules.time_limit);
        loop {
            let (v, at) = self
                .next
                .iter()
                .copied()
                .enumerate()
                .fold((0, f64::INFINITY), |(bv, bt), (i, x)| if x < bt { (i, x) } else { (bv, bt) });
            if at > until {
                break;
            }
            self.time = at;
            self.decide(v, rng);
            if self.conflicts == 0 {
                self.solved_at = Some(at);
                return;
            }
        }
        self.time = self.time.max(until);
    }

    fn decide(&mut self, v: usize, rng: &mut SimRng) {
        let c = counts(&self.adj, &self.color, v);
        let own = self.color[v];
        let new = if self.bot[v] {
            if rng.gen::<f64>() < self.rules.noise {
                rng.gen_range(0..COLORS as u32) as u8
            } else {
                greedy(c, own, rng)
            }
        } else if rng.gen::<f64>() < self.rules.errant {
            errant(c, own, rng)
        } else {
            greedy(c, own, rng)
        };
        self.conflicts = self.conflicts + c[new as usize] - c[own as usize];
        self.color[v] = new;
        self.decisions += 1;
        self.next[v] += if self.bot[v] {
            self.rules.bot_interval
        } else {
            wait(self.rules.reaction, rng)
        };
    }

    /// Plays to a proper coloring or the time limit; the solve time, if any.
    pub fn play(&mut self, rng: &mut SimRng) -> Option<f64> {
        self.advance_to(f64::INFINITY, rng);
        self.solved_at
    }
}

/// Extended Data Fig. 2's statistic: from `start`, a uniformly random node recolors to a
/// uniformly random color other than a uniformly random neighbor's, until the coloring is
/// proper; the number of recolorings, or None past `cap` (amendment 12).
pub fn convergence_steps(adj: &[Vec<u32>], start: &[u8], cap: u64, rng: &mut SimRng) -> Option<u64> {
    let mut color = start.to_vec();
    let mut conflicts = conflicts_of(adj, &color);
    let mut steps = 0;
    while conflicts > 0 {
        if steps == cap {
            return None;
        }
        steps += 1;
        let v = rng.gen_range(0..adj.len() as u32) as usize;
        if adj[v].is_empty() {
            continue;
        }
        let u = adj[v][rng.gen_range(0..adj[v].len() as u32) as usize] as usize;
        let new = (color[u] + 1 + rng.gen_range(0..2u32) as u8) % COLORS as u8;
        let c = counts(adj, &color, v);
        conflicts = conflicts + c[new as usize] - c[color[v] as usize];
        color[v] = new;
    }
    Some(steps)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid::coloring 2>&1 | tail -5`. Expected: 8 passed. If `sessions_count_conflicts_and_bots_decide_on_their_clock` finds the session solved before the bot's first decision (possible for some seeds), the test's `if !s.finished()` guard skips the clock check; change the seed only if the guard skips it, and record the seed change in the ledger.

- [ ] **Step 5: Commit**

```bash
git add crates/sugarscape-core/src/hybrid
git commit -m "hybrid: the coloring game, its bot, the reconstructed human and the solution count

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 7: Configuration, series, the runner, the world and `model.rs` (`config.rs`, `stats.rs`, `world.rs`)

**Files:**
- Create: `crates/sugarscape-core/src/hybrid/{config.rs,stats.rs,world.rs,view.rs}` (`view.rs` a stub until Task 8)
- Modify: `crates/sugarscape-core/src/hybrid/mod.rs`, `crates/sugarscape-core/src/model.rs` (17 sites)

**Interfaces:**
- Consumes: Tasks 1–6.
- Produces:
  - `config::{World::{Booker, Guo, Si, Coloring}, Structure::{WellMixed, Lattice, ErdosRenyi, BarabasiAlbert}, Bots::{Count, Fraction, Bernoulli}, StepUnit::{Paper, Sweep}}` (serde snake_case), re-exporting `BotKind`, `Accumulate`, `Neighborhood`, `Placement`, `BotAction`, `Framework`, `MixedDraw`, `Division`; `HybridConfig` (fields below) with `size() -> usize`, `bot_total() -> Option<usize>`, `reset_fields_changed(&Self) -> Vec<&'static str>`, `validate()`; `booker_defaults()`, `guo_defaults()`, `si_defaults()`, `coloring_defaults()`; `schema() -> Vec<Param>`
  - `stats::{SERIES: [&str; 7], HybridSnapshot, series_names()}`
  - `world::{Body::{Population, Coloring}, Runner { config, body, steps } with new(HybridConfig, seed) -> Result<Self, Vec<FieldError>>, step(), cooperation() -> f64, mean_over(steps: u64, window: u64) -> f64, finished(), rng() -> &SimRng; HybridWorld with new(HybridConfig, seed), runner(), reference(), payoff_now(i) -> f64, run(u32), step()}`; `booker_rules`, `guo_rules`, `si_rules`, `coloring_rules` (`pub(crate)`)
  - `ModelKind::Hybrid` ("hybrid"), `ModelConfig::Hybrid(HybridConfig)`, `ModelWorld::Hybrid(Box<HybridWorld>)`

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/hybrid/config.rs` with only its doc comment and tests:

```rust
//! The hybrid kind's configuration: the world, the structure, the bots, the imitation rule and
//! each world's game. Game parameters apply live; structure, population, framework, θ, the step
//! unit and the time limit apply on reset (amendment 13).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_world_opens_on_its_papers_setup() {
        let b = booker_defaults();
        assert_eq!((b.world, b.structure, b.side, b.neighborhood, b.periodic), (World::Booker, Structure::Lattice, 50, Neighborhood::VonNeumann, false));
        assert_eq!((b.bot_kind, b.b, b.c, b.beta, b.bot_total(), b.step), (BotKind::Samaritan, 2.0, 1.0, 0.1, Some(250), StepUnit::Paper));
        let g = guo_defaults();
        assert_eq!((g.world, g.side, g.periodic, g.bot_total(), g.beta, g.t, g.s, g.bot_action), (World::Guo, 100, true, Some(4871), 10.0, 1.4, 0.5, BotAction::Redraw));
        let s = si_defaults();
        assert_eq!((s.world, s.neighborhood, s.periodic, s.bots, s.bot_fraction, s.theta, s.r), (World::Si, Neighborhood::Moore, true, Bots::Bernoulli, 0.3, 0.9, 0.1));
        assert_eq!((s.framework, s.mixed_draw, s.division, s.beta), (Framework::Mixed, MixedDraw::PerInteraction, Division::Integer, 10.0));
        let c = coloring_defaults();
        assert_eq!((c.world, c.structure, c.nodes, c.links, c.bot_total(), c.placement), (World::Coloring, Structure::BarabasiAlbert, 20, 2, Some(3), Placement::Hubs));
        assert_eq!((c.noise, c.errant, c.reaction, c.bot_interval, c.time_limit), (0.1, 0.041, 1.59, 1.5, 300.0));
        for cfg in [b, g, s, c] {
            assert!(cfg.validate().is_ok(), "{cfg:?}");
        }
    }

    #[test]
    fn out_of_range_and_inconsistent_fields_are_named() {
        let bad = HybridConfig { side: 2, beta: -1.0, phi: 2.0, r: 1.5, theta: -0.1, c: 0.0, noise: 2.0, reaction: 0.0, time_limit: 0.0, initial_cooperation: 2.0, ..booker_defaults() };
        let fields: Vec<String> = bad.validate().unwrap_err().into_iter().map(|e| e.field).collect();
        for f in ["side", "beta", "phi", "r", "theta", "c", "noise", "reaction", "time_limit", "initial_cooperation"] {
            assert!(fields.contains(&f.to_string()), "{f} missing from {fields:?}");
        }
        let wm = HybridConfig { structure: Structure::WellMixed, ..booker_defaults() };
        assert_eq!(wm.validate().unwrap_err()[0].field, "structure", "Booker's well-mixed case is the exact chain");
        let lattice_coloring = HybridConfig { structure: Structure::Lattice, ..coloring_defaults() };
        assert_eq!(lattice_coloring.validate().unwrap_err()[0].field, "structure");
        let crowded = HybridConfig { bots: Bots::Count, bot_count: 2500, ..booker_defaults() };
        assert_eq!(crowded.validate().unwrap_err()[0].field, "bot_count");
        let all = HybridConfig { bot_fraction: 1.0, ..booker_defaults() };
        assert_eq!(all.validate().unwrap_err()[0].field, "bot_fraction");
        assert!(HybridConfig { bot_fraction: 1.0, bots: Bots::Bernoulli, ..si_defaults() }.validate().is_ok(), "a Bernoulli draw may leave no humans");
        assert!(HybridConfig { b: f64::NAN, ..booker_defaults() }.validate().is_err());
    }

    #[test]
    fn configs_round_trip_through_json_with_snake_case_tags() {
        let json = r#"{"world":"si","structure":"well_mixed","nodes":500,"bots":"bernoulli","framework":"discrete","division":"float","mixed_draw":"per_round","bot_kind":"discriminatory","bot_action":"per_interaction","neighborhood":"moore","placement":"leaves","step":"sweep","accumulate":"mean","graph_seed":3}"#;
        let c: HybridConfig = serde_json::from_str(json).unwrap();
        assert_eq!((c.world, c.structure, c.framework, c.division, c.mixed_draw), (World::Si, Structure::WellMixed, Framework::Discrete, Division::Float, MixedDraw::PerRound));
        assert_eq!((c.bot_kind, c.bot_action, c.placement, c.step, c.accumulate, c.graph_seed), (BotKind::Discriminatory, BotAction::PerInteraction, Placement::Leaves, StepUnit::Sweep, Accumulate::Mean, Some(3)));
        let back: HybridConfig = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert!(serde_json::from_str::<HybridConfig>(r#"{"proxy_a":true}"#).is_err());
    }

    #[test]
    fn reset_fields_are_the_structure_and_the_population() {
        let a = booker_defaults();
        let b = HybridConfig { beta: 5.0, b: 3.0, side: 40, framework: Framework::Discrete, ..booker_defaults() };
        assert_eq!(a.reset_fields_changed(&b), vec!["side", "framework"]);
        assert!(a.reset_fields_changed(&a).is_empty());
    }

    #[test]
    fn schema_round_trips_and_reset_fields_are_refused() {
        use crate::model::{ModelConfig, ModelWorld};
        for config in [booker_defaults(), guo_defaults(), si_defaults(), coloring_defaults()] {
            let config = ModelConfig::Hybrid(config);
            crate::schema::check_schema(&schema(), &config, || ModelWorld::new(config.clone(), 1).unwrap());
        }
    }
}
```

Create `crates/sugarscape-core/src/hybrid/stats.rs` with only its doc comment and tests:

```rust
//! Per-tick series: the humans' cooperation (their cooperating share, or mean strategy), the
//! bots' cooperating share, the bots' share of the nodes, the world's exact reference, and for
//! coloring the conflicts, the solve time and the game clock (NaN where a series does not apply).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_series_is_readable_and_defaults_to_nan() {
        let s = HybridSnapshot::default();
        assert_eq!(s.value("tick"), Some(0.0));
        for name in SERIES {
            assert!(s.value(name).unwrap().is_nan(), "{name}");
        }
        assert!(s.value("nope").is_none());
        assert_eq!(series_names().len(), 7);
    }
}
```

Create `crates/sugarscape-core/src/hybrid/world.rs` with only its doc comment and tests:

```rust
//! The hybrid world: Booker's, Guo's or Si et al.'s population, or a coloring session, stepped
//! one paper step per tick, with the exact reference for the current cell (Booker's all-C weight
//! for these N and M, Guo's highest stable master-equation root at this y, the coloring graph's
//! solution count). `Runner` is the same dynamics without the page's history; the survey drives it.

#[cfg(test)]
mod tests {
    use super::*;

    fn world(c: HybridConfig) -> HybridWorld {
        HybridWorld::new(c, 1).unwrap()
    }

    fn actions(w: &HybridWorld) -> Vec<bool> {
        match &w.runner().body {
            Body::Population(p) => p.action.clone(),
            Body::Coloring(_) => unreachable!(),
        }
    }

    #[test]
    fn tick_zero_carries_the_population_and_the_reference() {
        let w = world(booker_defaults());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.tick, 0);
        assert!((s.cooperation - 0.5).abs() < 0.05, "{s:?}");
        assert_eq!((s.bot_share, s.bot_cooperation), (0.1, 1.0), "250 Samaritans");
        let want = booker::Chain { n: 2250, m: 250, game: Donation { b: 2.0, c: 1.0 }, kind: BotKind::Samaritan, beta: 0.1 }.all_c_weight();
        assert_eq!(s.reference, want);
        assert!(s.conflicts.is_nan() && s.solved_at.is_nan() && s.game_time.is_nan());
        let g = world(HybridConfig { side: 20, bot_count: 100, ..guo_defaults() });
        let stable = guo::master_roots(&Plane { t: 1.4, s: 0.5 }, 100.0 / 300.0, 0.5, 10.0);
        assert_eq!(g.reference(), stable.iter().filter(|r| r.stable).map(|r| r.x).fold(f64::NAN, f64::max));
        assert!(world(HybridConfig { side: 10, ..si_defaults() }).reference().is_nan());
    }

    #[test]
    fn a_tick_is_the_papers_step() {
        let mut b = world(booker_defaults());
        let before = actions(&b);
        b.run(1);
        let changed = actions(&b).iter().zip(&before).filter(|(x, y)| x != y).count();
        assert!(changed <= 1, "Booker's step is one update: {changed}");
        assert_eq!((b.tick, b.runner().steps), (1, 1));
        let mut sweep = world(HybridConfig { step: StepUnit::Sweep, ..booker_defaults() });
        let before = actions(&sweep);
        sweep.run(1);
        assert!(actions(&sweep).iter().zip(&before).filter(|(x, y)| x != y).count() > 1, "N updates");
        let mut alg1 = world(HybridConfig { structure: Structure::WellMixed, nodes: 1000, bot_count: 487, ..guo_defaults() });
        let before = actions(&alg1);
        alg1.run(1);
        assert!(actions(&alg1).iter().zip(&before).filter(|(x, y)| x != y).count() <= 1, "Algorithm I: one comparison");
    }

    #[test]
    fn a_finished_coloring_world_holds() {
        let mut w = world(coloring_defaults());
        w.run(1);
        let s = w.stats.latest().unwrap();
        assert!(s.game_time <= 1.0 && s.cooperation.is_nan() && s.reference >= 6.0);
        w.run(400);
        assert!(w.finished() && w.holds_when_finished());
        let (tick, fp) = (w.tick, w.fingerprint());
        w.run(50);
        assert_eq!((w.tick, w.fingerprint()), (tick, fp), "a finished session runs no more ticks");
        let s = w.stats.latest().unwrap();
        assert!(s.game_time <= 300.0 && (s.solved_at.is_nan() || s.conflicts == 0.0), "{s:?}");
    }

    #[test]
    fn runner_means_read_after_every_step() {
        let c = HybridConfig { side: 10, ..booker_defaults() };
        let mut a = Runner::new(c.clone(), 3).unwrap();
        let mean = a.mean_over(200, 50);
        let mut b = Runner::new(c, 3).unwrap();
        let mut sum = 0.0;
        for t in 0..200 {
            b.step();
            if t >= 150 {
                sum += b.cooperation();
            }
        }
        assert_eq!(mean, sum / 50.0);
    }

    #[test]
    fn the_same_seed_gives_the_same_world_and_graph_seed_fixes_the_graph() {
        let c = HybridConfig { side: 20, ..si_defaults() };
        let (mut a, mut b) = (world(c.clone()), world(c.clone()));
        a.run(3);
        b.run(3);
        assert_eq!(a.fingerprint(), b.fingerprint());
        let mut other = HybridWorld::new(c, 2).unwrap();
        other.run(3);
        assert_ne!(a.fingerprint(), other.fingerprint());
        let ba = HybridConfig { structure: Structure::BarabasiAlbert, nodes: 500, bot_count: 100, graph_seed: Some(7), ..guo_defaults() };
        let graph = |c: &HybridConfig, seed| match Runner::new(c.clone(), seed).unwrap().body {
            Body::Population(p) => p.graph,
            Body::Coloring(_) => unreachable!(),
        };
        assert_eq!(graph(&ba, 1), graph(&ba, 2));
        let fresh = HybridConfig { graph_seed: None, ..ba };
        assert_ne!(graph(&fresh, 1), graph(&fresh, 2));
    }

    #[test]
    fn a_world_without_humans_runs_and_fingerprints() {
        let c = HybridConfig { structure: Structure::WellMixed, nodes: 5, bot_fraction: 1.0, ..si_defaults() };
        let (mut w, mut v) = (world(c.clone()), world(c));
        w.run(5);
        v.run(5);
        assert!(w.stats.latest().unwrap().cooperation.is_nan());
        assert_eq!(w.fingerprint(), v.fingerprint());
        let mut none = world(HybridConfig { bots: Bots::Count, bot_count: 0, side: 10, ..booker_defaults() });
        none.run(5);
        assert!(none.stats.latest().unwrap().bot_cooperation.is_nan());
    }

    #[test]
    fn game_parameters_apply_live_and_structure_is_refused() {
        let mut w = world(booker_defaults());
        let before = w.reference();
        w.set_config(ModelConfig::Hybrid(HybridConfig { beta: 1.0, ..booker_defaults() })).unwrap();
        assert_eq!(w.config.beta, 1.0);
        assert_ne!(w.reference(), before);
        let err = w.set_config(ModelConfig::Hybrid(HybridConfig { side: 40, ..booker_defaults() })).unwrap_err();
        assert_eq!((err[0].field.as_str(), err[0].message.as_str()), ("side", "changes only on reset"));
        let mut c = world(coloring_defaults());
        c.set_config(ModelConfig::Hybrid(HybridConfig { noise: 0.3, ..coloring_defaults() })).unwrap();
        match &c.runner().body {
            Body::Coloring(s) => assert_eq!(s.rules.noise, 0.3),
            Body::Population(_) => unreachable!(),
        }
    }
}
```

Add to `mod.rs`:

```rust
mod config;
mod stats;
mod view;
mod world;
pub use config::*;
pub use stats::{series_names, HybridSnapshot, SERIES};
pub use world::{Body, HybridWorld, Runner};
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: compile errors (`HybridConfig`, `HybridSnapshot`, `HybridWorld` undefined).

- [ ] **Step 3: Write `config.rs`**

Above its tests module:

```rust
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};

pub use super::booker::BotKind;
pub use super::engine::Accumulate;
pub use super::graph::{Neighborhood, Placement};
pub use super::guo::BotAction;
pub use super::si::{Division, Framework, MixedDraw};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum World {
    Booker,
    Guo,
    Si,
    Coloring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Structure {
    WellMixed,
    Lattice,
    ErdosRenyi,
    BarabasiAlbert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bots {
    /// `bot_count` bots, placed by `placement`.
    Count,
    /// round(`bot_fraction` · N) bots, placed by `placement`.
    Fraction,
    /// Each node a bot with probability `bot_fraction` (Si et al.'s code).
    Bernoulli,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepUnit {
    /// The paper's step: one update (Booker), one comparison (Guo, well-mixed), N updates (Guo
    /// on a graph), one round (Si), one second (coloring).
    Paper,
    /// N updates for Booker and well-mixed Guo (`sweep_steps`); the paper's step elsewhere.
    Sweep,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HybridConfig {
    pub world: World,
    pub structure: Structure,
    pub side: u32,
    pub nodes: u32,
    pub neighborhood: Neighborhood,
    pub periodic: bool,
    pub mean_degree: f64,
    pub links: u32,
    /// The graph's own seed (Booker's ten networks); None draws the graph from the run's seed.
    pub graph_seed: Option<u64>,
    pub bots: Bots,
    pub bot_count: u32,
    pub bot_fraction: f64,
    pub placement: Placement,
    pub beta: f64,
    pub step: StepUnit,
    pub accumulate: Accumulate,
    pub initial_cooperation: f64,
    pub bot_kind: BotKind,
    pub b: f64,
    pub c: f64,
    pub t: f64,
    pub s: f64,
    pub phi: f64,
    pub bot_action: BotAction,
    pub r: f64,
    pub theta: f64,
    pub framework: Framework,
    pub mixed_draw: MixedDraw,
    pub division: Division,
    pub noise: f64,
    pub errant: f64,
    pub reaction: f64,
    pub bot_interval: f64,
    pub time_limit: f64,
}

/// Booker et al.'s lattice (Figs. 4–6): 50 × 50, open, von Neumann; 10% Samaritan bots; β = 0.1;
/// b = 2, c = 1; humans start C with probability ½.
impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            world: World::Booker,
            structure: Structure::Lattice,
            side: 50,
            nodes: 1000,
            neighborhood: Neighborhood::VonNeumann,
            periodic: false,
            mean_degree: 4.0,
            links: 2,
            graph_seed: None,
            bots: Bots::Fraction,
            bot_count: 0,
            bot_fraction: 0.1,
            placement: Placement::Random,
            beta: 0.1,
            step: StepUnit::Paper,
            accumulate: Accumulate::Sum,
            initial_cooperation: 0.5,
            bot_kind: BotKind::Samaritan,
            b: 2.0,
            c: 1.0,
            t: 1.4,
            s: 0.5,
            phi: 0.5,
            bot_action: BotAction::Redraw,
            r: 0.1,
            theta: 0.9,
            framework: Framework::Mixed,
            mixed_draw: MixedDraw::PerInteraction,
            division: Division::Integer,
            noise: 0.1,
            errant: 0.041,
            reaction: 1.59,
            bot_interval: 1.5,
            time_limit: 300.0,
        }
    }
}

pub fn booker_defaults() -> HybridConfig {
    HybridConfig::default()
}

/// Guo et al.'s Fig. 6 lattice: 100 × 100, periodic, von Neumann; 4,871 bots (y = 0.95) at
/// random; K = 10; snowdrift T = 1.4, S = 0.5; φ = 0.5.
pub fn guo_defaults() -> HybridConfig {
    HybridConfig {
        world: World::Guo,
        side: 100,
        nodes: 10_000,
        periodic: true,
        bots: Bots::Count,
        bot_count: 4871,
        beta: 10.0,
        ..Default::default()
    }
}

/// Si et al.'s Fig. 3 lattice as the code runs it: 100 × 100, periodic, Moore; each node a bot
/// with probability 0.3; θ = 0.9; r = 0.1; β = 1/κ = 10; the mixed framework.
pub fn si_defaults() -> HybridConfig {
    HybridConfig {
        world: World::Si,
        side: 100,
        nodes: 10_000,
        neighborhood: Neighborhood::Moore,
        periodic: true,
        bots: Bots::Bernoulli,
        bot_fraction: 0.3,
        beta: 10.0,
        ..Default::default()
    }
}

/// Shirado and Christakis's game: a 20-node Barabási–Albert graph with two links per new node,
/// three bots on the highest-degree nodes with 10% noise, the reconstructed humans' errant rate
/// 0.041 and mean decision time 1.59 s, a 300 s limit.
pub fn coloring_defaults() -> HybridConfig {
    HybridConfig {
        world: World::Coloring,
        structure: Structure::BarabasiAlbert,
        nodes: 20,
        links: 2,
        bots: Bots::Count,
        bot_count: 3,
        placement: Placement::Hubs,
        ..Default::default()
    }
}

impl HybridConfig {
    /// The node count: side² on a lattice, `nodes` otherwise.
    pub fn size(&self) -> usize {
        if self.structure == Structure::Lattice {
            (self.side as usize).pow(2)
        } else {
            self.nodes as usize
        }
    }

    /// How many bots are placed (None for a Bernoulli draw).
    pub fn bot_total(&self) -> Option<usize> {
        match self.bots {
            Bots::Count => Some(self.bot_count as usize),
            Bots::Fraction => Some((self.bot_fraction * self.size() as f64).round() as usize),
            Bots::Bernoulli => None,
        }
    }

    /// The names of the reset-only fields that differ between `self` and `other`.
    pub fn reset_fields_changed(&self, other: &Self) -> Vec<&'static str> {
        [
            (self.world != other.world, "world"),
            (self.structure != other.structure, "structure"),
            (self.side != other.side, "side"),
            (self.nodes != other.nodes, "nodes"),
            (self.neighborhood != other.neighborhood, "neighborhood"),
            (self.periodic != other.periodic, "periodic"),
            (self.mean_degree != other.mean_degree, "mean_degree"),
            (self.links != other.links, "links"),
            (self.graph_seed != other.graph_seed, "graph_seed"),
            (self.bots != other.bots, "bots"),
            (self.bot_count != other.bot_count, "bot_count"),
            (self.bot_fraction != other.bot_fraction, "bot_fraction"),
            (self.placement != other.placement, "placement"),
            (self.step != other.step, "step"),
            (self.initial_cooperation != other.initial_cooperation, "initial_cooperation"),
            (self.bot_kind != other.bot_kind, "bot_kind"),
            (self.theta != other.theta, "theta"),
            (self.framework != other.framework, "framework"),
            (self.time_limit != other.time_limit, "time_limit"),
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
        let within = |v: f64, lo: f64, hi: f64| v.is_finite() && (lo..=hi).contains(&v);
        match self.world {
            World::Booker => check(
                self.structure != Structure::WellMixed,
                "structure",
                "Booker's runs need a graph; the well-mixed case is the exact chain, drawn as the reference",
            ),
            World::Coloring => check(
                self.structure == Structure::BarabasiAlbert && self.links <= 2 && self.nodes <= 200,
                "structure",
                "the coloring game runs on a Barabási–Albert graph of at most 200 nodes with one or two links per new node",
            ),
            World::Guo | World::Si => {}
        }
        if self.structure == Structure::Lattice {
            check((3..=400).contains(&self.side), "side", "must be between 3 and 400");
        } else {
            check((2..=200_000).contains(&self.nodes), "nodes", "must be between 2 and 200000");
        }
        check(within(self.mean_degree, 0.1, 50.0), "mean_degree", "must be between 0.1 and 50");
        check(
            (1..=10).contains(&self.links)
                && (self.structure != Structure::BarabasiAlbert || (self.links as usize) < self.size()),
            "links",
            "must be between 1 and 10 and below the node count",
        );
        check(within(self.bot_fraction, 0.0, 1.0), "bot_fraction", "must be between 0 and 1");
        if let Some(bots) = self.bot_total() {
            check(
                bots < self.size(),
                if self.bots == Bots::Count { "bot_count" } else { "bot_fraction" },
                "must leave at least one human",
            );
        }
        check(within(self.beta, 0.0, 1000.0), "beta", "must be between 0 and 1000");
        check(within(self.initial_cooperation, 0.0, 1.0), "initial_cooperation", "must be between 0 and 1");
        check(within(self.b, 0.0, 100.0), "b", "must be between 0 and 100");
        check(within(self.c, 0.01, 10.0), "c", "must be between 0.01 and 10");
        check(within(self.t, -10.0, 10.0), "t", "must be between -10 and 10");
        check(within(self.s, -10.0, 10.0), "s", "must be between -10 and 10");
        check(within(self.phi, 0.0, 1.0), "phi", "must be between 0 and 1");
        check(within(self.r, 0.0, 1.0), "r", "must be between 0 and 1");
        check(within(self.theta, 0.0, 1.0), "theta", "must be between 0 and 1");
        check(within(self.noise, 0.0, 1.0), "noise", "must be between 0 and 1");
        check(within(self.errant, 0.0, 1.0), "errant", "must be between 0 and 1");
        check(within(self.reaction, 0.01, 60.0), "reaction", "must be between 0.01 and 60 seconds");
        check(within(self.bot_interval, 0.01, 60.0), "bot_interval", "must be between 0.01 and 60 seconds");
        check(within(self.time_limit, 1.0, 3600.0), "time_limit", "must be between 1 and 3600 seconds");
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
        Param::choice("World", "world", "World", &[("booker", "Booker et al.: Samaritan or discriminatory bots"), ("guo", "Guo et al.: bots that cooperate with probability φ"), ("si", "Si et al.: cooperative bots in three frameworks"), ("coloring", "Shirado & Christakis: noisy bots in a coloring game")], Reset)
            .with_help("Each world opens on its paper's setup from the presets; switching here keeps the other fields."),
        Param::choice("Structure", "structure", "Structure", &[("well_mixed", "Well-mixed"), ("lattice", "Square lattice"), ("erdos_renyi", "Erdős–Rényi"), ("barabasi_albert", "Barabási–Albert")], Reset),
        Param::integer("Structure", "side", "Lattice side", (3, 400), Reset).shown_if("structure", "lattice"),
        Param::choice("Structure", "neighborhood", "Neighborhood", &[("von_neumann", "Von Neumann (4)"), ("moore", "Moore (8)")], Reset).shown_if("structure", "lattice"),
        Param::bool("Structure", "periodic", "Wrap around", Reset).shown_if("structure", "lattice"),
        Param::integer("Structure", "nodes", "Nodes", (2, 200_000), Reset)
            .with_help("Ignored on a lattice, where the side sets the size."),
        Param::number("Structure", "mean_degree", "Mean degree", (0.1, 50.0, 0.1), Reset).shown_if("structure", "erdos_renyi"),
        Param::integer("Structure", "links", "Links per new node (m)", (1, 10), Reset).shown_if("structure", "barabasi_albert"),
        Param::choice("Bots", "bots", "Bots by", &[("count", "Count"), ("fraction", "Fraction of the nodes"), ("bernoulli", "Each node with this probability (Si's code)")], Reset),
        Param::integer("Bots", "bot_count", "Bot count", (0, 200_000), Reset).shown_if("bots", "count"),
        Param::number("Bots", "bot_fraction", "Bot fraction", (0.0, 1.0, 0.01), Reset)
            .with_help("A fraction of the nodes, or each node's probability of being a bot."),
        Param::choice("Bots", "placement", "Placement", &[("random", "Random"), ("hubs", "Highest degree first"), ("leaves", "Lowest degree first")], Reset)
            .with_help("Ignored for a Bernoulli draw."),
        Param::number("Imitation", "beta", "Selection strength β", (0.0, 1000.0, 0.01), Live)
            .with_help("The Fermi rule 1/(1 + e^{−β(π_model − π_focal)}): Booker's β, Guo's K, Si's 1/κ."),
        Param::choice("Imitation", "step", "Step", &[("paper", "The paper's step"), ("sweep", "N updates (a Monte Carlo step)")], Reset),
        Param::choice("Imitation", "accumulate", "Payoffs", &[("sum", "Summed over neighbors"), ("mean", "Mean over neighbors")], Live),
        Param::number("Imitation", "initial_cooperation", "Humans cooperating at the start", (0.0, 1.0, 0.01), Reset),
        Param::choice("Booker", "bot_kind", "Bots", &[("samaritan", "Samaritan (always C)"), ("discriminatory", "Discriminatory (mirror the partner)"), ("malicious", "Malicious (always D)")], Reset),
        Param::number("Booker", "b", "Benefit b", (0.0, 100.0, 0.1), Live),
        Param::number("Booker", "c", "Cost c", (0.01, 10.0, 0.01), Live),
        Param::number("Guo", "t", "Temptation T", (-10.0, 10.0, 0.01), Live),
        Param::number("Guo", "s", "Sucker's payoff S", (-10.0, 10.0, 0.01), Live),
        Param::number("Guo", "phi", "Bot cooperation φ", (0.0, 1.0, 0.01), Live),
        Param::choice("Guo", "bot_action", "Bot action", &[("redraw", "Redrawn when chosen (Algorithm II)"), ("frozen", "Drawn once"), ("per_interaction", "Drawn for every game")], Live),
        Param::number("Si", "r", "Dilemma strength r", (0.0, 1.0, 0.01), Live),
        Param::number("Si", "theta", "Bot strategy θ", (0.0, 1.0, 0.01), Reset),
        Param::choice("Si", "framework", "Framework", &[("discrete", "Discrete"), ("continuous", "Continuous"), ("mixed", "Mixed")], Reset),
        Param::choice("Si", "mixed_draw", "Mixed actions", &[("per_interaction", "Drawn for every game (the code)"), ("per_round", "Drawn once a round (Fig. A1)")], Live).shown_if("framework", "mixed"),
        Param::choice("Si", "division", "Well-mixed payoff", &[("integer", "As the code runs it (integer division)"), ("float", "The stated game (Eq. 1)")], Live)
            .with_help("The code's (N − 1 − c)/(N − 1) is integer division, so S and P drop out unless nobody else cooperates."),
        Param::number("Coloring", "noise", "Bot noise", (0.0, 1.0, 0.01), Live),
        Param::number("Coloring", "errant", "Human errant rate (ours)", (0.0, 1.0, 0.001), Live),
        Param::number("Coloring", "reaction", "Human mean decision time (s, ours)", (0.01, 60.0, 0.01), Live),
        Param::number("Coloring", "bot_interval", "Bot decision interval (s)", (0.01, 60.0, 0.01), Live),
        Param::number("Coloring", "time_limit", "Time limit (s)", (1.0, 3600.0, 1.0), Reset),
    ]
}
```

- [ ] **Step 4: Write `stats.rs`**

Above its tests module:

```rust
use crate::stats::Series;
use serde::Serialize;

pub const SERIES: [&str; 7] = [
    "cooperation",
    "bot_cooperation",
    "bot_share",
    "reference",
    "conflicts",
    "solved_at",
    "game_time",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HybridSnapshot {
    pub tick: u64,
    pub cooperation: f64,
    pub bot_cooperation: f64,
    pub bot_share: f64,
    pub reference: f64,
    pub conflicts: f64,
    pub solved_at: f64,
    pub game_time: f64,
}

impl Default for HybridSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            cooperation: f64::NAN,
            bot_cooperation: f64::NAN,
            bot_share: f64::NAN,
            reference: f64::NAN,
            conflicts: f64::NAN,
            solved_at: f64::NAN,
            game_time: f64::NAN,
        }
    }
}

impl Series for HybridSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }
    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "cooperation" => self.cooperation,
            "bot_cooperation" => self.bot_cooperation,
            "bot_share" => self.bot_share,
            "reference" => self.reference,
            "conflicts" => self.conflicts,
            "solved_at" => self.solved_at,
            "game_time" => self.game_time,
            _ => return None,
        })
    }
}

pub fn series_names() -> Vec<String> {
    SERIES.iter().map(|s| s.to_string()).collect()
}
```

- [ ] **Step 5: Write `world.rs` and the view stub**

`view.rs` (replaced in Task 8):

```rust
//! Replaced in the next task.
use super::world::HybridWorld;
pub fn size(_w: &HybridWorld) -> (u32, u32) {
    (1, 1)
}
pub fn position(_w: &HybridWorld, _i: usize) -> (u32, u32) {
    (0, 0)
}
pub fn render(_w: &HybridWorld, _mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    buf.clear();
    buf.extend([0, 0, 0, 255]);
    Ok(())
}
pub fn inspect(_w: &HybridWorld, _x: u32, _y: u32) -> Result<String, String> {
    Err("no inspection yet".into())
}
```

`world.rs`, above its tests module:

```rust
use super::{
    booker::{self, Donation},
    coloring::{self, Session},
    config::*,
    engine::Population,
    graph::{bernoulli, erdos_renyi, lattice, place},
    guo::{self, Plane},
    si,
    stats::{HybridSnapshot, SERIES},
    view,
};
use crate::{
    config::FieldError,
    graph::{barabasi_albert, Graph},
    model::{wrong_model, Model, ModelConfig, ModelKind},
    rng::{self, SimRng},
    stats::{Series, Stats},
};
use rand::Rng;
use std::fmt::Write;

/// What a world runs: a population of humans and bots, or a coloring session.
#[derive(Clone, Debug)]
pub enum Body {
    Population(Population),
    Coloring(Session),
}

/// The dynamics without history.
#[derive(Clone, Debug)]
pub struct Runner {
    pub config: HybridConfig,
    pub body: Body,
    /// Paper steps taken.
    pub steps: u64,
    rng: SimRng,
}

pub(crate) fn booker_rules(c: &HybridConfig) -> booker::Rules {
    booker::Rules { game: Donation { b: c.b, c: c.c }, kind: c.bot_kind, beta: c.beta, accumulate: c.accumulate }
}

pub(crate) fn guo_rules(c: &HybridConfig) -> guo::Rules {
    guo::Rules { plane: Plane { t: c.t, s: c.s }, phi: c.phi, beta: c.beta, bot_action: c.bot_action, accumulate: c.accumulate }
}

pub(crate) fn si_rules(c: &HybridConfig) -> si::Rules {
    si::Rules { r: c.r, framework: c.framework, mixed_draw: c.mixed_draw, division: c.division, beta: c.beta, accumulate: c.accumulate }
}

pub(crate) fn coloring_rules(c: &HybridConfig) -> coloring::Rules {
    coloring::Rules { noise: c.noise, errant: c.errant, reaction: c.reaction, bot_interval: c.bot_interval, time_limit: c.time_limit }
}

/// The graph (None when well-mixed), drawn from `graph_seed`'s own stream when it is set and
/// from the run's stream otherwise.
fn graph_of(c: &HybridConfig, rng: &mut SimRng) -> Option<Vec<Vec<u32>>> {
    let mut own = c.graph_seed.map(rng::seeded);
    let g = match own.as_mut() {
        Some(r) => r,
        None => &mut *rng,
    };
    Some(match c.structure {
        Structure::WellMixed => return None,
        Structure::Lattice => lattice(c.side as usize, c.neighborhood, c.periodic),
        Structure::ErdosRenyi => erdos_renyi(c.nodes as usize, c.mean_degree, g),
        Structure::BarabasiAlbert => barabasi_albert(c.nodes as usize, c.links as usize, g),
    })
}

/// The bots: a Bernoulli draw per node, or a placed count (on degrees; all equal well-mixed).
fn bots_of(c: &HybridConfig, adj: Option<&Vec<Vec<u32>>>, rng: &mut SimRng) -> Vec<bool> {
    let n = c.size();
    match c.bot_total() {
        None => bernoulli(n, c.bot_fraction, rng),
        Some(count) => {
            let degrees: Vec<usize> = match adj {
                Some(a) => a.iter().map(Vec::len).collect(),
                None => vec![n - 1; n],
            };
            place(&degrees, count, c.placement, rng)
        }
    }
}

impl Runner {
    pub fn new(config: HybridConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let adj = graph_of(&config, &mut rng);
        let bot = bots_of(&config, adj.as_ref(), &mut rng);
        let body = if config.world == World::Coloring {
            let adj = adj.expect("the coloring game has a graph");
            assert!(coloring::colorings(&adj, 3) > 0, "a Barabási–Albert graph with m ≤ 2 is 3-colorable");
            Body::Coloring(Session::new(adj, bot, coloring_rules(&config), &mut rng))
        } else {
            let mut pop = Population::new(adj.map(Graph::from_lists).unwrap_or_default(), bot);
            match config.world {
                World::Booker => booker::init(&mut pop, config.bot_kind, config.initial_cooperation, &mut rng),
                World::Guo => guo::init(&mut pop, config.phi, config.initial_cooperation, &mut rng),
                World::Si => si::init(&mut pop, config.framework, config.theta, &mut rng),
                World::Coloring => unreachable!(),
            }
            Body::Population(pop)
        };
        Ok(Runner { config, body, steps: 0, rng })
    }

    /// One paper step (`StepUnit`).
    pub fn step(&mut self) {
        let c = &self.config;
        match &mut self.body {
            Body::Population(pop) => {
                let n = pop.nodes;
                let updates = if c.step == StepUnit::Sweep { n } else { 1 };
                match c.world {
                    World::Booker => {
                        let r = booker_rules(c);
                        for _ in 0..updates {
                            booker::micro(&r, pop, &mut self.rng);
                        }
                    }
                    World::Guo if pop.well_mixed() => {
                        let r = guo_rules(c);
                        for _ in 0..updates {
                            guo::micro_well_mixed(&r, pop, &mut self.rng);
                        }
                    }
                    World::Guo => {
                        let r = guo_rules(c);
                        for _ in 0..n {
                            guo::micro_network(&r, pop, &mut self.rng);
                        }
                    }
                    World::Si => si::round(&si_rules(c), pop, &mut self.rng),
                    World::Coloring => unreachable!(),
                }
            }
            Body::Coloring(s) => s.advance_to((self.steps + 1) as f64, &mut self.rng),
        }
        self.steps += 1;
    }

    /// The humans' cooperation now (NaN for coloring).
    pub fn cooperation(&self) -> f64 {
        match &self.body {
            Body::Population(p) => p.human_cooperation(),
            Body::Coloring(_) => f64::NAN,
        }
    }

    /// Runs `steps` steps and returns the mean human cooperation over the last `window`, read
    /// after each step.
    pub fn mean_over(&mut self, steps: u64, window: u64) -> f64 {
        assert!(window >= 1 && window <= steps, "the window lies inside the run");
        let mut sum = 0.0;
        for t in 0..steps {
            self.step();
            if t >= steps - window {
                sum += self.cooperation();
            }
        }
        sum / window as f64
    }

    pub fn finished(&self) -> bool {
        matches!(&self.body, Body::Coloring(s) if s.finished())
    }

    pub fn rng(&self) -> &SimRng {
        &self.rng
    }
}

#[derive(Clone)]
pub struct HybridWorld {
    pub config: HybridConfig,
    pub tick: u64,
    pub stats: Stats<HybridSnapshot>,
    seed: u64,
    runner: Runner,
    reference: f64,
}

/// A number's bytes for the fingerprint, with every NaN the same NaN.
fn bits(x: f64) -> [u8; 8] {
    (if x.is_nan() { f64::NAN } else { x }).to_bits().to_le_bytes()
}

impl HybridWorld {
    pub fn new(config: HybridConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        let runner = Runner::new(config.clone(), seed)?;
        let mut w = HybridWorld { config, tick: 0, stats: Stats::default(), seed, runner, reference: f64::NAN };
        w.reference = w.compute_reference();
        let s = w.snapshot();
        w.stats.push(s);
        Ok(w)
    }

    pub fn runner(&self) -> &Runner {
        &self.runner
    }

    pub fn reference(&self) -> f64 {
        self.reference
    }

    /// Booker: the all-C weight of the chain with these humans and bots; Guo: the highest stable
    /// master-equation root at y = bots/humans; coloring: the number of proper 3-colorings; Si:
    /// none (NaN).
    fn compute_reference(&self) -> f64 {
        let c = &self.config;
        match &self.runner.body {
            Body::Population(p) => {
                let humans = p.humans.len() as u32;
                let bots = p.nodes as u32 - humans;
                match c.world {
                    World::Booker if humans >= 2 => booker::Chain { n: humans, m: bots, game: Donation { b: c.b, c: c.c }, kind: c.bot_kind, beta: c.beta }.all_c_weight(),
                    World::Guo if humans > 0 => {
                        let y = f64::from(bots) / f64::from(humans);
                        guo::master_roots(&Plane { t: c.t, s: c.s }, y, c.phi, c.beta)
                            .iter()
                            .filter(|r| r.stable)
                            .map(|r| r.x)
                            .fold(f64::NAN, f64::max)
                    }
                    _ => f64::NAN,
                }
            }
            Body::Coloring(s) => coloring::colorings(&s.adj, 3) as f64,
        }
    }

    /// Node i's payoff from the stored actions (Inspect, the agents CSV); NaN in coloring.
    pub fn payoff_now(&self, i: usize) -> f64 {
        let c = &self.config;
        match &self.runner.body {
            Body::Population(p) => match c.world {
                World::Booker => booker::payoff(&booker_rules(c), p, i),
                World::Guo => guo::payoff_stored(&guo_rules(c), p, i),
                World::Si => si::payoff_stored(&si_rules(c), p, i),
                World::Coloring => f64::NAN,
            },
            Body::Coloring(_) => f64::NAN,
        }
    }

    pub fn step(&mut self) {
        self.runner.step();
        self.tick += 1;
        let s = self.snapshot();
        self.stats.push(s);
    }

    /// Runs `ticks` ticks, stopping once a coloring session has finished.
    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.runner.finished() {
                break;
            }
            self.step();
        }
    }

    fn snapshot(&self) -> HybridSnapshot {
        match &self.runner.body {
            Body::Population(p) => HybridSnapshot {
                tick: self.tick,
                cooperation: p.human_cooperation(),
                bot_cooperation: p.bot_cooperation(),
                bot_share: (p.nodes - p.humans.len()) as f64 / p.nodes as f64,
                reference: self.reference,
                ..Default::default()
            },
            Body::Coloring(s) => HybridSnapshot {
                tick: self.tick,
                bot_share: s.bot.iter().filter(|&&b| b).count() as f64 / s.bot.len() as f64,
                reference: self.reference,
                conflicts: s.conflicts as f64,
                solved_at: s.solved_at.unwrap_or(f64::NAN),
                game_time: s.time,
                ..Default::default()
            },
        }
    }
}

impl Model for HybridWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Hybrid(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        HybridWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        self.config.size()
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({
            "config": self.config, "seed": self.seed, "tick": self.tick, "steps": self.runner.steps,
        });
        let mut bytes = serde_json::to_vec(&state).unwrap();
        match &self.runner.body {
            Body::Population(p) => {
                for i in 0..p.nodes {
                    bytes.extend(bits(p.strategy[i]));
                    bytes.push(u8::from(p.action[i]));
                }
                bytes.extend((p.cooperating as u64).to_le_bytes());
            }
            Body::Coloring(s) => {
                bytes.extend(&s.color);
                for &t in &s.next {
                    bytes.extend(bits(t));
                }
                bytes.extend(bits(s.time));
                bytes.extend((s.conflicts as u64).to_le_bytes());
            }
        }
        bytes.extend(bits(self.reference));
        if let Some(s) = self.stats.latest() {
            for name in SERIES {
                bytes.extend(bits(s.value(name).unwrap()));
            }
        }
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for byte in bytes {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let mut stream = self.runner.rng().clone();
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
        match &self.runner.body {
            Body::Population(p) => {
                out.push_str("node,x,y,bot,strategy,action,degree,payoff\n");
                for i in 0..p.nodes {
                    let (x, y) = view::position(self, i);
                    let action = if p.action[i] { "C" } else { "D" };
                    writeln!(out, "{i},{x},{y},{},{},{action},{},{}", p.bot[i], p.strategy[i], p.degree(i), self.payoff_now(i)).unwrap();
                }
            }
            Body::Coloring(s) => {
                out.push_str("node,x,y,bot,color,degree\n");
                for i in 0..s.color.len() {
                    let (x, y) = view::position(self, i);
                    writeln!(out, "{i},{x},{y},{},{},{}", s.bot[i], s.color[i], s.adj[i].len()).unwrap();
                }
            }
        }
        out
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        view::inspect(self, x, y)
    }
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        (id < self.config.size() as u64).then(|| view::position(self, id as usize))
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Hybrid(next) = next else {
            return Err(wrong_model(ModelKind::Hybrid, &next));
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
        self.config = next.clone();
        self.runner.config = next;
        if let Body::Coloring(s) = &mut self.runner.body {
            s.rules = coloring_rules(&self.config);
        }
        self.reference = self.compute_reference();
        Ok(())
    }
    fn finished(&self) -> bool {
        self.runner.finished()
    }
    fn holds_when_finished(&self) -> bool {
        self.config.world == World::Coloring
    }
}
```

- [ ] **Step 6: Register the kind in `model.rs` (the same 17 sites as `misaligned`)**

In `crates/sugarscape-core/src/model.rs`, with `grep -n Misaligned` as the guide, add after each misaligned line:

1. Imports (line 29): `use crate::hybrid::{HybridConfig, HybridWorld};`
2. `pub enum ModelKind`: `Hybrid,` after `Misaligned,`.
3. `pub const ALL: [ModelKind; 35]` → `36`, and `ModelKind::Hybrid,` after `ModelKind::Misaligned,`.
4. `as_str`: `ModelKind::Hybrid => "hybrid",`.
5. `schema`: `ModelKind::Hybrid => crate::hybrid::schema(),`.
6. `pub enum ModelConfig`: `Hybrid(HybridConfig),`.
7. `enum Tagged<'a>`: `Hybrid(&'a HybridConfig),`.
8. `impl Serialize for ModelConfig`: `ModelConfig::Hybrid(c) => Tagged::Hybrid(c).serialize(s),`.
9. `kind()`: `ModelConfig::Hybrid(_) => ModelKind::Hybrid,`.
10. The tag parser: `"hybrid" => serde_json::from_value(value).map(ModelConfig::Hybrid).map_err(|e| FieldError::new("config", e.to_string())),` in the misaligned arm's shape, and the expected list's tail `gridworlds, misaligned or polarity` → `gridworlds, misaligned, hybrid or polarity`.
11. `validate`: `ModelConfig::Hybrid(c) => c.validate(),`.
12. `set_path`: `ModelConfig::Hybrid(c) => set_path(c, path, value).map(ModelConfig::Hybrid),`.
13. The `None` arm listing `| ModelConfig::Misaligned(_)`: add `| ModelConfig::Hybrid(_)`.
14. `series_names`: `ModelConfig::Hybrid(_) => crate::hybrid::series_names(),`.
15. `pub enum ModelWorld`: `Hybrid(Box<HybridWorld>),`.
16. `ModelWorld::new`: `ModelConfig::Hybrid(c) => ModelWorld::Hybrid(Box::new(HybridWorld::new(c, seed)?)),`.
17. `kind()`, `model()`, `model_mut()`, the `copy_without_history!(Hybrid, w)` arm and the `(ModelWorld::Hybrid(live), ModelWorld::Hybrid(kept)) => { restore_into!(live, kept) }` arm, each beside the misaligned one; and `"hybrid",` after `"misaligned",` in the kind list of the test near line 1995.

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: 60 passed (47 from Tasks 1–6, then 5 config, 1 stats and 7 world tests). Then `cargo test -p sugarscape-core 2>&1 | tail -3` for the whole crate. Expected: green (no presets yet, so the title and golden tests are unaffected; the `model.rs` kind-list test passes with `"hybrid"`).

- [ ] **Step 8: Commit**

```bash
git add crates/sugarscape-core/src/hybrid crates/sugarscape-core/src/model.rs
git commit -m "hybrid: configuration, series, the runner and the world

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 8: Frame, inspection, presets, titles, goldens, WASM and the identity receipts

**Files:**
- Replace: `crates/sugarscape-core/src/hybrid/view.rs`
- Create: `crates/sugarscape-core/src/hybrid/presets.rs`
- Modify: `crates/sugarscape-core/src/hybrid/mod.rs` (`mod presets; pub use presets::presets;`), `crates/sugarscape-core/src/presets.rs` (catalog), `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-wasm/tests/web.rs`, `web/src/episodes/test-fixtures/foraging-source-compatibility.json`

**Interfaces:**
- Consumes: Task 7's `HybridWorld`, `Body`, `payoff_now`, `reference`.
- Produces: `view::{layout(&HybridWorld) -> (usize, usize), size, position(&HybridWorld, i) -> (u32, u32), render, inspect}` with color mode `"strategy"` (also `""`); inspection JSON with `panel` ∈ {`"node"`, `"empty"`}, `world`, `tick`, `x`, `y`, `reference` (null when NaN), `agent: null`, and per panel the keys of Step 3 (a population node adds `replicator`, Booker's largest stable replicator rest point, and `phase`, Guo's replicator phase: the reference panel's second line); `presets() -> Vec<ModelPreset>` with the eight ids below.

Layout: on a lattice node i sits at (i mod side, i div side); otherwise nodes fill, in index order, the smallest square-ish frame `wide = ⌈√N⌉`, `tall = ⌈N / wide⌉`. Colors: humans from defecting red `[230, 70, 70]` to cooperating blue `[70, 130, 230]` by strategy (pure strategies hit the ends); bots acting C green `[90, 200, 120]`, acting D orange `[245, 165, 50]`, Booker's discriminatory bots purple `[160, 100, 220]`; coloring: the game's colors `[80, 180, 80]`, `[240, 150, 40]`, `[150, 90, 200]`, bots at 3/5 brightness.

- [ ] **Step 1: Write the failing tests**

Replace `view.rs`'s stub with only this doc comment and tests for now (keep the stub's functions until Step 3 replaces them):

```rust
//! Frame and inspection: one cell per node, on a lattice in place and otherwise in index order on
//! the smallest square-ish frame that holds them. Humans run from defecting red to cooperating
//! blue by strategy; bots have their own shades (green acting C, orange acting D, purple for
//! Booker's discriminatory bots); coloring shows the game's three colors, bots darker.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hybrid::config::*, model::Model};

    #[test]
    fn frames_have_the_layout_size_and_draw_every_node() {
        let w = HybridWorld::new(booker_defaults(), 1).unwrap();
        assert_eq!(w.size(), (50, 50));
        let mut buf = vec![];
        for mode in ["", "strategy"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 50 * 50 * 4, "{mode}");
        }
        assert!(w.render("payoff", "", &mut buf).is_err());
        w.render("strategy", "", &mut buf).unwrap();
        let Body::Population(p) = &w.runner().body else { unreachable!() };
        for i in 0..2500 {
            let want = if p.bot[i] { BOT_C } else if p.action[i] { HUMAN_C } else { HUMAN_D };
            assert_eq!(&buf[i * 4..i * 4 + 3], &want, "node {i}");
        }
        let c = HybridWorld::new(coloring_defaults(), 1).unwrap();
        assert_eq!(c.size(), (5, 4), "20 nodes, 5 wide");
        let g = HybridWorld::new(HybridConfig { structure: Structure::BarabasiAlbert, nodes: 10, bot_count: 2, ..guo_defaults() }, 1).unwrap();
        assert_eq!((g.size(), position(&g, 9)), ((4, 3), (1, 2)));
        assert_eq!(g.locate(9), Some((1, 2)));
        assert_eq!(g.locate(10), None);
    }

    #[test]
    fn continuous_strategies_blend_and_coloring_bots_are_darker() {
        let si = HybridWorld::new(HybridConfig { side: 10, framework: Framework::Continuous, ..si_defaults() }, 1).unwrap();
        let mut buf = vec![];
        si.render("", "", &mut buf).unwrap();
        let Body::Population(p) = &si.runner().body else { unreachable!() };
        let i = (0..100).find(|&i| !p.bot[i]).unwrap();
        assert_eq!(&buf[i * 4..i * 4 + 3], &blend(p.strategy[i]));
        let c = HybridWorld::new(coloring_defaults(), 1).unwrap();
        c.render("", "", &mut buf).unwrap();
        let Body::Coloring(s) = &c.runner().body else { unreachable!() };
        for i in 0..20 {
            let base = GAME[s.color[i] as usize];
            assert_eq!(&buf[i * 4..i * 4 + 3], &if s.bot[i] { darker(base) } else { base });
        }
    }

    #[test]
    fn inspection_reads_nodes_padding_and_refuses_outside() {
        let g = HybridWorld::new(HybridConfig { structure: Structure::BarabasiAlbert, nodes: 10, bot_count: 2, placement: Placement::Hubs, ..guo_defaults() }, 1).unwrap();
        let node: serde_json::Value = serde_json::from_str(&g.inspect_json(1, 2).unwrap()).unwrap();
        assert_eq!((node["panel"].as_str(), node["node"].as_u64()), (Some("node"), Some(9)));
        assert!(node["payoff"].is_number() && node["degree"].as_u64().unwrap() >= 2 && node["bot"].is_boolean());
        assert!(node["reference"].is_number() && node["agent"].is_null());
        assert!(node["phase"].is_string() && node["replicator"].is_null(), "Guo: the replicator phase");
        let b = HybridWorld::new(HybridConfig { side: 10, ..booker_defaults() }, 1).unwrap();
        let bv: serde_json::Value = serde_json::from_str(&b.inspect_json(0, 0).unwrap()).unwrap();
        assert!(bv["replicator"].is_number() && bv["phase"].is_null(), "Booker: the replicator rest point");
        let pad: serde_json::Value = serde_json::from_str(&g.inspect_json(3, 2).unwrap()).unwrap();
        assert_eq!(pad["panel"], "empty");
        assert!(g.inspect_json(4, 0).is_err() && g.inspect_json(0, 3).is_err());
        let si = HybridWorld::new(HybridConfig { side: 10, ..si_defaults() }, 1).unwrap();
        let v: serde_json::Value = serde_json::from_str(&si.inspect_json(0, 0).unwrap()).unwrap();
        assert!(v["reference"].is_null(), "no reference in Si's world: null, not NaN");
        let c = HybridWorld::new(coloring_defaults(), 1).unwrap();
        let v: serde_json::Value = serde_json::from_str(&c.inspect_json(0, 0).unwrap()).unwrap();
        assert!(v["color"].as_u64().unwrap() < 3 && v["solutions"].as_f64().unwrap() >= 6.0);
        assert!(v["conflicts_here"].is_number() && v["time"].is_number());
    }
}
```

Create `crates/sugarscape-core/src/hybrid/presets.rs` with only its doc comment and tests:

```rust
//! Eight source-labeled presets, two per world; the last task appends the survey's measured
//! results to their descriptions.

#[cfg(test)]
mod tests {
    use super::*;

    const IDS: [&str; 8] = [
        "hybrid-booker-samaritan",
        "hybrid-booker-discriminatory",
        "hybrid-guo-snowdrift-lattice",
        "hybrid-guo-snowdrift-scale-free",
        "hybrid-guo-hub-bots",
        "hybrid-si-mixed-lattice",
        "hybrid-si-discrete-lattice",
        "hybrid-coloring-central",
    ];

    #[test]
    fn eight_presets_cover_the_four_worlds() {
        let all = presets();
        assert_eq!(all.iter().map(|p| p.id).collect::<Vec<_>>(), IDS);
        let mut worlds = vec![];
        for p in &all {
            let ModelConfig::Hybrid(c) = &p.config else { panic!("{}", p.id) };
            assert!(c.validate().is_ok(), "{}", p.id);
            assert!(p.name.len() > 10 && !p.name.contains("Flump") && !p.description.contains("Flump"), "{}", p.id);
            worlds.push(c.world);
        }
        for w in [World::Booker, World::Guo, World::Si, World::Coloring] {
            assert!(worlds.contains(&w), "{w:?}");
        }
        let ModelConfig::Hybrid(hubs) = &all[4].config else { unreachable!() };
        assert_eq!((hubs.bot_total(), hubs.placement, hubs.phi, hubs.structure), (Some(10), Placement::Hubs, 1.0, Structure::BarabasiAlbert));
    }
}
```

Add `mod presets;` and `pub use presets::presets;` to `mod.rs`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: compile errors (`presets`, `BOT_C`, `blend`, `darker`, `GAME`, `position` with the stub's shape) — the view tests and the presets test cannot pass yet.

- [ ] **Step 3: Write `view.rs`**

Replace the stub's functions (keep the doc comment and tests):

```rust
use super::{
    booker::{Donation, Replicator},
    coloring::counts,
    config::{BotKind, Structure, World},
    guo::{Phase, Plane},
    world::{Body, HybridWorld},
};
use crate::opinions::Canvas;
use serde_json::json;

pub const HUMAN_C: [u8; 3] = [70, 130, 230];
pub const HUMAN_D: [u8; 3] = [230, 70, 70];
pub const BOT_C: [u8; 3] = [90, 200, 120];
pub const BOT_D: [u8; 3] = [245, 165, 50];
pub const BOT_MIRROR: [u8; 3] = [160, 100, 220];
pub const GAME: [[u8; 3]; 3] = [[80, 180, 80], [240, 150, 40], [150, 90, 200]];

/// The frame's width and height in cells.
pub fn layout(w: &HybridWorld) -> (usize, usize) {
    if w.config.structure == Structure::Lattice {
        let s = w.config.side as usize;
        return (s, s);
    }
    let n = w.config.size();
    let wide = (n as f64).sqrt().ceil() as usize;
    (wide, n.div_ceil(wide))
}

pub fn size(w: &HybridWorld) -> (u32, u32) {
    let (wide, tall) = layout(w);
    (wide as u32, tall as u32)
}

/// Where node i is drawn.
pub fn position(w: &HybridWorld, i: usize) -> (u32, u32) {
    let (wide, _) = layout(w);
    ((i % wide) as u32, (i / wide) as u32)
}

/// A human's color: red at strategy 0, blue at 1, the blend between.
pub fn blend(s: f64) -> [u8; 3] {
    let mix = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * s).round() as u8;
    [mix(HUMAN_D[0], HUMAN_C[0]), mix(HUMAN_D[1], HUMAN_C[1]), mix(HUMAN_D[2], HUMAN_C[2])]
}

pub fn darker(c: [u8; 3]) -> [u8; 3] {
    c.map(|v| (u16::from(v) * 3 / 5) as u8)
}

pub fn render(w: &HybridWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "strategy"].contains(&mode) {
        return Err(format!("unknown color mode {mode:?} (expected strategy)"));
    }
    let (wide, tall) = layout(w);
    let mut canvas = Canvas { buf, wide: 0 };
    canvas.clear(wide, tall);
    match &w.runner().body {
        Body::Population(p) => {
            let mirror = w.config.world == World::Booker && w.config.bot_kind == BotKind::Discriminatory;
            for i in 0..p.nodes {
                let color = match (p.bot[i], mirror, p.action[i]) {
                    (true, true, _) => BOT_MIRROR,
                    (true, false, true) => BOT_C,
                    (true, false, false) => BOT_D,
                    (false, ..) => blend(p.strategy[i]),
                };
                canvas.put(i % wide, i / wide, color);
            }
        }
        Body::Coloring(s) => {
            for i in 0..s.color.len() {
                let c = GAME[s.color[i] as usize];
                canvas.put(i % wide, i / wide, if s.bot[i] { darker(c) } else { c });
            }
        }
    }
    Ok(())
}

/// The reference panel's second line: Booker's largest stable replicator rest point (Eq. 18 or
/// 20) at α = bots/N, and Guo's replicator phase (Theorems 1–3) at y = bots/humans.
fn reference_detail(w: &HybridWorld) -> (Option<f64>, Option<Phase>) {
    let c = &w.config;
    let Body::Population(p) = &w.runner().body else {
        return (None, None);
    };
    let bots = (p.nodes - p.humans.len()) as f64;
    match c.world {
        World::Booker => {
            let r = Replicator { game: Donation { b: c.b, c: c.c }, alpha: bots / p.nodes as f64, beta_h: c.beta, beta_ai: c.beta, kind: c.bot_kind };
            (r.top_stable(), None)
        }
        World::Guo if !p.humans.is_empty() => (None, Some(Plane { t: c.t, s: c.s }.phase_closed(bots / p.humans.len() as f64, c.phi))),
        _ => (None, None),
    }
}

pub fn inspect(w: &HybridWorld, x: u32, y: u32) -> Result<String, String> {
    let (wide, tall) = layout(w);
    if x as usize >= wide || y as usize >= tall {
        return Err(format!("({x}, {y}) is not in the frame"));
    }
    let i = y as usize * wide + x as usize;
    let mut v = match &w.runner().body {
        _ if i >= w.config.size() => json!({ "panel": "empty" }),
        Body::Population(p) => {
            let (replicator, phase) = reference_detail(w);
            json!({
                "panel": "node", "node": i, "bot": p.bot[i], "strategy": p.strategy[i],
                "action": if p.action[i] { "C" } else { "D" }, "degree": p.degree(i),
                "payoff": w.payoff_now(i), "cooperation": p.human_cooperation(),
                "replicator": replicator, "phase": phase,
            })
        }
        Body::Coloring(s) => json!({
            "panel": "node", "node": i, "bot": s.bot[i], "color": s.color[i], "degree": s.adj[i].len(),
            "conflicts_here": counts(&s.adj, &s.color, i)[s.color[i] as usize],
            "solutions": w.reference(), "conflicts": s.conflicts, "time": s.time, "solved_at": s.solved_at,
        }),
    };
    let map = v.as_object_mut().expect("an object");
    // serde_json writes a NaN as null.
    for (k, val) in [
        ("world", json!(w.config.world)),
        ("tick", json!(w.tick)),
        ("x", json!(x)),
        ("y", json!(y)),
        ("reference", json!(w.reference())),
        ("agent", serde_json::Value::Null),
    ] {
        map.insert(k.into(), val);
    }
    Ok(v.to_string())
}
```

- [ ] **Step 4: Write `presets.rs`**

Above its tests:

```rust
use super::config::*;
use crate::{model::ModelConfig, presets::ModelPreset};

const BOOKER: &str = "Booker et al. 2023, Discriminatory or Samaritan — which AI is needed for humanity? (arXiv 2306.17747v2)";
const GUO: &str = "Guo et al. 2023, Facilitating cooperation in human-agent hybrid populations through autonomous agents, iScience (arXiv 2307.01123)";
const SI: &str = "Si, He, Shen & Tanimoto 2025, Cooperative bots exhibit nuanced effects on cooperation across strategic frameworks, J. R. Soc. Interface 22:20240427";
const SHIRADO: &str = "Shirado & Christakis 2017, Locally noisy autonomous agents improve global human coordination in network experiments, Nature 545:370";

fn scale_free(c: HybridConfig) -> HybridConfig {
    HybridConfig { structure: Structure::BarabasiAlbert, nodes: 10_000, links: 2, ..c }
}

pub fn presets() -> Vec<ModelPreset> {
    let p = |id, name, source, description, config| ModelPreset { id, name, source, description, config: ModelConfig::Hybrid(config) };
    vec![
        p("hybrid-booker-samaritan", "Samaritan bots among noisy imitators on a lattice", BOOKER,
          "Booker et al.'s lattice (Figs. 4–6): 2,500 players on an open 50 × 50 grid, a quarter of them Samaritan bots that always cooperate, the rest humans who copy a random neighbor's action by the Fermi rule at β = 0.1, in the donation game with b = 2, c = 1. One tick is one update, the paper's step; the paper runs 100,000.",
          HybridConfig { bot_fraction: 0.25, ..booker_defaults() }),
        p("hybrid-booker-discriminatory", "Bots that mirror their partner, among sharp imitators", BOOKER,
          "Discriminatory bots (perfect intention recognition: each cooperates with a cooperator and defects against a defector) are never copied; they only make cooperating pay. The paper studies them in the well-mixed chain (Fig. 1), drawn here as the reference; this lattice run, with a quarter of the players such bots at β = 2.5, is ours.",
          HybridConfig { bot_fraction: 0.25, bot_kind: BotKind::Discriminatory, beta: 2.5, ..booker_defaults() }),
        p("hybrid-guo-snowdrift-lattice", "Coin-flip bots in a snowdrift game on a lattice", GUO,
          "Guo et al.'s Fig. 6 lattice: 10,000 players on a periodic 100 × 100 grid, 4,871 of them bots (y = 0.95) that cooperate with probability φ = 0.5, redrawn whenever one is chosen (Algorithm II); humans copy a random neighbor at K = 10 in the snowdrift game T = 1.4, S = 0.5. One tick is 10,000 updates.",
          guo_defaults()),
        p("hybrid-guo-snowdrift-scale-free", "The same bots and game on a scale-free network", GUO,
          "The lattice preset on a Barabási–Albert network of 10,000 nodes with two links per new node (mean degree 4). Guo et al. report that cooperation falls with φ on the lattice and rises on random and scale-free graphs, and attribute the reversal to hubs.",
          scale_free(guo_defaults())),
        p("hybrid-guo-hub-bots", "Ten cooperative bots on the hubs of a scale-free network", GUO,
          "Guo et al.'s Fig. 7: only ten bots, all cooperating (φ = 1), on the ten highest-degree nodes of a 10,000-node Barabási–Albert network, in the Prisoner's Dilemma T = 1.3, S = −0.2. Set the placement to random to compare ten random bots.",
          HybridConfig { bot_count: 10, placement: Placement::Hubs, phi: 1.0, t: 1.3, s: -0.2, ..scale_free(guo_defaults()) }),
        p("hybrid-si-mixed-lattice", "Bots that defect one time in ten, among imitators on a lattice", SI,
          "Si et al.'s Fig. 3C as the published code runs it: a periodic 100 × 100 Moore lattice, each node a bot with probability 0.5; bots cooperate with probability θ = 0.9, drawing an action for every game; humans hold a cooperation probability and copy a random neighbor's at β = 10 (κ = 0.1); r = 0.1.",
          HybridConfig { bot_fraction: 0.5, ..si_defaults() }),
        p("hybrid-si-discrete-lattice", "Fixed cooperators and defectors among imitators on a lattice", SI,
          "Si et al.'s Fig. 3A as the code runs it: the same lattice and bots, but each bot is drawn once as a cooperator (probability θ = 0.9) or a defector, and humans play pure C or D.",
          HybridConfig { bot_fraction: 0.5, framework: Framework::Discrete, ..si_defaults() }),
        p("hybrid-coloring-central", "Three slightly noisy bots at the center of a coloring game", SHIRADO,
          "Shirado and Christakis's game: 20 players on a Barabási–Albert graph must reach a proper 3-coloring within 300 seconds, each seeing only its neighbors. The three highest-degree nodes are bots that recolor every 1.5 s, choosing at random 10% of the time and otherwise taking the least common color around them. The 17 humans are our reconstruction: greedy, with an errant-move rate of 0.041 (the paper's measured human mean) and decision times averaging 1.59 s. One tick is one second.",
          coloring_defaults()),
    ]
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core hybrid:: 2>&1 | tail -5`. Expected: 64 passed.

- [ ] **Step 6: Titles, catalog and goldens**

- `crates/sugarscape-core/src/presets.rs`, in `catalog()`: `out.extend(crate::hybrid::presets());` right after the misaligned line (1603).
- `crates/sugarscape-core/src/titles.rs`: `TITLES` length `458` → `466`, and after `misaligned-logarithmic-floor`'s line the eight `(id, title)` pairs with exactly the preset names above.
- `crates/sugarscape-core/tests/golden.rs`, `MODEL_GOLDEN`: run `cargo test -p sugarscape-core --test golden print_golden -- --ignored --nocapture 2>&1 | grep hybrid-` and paste the eight lines after the misaligned block under the comment `// Hybrid human–machine populations: 200 ticks, seed 1.`

Run: `cargo test -p sugarscape-core 2>&1 | tail -3`. Expected: the whole crate green.

- [ ] **Step 7: WASM tests**

In `crates/sugarscape-wasm/tests/web.rs`:

- in `presets_list_every_model_with_sugarscape_configs_untagged`, after the misaligned assertion (line 862): `assert_eq!(model("hybrid-booker-samaritan").as_deref(), Some("hybrid"));`
- append, with the eight fingerprints from `golden.rs` as zero-padded 16-digit hex strings:

```rust
#[wasm_bindgen_test]
fn hybrid_presets_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: f64 sums, the portable exponential and
    // logarithm, and the shared RNG only, with NaNs canonicalized before hashing, so WASM repeats
    // the native fingerprints.
    for (id, expected) in [
        ("hybrid-booker-samaritan", "0x…"),
        ("hybrid-booker-discriminatory", "0x…"),
        ("hybrid-guo-snowdrift-lattice", "0x…"),
        ("hybrid-guo-snowdrift-scale-free", "0x…"),
        ("hybrid-guo-hub-bots", "0x…"),
        ("hybrid-si-mixed-lattice", "0x…"),
        ("hybrid-si-discrete-lattice", "0x…"),
        ("hybrid-coloring-central", "0x…"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        sim.step(200);
        assert_eq!(sim.fingerprint(), expected, "{id}");
        assert_ne!(sim.render("", "sugar").unwrap(), 0);
    }
    let sim = Sim::new(&preset_json("hybrid-coloring-central"), 1, JsValue::NULL).unwrap();
    let view: serde_json::Value = serde_json::from_str(&sim.inspect(0, 0).unwrap()).unwrap();
    assert_eq!(view["panel"], "node");
}
```

Each `"0x…"` is replaced by the value Step 6 printed; none stays as written. Run: `wasm-pack test --node crates/sugarscape-wasm 2>&1 | tail -5`. Expected: all pass. A native–WASM difference means a non-portable float path: look for an `f64::exp`/`ln`/`powf`, or a NaN reaching the hash without `bits`.

- [ ] **Step 8: The identity receipts**

Write `$SCRATCH/m42/receipts.py` and run it with `python3 -I $SCRATCH/m42/receipts.py`:

```python
"""Set reviewed_sha256 of the model/presets/sweep/titles receipts to the live files' sha256."""
import hashlib, pathlib, re
root = pathlib.Path("/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/misaligned")
fixture = root / "web/src/episodes/test-fixtures/foraging-source-compatibility.json"
text = fixture.read_text(encoding="utf-8")
for name in ["model", "presets", "sweep", "titles"]:
    live = hashlib.sha256((root / f"crates/sugarscape-core/src/{name}.rs").read_bytes()).hexdigest()
    pattern = (r'("path": "crates/sugarscape-core/src/%s\.rs",\s*"fixture": "[^"]*",\s*'
               r'"historical_sha256": "[0-9a-f]{64}",\s*"reviewed_sha256": ")[0-9a-f]{64}"') % name
    text, n = re.subn(pattern, lambda m: m.group(1) + live + '"', text)
    assert n == 1, (name, n)
    print(name, live)
fixture.write_text(text, encoding="utf-8")
```

Run it after the last edit and the last `cargo fmt` of each pinned file in this branch (Task 9 edits `sweep.rs` and Task 13 the titles and preset descriptions: run it again then). Run: `cd web && npx vitest run src/episodes/caching-parity.test.ts src/episodes/spatial-parity.test.ts 2>&1 | tail -5`. Expected: pass.

- [ ] **Step 9: Commit**

```bash
git add crates web/src/episodes
git commit -m "hybrid: frame, inspection, presets, titles, goldens and the identity receipts

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 9: Built-in sweeps and the CI smoke test

**Files:**
- Create: `sweeps/hybrid-booker-lattice-alpha.json`, `sweeps/hybrid-guo-phi-networks.json`, `sweeps/hybrid-si-lattice-r.json`
- Modify: `crates/sugarscape-core/src/sweep.rs` (`BUILTINS` 236 → 239; the id list in its tests near line 3079)
- Modify: `crates/sugarscape-wasm/tests/web.rs` (`builtins_and_series_names_are_listed`: three ids after `misaligned-floor`)
- Modify: `survey/war1_study/fixtures/core-literal-inputs-v1.txt` (three lines), `survey/war1_study/test_registration.py` (four counts 250 → 253, lines 72, 93, 135, 165), `survey/tests/war1_study_cli.rs` (line 207: 250 → 253)
- Create: `crates/sugarscape-core/tests/hybrid_smoke.rs`
- Modify: `web/src/episodes/test-fixtures/foraging-source-compatibility.json` (the `sweep.rs` receipt)

**Interfaces:**
- Consumes: Task 7's config paths and series; Task 8's presets.
- Produces: the three builtin ids above.

- [ ] **Step 1: Write the failing smoke test**

`crates/sugarscape-core/tests/hybrid_smoke.rs`:

```rust
//! Every hybrid preset runs 200 ticks at seed 1 with sane series: CI's smoke test for
//! milestone 42 (it scores nothing; the survey scores).
use sugarscape_core::{
    model::{ModelConfig, ModelKind, ModelWorld},
    presets, sweep,
};

#[test]
fn every_preset_runs_two_hundred_ticks_with_sane_series() {
    let mut seen = 0;
    for p in presets::catalog() {
        let ModelConfig::Hybrid(c) = &p.config else { continue };
        seen += 1;
        let mut w = ModelWorld::new(p.config.clone(), 1).unwrap();
        w.model_mut().run(200);
        let m = w.model();
        assert_eq!(w.kind(), ModelKind::Hybrid);
        let share = m.latest_value("bot_share").unwrap();
        assert!((0.0..1.0).contains(&share), "{}: bot share {share}", p.id);
        if c.world == sugarscape_core::hybrid::World::Coloring {
            let t = m.latest_value("game_time").unwrap();
            assert!((0.0..=300.0).contains(&t) && m.latest_value("conflicts").unwrap().is_finite(), "{}", p.id);
        } else {
            let coop = m.latest_value("cooperation").unwrap();
            assert!((0.0..=1.0).contains(&coop), "{}: cooperation {coop}", p.id);
            assert_eq!(m.tick(), 200);
        }
    }
    assert_eq!(seen, 8);
}

#[test]
fn the_builtin_sweeps_parse_and_name_real_paths() {
    for id in ["hybrid-booker-lattice-alpha", "hybrid-guo-phi-networks", "hybrid-si-lattice-r"] {
        let s = sweep::builtin(id).unwrap_or_else(|| panic!("no builtin {id}"));
        assert!(!s.points().unwrap().is_empty(), "{id}");
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p sugarscape-core --test hybrid_smoke 2>&1 | tail -5`. Expected: the sweeps test fails on the first missing id; the preset test passes.

- [ ] **Step 3: Write the sweeps**

`sweeps/hybrid-booker-lattice-alpha.json`:

```json
{
  "name": "Hybrid populations: Samaritan bots on Booker's lattice",
  "description": "Booker et al. (arXiv 2306.17747), Figs. 4–6: human cooperation on the open 50 × 50 lattice, averaged over the last 1,000 of 100,000 single updates, against the share of Samaritan bots, one line per selection strength β (donation game b = 2, c = 1). Five seeds here; the survey runs the paper's thirty.",
  "base": { "config": { "model": "hybrid", "world": "booker" } },
  "x": { "label": "Bot fraction α", "path": "bot_fraction", "values": [0, 0.01, 0.1, 0.25, 0.5, 0.75] },
  "series": {
    "label": "Selection strength",
    "values": [
      { "at": 0, "name": "β = 0.1", "set": { "beta": 0.1 } },
      { "at": 1, "name": "β = 1", "set": { "beta": 1 } },
      { "at": 2, "name": "β = 5", "set": { "beta": 5 } }
    ]
  },
  "seeds": { "from": 1, "count": 5 },
  "ticks": 100000,
  "metric": { "kind": "window_mean", "series": "cooperation", "from": 99000 }
}
```

`sweeps/hybrid-guo-phi-networks.json`:

```json
{
  "name": "Hybrid populations: Guo's bots in snowdrift on three networks",
  "description": "Guo et al. (iScience 2023), Fig. 6B: human cooperation against the bots' cooperation probability φ in the snowdrift game T = 1.4, S = 0.5 with 4,871 random bots among 10,000 players (K = 10), on the periodic lattice, an Erdős–Rényi graph and a Barabási–Albert graph (mean degree 4). Shortened to 5,000 steps averaged over the last 1,000, two seeds; the survey runs the paper's 50,000 steps, the last 5,000, sixty runs.",
  "base": { "config": { "model": "hybrid", "world": "guo", "structure": "lattice", "side": 100, "nodes": 10000, "periodic": true, "bots": "count", "bot_count": 4871, "beta": 10, "t": 1.4, "s": 0.5 } },
  "x": { "label": "Bot cooperation φ", "path": "phi", "values": [0, 0.2, 0.4, 0.6, 0.8, 1] },
  "series": {
    "label": "Network",
    "values": [
      { "at": 0, "name": "Lattice", "set": { "structure": "lattice" } },
      { "at": 1, "name": "Erdős–Rényi", "set": { "structure": "erdos_renyi" } },
      { "at": 2, "name": "Barabási–Albert", "set": { "structure": "barabasi_albert" } }
    ]
  },
  "seeds": { "from": 1, "count": 2 },
  "ticks": 5000,
  "metric": { "kind": "window_mean", "series": "cooperation", "from": 4000 }
}
```

`sweeps/hybrid-si-lattice-r.json`:

```json
{
  "name": "Hybrid populations: Si's bots across three frameworks on the lattice",
  "description": "Si et al. (J. R. Soc. Interface 2025), Fig. 3, as the published code runs it: human cooperation on the periodic 100 × 100 Moore lattice against the dilemma strength r, without bots and with each node a bot with probability 0.5 (θ = 0.9, β = 10), in the discrete, continuous and mixed frameworks. Shortened to 3,000 rounds averaged over the last 1,000, two seeds; the survey runs the code's 10,000 rounds, the last 2,000, a hundred runs.",
  "base": { "config": { "model": "hybrid", "world": "si", "structure": "lattice", "side": 100, "neighborhood": "moore", "periodic": true, "bots": "bernoulli", "beta": 10 } },
  "x": { "label": "Dilemma strength r", "path": "r", "values": [0, 0.02, 0.04, 0.06, 0.08, 0.1, 0.12, 0.14, 0.16, 0.18, 0.2] },
  "series": {
    "label": "Framework and bots",
    "values": [
      { "at": 0, "name": "Discrete, no bots", "set": { "framework": "discrete", "bot_fraction": 0 } },
      { "at": 1, "name": "Discrete, ρ = 0.5", "set": { "framework": "discrete", "bot_fraction": 0.5 } },
      { "at": 2, "name": "Continuous, no bots", "set": { "framework": "continuous", "bot_fraction": 0 } },
      { "at": 3, "name": "Continuous, ρ = 0.5", "set": { "framework": "continuous", "bot_fraction": 0.5 } },
      { "at": 4, "name": "Mixed, no bots", "set": { "framework": "mixed", "bot_fraction": 0 } },
      { "at": 5, "name": "Mixed, ρ = 0.5", "set": { "framework": "mixed", "bot_fraction": 0.5 } }
    ]
  },
  "seeds": { "from": 1, "count": 2 },
  "ticks": 3000,
  "metric": { "kind": "window_mean", "series": "cooperation", "from": 2000 }
}
```

- [ ] **Step 4: Register the builtins and the literal inventory**

- `crates/sugarscape-core/src/sweep.rs`: `const BUILTINS: [Builtin; 236]` → `239`; after the `misaligned-floor` entry (line 1997–2000), before `polarity-predators`, three `Builtin { id: "hybrid-…", json: include_str!("../../../sweeps/hybrid-….json") }` entries in the order booker, guo, si; the same three ids after `"misaligned-floor",` in the test list near line 3079, and in `crates/sugarscape-wasm/tests/web.rs`'s `builtins_and_series_names_are_listed` after `"misaligned-floor",` (line 492).
- `survey/war1_study/fixtures/core-literal-inputs-v1.txt`: insert `sweeps/hybrid-booker-lattice-alpha.json`, `sweeps/hybrid-guo-phi-networks.json`, `sweeps/hybrid-si-lattice-r.json` after `sweeps/hoard-recovery.json` (line 174). Check with `LC_ALL=C sort -c survey/war1_study/fixtures/core-literal-inputs-v1.txt` (no output means sorted).
- `survey/war1_study/test_registration.py`: the four `250` → `253`; `survey/tests/war1_study_cli.rs` line 207: `==250` → `==253`.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p sugarscape-core --test hybrid_smoke 2>&1 | tail -3`, `cargo test -p sugarscape-core sweep 2>&1 | tail -3`, `python3 -m unittest discover -s survey/war1_study -p 'test_*.py' 2>&1 | tail -3`, and `cargo test --manifest-path survey/Cargo.toml --locked --features war-studies --bin war1-study --test war1_study_cli 2>&1 | tail -3`. Expected: all pass. Then run one sweep at a small size to see it end to end: `cargo run --release -p sugarscape-cli -- sweep --help` for the flags, then `hybrid-booker-lattice-alpha` with one seed and two x values.

- [ ] **Step 6: Refresh the `sweep.rs` receipt and commit**

Run `cargo fmt --all`, then `python3 -I $SCRATCH/m42/receipts.py`, then:

```bash
git add sweeps crates/sugarscape-core/src/sweep.rs crates/sugarscape-core/tests/hybrid_smoke.rs crates/sugarscape-wasm/tests/web.rs survey/war1_study survey/tests/war1_study_cli.rs web/src/episodes/test-fixtures/foraging-source-compatibility.json
git commit -m "hybrid: built-in sweeps and the CI smoke test

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 10: The page

**Files:**
- Modify: `web/src/types.ts` (`ModelKind`, `ModelConfig`, `ModelStats`, `AnyInspection`; new interfaces after the misaligned block at line 2472)
- Modify: `web/src/models.ts` (`MODELS`, `MODEL_LABELS`, `modelOf`, `isHybridView`, `COLOR_MODES`, `MODEL_OVERLAYS`, `ticksLeft`, `finishesUnpredictably`)
- Modify: `web/src/schema-form.ts` (`schemaVariant`), `web/src/ui/schema-panel.ts` (`hybridParams`)
- Modify: `web/src/ui/series-data.ts` (`MODEL_CHARTS.hybrid`), `web/src/ui/grid-view.ts`, `web/src/ui/inspect-panel.ts`, `web/src/experiments/form.ts`, `web/src/compare-presets.ts`, `web/src/determinism.test.ts`
- Create: `web/src/hybrid.ts`, `web/src/hybrid.test.ts`
- Modify (tests): `web/src/models.test.ts`, `web/src/schema-form.test.ts`, `web/src/experiments/form.test.ts`

**Interfaces:**
- Consumes: Task 8's inspection JSON and Task 7's schema groups (`World`, `Structure`, `Bots`, `Imitation`, `Booker`, `Guo`, `Si`, `Coloring`).
- Produces: `hybridParams(params: Param[], config: HybridConfig): Param[]`, `hybridRows(view: HybridInspection): [string, string][]`, `hybridLegend(config: HybridConfig, mode: ColorMode): { status: string; items: LegendItem[] }`, `isHybridView(v: AnyInspection): v is HybridInspection`.

- [ ] **Step 1: Write the failing tests**

`web/src/hybrid.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { hybridLegend, hybridParams, hybridRows } from './hybrid';
import type { HybridConfig, HybridInspection, Param } from './types';

const param = (path: string, group: string): Param => ({ path, label: path, kind: 'number', apply: 'live', group });
const PARAMS = [
  param('world', 'World'), param('structure', 'Structure'), param('bot_count', 'Bots'), param('beta', 'Imitation'),
  param('b', 'Booker'), param('t', 'Guo'), param('r', 'Si'), param('noise', 'Coloring'),
];
const config = (world: HybridConfig['world']): HybridConfig => ({ model: 'hybrid', world, framework: 'mixed', bot_kind: 'samaritan', structure: 'lattice', beta: 10 } as HybridConfig);

describe('hybrid page helpers', () => {
  it('shows the shared groups and only the chosen world’s own', () => {
    expect(hybridParams(PARAMS, config('booker')).map((p) => p.path)).toEqual(['world', 'structure', 'bot_count', 'beta', 'b']);
    expect(hybridParams(PARAMS, config('si')).map((p) => p.path)).toEqual(['world', 'structure', 'bot_count', 'beta', 'r']);
    expect(hybridParams(PARAMS, config('coloring')).map((p) => p.path)).toEqual(['world', 'structure', 'bot_count', 'noise']);
  });

  it('rows describe a node, a coloring node and an empty cell', () => {
    const node: HybridInspection = { panel: 'node', world: 'guo', tick: 3, x: 1, y: 2, reference: 0.4123, agent: null, node: 9, bot: false, strategy: 1, action: 'C', degree: 4, payoff: 2.5, cooperation: 0.61, replicator: null, phase: 'coexistence' };
    const rows = hybridRows(node);
    expect(rows[0]).toEqual(['Player', 'node 9 · human · plays C']);
    expect(rows.find(([k]) => k === 'Payoff')?.[1]).toBe('2.500 over 4 games');
    expect(rows.find(([k]) => k === 'Reference')?.[1]).toBe('highest stable master-equation root 0.412 · replicator phase: coexistence');
    const booker: HybridInspection = { ...node, world: 'booker', reference: 0.98, replicator: 1, phase: null };
    expect(hybridRows(booker).find(([k]) => k === 'Reference')?.[1]).toBe('all-C weight of the well-mixed chain 0.980 · replicator rest point 1.000');
    const si: HybridInspection = { ...node, world: 'si', reference: null, bot: true, strategy: 0.9, action: 'D', phase: null };
    expect(hybridRows(si)[0]).toEqual(['Player', 'node 9 · bot · plays D']);
    expect(hybridRows(si).find(([k]) => k === 'Strategy')?.[1]).toBe('cooperates with probability 0.900');
    expect(hybridRows(si).find(([k]) => k === 'Reference')?.[1]).toBe('none (no closed form)');
    const color: HybridInspection = { panel: 'node', world: 'coloring', tick: 12, x: 0, y: 0, reference: 48, agent: null, node: 0, bot: true, color: 2, degree: 7, conflicts_here: 1, solutions: 48, conflicts: 3, time: 12.4, solved_at: null };
    const crow = hybridRows(color);
    expect(crow[0]).toEqual(['Player', 'node 0 · bot · purple']);
    expect(crow.find(([k]) => k === 'Conflicts')?.[1]).toBe('1 here · 3 in all');
    expect(crow.find(([k]) => k === 'Clock')?.[1]).toBe('12.4 s, unsolved');
    expect(crow.find(([k]) => k === 'Solutions')?.[1]).toBe('48 proper 3-colorings');
    expect(hybridRows({ panel: 'empty', world: 'guo', tick: 0, x: 3, y: 2, reference: null, agent: null })).toEqual([['Cell', 'no node here']]);
  });

  it('legend names the world and its colors', () => {
    const booker = hybridLegend({ ...config('booker'), bot_kind: 'discriminatory' }, 'strategy');
    expect(booker.status).toContain('Booker et al.');
    expect(booker.items.map((i) => i.label)).toContain('Discriminatory bot');
    expect(hybridLegend(config('si'), 'strategy').status).toContain('as the published code runs it');
    expect(hybridLegend(config('coloring'), 'strategy').items.map((i) => i.label)).toEqual(['Green', 'Orange', 'Purple', 'Bots: darker']);
  });
});
```

Add to `web/src/models.test.ts`, after the misaligned `describe`:

```ts
describe('the hybrid model', () => {
  it('is read by its tag, and its inspections by their panel and world', () => {
    expect(modelOf({ model: 'hybrid' } as unknown as ModelConfig)).toBe('hybrid');
    const node = { panel: 'node', world: 'si', node: 1, tick: 0, x: 1, y: 0, reference: null, agent: null } as unknown as AnyInspection;
    const empty = { panel: 'empty', world: 'guo', tick: 0, x: 3, y: 2, reference: null, agent: null } as unknown as AnyInspection;
    const bar = { panel: 'attribute', index: 2, name: 'c', outcome: { utility_start: 16 }, agent: null } as unknown as AnyInspection;
    expect([node, empty, bar].map(isHybridView)).toEqual([true, true, false]);
    expect(isMisalignedView(node)).toBe(false);
  });

  it('colors by strategy, has no overlays, and only the coloring game ends', () => {
    expect(COLOR_MODES.hybrid.map(([m]) => m)).toEqual(['strategy']);
    expect(MODEL_OVERLAYS.hybrid).toEqual([]);
    const pop = { model: 'hybrid', world: 'si' } as unknown as ModelConfig;
    const game = { model: 'hybrid', world: 'coloring', time_limit: 300 } as unknown as ModelConfig;
    expect([ticksLeft(pop, 10), ticksLeft(game, 10)]).toEqual([Infinity, 290]);
    expect([finishesUnpredictably(pop), finishesUnpredictably(game)]).toEqual([false, true]);
  });
});
```

(and add `isHybridView` to that file's import from `./models`). In `web/src/schema-form.test.ts`'s first test add `expect(schemaVariant('hybrid', { model: 'hybrid', world: 'coloring' } as unknown as ModelConfig)).toBe('coloring');`. In `web/src/experiments/form.test.ts`, after the misaligned expectation, add:

```ts
    expect(defaultForm('hybrid')).toMatchObject({
      x: { path: 'bot_fraction', values: '0,0.01,0.1,0.25,0.5,0.75' },
      seeds: 5,
      ticks: 100000,
      metric: { kind: 'window_mean', series: 'cooperation', from: 99000 },
    });
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cd web && npx vitest run src/hybrid.test.ts src/models.test.ts src/schema-form.test.ts src/experiments/form.test.ts 2>&1 | tail -8`. Expected: failures (no `./hybrid` module, `COLOR_MODES.hybrid` undefined).

- [ ] **Step 3: Types**

In `web/src/types.ts`: append `| 'hybrid'` to `ModelKind`, `| HybridConfig` to `ModelConfig`, `| HybridStats` to `ModelStats`, `| HybridInspection` to `AnyInspection`; after the misaligned interfaces (line 2472) add:

```ts
/** Hybrid human–machine populations (milestone 42); structure, population, framework, θ, the step and the time limit apply at reset. */
export interface HybridConfig {
  model: 'hybrid'; world: 'booker' | 'guo' | 'si' | 'coloring';
  structure: 'well_mixed' | 'lattice' | 'erdos_renyi' | 'barabasi_albert'; side: number; nodes: number; neighborhood: 'von_neumann' | 'moore'; periodic: boolean; mean_degree: number; links: number; graph_seed: number | null;
  bots: 'count' | 'fraction' | 'bernoulli'; bot_count: number; bot_fraction: number; placement: 'random' | 'hubs' | 'leaves';
  beta: number; step: 'paper' | 'sweep'; accumulate: 'sum' | 'mean'; initial_cooperation: number;
  bot_kind: 'samaritan' | 'discriminatory' | 'malicious'; b: number; c: number;
  t: number; s: number; phi: number; bot_action: 'redraw' | 'frozen' | 'per_interaction';
  r: number; theta: number; framework: 'discrete' | 'continuous' | 'mixed'; mixed_draw: 'per_interaction' | 'per_round'; division: 'integer' | 'float';
  noise: number; errant: number; reaction: number; bot_interval: number; time_limit: number;
}
export interface HybridStats {
  tick: number; cooperation: number | null; bot_cooperation: number | null; bot_share: number | null; reference: number | null;
  conflicts: number | null; solved_at: number | null; game_time: number | null;
}
interface HybridCell { world: HybridConfig['world']; tick: number; x: number; y: number; reference: number | null; agent: null }
export interface HybridEmptyInspection extends HybridCell { panel: 'empty' }
export interface HybridNodeInspection extends HybridCell {
  panel: 'node'; node: number; bot: boolean; strategy: number; action: 'C' | 'D'; degree: number; payoff: number | null; cooperation: number | null;
  /** Booker: the largest stable replicator rest point; Guo: the replicator phase. */
  replicator: number | null; phase: 'full_c' | 'full_d' | 'coexistence' | 'bistable' | null;
}
export interface HybridColoringInspection extends HybridCell {
  panel: 'node'; node: number; bot: boolean; color: number; degree: number; conflicts_here: number; solutions: number; conflicts: number; time: number; solved_at: number | null;
}
export type HybridInspection = HybridEmptyInspection | HybridNodeInspection | HybridColoringInspection;
```

- [ ] **Step 4: `models.ts`, the schema variant and the panel**

- `MODELS`: append `'hybrid'`. `MODEL_LABELS`: `hybrid: 'Hybrid Populations',`. `modelOf`: add `|| tag === 'hybrid'` to the chain. Import `HybridConfig`, `HybridInspection` in the type import list.
- After `isMisalignedView`:

```ts
/** A hybrid frame: a node or an empty cell, tagged with its world. */
export function isHybridView(v: AnyInspection): v is HybridInspection {
  return 'panel' in v && (v.panel === 'node' || v.panel === 'empty') && 'world' in v && ['booker', 'guo', 'si', 'coloring'].includes(String((v as { world: unknown }).world));
}
```

- `COLOR_MODES`: `hybrid: [['strategy', 'Strategies and bots']],`. `MODEL_OVERLAYS`: `hybrid: [],`.
- `ticksLeft`, before its last line: `if (modelOf(c) === 'hybrid' && (c as HybridConfig).world === 'coloring') return Math.max(0, Math.ceil((c as HybridConfig).time_limit) - tick);`
- `finishesUnpredictably`, before `return model === 'civil' …`: `if (model === 'hybrid') return (c as HybridConfig).world === 'coloring';`
- `web/src/schema-form.ts`, `schemaVariant`: `return model === 'misaligned' || model === 'hybrid' ? String((config as { world?: unknown }).world ?? '') : '';`, and in its doc comment add "and the hybrid kind, whose panel shows only the chosen world's group (`hybridParams`)".
- `web/src/ui/schema-panel.ts`: import `hybridParams` from `'../hybrid'` and `HybridConfig`; in the `sections` chain (line 73) add `: model === 'hybrid' ? hybridParams(params, this.engine.config as HybridConfig)` before the final `: params`.

- [ ] **Step 5: `hybrid.ts`**

```ts
import type { ColorMode, HybridConfig, HybridInspection, Param } from './types';
import type { LegendItem } from './ui/legend';

const fmt = (v: number | null | undefined, digits = 3): string => (v === null || v === undefined || Number.isNaN(v) ? '—' : v.toFixed(digits).replace('-', '−'));
const WORLD_GROUP: Record<HybridConfig['world'], string> = { booker: 'Booker', guo: 'Guo', si: 'Si', coloring: 'Coloring' };
const WORLD_GROUPS = Object.values(WORLD_GROUP);
const COLOR_NAMES = ['green', 'orange', 'purple'];
const swatch = (label: string, color: string): LegendItem => ({ label, mark: { kind: 'swatch', color } });

/** The Rules panel for one world: the shared groups and the world's own; the coloring game has no imitation rule. */
export function hybridParams(params: Param[], config: HybridConfig): Param[] {
  const own = WORLD_GROUP[config.world];
  return params.filter((p) => (WORLD_GROUPS.includes(p.group) ? p.group === own : !(config.world === 'coloring' && p.group === 'Imitation')));
}

const PHASES = { full_c: 'full cooperation', full_d: 'full defection', coexistence: 'coexistence', bistable: 'bistable' } as const;

const referenceText = (view: HybridInspection): string => {
  const node = view.panel === 'node' && 'phase' in view ? view : null;
  switch (view.world) {
    case 'booker': return `all-C weight of the well-mixed chain ${fmt(view.reference)} · replicator rest point ${fmt(node?.replicator)}`;
    case 'guo': return `highest stable master-equation root ${fmt(view.reference)} · replicator phase: ${node?.phase ? PHASES[node.phase] : '—'}`;
    case 'coloring': return `${fmt(view.reference, 0)} proper 3-colorings`;
    default: return 'none (no closed form)';
  }
};

/** Inspect rows: a population node, a coloring node, or an empty cell. */
export function hybridRows(view: HybridInspection): [string, string][] {
  if (view.panel === 'empty') return [['Cell', 'no node here']];
  if ('color' in view) {
    return [
      ['Player', `node ${view.node} · ${view.bot ? 'bot' : 'human (ours)'} · ${COLOR_NAMES[view.color]}`],
      ['Conflicts', `${view.conflicts_here} here · ${view.conflicts} in all`],
      ['Clock', view.solved_at === null ? `${fmt(view.time, 1)} s, unsolved` : `solved at ${fmt(view.solved_at, 1)} s`],
      ['Solutions', `${view.solutions} proper 3-colorings`],
    ];
  }
  return [
    ['Player', `node ${view.node} · ${view.bot ? 'bot' : 'human'} · plays ${view.action}`],
    ['Strategy', `cooperates with probability ${fmt(view.strategy)}`],
    ['Payoff', `${fmt(view.payoff)} over ${view.degree} games`],
    ['Humans cooperating', fmt(view.cooperation)],
    ['Reference', referenceText(view)],
  ];
}

/** The frame's status line and legend. */
export function hybridLegend(config: HybridConfig, _mode: ColorMode): { status: string; items: LegendItem[] } {
  if (config.world === 'coloring') {
    return {
      status: `Shirado & Christakis's coloring game · ${config.bot_count} bots, noise ${config.noise} · humans are our reconstruction (errant rate ${config.errant}) · one tick is one second, up to ${config.time_limit} s`,
      items: [swatch('Green', '#50b450'), swatch('Orange', '#f09628'), swatch('Purple', '#965ac8'), swatch('Bots: darker', '#303030')],
    };
  }
  const who = { booker: 'Booker et al.', guo: 'Guo et al.', si: 'Si et al., as the published code runs it' }[config.world];
  const game = config.world === 'booker' ? `donation game b ${config.b}, c ${config.c}` : config.world === 'guo' ? `T ${config.t}, S ${config.s}, bots cooperate with φ ${config.phi}` : `r ${config.r}, ${config.framework} framework`;
  const bots = config.world === 'booker' && config.bot_kind === 'discriminatory'
    ? [swatch('Discriminatory bot', '#a064dc')]
    : [swatch('Bot acting C', '#5ac878'), swatch('Bot acting D', '#f5a532')];
  return {
    status: `${who} · ${config.structure.replace('_', ' ')} · ${game} · β ${config.beta} · humans blend from red (defect) to blue (cooperate)`,
    items: [swatch('Human cooperating', '#4682e6'), swatch('Human defecting', '#e64646'), ...bots],
  };
}
```

- [ ] **Step 6: Charts, legend, Inspect rows, the sweep form, Compare, goldens**

- `web/src/ui/series-data.ts`, `MODEL_CHARTS`, after the misaligned entry:

```ts
  hybrid: [
    { title: 'Cooperation', range: [0, 1], shown: (c) => (c as HybridConfig).world !== 'coloring', lines: [{ key: 'cooperation', label: 'Humans', color: '--c1' }, { key: 'bot_cooperation', label: 'Bots acting C', color: '--c2' }, { key: 'reference', label: 'Exact reference (Booker, Guo)', color: '--c3' }] },
    { title: 'Coloring', shown: (c) => (c as HybridConfig).world === 'coloring', lines: [{ key: 'conflicts', label: 'Conflicting edges', color: '--red' }, { key: 'game_time', label: 'Game clock (s)', color: '--c1' }] },
    { title: 'Bots', range: [0, 1], lines: [{ key: 'bot_share', label: 'Share of the nodes', color: '--c1' }] },
  ],
```

  (import `HybridConfig` there.)
- `web/src/ui/grid-view.ts`: import `hybridLegend` from `'../hybrid'` and `HybridConfig`; beside the misaligned line (280) add `const hybrid = this.engine.model === 'hybrid' ? hybridLegend(this.engine.config as HybridConfig, this.engine.colorMode) : null;` and put it first in both chains: `(hybrid ? hybrid.status : misaligned ? misaligned.status : …` and `hybrid ? hybrid.items : misaligned ? misaligned.items : …`.
- `web/src/ui/inspect-panel.ts`: import `hybridRows` from `'../hybrid'` and `isHybridView` from `'../models'`; before the misaligned branch (line 847) add `: isHybridView(view) ? hybridRows(view).map(([k, v]) => h('tr', {}, h('th', {}, k), h('td', {}, v)))`.
- `web/src/experiments/form.ts`, before the misaligned branch (line 125):

```ts
  if (model === 'hybrid') {
    return { ...form, description: "Booker et al.'s lattice: human cooperation over the last 1,000 of 100,000 updates against the share of Samaritan bots. Pick another world on the Rules panel for Guo's, Si's or the coloring game, and change the axis to match.", x: { path: 'bot_fraction', values: '0,0.01,0.1,0.25,0.5,0.75' }, seeds: 5, ticks: 100000, metric: { ...form.metric, kind: 'window_mean', series: 'cooperation', from: 99000, to: null } };
  }
```

- `web/src/compare-presets.ts`, after the misaligned pairs (line 234): `{ id: 'hybrid-lattice-vs-scale-free', label: 'Coin-flip bots in snowdrift on a lattice and on a scale-free network — Hybrid Populations (Compare)', a: 'hybrid-guo-snowdrift-lattice', b: 'hybrid-guo-snowdrift-scale-free' },` and `{ id: 'hybrid-si-mixed-vs-discrete', label: 'Noisy bots against fixed bots on the lattice — Hybrid Populations (Compare)', a: 'hybrid-si-mixed-lattice', b: 'hybrid-si-discrete-lattice' },`.
- `web/src/determinism.test.ts`, `GOLDEN_MODELS`: add `['hybrid-guo-hub-bots', '0x…']` with that preset's fingerprint from `golden.rs` (replace `0x…`).

- [ ] **Step 7: Run the page checks**

Run: `cd web && npm run wasm && npx tsc --noEmit && npx vitest run src/hybrid.test.ts src/models.test.ts src/schema-form.test.ts src/experiments/form.test.ts src/determinism.test.ts src/brand.test.ts 2>&1 | tail -8`. Expected: a clean typecheck and every test passing.

Then open the page (`npm run dev`), choose "Hybrid Populations", and check: each preset draws its frame; the Rules panel shows only the chosen world's group (switch `world` and watch the panel rebuild); Inspect reads a node, a padding cell of the coloring frame (row 3 of 5 × 4 is full; try a 10-node graph for padding) and shows no "NaN"; the Compare pairs run side by side; the coloring preset stops at its solve time or 300 s. Record anything off for the fix pass rather than widening this task.

- [ ] **Step 8: Commit**

```bash
git add web/src
git commit -m "hybrid: the page

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 11: The docking reference from Si et al.'s published code (`survey/hybrid_si/`)

**Files:**
- Create: `survey/hybrid_si/dock.py`, `survey/hybrid_si/test_dock.py`
- Create (outputs, tracked): `survey/out/hybrid/si-reference/{fig1a,fig1b,fig1c,fig3a,fig3b,fig3c}.csv`, `survey/out/hybrid/si-reference/receipts.json`

**Interfaces:**
- Consumes: `papers/alignment/code/si-et-al-2024-jrsi-cooperative-bots-zenodo-13957161/*.cpp` (read only, never compiled in place).
- Produces: one CSV per file with header `r,rho,chunk,seed,run,cooperation` and one row per run (100 per point unless `--runs` is lowered to match a reduced survey row); `receipts.json` with each source's sha256, each patched source's sha256, the patches applied, the compiler's `--version` line, the flags, the sandbox profile, and wall times.

The S1 points (spec): lattice files Fig. 3A–C at r ∈ {0.05, 0.1, 0.15} × ρ ∈ {0, 0.3, 0.5}; well-mixed files Fig. 1A–C at r ∈ {0.05, 0.1, 0.15}, ρ = 0.3. Point p (in the order file, r, ρ) runs as four processes of `runs/4` runs with seeds 4p + 1 … 4p + 4 (amendment 2).

- [ ] **Step 1: Write the failing tests**

`survey/hybrid_si/test_dock.py`:

```python
"""Tests of the docking harness's patcher and parsers; nothing here compiles or runs C++."""
import pathlib, sys, unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import dock  # noqa: E402

WELL_MIXED = """c#include <iostream>
int loop = 100;  // Number of loops
int main(void)
{
    sgenrand(time(0));  // seed
    for(r = 0.00; r <= 0.21 ; r += 0.01){
        for(rho = 0.0; rho <= 0.5; rho += 0.01) {
            double record_fc[loop]={0};
        }
    }
}
"""
LATTICE = """#include <iostream>
#define loop 100 // 100 loops
int main(void) {
    sgenrand(time(0));
    for (r = 0; r < 0.21; r += 0.01) {
        for (rho = 0; rho < 0.51; rho += 0.01) {
        }
    }
}
"""


class Patches(unittest.TestCase):
    def test_well_mixed_source_is_pinned_to_one_point(self):
        out, applied = dock.patch(WELL_MIXED)
        self.assertTrue(out.startswith("#include <iostream>"))
        self.assertIn("const int loop = SI_LOOP;", out)
        self.assertIn("sgenrand(SI_SEED);", out)
        self.assertIn("for (r = SI_R; r <= SI_R; r += 1)", out)
        self.assertIn("for (rho = SI_RHO; rho <= SI_RHO; rho += 1)", out)
        self.assertEqual([a["name"] for a in applied], ["stray characters before #include", "constant run count", "seed", "r loop", "rho loop"])

    def test_lattice_source_keeps_its_define(self):
        out, applied = dock.patch(LATTICE)
        self.assertIn("#define loop SI_LOOP // 100 loops", out)
        self.assertNotIn("stray characters before #include", [a["name"] for a in applied])

    def test_a_missing_pattern_is_an_error(self):
        with self.assertRaises(ValueError):
            dock.patch(LATTICE.replace("sgenrand(time(0));", ""))


class Parsers(unittest.TestCase):
    def test_well_mixed_lines(self):
        out = "r,rho,Kappa,theta,loop,c_fraction\n0.050000,0.300000,10.000000,0.900000,0,0.123400\n0.050000,0.300000,10.000000,0.900000,1,0.234500\n"
        self.assertEqual(dock.parse("well_mixed", "discrete", out), [(0.05, 0.3, 0.1234), (0.05, 0.3, 0.2345)])

    def test_lattice_lines_by_framework(self):
        out = "*****start*****\n0.100000\t0.500000\t0.612300\t0.387700\n"
        self.assertEqual(dock.parse("lattice", "mixed", out), [(0.1, 0.5, 0.6123)])
        cont = "0.100000\t0.500000\t0.900000\t0.555000\t0.445000\n"
        self.assertEqual(dock.parse("lattice", "continuous", cont), [(0.1, 0.5, 0.555)])

    def test_points_and_seeds(self):
        points = dock.points()
        self.assertEqual(len(points), 9 + 27)
        self.assertEqual(points[0], ("fig1a", 0.05, 0.3))
        self.assertEqual(dock.seeds(0), [1, 2, 3, 4])
        self.assertEqual(dock.seeds(35), [141, 142, 143, 144])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run them to verify they fail**

Run: `python3 -I -m unittest discover -s survey/hybrid_si -p 'test_*.py' 2>&1 | tail -3`. Expected: `ModuleNotFoundError: No module named 'dock'` (an import error is the failure here).

- [ ] **Step 3: Write `dock.py`**

```python
#!/usr/bin/env python3
"""The docking reference for milestone 42 (spec S1, amendment 2): copy Si et al.'s published C++
into a scratch directory, patch it minimally, compile it, run every S1 point under sandbox-exec
with the network denied, and keep every run's value.

    python3 -I survey/hybrid_si/dock.py --source papers/alignment/code/si-et-al-2024-jrsi-cooperative-bots-zenodo-13957161 \
        --scratch $SCRATCH/m42/si-dock --out survey/out/hybrid/si-reference [--runs 100] [--jobs 10] [--smoke]

The patches, each required exactly once where it applies: strip stray characters before the first
#include (three files ship with them), make the run count a constant (clang rejects the original
variable-length array initializer) settable as SI_LOOP, set the seed to SI_SEED, and pin the r and
ρ loops to SI_R and SI_RHO. Nothing else changes.
"""
import argparse, concurrent.futures, hashlib, json, os, pathlib, re, subprocess, time

FILES = {  # slug: (source file, structure, framework)
    "fig1a": ("For data_Figure1A_Discrete.cpp", "well_mixed", "discrete"),
    "fig1b": ("For data_Figure1B_Continuous.cpp", "well_mixed", "continuous"),
    "fig1c": ("For data_Figure1C_Mixed.cpp", "well_mixed", "mixed"),
    "fig3a": ("For data_Figure3A_Discrete strategy.cpp", "lattice", "discrete"),
    "fig3b": ("For data_Figure3B_Continuous strategy.cpp", "lattice", "continuous"),
    "fig3c": ("For data_Figure3C_Mixed strategy.cpp", "lattice", "mixed"),
}
R = [0.05, 0.1, 0.15]
RHO = {"well_mixed": [0.3], "lattice": [0.0, 0.3, 0.5]}
CHUNKS = 4
SANDBOX = "(version 1)(allow default)(deny network*)"
FLAGS = ["-std=c++17", "-O2"]

# (name, pattern, replacement, optional): an optional patch may match zero times; every patch
# that matches must match exactly once.
PATCHES = [
    ("stray characters before #include", r"\A[^#\n]+(#include)", r"\1", True),
    ("constant run count", r"^int loop = 100;", "const int loop = SI_LOOP;", True),
    ("constant run count", r"^#define loop 100\b", "#define loop SI_LOOP", True),
    ("seed", r"sgenrand\(time\(0\)\);", "sgenrand(SI_SEED);", False),
    ("r loop", r"for\s*\(\s*r\s*=\s*0(?:\.0+)?\s*;[^)]*\)", "for (r = SI_R; r <= SI_R; r += 1)", False),
    ("rho loop", r"for\s*\(\s*rho\s*=\s*0(?:\.0+)?\s*;[^)]*\)", "for (rho = SI_RHO; rho <= SI_RHO; rho += 1)", False),
]


def patch(source):
    """The patched source and the list of patches applied; ValueError if a required one is missing."""
    applied = []
    for name, pattern, replacement, optional in PATCHES:
        out, n = re.subn(pattern, replacement, source, flags=re.MULTILINE)
        if n > 1 or (n == 0 and not optional):
            raise ValueError(f"{name}: {n} matches")
        if n == 1:
            applied.append({"name": name, "pattern": pattern, "replacement": replacement})
            source = out
    if not any(a["name"] == "constant run count" for a in applied):
        raise ValueError("constant run count: no match")
    return source, applied


def parse(structure, framework, stdout):
    """(r, ρ, cooperation) for every run line the program printed."""
    rows = []
    for line in stdout.splitlines():
        if structure == "well_mixed":
            f = line.split(",")
            if len(f) == 6 and f[0][:1].isdigit():
                rows.append((float(f[0]), float(f[1]), float(f[5])))
        else:
            f = line.split("\t")
            if len(f) in (4, 5) and f[0][:1].isdigit():
                rows.append((float(f[0]), float(f[1]), float(f[3] if framework == "continuous" else f[2])))
    return rows


def points():
    return [(slug, r, rho) for slug, (_, structure, _) in FILES.items() for r in R for rho in RHO[structure]]


def seeds(p):
    return [CHUNKS * p + k + 1 for k in range(CHUNKS)]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def run_chunk(job):
    slug, r, rho, seed, per, patched, scratch, chunk = job
    _, structure, framework = FILES[slug]
    box = scratch / f"{slug}-r{r}-rho{rho}-seed{seed}"
    box.mkdir(parents=True, exist_ok=True)
    (box / "x.cpp").write_text(patched, encoding="utf-8")
    defines = [f"-DSI_R={r}", f"-DSI_RHO={rho}", f"-DSI_SEED={seed}", f"-DSI_LOOP={per}"]
    subprocess.run(["c++", *FLAGS, *defines, "-o", "run", "x.cpp"], cwd=box, check=True, capture_output=True)
    started = time.time()
    done = subprocess.run(["sandbox-exec", "-p", SANDBOX, "./run"], cwd=box, check=True, capture_output=True, text=True)
    rows = parse(structure, framework, done.stdout)
    if len(rows) != per or any(abs(a - r) > 1e-6 or abs(b - rho) > 1e-6 for a, b, _ in rows):
        raise RuntimeError(f"{box}: expected {per} runs at r {r}, rho {rho}, got {rows[:3]}…")
    return slug, r, rho, chunk, seed, [c for _, _, c in rows], time.time() - started


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", required=True, type=pathlib.Path)
    ap.add_argument("--scratch", required=True, type=pathlib.Path)
    ap.add_argument("--out", required=True, type=pathlib.Path)
    ap.add_argument("--runs", type=int, default=100)
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--smoke", action="store_true", help="one run of the first well-mixed point only")
    a = ap.parse_args()
    assert a.runs % CHUNKS == 0, "runs split into four chunks"
    a.scratch.mkdir(parents=True, exist_ok=True)
    a.out.mkdir(parents=True, exist_ok=True)
    sources, jobs = {}, []
    for slug, (name, _, _) in FILES.items():
        raw = (a.source / name).read_bytes()
        patched, applied = patch(raw.decode("utf-8"))
        sources[slug] = {"file": name, "sha256": sha(raw), "patched_sha256": sha(patched.encode("utf-8")), "patches": applied, "patched": patched}
    todo = points()[:1] if a.smoke else points()
    for p, (slug, r, rho) in enumerate(todo):
        for chunk, seed in enumerate(seeds(p)[:1] if a.smoke else seeds(p)):
            per = 1 if a.smoke else a.runs // CHUNKS
            jobs.append((slug, r, rho, seed, per, sources[slug]["patched"], a.scratch, chunk))
    results, started = [], time.time()
    with concurrent.futures.ThreadPoolExecutor(max_workers=a.jobs) as pool:
        for out in pool.map(run_chunk, jobs):
            results.append(out)
            print(out[0], out[1], out[2], "chunk", out[3], f"{out[6]:.0f} s", flush=True)
    for slug in FILES:
        lines = ["r,rho,chunk,seed,run,cooperation"]
        for s, r, rho, chunk, seed, values, _ in sorted(x for x in results if x[0] == slug):
            lines += [f"{r},{rho},{chunk},{seed},{i},{v:.6f}" for i, v in enumerate(values)]
        if len(lines) > 1:
            (a.out / f"{slug}.csv").write_text("\n".join(lines) + "\n", encoding="utf-8")
    compiler = subprocess.run(["c++", "--version"], capture_output=True, text=True).stdout.splitlines()[0]
    receipts = {
        "sources": {k: {x: v[x] for x in ("file", "sha256", "patched_sha256", "patches")} for k, v in sources.items()},
        "compiler": compiler, "flags": FLAGS, "sandbox": SANDBOX, "runs_per_point": 1 if a.smoke else a.runs,
        "wall_seconds": round(time.time() - started, 1),
        "chunks": [{"slug": s, "r": r, "rho": rho, "chunk": c, "seed": seed, "seconds": round(t, 1)} for s, r, rho, c, seed, _, t in sorted(results)],
    }
    (a.out / "receipts.json").write_text(json.dumps(receipts, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python3 -I -m unittest discover -s survey/hybrid_si -p 'test_*.py' 2>&1 | tail -3`. Expected: `OK` (6 tests).

- [ ] **Step 5: Smoke the pipeline, then run the reference**

Write `$SCRATCH/m42/dock.sh`:

```bash
#!/bin/bash
set -euo pipefail
cd /Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/misaligned
SRC=papers/alignment/code/si-et-al-2024-jrsi-cooperative-bots-zenodo-13957161
SCRATCH=/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-SugarScape/1082ac4c-3302-454b-aa9f-77a920225c83/scratchpad
python3 -I survey/hybrid_si/dock.py --source "$SRC" --scratch "$SCRATCH/m42/si-dock-$1" --out "$SCRATCH/m42/si-out-$1" ${2:-}
```

Run `bash $SCRATCH/m42/dock.sh smoke --smoke`. Expected: one line `fig1a 0.05 0.3 chunk 0 …`, `$SCRATCH/m42/si-out-smoke/fig1a.csv` with one run, and a `receipts.json` naming four patches for each file and five for `fig3b` and `fig3c` (the stray-character patch). If `sandbox-exec` is missing or refuses the profile, stop: running the deposit without the network denied is outside the approval, so report it to your human partner instead of working around it. Then the full reference, in the background (it takes hours; the lattice mixed file is the slowest): `bash $SCRATCH/m42/dock.sh full` with `run_in_background`, and watch it with `kill -0` on its PID (never `pgrep -f`). When it ends, copy `$SCRATCH/m42/si-out-full/*` into `survey/out/hybrid/si-reference/` and check each CSV has 100 rows per point (`wc -l`: 301 for the well-mixed files, 901 for the lattice files). Task 12 can proceed meanwhile; only its S1 row reads these files.

- [ ] **Step 6: Commit**

```bash
git add survey/hybrid_si survey/out/hybrid/si-reference
git commit -m "hybrid: the sandboxed docking reference from Si et al.'s published code

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

(If the full run is still going, commit the harness and its tests now and the reference files in a second commit when it ends.)

---

### Task 12: The survey: claims, pilot and the full run

**Files:**
- Create: `survey/src/claims/hybrid.rs`
- Modify: `survey/src/claims/mod.rs` (`mod hybrid;` after `mod gridworlds;`, and `hybrid::claims(),` after `misaligned::claims(),`)
- Create: `survey/out/hybrid/results.md` (the survey's output, tracked)

**Interfaces:**
- Consumes: `sugarscape_core::hybrid::{booker, coloring, graph, guo, Runner, HybridConfig, …}`, `crate::claim::{all_of, Claim, Outcome, Source, Verdict}`, `crate::runner::on_threads`, Task 11's CSVs.
- Produces: claims `hybrid.pilot`, `hybrid.b1-switch`, `hybrid.b2-samaritan-vs-discriminatory`, `hybrid.b3-eq18`, `hybrid.b3-eq20`, `hybrid.b4-errata`, `hybrid.b5-lattice`, `hybrid.b5-lattice-sweep-steps`, `hybrid.b6-topology`, `hybrid.g1-phase-diagrams`, `hybrid.g2-master-equation`, `hybrid.g3-algorithm-i`, `hybrid.g4-networks`, `hybrid.g5-hubs`, `hybrid.g-hub-explanation`, `hybrid.s1-docking`, `hybrid.s2-well-mixed-code`, `hybrid.s2-well-mixed-stated`, `hybrid.s3-lattice`, `hybrid.s4-optimum-lattice`, `hybrid.s4-optimum-well-mixed`, `hybrid.s3-per-round`, `hybrid.s2-preprint-n`, `hybrid.s2-journal-n`, `hybrid.s3-journal-n`, `hybrid.c1-convergence`, `hybrid.c2-mechanism`, `hybrid.c2-observed-e`.

Every scored row is the spec's rule (as amended), coded once; the measured string carries the numbers a reader needs to check the verdict by hand. Evaluation rows (`Untestable`, "reported without a target") never count as the paper's claims.

- [ ] **Step 1: Write the claims**

`survey/src/claims/hybrid.rs`:

```rust
//! Milestone 42: the spec's rules, fixed in
//! docs/superpowers/specs/2026-10-17-hybrid-populations-design.md (with its 2026-10-20 planning
//! amendments) before any measurement. Exact rows take no RNG. Simulated rows run the papers'
//! sizes and protocols; a run count below the protocol's comes only from the pilot rule
//! (amendment 8), and the row then says "reduced".
use std::path::Path;
use std::time::Instant;

use rand::Rng;
use sugarscape_core::graph::barabasi_albert;
use sugarscape_core::hybrid::booker::{alpha_c_corrected, alpha_c_printed, alpha_c_scan, BotKind, Chain, Donation, Replicator};
use sugarscape_core::hybrid::coloring::{self, Session};
use sugarscape_core::hybrid::graph::{place, Placement};
use sugarscape_core::hybrid::guo::{self, Phase, Plane};
use sugarscape_core::hybrid::{
    booker_defaults, guo_defaults, si_defaults, Accumulate, Bots, Division, Framework, HybridConfig, MixedDraw, Runner, StepUnit, Structure,
};
use sugarscape_core::rng::seeded;

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::on_threads;

const BOOKER: &str = "Booker et al. 2023, arXiv 2306.17747v2; rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const GUO: &str = "Guo et al. 2023, iScience (arXiv 2307.01123); rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const SI: &str = "Si, He, Shen & Tanimoto 2025, J. R. Soc. Interface 22:20240427, code Zenodo 10.5281/zenodo.13957161; rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const SHIRADO: &str = "Shirado & Christakis 2017, Nature 545:370 (humans reconstructed); rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const OURS: &str = "Our evaluation; docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";

// The protocols' run counts. The pilot (amendment 8) may lower a RUNS_* constant; `reduced` then
// labels its rows. Sizes, lengths and windows never change.
const PROTOCOL_BOOKER_LATTICE: u64 = 30;
const RUNS_BOOKER_LATTICE: u64 = 30;
const BOOKER_NETWORKS: u64 = 10;
const RUNS_PER_NETWORK: u64 = 20;
const PROTOCOL_GUO: u64 = 60;
const RUNS_GUO: u64 = 60;
const PROTOCOL_ALG_I: u64 = 20;
const RUNS_ALG_I: u64 = 20;
const PROTOCOL_SI: u64 = 100;
const RUNS_SI: u64 = 100;
const SESSIONS: u64 = 200;

const BOOKER_STEPS: u64 = 100_000;
const BOOKER_WINDOW: u64 = 1_000;
const ALG_I_STEPS: u64 = 1_000_000;
const ALG_I_WINDOW: u64 = 500_000;
const GUO_STEPS: u64 = 50_000;
const GUO_WINDOW: u64 = 5_000;
const SI_STEPS: u64 = 10_000;
const SI_WINDOW: u64 = 2_000;
/// The 8-hour budget of amendment 8.
const BUDGET_HOURS: f64 = 8.0;

const SD3: Plane = Plane { t: 1.2, s: 0.5 }; // Figs. 3 and 5
const SH4: Plane = Plane { t: 0.6, s: -0.2 }; // Figs. 4 and 5
const PD: Plane = Plane { t: 1.3, s: -0.2 }; // Figs. 2, 5, 6 and 7
const SD6: Plane = Plane { t: 1.4, s: 0.5 }; // Figs. 6 and 7
const SH6: Plane = Plane { t: 0.6, s: -0.5 }; // Figs. 6 and 7
const PHIS: [f64; 6] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
const ALPHAS: [f64; 6] = [0.0, 0.01, 0.1, 0.25, 0.5, 0.75];
const BETAS: [f64; 3] = [0.1, 1.0, 5.0];
const FRAMEWORKS: [(Framework, &str); 3] = [(Framework::Discrete, "discrete"), (Framework::Continuous, "continuous"), (Framework::Mixed, "mixed")];
const DOCK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/out/hybrid/si-reference");

fn claim(id: &'static str, item: &'static str, citation: &'static str, text: &'static str, check: fn(&[u64]) -> Outcome) -> Claim {
    Claim { id, item, source: if citation == OURS { Source::App } else { Source::Book }, citation, text, check }
}

fn scored(holds: bool, measured: String) -> Outcome {
    Outcome { verdict: if holds { Verdict::Holds } else { Verdict::Fails }, measured, detail: String::new() }
}

fn reported(measured: String) -> Outcome {
    Outcome { verdict: Verdict::Untestable, measured, detail: "reported without a target".into() }
}

/// An evaluation row: what the scored rule would say, never scored.
fn as_reported(o: Outcome) -> Outcome {
    reported(format!("by the scored rule this would read {:?}: {}", o.verdict, o.measured))
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The standard error of the mean.
fn se(v: &[f64]) -> f64 {
    let m = mean(v);
    let n = v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt()
}

fn combined(a: &[f64], b: &[f64]) -> f64 {
    (se(a).powi(2) + se(b).powi(2)).sqrt()
}

fn reduced(protocol: u64, used: u64) -> String {
    if used < protocol {
        format!(" [reduced: {used} of {protocol} runs]")
    } else {
        String::new()
    }
}

fn threads() -> f64 {
    std::thread::available_parallelism().map_or(4, |n| n.get()) as f64
}

/// Each run's mean cooperation: seeds 1..=runs, `steps` paper steps, the last `window` averaged.
fn runs_of(c: &HybridConfig, runs: u64, steps: u64, window: u64) -> Vec<f64> {
    let seeds: Vec<u64> = (1..=runs).collect();
    on_threads(&seeds, |seed| Runner::new(c.clone(), seed).expect("a valid config").mean_over(steps, window))
}

/// Hours for `runs` runs of `c` at `steps` steps, from a timed twentieth of one run.
fn projected_hours(c: &HybridConfig, steps: u64, runs: u64) -> f64 {
    let sample = (steps / 20).max(1);
    let started = Instant::now();
    let mut r = Runner::new(c.clone(), 1).expect("a valid config");
    for _ in 0..sample {
        r.step();
    }
    started.elapsed().as_secs_f64() * (steps as f64 / sample as f64) * runs as f64 / threads() / 3600.0
}

/// An evaluation row that runs only if it fits the budget.
fn guarded(probe: &HybridConfig, steps: u64, runs: u64, row: impl FnOnce() -> Outcome) -> Outcome {
    let hours = projected_hours(probe, steps, runs);
    if hours > BUDGET_HOURS {
        reported(format!("not run: projected {hours:.1} h, over the {BUDGET_HOURS} h budget (amendment 8)"))
    } else {
        row()
    }
}

// ---- Booker et al. --------------------------------------------------------------------------

fn chain(m: u32, b: f64, kind: BotKind, beta: f64) -> Chain {
    Chain { n: 100, m, game: Donation { b, c: 1.0 }, kind, beta }
}

fn booker_lattice(alpha: f64, beta: f64) -> HybridConfig {
    HybridConfig { bot_fraction: alpha, beta, ..booker_defaults() }
}

fn booker_ba(alpha: f64, beta: f64) -> HybridConfig {
    HybridConfig { structure: Structure::BarabasiAlbert, nodes: 1000, links: 2, bot_fraction: alpha, beta, ..booker_defaults() }
}

/// Ten networks (graph seeds 1–10) × twenty runs (amendment 9).
fn booker_ba_runs(alpha: f64, beta: f64) -> Vec<f64> {
    let seeds: Vec<u64> = (1..=BOOKER_NETWORKS).flat_map(|g| (1..=RUNS_PER_NETWORK).map(move |r| g * 1000 + r)).collect();
    on_threads(&seeds, |seed| {
        Runner::new(HybridConfig { graph_seed: Some(seed / 1000), ..booker_ba(alpha, beta) }, seed)
            .expect("a valid config")
            .mean_over(BOOKER_STEPS, BOOKER_WINDOW)
    })
}

fn b1(_: &[u64]) -> Outcome {
    let first = |beta: f64| (0..=100).find(|&m| chain(m, 2.0, BotKind::Samaritan, beta).all_c_weight() >= 0.5);
    let (a, b, c) = (first(0.1), first(1.0), first(2.5));
    let holds = a.is_some_and(|m| (1..=8).contains(&m)) && b.is_some_and(|m| (50..=65).contains(&m)) && c.is_none();
    scored(holds, format!("first M with all-C weight ≥ 0.5 (N 100, b 2): β 0.1 → {a:?}, β 1 → {b:?}, β 2.5 → {c:?}"))
}

fn b2(_: &[u64]) -> Outcome {
    let grid = |kind: BotKind, beta: f64| {
        let w: Vec<f64> = (0..=100)
            .flat_map(|m| (0..20).map(move |k| chain(m, 1.1 + 0.1 * f64::from(k), kind, beta).all_c_weight()))
            .collect();
        mean(&w)
    };
    let (s_low, d_low) = (grid(BotKind::Samaritan, 0.1), grid(BotKind::Discriminatory, 0.1));
    let (s_high, d_high) = (grid(BotKind::Samaritan, 2.5), grid(BotKind::Discriminatory, 2.5));
    all_of(vec![
        ("β 0.1, Samaritan higher".into(), scored(s_low > d_low, format!("Samaritan {s_low:.4}, discriminatory {d_low:.4}"))),
        ("β 2.5, discriminatory higher".into(), scored(d_high > s_high, format!("Samaritan {s_high:.4}, discriminatory {d_high:.4}"))),
    ])
}

/// B3 on Eq. 18 or Eq. 20 (amendment 7).
fn b3(kind: BotKind) -> Outcome {
    let grid: Vec<f64> = (0..=190).map(|k| 1.1 + 0.01 * f64::from(k)).collect();
    let rep = |b: f64, alpha: f64, beta_h: f64, beta_ai: f64| Replicator { game: Donation { b, c: 1.0 }, alpha, beta_h, beta_ai, kind };
    let b_star = |beta_h: f64| grid.iter().copied().find(|&b| rep(b, 0.1, beta_h, 1.0).top_stable().unwrap() >= 0.5);
    let (low, high) = (b_star(0.1), b_star(0.5));
    let first = match (low, high) {
        (Some(l), Some(h)) => l < h,
        (Some(_), None) => true,
        _ => false,
    };
    let worst = grid.iter().map(|&b| rep(b, 0.5, 0.5, 1.0).rate(0.0).unwrap().abs()).fold(0.0, f64::max);
    let spread = (0..=100)
        .map(|i| {
            let x = f64::from(i) / 100.0;
            let v: Vec<f64> = grid.iter().map(|&b| rep(b, 0.1, 0.1, 2.0).rate(x).unwrap()).collect();
            v.iter().copied().fold(f64::NEG_INFINITY, f64::max) - v.iter().copied().fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max);
    all_of(vec![
        ("lower β cooperates at lower b".into(), scored(first, format!("b* at β_H 0.1: {low:?}; at β_H 0.5: {high:?} (α 0.1, β_AI 1)"))),
        ("x = 0 is a fixed point at α = 0.5".into(), scored(worst < 1e-12, format!("largest |ẋ(0)| over b: {worst:.3e}"))),
    ])
    .with(&format!("largest spread of ẋ(x) across b at α 0.1, β_H 0.1, β_AI 2: {spread:.3e}"))
}

fn b4(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for m in [1u32, 10, 50, 100] {
        let c = Chain { n: 100, m, game: Donation { b: 2.0, c: 1.0 }, kind: BotKind::Malicious, beta: 1.0 };
        let direct = c.ln_g_direct();
        let printed = ((c.ln_g_d_printed() - direct).exp() - 1.0).abs();
        let corrected = ((c.ln_g_closed() - direct).exp() - 1.0).abs();
        parts.push((format!("G_D at M {m}"), scored(printed > 1e-9 && corrected < 1e-9, format!("printed off by {printed:.3e} relative, corrected by {corrected:.3e}"))));
    }
    for beta in [0.1, 1.0, 2.5] {
        let scan = alpha_c_scan(Donation { b: 2.0, c: 1.0 }, beta, beta);
        let (p, q) = (alpha_c_printed(beta, 1.0), alpha_c_corrected(beta, 1.0));
        let (printed, corrected) = ((p - scan).abs() / scan, (q - scan).abs() / scan);
        parts.push((format!("α_c at β {beta}"), scored(printed > 1e-9 && corrected < 1e-9, format!("root scan {scan:.6}; printed {p:.6} ({printed:.3e}); corrected {q:.6} ({corrected:.3e})"))));
    }
    all_of(parts)
}

fn b5_with(step: StepUnit) -> Outcome {
    let mut means = [[0.0; 6]; 3];
    let mut table = vec![];
    for (bi, &beta) in BETAS.iter().enumerate() {
        for (ai, &alpha) in ALPHAS.iter().enumerate() {
            let runs = runs_of(&HybridConfig { step, ..booker_lattice(alpha, beta) }, RUNS_BOOKER_LATTICE, BOOKER_STEPS, BOOKER_WINDOW);
            means[bi][ai] = mean(&runs);
            table.push(format!("β {beta} α {alpha}: {:.3} ± {:.3}", means[bi][ai], se(&runs)));
        }
    }
    let low = means[0][3] >= 0.5 && means[0][4] >= 0.5 && means[0][1] <= 0.2;
    let high = (1..=4).all(|ai| means[2][ai] - means[2][0] <= 0.2);
    all_of(vec![
        ("β 0.1: ≥ 0.5 at α 0.25 and 0.5, ≤ 0.2 at α 0.01".into(), scored(low, format!("{:.3}, {:.3}, {:.3}", means[0][3], means[0][4], means[0][1]))),
        ("β 5: every α ≤ 0.5 within 0.2 above α 0".into(), scored(high, format!("α 0: {:.3}; largest rise {:.3}", means[2][0], (1..=4).map(|ai| means[2][ai] - means[2][0]).fold(f64::NEG_INFINITY, f64::max)))),
    ])
    .with(&(table.join(" · ") + &reduced(PROTOCOL_BOOKER_LATTICE, RUNS_BOOKER_LATTICE)))
}

fn b6(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for alpha in [0.1, 0.2, 0.3, 0.4] {
        for beta in BETAS {
            let lattice = mean(&runs_of(&booker_lattice(alpha, beta), RUNS_BOOKER_LATTICE, BOOKER_STEPS, BOOKER_WINDOW));
            let ba = mean(&booker_ba_runs(alpha, beta));
            parts.push((format!("α {alpha} β {beta}"), scored((lattice - ba).abs() <= 0.1, format!("lattice {lattice:.3}, Barabási–Albert {ba:.3}"))));
        }
    }
    all_of(parts).with(&reduced(PROTOCOL_BOOKER_LATTICE, RUNS_BOOKER_LATTICE))
}

// ---- Guo et al. ------------------------------------------------------------------------------

fn stable_roots(plane: &Plane, y: f64, phi: f64, k: f64) -> Vec<f64> {
    guo::master_roots(plane, y, phi, k).into_iter().filter(|r| r.stable).map(|r| r.x).collect()
}

/// (lowest, highest) of a set of roots; (∞, −∞) for none, which fails every comparison.
fn ends(v: &[f64]) -> (f64, f64) {
    (v.iter().copied().fold(f64::INFINITY, f64::min), v.iter().copied().fold(f64::NEG_INFINITY, f64::max))
}

fn g1(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (name, plane) in [("snowdrift", SD3), ("stag hunt", SH4), ("dilemma", PD)] {
        let (mut compared, mut skipped, mut mismatched) = (0, 0, vec![]);
        for i in 0..=100 {
            for j in 0..=100 {
                let (y, phi) = (0.05 * f64::from(i), 0.01 * f64::from(j));
                if (plane.snowdrift() || plane.stag_hunt()) && plane.boundary_distance(y, phi) <= 1e-9 {
                    skipped += 1;
                    continue;
                }
                compared += 1;
                if guo::phase_numeric(&plane, y, phi) != Some(plane.phase_closed(y, phi)) {
                    mismatched.push(format!("({y:.2}, {phi:.2})"));
                }
            }
        }
        let shown = if mismatched.is_empty() { "none".to_string() } else { mismatched.iter().take(10).cloned().collect::<Vec<_>>().join(" ") };
        parts.push((name.to_string(), scored(mismatched.is_empty(), format!("{compared} cells compared, {skipped} on a boundary; mismatches: {shown}"))));
    }
    let full = |y: f64| guo::phase_numeric(&SD3, y, 0.0) == Some(Phase::FullC);
    let (mut lo, mut hi) = (0.0, 5.0);
    for _ in 0..200 {
        let m = 0.5 * (lo + hi);
        if full(m) {
            hi = m;
        } else {
            lo = m;
        }
    }
    parts.push(("y_min".into(), scored((hi - 0.4).abs() < 1e-9, format!("the numerical switch to full C at φ 0: y = {hi:.12} (closed form {})", SD3.y_min()))));
    all_of(parts)
}

fn g2(_: &[u64]) -> Outcome {
    let ys: Vec<f64> = (0..=100).map(|i| 0.05 * f64::from(i)).collect();
    let worst = ys.iter().flat_map(|&y| [0.1, 0.9].into_iter().flat_map(move |phi| stable_roots(&PD, y, phi, 100.0))).fold(0.0, f64::max);
    let mut parts = vec![("K 100, dilemma: every stable root ≤ 0.05".to_string(), scored(worst <= 0.05, format!("largest stable root {worst:.4}")))];
    for (phi, rising) in [(0.1, true), (0.9, false)] {
        let (lows, highs): (Vec<f64>, Vec<f64>) = ys.iter().map(|&y| ends(&stable_roots(&SD3, y, phi, 100.0))).unzip();
        let ok = |v: &[f64]| {
            let steps = v.windows(2).all(|w| if rising { w[1] >= w[0] - 1e-9 } else { w[1] <= w[0] + 1e-9 });
            let net = if rising { v[v.len() - 1] > v[0] } else { v[v.len() - 1] < v[0] };
            steps && net
        };
        parts.push((
            format!("K 100, snowdrift, φ {phi}: the stable root {}", if rising { "rises" } else { "falls" }),
            scored(ok(&lows) && ok(&highs), format!("highest root from {:.4} at y 0 to {:.4} at y 5", highs[0], highs[highs.len() - 1])),
        ));
    }
    for (name, plane) in [("dilemma", PD), ("snowdrift", SD3), ("stag hunt", SH4)] {
        let fails: Vec<String> = ys[1..]
            .iter()
            .filter(|&&y| {
                let (l1, h1) = ends(&stable_roots(&plane, y, 0.1, 1.0));
                let (l9, h9) = ends(&stable_roots(&plane, y, 0.9, 1.0));
                !(l9 > l1 && h9 > h1)
            })
            .map(|y| format!("{y:.2}"))
            .collect();
        let shown = if fails.is_empty() { "none".to_string() } else { fails.join(" ") };
        parts.push((format!("K 1, {name}: φ 0.9 above φ 0.1"), scored(fails.is_empty(), format!("y where it is not: {shown}"))));
    }
    all_of(parts)
}

fn alg_i(plane: Plane, y: f64, phi: f64, k: f64, p: f64) -> HybridConfig {
    let bots = (1000.0 * y / (1.0 + y)).round() as u32;
    HybridConfig { structure: Structure::WellMixed, nodes: 1000, bots: Bots::Count, bot_count: bots, t: plane.t, s: plane.s, phi, beta: k, initial_cooperation: p, ..guo_defaults() }
}

fn g3(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (name, plane) in [("dilemma", PD), ("snowdrift", SD3), ("stag hunt", SH4)] {
        let (mut ok, mut rows) = (true, vec![]);
        for y in [0.5, 1.0, 2.0, 5.0] {
            for phi in [0.1, 0.9] {
                for k in [1.0, 10.0, 100.0] {
                    let roots = stable_roots(&plane, y, phi, k);
                    let (lo, hi) = ends(&roots);
                    let starts = if roots.len() == 1 { vec![(0.5, roots[0])] } else { vec![(0.0, lo), (1.0, hi)] };
                    for (p, root) in starts {
                        let m = mean(&runs_of(&alg_i(plane, y, phi, k, p), RUNS_ALG_I, ALG_I_STEPS, ALG_I_WINDOW));
                        let good = (m - root).abs() <= 0.05;
                        ok &= good;
                        rows.push(format!("{}y {y} φ {phi} K {k} p {p}: {m:.3} vs {root:.3}", if good { "" } else { "OFF " }));
                    }
                }
            }
        }
        parts.push((name.to_string(), scored(ok, rows.join(" · "))));
    }
    all_of(parts).with(&reduced(PROTOCOL_ALG_I, RUNS_ALG_I))
}

fn guo_net(structure: Structure, plane: Plane) -> HybridConfig {
    HybridConfig { structure, t: plane.t, s: plane.s, ..guo_defaults() }
}

fn phi_runs(base: &HybridConfig) -> Vec<Vec<f64>> {
    PHIS.iter().map(|&phi| runs_of(&HybridConfig { phi, ..base.clone() }, RUNS_GUO, GUO_STEPS, GUO_WINDOW)).collect()
}

/// The least-squares slope of cooperation against φ.
fn slope(means: &[f64]) -> f64 {
    let (mx, my) = (mean(&PHIS), mean(means));
    PHIS.iter().zip(means).map(|(x, y)| (x - mx) * (y - my)).sum::<f64>() / PHIS.iter().map(|x| (x - mx).powi(2)).sum::<f64>()
}

/// The slope and its 95% percentile bootstrap interval: 2,000 resamples of runs within each φ,
/// survey seed 42 (amendment 11).
fn slope_ci(runs: &[Vec<f64>]) -> (f64, f64, f64) {
    let point = slope(&runs.iter().map(|r| mean(r)).collect::<Vec<_>>());
    let mut rng = seeded(42);
    let mut boot: Vec<f64> = (0..2000)
        .map(|_| {
            let means: Vec<f64> = runs.iter().map(|r| (0..r.len()).map(|_| r[rng.gen_range(0..r.len())]).sum::<f64>() / r.len() as f64).collect();
            slope(&means)
        })
        .collect();
    boot.sort_by(f64::total_cmp);
    (point, boot[50], boot[1949])
}

fn g4(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (game, plane) in [("snowdrift", SD6), ("dilemma", PD)] {
        for (name, structure) in [("lattice", Structure::Lattice), ("Erdős–Rényi", Structure::ErdosRenyi), ("Barabási–Albert", Structure::BarabasiAlbert)] {
            let rising = !(game == "snowdrift" && structure == Structure::Lattice);
            let runs = phi_runs(&guo_net(structure, plane));
            let (s, lo, hi) = slope_ci(&runs);
            let ok = if rising { lo > 0.0 } else { hi < 0.0 };
            let means: Vec<String> = runs.iter().map(|r| format!("{:.3}", mean(r))).collect();
            parts.push((
                format!("{game}, {name}: slope {}", if rising { "positive" } else { "negative" }),
                scored(ok, format!("slope {s:+.4} (95% CI {lo:+.4} to {hi:+.4}); means by φ {}", means.join(", "))),
            ));
        }
    }
    all_of(parts).with(&reduced(PROTOCOL_GUO, RUNS_GUO))
}

fn g5(_: &[u64]) -> Outcome {
    let ba = |plane: Plane| guo_net(Structure::BarabasiAlbert, plane);
    let mut parts = vec![];
    for (game, plane, hubs_scored) in [("dilemma", PD, true), ("snowdrift", SD6, false), ("stag hunt", SH6, true)] {
        let none = runs_of(&HybridConfig { bot_count: 0, ..ba(plane) }, RUNS_GUO, GUO_STEPS, GUO_WINDOW);
        let hubs = phi_runs(&HybridConfig { bot_count: 10, placement: Placement::Hubs, ..ba(plane) });
        let random = phi_runs(&HybridConfig { bot_count: 10, placement: Placement::Random, ..ba(plane) });
        let hub_means: Vec<String> = hubs.iter().map(|r| format!("{:.3}", mean(r))).collect();
        if hubs_scored {
            let (gap, two) = (mean(&hubs[5]) - mean(&none), 2.0 * combined(&hubs[5], &none));
            let (s, lo, _) = slope_ci(&hubs);
            parts.push((
                format!("{game}: ten hub bots move cooperation"),
                scored(gap > two && lo > 0.0, format!("φ 1: {:.3} vs none {:.3} (gap {gap:.3}, 2 SE {two:.3}); slope {s:+.4}, CI lower {lo:+.4}; hub means by φ {}", mean(&hubs[5]), mean(&none), hub_means.join(", "))),
            ));
        }
        let off: Vec<String> = PHIS
            .iter()
            .zip(&random)
            .filter(|(_, r)| (mean(r) - mean(&none)).abs() > 2.0 * combined(r, &none))
            .map(|(phi, r)| format!("φ {phi}: {:.3}", mean(r)))
            .collect();
        let shown = if off.is_empty() { "none".to_string() } else { off.join(", ") };
        parts.push((format!("{game}: ten random bots do not"), scored(off.is_empty(), format!("none {:.3}; outside 2 SE: {shown}", mean(&none)))));
    }
    all_of(parts).with(&reduced(PROTOCOL_GUO, RUNS_GUO))
}

fn guo_hub_explanation(_: &[u64]) -> Outcome {
    let base = guo_net(Structure::BarabasiAlbert, SD6);
    let (s_sum, lo_sum, hi_sum) = slope_ci(&phi_runs(&base));
    let (s_mean, lo_mean, hi_mean) = slope_ci(&phi_runs(&HybridConfig { accumulate: Accumulate::Mean, ..base }));
    reported(format!(
        "snowdrift on Barabási–Albert, 4,871 random bots: slope with summed payoffs {s_sum:+.4} ({lo_sum:+.4} to {hi_sum:+.4}); with mean payoffs {s_mean:+.4} ({lo_mean:+.4} to {hi_mean:+.4}){}",
        reduced(PROTOCOL_GUO, RUNS_GUO)
    ))
}

// ---- Si et al. -------------------------------------------------------------------------------

fn si_lattice(framework: Framework, r: f64, rho: f64, side: u32) -> HybridConfig {
    HybridConfig { framework, r, bot_fraction: rho, side, nodes: side * side, ..si_defaults() }
}

fn si_well_mixed(framework: Framework, r: f64, rho: f64, division: Division, nodes: u32) -> HybridConfig {
    HybridConfig { structure: Structure::WellMixed, nodes, framework, r, bot_fraction: rho, division, ..si_defaults() }
}

/// The published code's runs at (r, ρ) from Task 11's CSV.
fn theirs(slug: &str, r: f64, rho: f64) -> Result<Vec<f64>, String> {
    let path = Path::new(DOCK).join(format!("{slug}.csv"));
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e} (run survey/hybrid_si/dock.py first)", path.display()))?;
    let v: Vec<f64> = text
        .lines()
        .skip(1)
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').collect();
            let (lr, lrho, c): (f64, f64, f64) = (f.first()?.parse().ok()?, f.get(1)?.parse().ok()?, f.get(5)?.parse().ok()?);
            ((lr - r).abs() < 1e-9 && (lrho - rho).abs() < 1e-9).then_some(c)
        })
        .collect();
    if v.is_empty() {
        Err(format!("{slug}: no runs at r {r}, ρ {rho}"))
    } else {
        Ok(v)
    }
}

fn s1(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (slug, lattice, (framework, name)) in [
        ("fig1a", false, FRAMEWORKS[0]),
        ("fig1b", false, FRAMEWORKS[1]),
        ("fig1c", false, FRAMEWORKS[2]),
        ("fig3a", true, FRAMEWORKS[0]),
        ("fig3b", true, FRAMEWORKS[1]),
        ("fig3c", true, FRAMEWORKS[2]),
    ] {
        let rhos: &[f64] = if lattice { &[0.0, 0.3, 0.5] } else { &[0.3] };
        for r in [0.05, 0.1, 0.15] {
            for &rho in rhos {
                let them = match theirs(slug, r, rho) {
                    Ok(v) => v,
                    Err(e) => return Outcome { verdict: Verdict::Untestable, measured: e, detail: String::new() },
                };
                let config = if lattice { si_lattice(framework, r, rho, 100) } else { si_well_mixed(framework, r, rho, Division::Integer, 10_000) };
                let us = runs_of(&config, RUNS_SI, SI_STEPS, SI_WINDOW);
                let d = mean(&us) - mean(&them);
                let ok = d.abs() <= 3.0 * combined(&us, &them) || d.abs() < 0.02;
                parts.push((
                    format!("{} {name}, r {r}, ρ {rho}", if lattice { "lattice" } else { "well-mixed" }),
                    scored(ok, format!("ours {:.4} ± {:.4} ({} runs), theirs {:.4} ± {:.4} ({} runs)", mean(&us), se(&us), us.len(), mean(&them), se(&them), them.len())),
                ));
            }
        }
    }
    all_of(parts).with(&reduced(PROTOCOL_SI, RUNS_SI))
}

fn s2_with(division: Division, nodes: u32) -> Outcome {
    let rs = [0.0, 0.04, 0.08, 0.12, 0.16, 0.2];
    let rhos = [0.0, 0.1, 0.3, 0.5];
    let mut parts = vec![];
    for (framework, name) in FRAMEWORKS {
        let m: Vec<Vec<f64>> = rhos
            .iter()
            .map(|&rho| rs.iter().map(|&r| mean(&runs_of(&si_well_mixed(framework, r, rho, division, nodes), RUNS_SI, SI_STEPS, SI_WINDOW))).collect())
            .collect();
        let table: Vec<String> = rhos.iter().zip(&m).map(|(rho, row)| format!("ρ {rho}: {}", row.iter().map(|v| format!("{v:.3}")).collect::<Vec<_>>().join(" "))).collect();
        let raised = (1..4).all(|k| (0..rs.len()).all(|j| m[k][j] > m[0][j]));
        parts.push((format!("{name}: bots raise cooperation at every r"), scored(raised, format!("r {rs:?}; {}", table.join("; ")))));
        if framework != Framework::Discrete {
            let near = (0..3).all(|j| m[3][j] >= 0.8);
            parts.push((format!("{name}: ≥ 0.8 at ρ 0.5 for r ≤ 0.1"), scored(near, format!("{:.3}, {:.3}, {:.3}", m[3][0], m[3][1], m[3][2]))));
        }
    }
    all_of(parts).with(&format!(" N {nodes}, division {division:?}{}", reduced(PROTOCOL_SI, RUNS_SI)))
}

fn s3_with(mixed_draw: MixedDraw, side: u32, frameworks: &[(Framework, &str)]) -> Outcome {
    let rs: Vec<f64> = (0..=20).map(|k| 0.01 * f64::from(k)).collect();
    let mut parts = vec![];
    for &(framework, name) in frameworks {
        let promote = framework == Framework::Mixed;
        let (mut ok, mut rows) = (true, vec![]);
        for (k, &r) in rs.iter().enumerate() {
            if !promote && k >= 13 {
                continue;
            }
            let at = |rho: f64| runs_of(&HybridConfig { mixed_draw, ..si_lattice(framework, r, rho, side) }, RUNS_SI, SI_STEPS, SI_WINDOW);
            let (none, half) = (at(0.0), at(0.5));
            let (d, two) = (mean(&half) - mean(&none), 2.0 * combined(&half, &none));
            let good = if promote { d > two } else { d < -two };
            ok &= good;
            rows.push(format!("{}r {r:.2}: {:.3} vs {:.3}", if good { "" } else { "OFF " }, mean(&half), mean(&none)));
        }
        parts.push((format!("{name}: bots {}", if promote { "promote at every r" } else { "suppress for r < 0.13" }), scored(ok, rows.join(" · "))));
    }
    all_of(parts).with(&format!(" side {side}, {mixed_draw:?}{}", reduced(PROTOCOL_SI, RUNS_SI)))
}

fn s4(lattice: bool) -> Outcome {
    let thetas: Vec<f64> = (0..=10).map(|k| 0.1 * f64::from(k)).collect();
    let base = if lattice { si_lattice(Framework::Mixed, 0.01, 0.3, 100) } else { si_well_mixed(Framework::Mixed, 0.01, 0.3, Division::Integer, 10_000) };
    let runs: Vec<Vec<f64>> = thetas.iter().map(|&theta| runs_of(&HybridConfig { theta, ..base.clone() }, RUNS_SI, SI_STEPS, SI_WINDOW)).collect();
    let means: Vec<f64> = runs.iter().map(|r| mean(r)).collect();
    let best = (0..means.len()).max_by(|&a, &b| means[a].total_cmp(&means[b])).unwrap();
    let (margin, two) = (means[best] - means[10], 2.0 * combined(&runs[best], &runs[10]));
    let shown: Vec<String> = means.iter().map(|m| format!("{m:.3}")).collect();
    scored(
        best > 0 && best < 10 && margin > two,
        format!("argmax θ {:.1} ({:.4}); θ 1 {:.4}; margin {margin:.4} vs 2 SE {two:.4}; by θ {}{}", thetas[best], means[best], means[10], shown.join(" "), reduced(PROTOCOL_SI, RUNS_SI)),
    )
}

// ---- Shirado and Christakis ------------------------------------------------------------------

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let (mx, my) = (mean(x), mean(y));
    let sxy: f64 = x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum();
    let sxx: f64 = x.iter().map(|a| (a - mx).powi(2)).sum();
    let syy: f64 = y.iter().map(|b| (b - my).powi(2)).sum();
    sxy / (sxx * syy).sqrt()
}

fn c1(_: &[u64]) -> Outcome {
    let seeds: Vec<u64> = (1..=230).collect();
    let rows: Vec<(f64, f64, u32)> = on_threads(&seeds, |g| {
        let mut rng = seeded(g);
        let adj = barabasi_albert(20, 2, &mut rng);
        let start: Vec<u8> = (0..20).map(|_| rng.gen_range(0..3u32) as u8).collect();
        let solutions = coloring::colorings(&adj, 3) as f64;
        let (mut total, mut censored) = (0.0, 0);
        for _ in 0..100 {
            match coloring::convergence_steps(&adj, &start, 10_000_000, &mut rng) {
                Some(s) => total += s as f64,
                None => {
                    censored += 1;
                    total += 1e7;
                }
            }
        }
        (solutions.ln(), (total / 100.0).max(1.0).ln(), censored)
    });
    let (x, y): (Vec<f64>, Vec<f64>) = rows.iter().map(|&(a, b, _)| (a, b)).unzip();
    let censored: u32 = rows.iter().map(|r| r.2).sum();
    let r = pearson(&x, &y);
    scored(r <= -0.9, format!("Pearson r of ln solutions and ln mean steps over 230 graphs: {r:.4} (the paper's −0.990); censored repeats (counted at the 10⁷ cap): {censored}"))
}

/// The share of `SESSIONS` sessions solved within 300 s with `bots` bots on the hubs.
fn solve_rate(bots: usize, noise: f64, errant: f64) -> f64 {
    let seeds: Vec<u64> = (1..=SESSIONS).collect();
    let solved = on_threads(&seeds, |s| {
        let mut rng = seeded(s);
        let adj = barabasi_albert(20, 2, &mut rng);
        let degrees: Vec<usize> = adj.iter().map(Vec::len).collect();
        let bot = place(&degrees, bots, Placement::Hubs, &mut rng);
        let rules = coloring::Rules { noise, errant, reaction: 1.59, bot_interval: 1.5, time_limit: 300.0 };
        f64::from(u8::from(Session::new(adj, bot, rules, &mut rng).play(&mut rng).is_some()))
    });
    mean(&solved)
}

/// The errant rate whose control solve fraction is nearest 0.67 (20 humans; ties to the lower e).
fn fitted_errant() -> (f64, f64) {
    (0..=40)
        .map(|k| {
            let e = 0.005 * f64::from(k);
            (e, solve_rate(0, 0.0, e))
        })
        .min_by(|a, b| (a.1 - 0.67).abs().total_cmp(&(b.1 - 0.67).abs()))
        .unwrap()
}

fn c2_at(errant: f64, label: String) -> Outcome {
    let (p0, p10, p30) = (solve_rate(3, 0.0, errant), solve_rate(3, 0.1, errant), solve_rate(3, 0.3, errant));
    let n = SESSIONS as f64;
    let two = |a: f64, b: f64| 2.0 * ((a * (1.0 - a) + b * (1.0 - b)) / n).sqrt();
    scored(
        p10 - p0 > two(p10, p0) && p10 - p30 > two(p10, p30),
        format!("{label}: solved within 300 s with three central bots at noise 0, 0.1, 0.3: {p0:.3}, {p10:.3}, {p30:.3} ({SESSIONS} sessions each; 2 SE {:.3} and {:.3})", two(p10, p0), two(p10, p30)),
    )
}

fn c2(_: &[u64]) -> Outcome {
    let (e, rate) = fitted_errant();
    c2_at(e, format!("fitted e {e:.3} (control solve fraction {rate:.3}; the paper's 20/30; its measured human mean e is 0.041)"))
}

fn c2_observed(_: &[u64]) -> Outcome {
    c2_at(0.041, "e 0.041".into())
}

// ---- The pilot -------------------------------------------------------------------------------

fn pilot(_: &[u64]) -> Outcome {
    let rows: Vec<(&str, HybridConfig, u64, u64)> = vec![
        ("Booker lattice, B5 and B6", booker_lattice(0.25, 1.0), BOOKER_STEPS, 30 * RUNS_BOOKER_LATTICE),
        ("Booker lattice under sweep_steps, B5", HybridConfig { step: StepUnit::Sweep, ..booker_lattice(0.25, 1.0) }, BOOKER_STEPS, 18 * RUNS_BOOKER_LATTICE),
        ("Booker Barabási–Albert, B6", booker_ba(0.25, 1.0), BOOKER_STEPS, 12 * BOOKER_NETWORKS * RUNS_PER_NETWORK),
        ("Guo Algorithm I, G3", alg_i(SD3, 1.0, 0.9, 10.0, 0.5), ALG_I_STEPS, 144 * RUNS_ALG_I),
        ("Guo networks, G4", guo_net(Structure::BarabasiAlbert, SD6), GUO_STEPS, 36 * RUNS_GUO),
        ("Guo Barabási–Albert, G5 and the hub evaluation", guo_net(Structure::BarabasiAlbert, PD), GUO_STEPS, 51 * RUNS_GUO),
        ("Si lattice (mixed, the costliest), S1, S3, S4, per_round", si_lattice(Framework::Mixed, 0.1, 0.3, 100), SI_STEPS, 206 * RUNS_SI),
        ("Si well-mixed, S1, S2 twice, S4", si_well_mixed(Framework::Mixed, 0.1, 0.3, Division::Integer, 10_000), SI_STEPS, 164 * RUNS_SI),
    ];
    let out: Vec<String> = rows
        .into_iter()
        .map(|(name, config, steps, runs)| {
            let hours = projected_hours(&config, steps, runs);
            format!("{name}: {runs} runs ≈ {hours:.2} h on {} threads{}", threads(), if hours > BUDGET_HOURS { " — over budget" } else { "" })
        })
        .collect();
    reported(out.join(" · "))
}

pub fn claims() -> Vec<Claim> {
    vec![
        claim("hybrid.pilot", "hybrid-si-mixed-lattice", OURS, "throughput of each simulated protocol and its projected wall time (amendment 8)", pilot),
        claim("hybrid.b1-switch", "hybrid-booker-samaritan", BOOKER, "Fig. 1, Samaritan bots: the all-C weight switches on at a few bots at β 0.1, at about 55–60 at β 1, and not by 100 at β 2.5 (N = 100, b = 2)", b1),
        claim("hybrid.b2-samaritan-vs-discriminatory", "hybrid-booker-discriminatory", BOOKER, "Samaritan bots beat discriminatory bots at low β and lose to them at high β (Fig. 1)", b2),
        claim("hybrid.b3-eq18", "hybrid-booker-samaritan", BOOKER, "Figs. 2–3 on Eq. 18: lower β cooperates at lower b, and x = 0 is a fixed point at α = 0.5 (registered expectation: fails)", |_| b3(BotKind::Samaritan)),
        claim("hybrid.b3-eq20", "hybrid-booker-discriminatory", BOOKER, "the same two statements on Eq. 20 (discriminatory bots), judged separately", |_| b3(BotKind::Discriminatory)),
        claim("hybrid.b4-errata", "hybrid-booker-samaritan", BOOKER, "the printed G_D (Eq. 14) and simplified α_c differ from the formulas they come from; the corrected forms agree", b4),
        claim("hybrid.b5-lattice", "hybrid-booker-lattice-alpha", BOOKER, "on the lattice, Samaritan bots raise cooperation at β 0.1 as α grows and barely help at β 5", |_| b5_with(StepUnit::Paper)),
        claim("hybrid.b5-lattice-sweep-steps", "hybrid-booker-lattice-alpha", BOOKER, "B5 with N-update steps (variant row, amendment 10)", |_| b5_with(StepUnit::Sweep)),
        claim("hybrid.b6-topology", "hybrid-booker-samaritan", BOOKER, "topology does not change the final state: Barabási–Albert and lattice agree", b6),
        claim("hybrid.g1-phase-diagrams", "hybrid-guo-snowdrift-lattice", GUO, "Theorems 1–3, Figs. 3–4: the replicator phase diagrams; snowdrift y_min = 0.4", g1),
        claim("hybrid.g2-master-equation", "hybrid-guo-snowdrift-lattice", GUO, "Fig. 5 lines: the master equation's stable roots", g2),
        claim("hybrid.g3-algorithm-i", "hybrid-guo-snowdrift-lattice", GUO, "Fig. 5 dots: Algorithm I with N = 1,000 agrees with the master equation", g3),
        claim("hybrid.g4-networks", "hybrid-guo-phi-networks", GUO, "Fig. 6: snowdrift cooperation falls with φ on the lattice and rises on Erdős–Rényi and Barabási–Albert; dilemma cooperation rises on all three", g4),
        claim("hybrid.g5-hubs", "hybrid-guo-hub-bots", GUO, "Fig. 7: ten hub bots move cooperation; ten random bots do not", g5),
        claim("hybrid.g-hub-explanation", "hybrid-guo-snowdrift-scale-free", OURS, "snowdrift on Barabási–Albert with random bots: the φ slope with summed and with mean payoffs (reported)", guo_hub_explanation),
        claim("hybrid.s1-docking", "hybrid-si-mixed-lattice", SI, "our runs dock with the published code at 27 lattice and 9 well-mixed points", s1),
        claim("hybrid.s2-well-mixed-code", "hybrid-si-mixed-lattice", SI, "Fig. 1 as the code runs it: bots raise well-mixed cooperation; continuous and mixed stay ≥ 0.8 up to r ≈ 0.1 at ρ 0.5", |_| s2_with(Division::Integer, 10_000)),
        claim("hybrid.s2-well-mixed-stated", "hybrid-si-mixed-lattice", SI, "Fig. 1 on the stated game (division: float, amendment 1)", |_| s2_with(Division::Float, 10_000)),
        claim("hybrid.s3-lattice", "hybrid-si-lattice-r", SI, "Fig. 3: on the lattice bots suppress cooperation in discrete and continuous for r < 0.13 and promote it in mixed at every r", |_| s3_with(MixedDraw::PerInteraction, 100, &FRAMEWORKS)),
        claim("hybrid.s4-optimum-lattice", "hybrid-si-mixed-lattice", SI, "Fig. 6 as captioned (lattice): an interior optimal θ in mixed at r 0.01", |_| s4(true)),
        claim("hybrid.s4-optimum-well-mixed", "hybrid-si-mixed-lattice", SI, "Fig. 6 as the deposited code runs it (well-mixed): an interior optimal θ in mixed at r 0.01", |_| s4(false)),
        claim("hybrid.s3-per-round", "hybrid-si-mixed-lattice", OURS, "S3's mixed rows with one action per round (Fig. A1), reported", |_| as_reported(s3_with(MixedDraw::PerRound, 100, &FRAMEWORKS[2..]))),
        claim("hybrid.s2-preprint-n", "hybrid-si-mixed-lattice", OURS, "S2 with the preprint's N = 500, reported", |_| as_reported(s2_with(Division::Integer, 500))),
        claim("hybrid.s2-journal-n", "hybrid-si-mixed-lattice", OURS, "S2 with the journal's N = 10⁵, reported if affordable", |_| {
            guarded(&si_well_mixed(Framework::Mixed, 0.1, 0.3, Division::Integer, 100_000), SI_STEPS, 72 * RUNS_SI, || as_reported(s2_with(Division::Integer, 100_000)))
        }),
        claim("hybrid.s3-journal-n", "hybrid-si-lattice-r", OURS, "S3 on a 316 × 316 lattice (the journal's N = 10⁵, nearest square), reported if affordable", |_| {
            guarded(&si_lattice(Framework::Mixed, 0.1, 0.3, 316), SI_STEPS, 126 * RUNS_SI, || as_reported(s3_with(MixedDraw::PerInteraction, 316, &FRAMEWORKS)))
        }),
        claim("hybrid.c1-convergence", "hybrid-coloring-central", SHIRADO, "Extended Data Fig. 2: ln solutions and ln mean convergence steps correlate at −0.990", c1),
        claim("hybrid.c2-mechanism", "hybrid-coloring-central", SHIRADO, "with e fit to the control arm, 10% central bots solve more often than 0% and 30% central bots (mechanism sufficiency, not the human result)", c2),
        claim("hybrid.c2-observed-e", "hybrid-coloring-central", SHIRADO, "the same at the paper's measured human errant rate, 0.041", c2_observed),
    ]
}
```

Register in `survey/src/claims/mod.rs`: `mod hybrid;` after `mod gridworlds;` and `hybrid::claims(),` after `misaligned::claims(),`.

- [ ] **Step 2: Build, lint and test the survey**

Run: `cargo build --release --manifest-path survey/Cargo.toml 2>&1 | tail -3`, `cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings 2>&1 | tail -5`, `cargo test --manifest-path survey/Cargo.toml 2>&1 | tail -3`. Expected: clean. Clippy may ask for `is_some_and` forms or flag the large `vec!`; follow its suggestions without changing a rule.

- [ ] **Step 3: Pilot**

Run: `cargo run --release --manifest-path survey/Cargo.toml -- --only hybrid.pilot 2>&1 | tail -5`. Read each projection. For a scored protocol over 8 hours, lower its `RUNS_*` constant to the largest count that fits (at least 10), leave its `PROTOCOL_*` constant alone so the rows say "reduced", and record the pilot's numbers and each change as a dated note for the findings' provenance section. If `RUNS_SI` is lowered, rerun Task 11's reference with `--runs` set to the same count (rounded down to a multiple of 4), so both sides use the same count (spec, Measurement and compute). Commit the claims file (and any constant change) before the full run:

```bash
git add survey/src/claims
git commit -m "hybrid: the survey's claims for milestone 42

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

- [ ] **Step 4: The full run**

Write `$SCRATCH/m42/survey.sh`:

```bash
#!/bin/bash
set -euo pipefail
cd /Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/misaligned
mkdir -p survey/out/hybrid
started=$(date +%s)
cargo run --release --manifest-path survey/Cargo.toml -- --only hybrid 2>&1 | grep -v '^running' > survey/out/hybrid/results.md
echo "wall seconds: $(( $(date +%s) - started ))"
git rev-parse --short HEAD
shasum -a 256 survey/out/hybrid/results.md
```

Run it with `run_in_background` and wait on its PID with `kill -0` checks (never `pgrep -f`), roughly hourly. Expected: hours of wall time; one row per claim in the shape of `survey/out/misaligned/results.md`. Read every verdict. A `Fails` is reported as measured; it is never tuned away. If a claim panics, the survey prints `Error` for that row: fix the code's defect (not the rule), commit, and rerun only that claim with `--only <id>`, then splice its row into `results.md` and say so in the provenance.

- [ ] **Step 5: Commit the results**

```bash
git add survey/out/hybrid/results.md
git commit -m "hybrid: survey results for milestone 42

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

---

### Task 13: Independent review of rules and measurements, findings, catalog and docs

**Files:**
- Create: `docs/superpowers/specs/2026-10-17-hybrid-populations-findings.md`
- Modify: `crates/sugarscape-core/src/hybrid/presets.rs` (measured results appended to the descriptions), `crates/sugarscape-core/src/titles.rs` (titles revised to what happens, where the results support it), `web/src/episodes/test-fixtures/foraging-source-compatibility.json` (the `titles.rs` receipt; `presets.rs` here is the hybrid module's, which no receipt pins)
- Modify: `docs/papers.md`, `docs/reproducibility.md`, `docs/roadmap.md`, `docs/studies/2026-10-11-alignment-program.md`, `README.md`
- Modify (memory, outside the repo): `/Users/nathan/.claude/projects/-Users-nathan-Projects-ndouglas-SugarScape/memory/` (`model-queue.md`, a new `hybrid-milestone.md`, `MEMORY.md`)

**Interfaces:**
- Consumes: `survey/out/hybrid/results.md`, `survey/out/hybrid/si-reference/receipts.json`, the pilot's notes, the ledger's rulings.

- [ ] **Step 1: Independent review of the rules and the measurements**

The spec's Process requires a review before anything is called complete. Dispatch a fresh reviewer (the most capable model; a general-purpose agent) with: the spec and its amendments; `survey/src/claims/hybrid.rs`; `survey/out/hybrid/results.md`; `survey/out/hybrid/si-reference/receipts.json` and the CSVs; and the core files each claim drives (`booker.rs`, `guo.rs`, `si.rs`, `coloring.rs`, `world.rs`). Ask it to check, claim by claim: that the code implements the registered rule and nothing else (grid, tolerance, statistic, direction); that each measured string supports its verdict; that every reduced row is labeled and both sides of S1 use the same run count; that the Si port matches the C++ line by line where S1 depends on it (payoffs, update order, who may act, the measure); that the docking patches change nothing but the five listed things; and that nothing reads as tuned to a verdict. It reports findings with file and line; it does not edit. A finding that a rule's code is wrong is fixed in the code (never in the rule), tested, committed, and the affected claims rerun with `--only <id>`; a finding that a rule itself is unclear gets a dated note in the findings and its own verdict, never a silent change. Record each finding and its disposition in the ledger.

- [ ] **Step 2: Findings**

Write `docs/superpowers/specs/2026-10-17-hybrid-populations-findings.md` in the shape of `2026-10-14-misaligned-ai-findings.md`:

- **Sources.** Booker et al. arXiv v2; Guo et al. arXiv v1 with the supplement's source; Si et al.'s journal version (and preprint), with the Zenodo code; Shirado and Christakis with its Supplementary Information; the cited-only papers (Fernández Domingos 2022, Ishowo-Oloko 2019 and its OSF deposit, Crandall 2018).
- **Summary**, a short paragraph per world.
- **Provenance**: the survey command, the machine and its thread count, the core commit, the wall time, the pilot's projections and every run-count change (with its date), `results.md`'s sha256; the docking build (compiler line, flags, sandbox profile, the patches and their hashes from `receipts.json`, wall time).
- **Claims**: one table row per claim with the measured values and the verdict copied from `results.md`, the evaluation rows marked as such, B3's and B5's variant rows beside their scored rows, S2 and S4 each with both verdicts.
- **What the sources say and what the code does**: Si et al.'s integer division and what it does to the well-mixed game; the compile defects; the Fig. 6 code–caption mismatch and which reading each S4 verdict judges; κ as a temperature on the lattice and an inverse temperature well-mixed (both β = 10); N in the code (10⁴) against the journal (10⁵) and the preprint (500); Booker's b-independent Eq. 18, the x = 0 statement, the printed G_D and α_c; Guo's hub inference and what the evaluation row shows; the coloring game's reconstruction and that C2 tests mechanism sufficiency, not the human result.
- **Human data, cited and not reproduced**: Shirado's 17/20 against 20/30 at 10% central noise (log-rank P = 0.015, one of nine cells, against the omnibus P = 0.024); Ishowo-Oloko's machine penalty; Fernández Domingos's 66.7% against 86.7% group success.
- **Limits and next steps**: the OSF deposit makes S++ reproducible from code in a later milestone; what any reduced row would need to run in full.

Keep these working notes distinct from publication-ready claims (AGENTS.md): the findings report measurements and verdicts, and label interpretations as such.

- [ ] **Step 3: Presets and titles**

- `crates/sugarscape-core/src/hybrid/presets.rs`: append to each description a sentence with the survey's measured result for that preset's claim (for example "Survey (30 runs, the paper's protocol): …"), plain and sourced from `results.md`.
- `crates/sugarscape-core/src/titles.rs`: where a result supports a plain statement of what happens, retitle the preset to it (the titles in Task 8 describe the setup because nothing was measured then); where the result is mixed or fails, keep the setup title. Make the preset's `name` the same text.
- Run `cargo fmt --all`, `cargo test -p sugarscape-core 2>&1 | tail -3` (the title test checks one title per preset), then `python3 -I $SCRATCH/m42/receipts.py` and `cd web && npx vitest run src/episodes/caching-parity.test.ts 2>&1 | tail -3`. Preset descriptions do not enter fingerprints, so the goldens do not move; if one does, stop and find out why.

- [ ] **Step 4: Catalog, register, roadmap, program note, README, memory**

- `docs/papers.md`: remove row 42 from the Queue table; in the Queue intro change "items 41, 42 and 44 are its reproduction queue" to "items 41 and 44 are its reproduction queue"; add to the Reproduced table after row 43: `| 42 | \`hybrid\` | \`alignment/booker-et-al-2023-…\`; \`alignment/guo-et-al-2023-iscience-…\` (with the arXiv supplement source); \`alignment/si-he-shen-tanimoto-2025-jrsi-cooperative-bots.pdf\` (and the Zenodo code); \`alignment/shirado-christakis-2017-nature-…\` (model only); cited: \`alignment/fernandez-domingos-et-al-2022-scirep-…\`, \`alignment/ishowo-oloko-et-al-2019-…\` (with its OSF deposit), \`alignment/crandall-et-al-2018-natcomms-…\` | <headline from the findings> ([findings](superpowers/specs/2026-10-17-hybrid-populations-findings.md)) |`, using the exact file names under `papers/alignment/` (`ls papers/alignment`).
- `docs/reproducibility.md`: a summary row `| 42 | Booker, Guo, Si, Shirado; hybrid | … | [RR-HY-01–NN](#rr-hy-01) |` beside row 43; a `### Milestone 42 — hybrid human–machine populations, \`hybrid\`` subsection beside milestone 43's, with RR-HY-01 for provenance and one entry per scored claim in the same four columns; and `[hybrid]: superpowers/specs/2026-10-17-hybrid-populations-findings.md` beside `[misaligned]:`.
- `docs/roadmap.md`: in the alignment program bullet (line 435), after the milestone 43 clause add "hybrid human–machine populations, done <date> as milestone 42 `hybrid` with Booker's, Guo's and Si et al.'s bots and Shirado's coloring game ([findings](superpowers/specs/2026-10-17-hybrid-populations-findings.md));" and remove "hybrid human–machine populations" from the remaining list.
- `docs/studies/2026-10-11-alignment-program.md`: item 42's row (line 82) gains `**done <date>** as milestone 42 \`hybrid\` ([findings](../superpowers/specs/2026-10-17-hybrid-populations-findings.md))`.
- `README.md`: after the Misaligned AI section (before `### Emergent polarity`, line 3962), a `### Hybrid populations (Booker 2023, Guo 2023, Si 2025, Shirado & Christakis 2017)` section in the same shape: the model in two sentences, the eight presets by what they show, the headline results, the three sweeps, links to the findings and the design.
- Memory: write `hybrid-milestone.md` (what landed, the merge commit once merged, the headline verdicts, the Si integer-division finding and the docking result), update `model-queue.md` (milestone 42 done; remaining 41, 44, 2, 26) and add the pointer line to `MEMORY.md`.

- [ ] **Step 5: Full verification**

Write `$SCRATCH/m42/verify.sh` and run `bash $SCRATCH/m42/verify.sh 2>&1 | tail -40`:

```bash
#!/bin/bash
set -euo pipefail
cd /Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/misaligned
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test -p sugarscape-core 2>&1 | tail -3
wasm-pack test --node crates/sugarscape-wasm 2>&1 | tail -3
(cd web && npm run wasm >/dev/null && npx tsc --noEmit && npx vitest run 2>&1 | tail -4)
cargo fmt --manifest-path survey/Cargo.toml --check
cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings
python3 -m unittest discover -s survey/war1_study -p 'test_*.py' 2>&1 | tail -2
python3 -I -m unittest discover -s survey/hybrid_si -p 'test_*.py' 2>&1 | tail -2
git diff --quiet main -- Cargo.toml Cargo.lock && echo "root manifest unchanged"
```

Expected: all green, `caching-parity` and `spatial-parity` included, and the last line printed. (The native-parity vitest files are slow; if they time out under load, rerun them alone after `cargo build --release -p sugarscape-cli`.)

- [ ] **Step 6: Commit**

```bash
git add docs README.md crates/sugarscape-core/src/hybrid/presets.rs crates/sugarscape-core/src/titles.rs web/src/episodes/test-fixtures/foraging-source-compatibility.json
git commit -m "hybrid: findings, catalog and docs for milestone 42

Claude-Session: https://claude.ai/code/session_01X83MHaPG46PZwAy1U5mykR"
git push
```

The branch is then ready for the whole-branch review and superpowers:finishing-a-development-branch (merge to main, push, CI, Pages).

---

## Self-review

**Spec coverage.** Engine (population, bots, Fermi imitation, step unit, payoff accumulation): Tasks 1 and 4–5, with the step units in Task 7's `Runner`. World `booker` (game, bot kinds, chain, closed forms, replicator equations, α_c, lattice and Barabási–Albert runs, `sweep_steps`): Tasks 2, 4, 7, 12. World `guo` (replicator phases, master equation, Algorithms I and II, `frozen` and `per_interaction`, Figs. 6–7): Tasks 3, 4, 7, 12. World `si` (three frameworks, lattice and well-mixed, the code's payoffs and update order, `per_round`, `journal_n` and `preprint_n` as sizes, the integer-division default and `float` switch): Tasks 5, 7, 12. World `coloring` (graph, bot, reconstructed human, solution count, Extended Data Fig. 2): Tasks 6, 7, 12. Decision rules B1–B6, G1–G5, S1–S4 (S2 and S4 twice), C1–C2 and every evaluation row: Task 12. Architecture and registration: Tasks 1, 7, 8, 9. Configuration and live changes: Task 7. Measurement, the pilot and the docking build: Tasks 11–12. Playground (kind, world picker, view, inspector, reference panel, presets): Tasks 8 and 10. Testing (Fig. A1, Eq. 2 = E[Eq. 1], closed forms and errata, the classification, θ = 1 mixed bots, graphs, placement, the coloring counts and rule, determinism, goldens, fixtures, the literal inventory, web tests): Tasks 1–10. Process (findings, register, catalog, independent review): Task 13. The amendments are Task 1 Step 1.

**Placeholder scan.** The `0x…` strings in Task 8 Step 7 and Task 10 Step 6 are golden fingerprints that Task 8 Step 6 prints; the findings' headline, the register's entry count and the preset results come from the measured survey in Task 13. No TBD or TODO.

**Type consistency.** `Population` and its methods (Task 1) are used with the same names and signatures in Tasks 4, 5 and 7. `booker::{Rules, play, payoff, init, micro}`, `guo::{Rules, BotAction, init, payoff, payoff_stored, micro_network, expected, micro_well_mixed}` and `si::{Rules, init, payoff_graph, payoff_stored, payoff_well_mixed, round}` match Task 7's `booker_rules`, `guo_rules`, `si_rules` and `Runner::step`. `coloring::{Rules, Session, colorings, counts, convergence_steps}` match Tasks 7, 8 and 12. `Chain`, `Replicator` and the α_c functions (Task 2) and `Plane`, `Phase`, `phase_numeric`, `master_roots` (Task 3) match Task 12's calls. The inspection keys of Task 8 (`panel`, `node`, `bot`, `strategy`, `action`, `degree`, `payoff`, `cooperation`, `replicator`, `phase`, `color`, `conflicts_here`, `solutions`, `conflicts`, `time`, `solved_at`, `world`, `tick`, `x`, `y`, `reference`, `agent`) are Task 10's `HybridInspection` fields. The series names of Task 7 are the keys of Task 10's charts and the sweeps' metrics in Task 9. Config field names and serde tags are the same in Rust (Task 7), the sweeps (Task 9), TypeScript (Task 10) and the survey (Task 12).

**Review Focus.** Each of the five lines names its pinning test: Task 1's `isolated_nodes_have_no_partner_and_mean_payoffs_of_zero` and `a_population_without_humans_reports_nan`, Task 2's `the_chain_is_finite_at_strong_selection`, Task 4's `guo_isolated_humans_do_nothing`, Task 6's `a_finished_session_stays_finished`, Task 7's `a_world_without_humans_runs_and_fingerprints` and `a_finished_coloring_world_holds`, Task 8's `inspection_reads_nodes_padding_and_refuses_outside`.
