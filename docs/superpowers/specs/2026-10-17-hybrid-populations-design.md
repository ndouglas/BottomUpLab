# SugarScape milestone — Hybrid human–machine populations: bots as a second species — Design

**Date:** 2026-10-17. **Milestone number:** 42 (queue item 42 in `docs/papers.md`), assigned at merge.
**Kind:** `hybrid`, displayed as **Hybrid Populations**.
**Design basis:** scope (shared engine plus Shirado's coloring game), statistical docking against
Si et al.'s published code, architecture A (one kind, four worlds) and the three design sections
below, all approved in chat on 2026-10-16/17.
**Program:** the [assistance and alignment program](../../studies/2026-10-11-alignment-program.md),
item 42 ("the second-species idea, already done on replicator dynamics; our lattice and network
games are the next step").
**Closest relatives:** milestone 13, Spatial Games (`2026-09-25-spatial-games-design.md`), for the
lattice Prisoner's Dilemma and its conventions; milestone 43, Misaligned AI
(`2026-10-14-misaligned-ai-design.md`), for worlds inside one kind, closed forms checked against
runs, per-world rules-panel variants and survey wiring. `graph.rs` already supplies
`barabasi_albert`, `gnp`, `gnp_sparse` and `Graph`; the new kind reuses them.

**Primary sources** (local copies in `papers/alignment/`, gitignored; reproduction briefs made on
2026-10-16 and kept with the session's notes):

- Booker, Miranda, Moreno López, et al., "Discriminatory or Samaritan — which AI is needed for humanity?",
  arXiv 2306.17747v2 (the latest version; no published version found). Donation-game Prisoner's
  Dilemma; Samaritan (always C), discriminatory (perfect intention recognition) and malicious AIs; a
  finite well-mixed Markov chain (Eqs 1–15), infinite-population replicator equations (Eqs 16–20),
  a 50×50 lattice and Barabási–Albert networks (Samaritan AIs only). No code.
- Guo et al., "Facilitating cooperation in human-agent hybrid populations through autonomous
  agents," iScience 2023, arXiv 2307.01123. Autonomous agents (AAs) cooperate with probability φ; R
  = 1, P = 0, a (T, S) plane; replicator dynamics (Eqs 3–5), the pairwise-comparison master
  equation (Eqs 6–10), lattice, Erdős–Rényi and Barabási–Albert runs. The supplement's Theorems 1–3
  and Algorithms I–II come from the arXiv source (`SI/2supp.tex`, saved under
  `papers/alignment/code/guo-et-al-2023-iscience-supplement-arxiv-source/`). No code.
- Si, He, Shen and Tanimoto, "Cooperative bots exhibit nuanced effects on cooperation across
  strategic frameworks," arXiv 2406.14913v1, published J. R. Soc. Interface 22:20240427 (2025; the
  journal PDF is not yet local). The Prisoner's Dilemma R = 1, S = −r, T = 1 + r, P = 0 with
  discrete, continuous and mixed strategies, well-mixed and on a Moore lattice. **Code:** Zenodo
  10.5281/zenodo.13957161 (22 C++ files, one per panel), saved under
  `papers/alignment/code/si-et-al-2024-jrsi-cooperative-bots-zenodo-13957161/`.
- Shirado and Christakis, "Locally noisy autonomous agents improve global human coordination in
  network experiments," Nature 545:370 (2017), with its Supplementary Information. A human-subject
  network 3-coloring experiment; the bot rule, the network generator, the solution count and
  Extended Data Fig. 2 are computational. Kearns, Suri and Montfort (Science 2006) originated the
  coloring experiment and is the game's reference.

**Cited, not reproduced** (catalog and findings only): Fernández Domingos et al., Sci. Rep. 12:8492
(2022), a lab experiment on delegation in the collective-risk dilemma; Ishowo-Oloko et al., Nat.
Mach. Intell. 1:517 (2019), the "machine penalty," human data (its OSF deposit, said to hold code,
was unreachable on 2026-10-16; retry); Crandall et al., Nat. Commun. 9:233 (2018), S++ and S#, whose
Supplementary Notes and the 2014 JAIR paper are now local for a later milestone.

## Goal and intended use

Reproduce the evolutionary game theory of hybrid populations, where fixed-behavior machines share a
population with imitating humans, faithfully enough to test each paper's claims one by one, and
then take the idea to the lattice and network games the alignment program names as the next step.

Four worlds share one engine:

1. **`booker`**: the donation game with Samaritan or discriminatory bots. Exact finite chain and
   replicator equations; lattice and Barabási–Albert runs with Samaritan bots.
2. **`guo`**: R = 1, P = 0 games on the (T, S) plane with probabilistic bots. Exact replicator phase
   diagrams and master-equation roots; lattice, Erdős–Rényi and Barabási–Albert runs; hub placement.
3. **`si`**: the r-family Prisoner's Dilemma in three strategy frameworks, well-mixed and on the
   Moore lattice, matching the published code, which we compile and run to dock against.
4. **`coloring`**: Shirado's 20-node, 3-color network coloring game with greedy noisy bots; the
   human players are a labeled reconstruction.

Several findings are in view before any measurement and are to be tested, not assumed:

- In Booker's donation game the Samaritan payoff difference does not depend on b (R − T = S − P =
  −c), so Eq 18 gives the same curve for every b, and h(0) = α·p > 0 means x = 0 is never a fixed
  point. Figures 2–3 vary b and the text says 0 is a fixed point at α = 0.5.
- Booker's printed G_D (Eq 14) and simplified α_c disagree with the general formulas they come
  from: G_D should be (N−1)!M!/(N+M−1)!, and α_c for equal β is 1 − e^{−β(T−R)}.
- Booker's network protocol, 10⁵ single updates on 2,500 nodes, is about 40 updates per node, and
  their own β = 0.1 snapshots are still changing at 10⁵.
- Guo attributes the network reversal in snowdrift (cooperation falls with φ on the lattice, rises
  on random and scale-free graphs) to hubs by inference. A discriminating test is possible.
- Si's code differs from the prose: the mixed strategy redraws an action per neighbor (the Fig. A1
  example draws once per round), the well-mixed N is 10⁴ (the prose says 500), and κ is a
  temperature on the lattice but an inverse temperature in the well-mixed files.

## Binding constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add a new kind and new goldens.
- One sequential path serves native and WASM. Follow the seeded RNG and portable arithmetic
  conventions: `f64` draws, `portable::exp_neg` for every Fermi probability (symmetrized so the
  argument is never positive), `portable::ln` where a log is needed, no `f64::exp`.
- The exact references (Booker's chain, fixation probabilities, closed forms; Guo's replicator
  thresholds and master-equation roots; the chromatic polynomial) take no RNG and are pure functions
  of the config, so tests check them without a world.
- Each world's defaults are its paper's literal setup (originator first). Where prose and code or
  two passages disagree, the default follows the stated priority below and the other reading is a
  named switch.
- Decision rules below are fixed before any measurement. A rule added after looking at results needs
  a dated amendment and a separate verdict.
- Root `Cargo.toml` and `Cargo.lock` do not change (they are compiled into core as provenance). The
  survey work lives in `survey/` and runs by manifest path.
- Paper PDFs and downloaded code stay gitignored in `papers/`. Si et al.'s C++ is compiled and run
  only in its own scratch directory, with no network, to produce reference CSVs; its raw outputs
  become receipts.

**Default priority when sources disagree.** Booker and Guo: the paper's equations and pseudocode
over prose. Si: the published code over the prose, because the code produced the figures; the
prose reading is the named variant.

## Source model and targets

### Engine (shared)

- **Population.** A `Graph` of nodes; each node is a human or a bot. Generators: complete
  (well-mixed), square lattice (von Neumann 4 or Moore 8; periodic or open), Erdős–Rényi with a
  target mean degree, Barabási–Albert with m (m = 2, mean degree 4, unless a paper says otherwise).
  Well-mixed worlds with an analytic payoff (Si's code, Guo's Algorithm I) compute expected payoffs
  from counts and never build the complete graph.
- **Bots.** A count or fraction; placement random (default), hubs (highest degree first, ties by
  node index) or leaves (lowest degree first). Bots never update and are never focal.
- **Humans.** Asynchronous Fermi imitation: pick a uniformly random node (Si, Guo networks: with
  replacement over all N nodes, acting only if human; Booker: one human) and a uniformly random
  neighbor as the role model; adopt the model's strategy with probability 1/(1 + e^{−β(π_model −
  π_focal)}). Whether a bot can be a role model is per world (below).
- **Time.** One Monte Carlo step (MCS) is N micro-updates unless the world's paper defines a step
  otherwise; the paper's own unit is the default and the other is a named switch.
- **Payoffs.** Sum over neighbors (all three papers' network runs) by default; mean over neighbors
  is a named variant. Payoffs are recomputed for the focal and the model at each micro-update.

### World `booker`

- Game: T = b, R = b − c, P = 0, S = −c, c = 1. Bot type: Samaritan (C), discriminatory (mirrors the
  partner's action) or malicious (D). Humans copy Samaritan and malicious bots; discriminatory bots
  are never role models (in Eq 6 they only enlarge the sampling denominator).
- Exact: T⁺(k), T⁻(k) (Eq 6), fixation probabilities (Eq 7), stationary all-C weight r/(1 + r) with
  r = ∏ T⁺/T⁻, the closed forms F (Eqs 9–11) and G (Eqs 13, 15, corrected 14). Replicator equations
  18 and 20, their fixed points by bracketing root scans, and α_c from the general formula.
- Networks: 50 × 50 von Neumann lattice with open boundaries; Barabási–Albert N = 1000, m = 2, ten
  networks × 20 runs. Samaritan bots placed at random, humans start C or D with probability ½. One
  micro-update per step, 10⁵ steps, the last 1,000 averaged, 30 runs (lattice). β ∈ {0.1, 1, 5},
  b = 2. Bots are role models. The N-update MCS is the named variant `sweep_steps`.

### World `guo`

- Game: R = 1, P = 0, T and S free. Bots play C with probability φ.
- Exact: replicator equation (Eq 3), interior root (Eq 5), the full-C / full-D / coexistence /
  bistable classification over (y, φ); the master equation (Eqs 6–9) roots for y ∈ [0, 5] by
  bracketing scans, with stability from the sign change.
- Well-mixed simulation (Algorithm I): N = 1000, one pairwise comparison per step, expected payoffs
  from the population state, bots counted as role models with the payoff of their action.
- Networks (Algorithm II): N = 10,000 total, mean degree 4, y = 0.95 (4,871 bots), K = 10; periodic
  von Neumann lattice, Erdős–Rényi, Barabási–Albert; one step is N micro-updates; a bot picked as
  focal redraws its stored action with probability φ (default, the pseudocode). Variants: `frozen`
  (drawn once) and `per_interaction` (redrawn for every game). 50,000 steps, the last 5,000
  averaged, 60 runs. Games from Figs 6–7: PD T = 1.3, S = −0.2; snowdrift T = 1.4, S = 0.5; stag
  hunt T = 0.6, S = −0.5. Fig. 7: ten bots on the hubs, ten at random, none, on Barabási–Albert,
  swept over φ with the Fig. 6 settings.

### World `si`

- Game: R = 1, S = −r, T = 1 + r, P = 0. Frameworks: discrete (pure C/D; a bot is C with
  probability θ, drawn once), continuous (s ∈ [0, 1]; π = Σ −r·s_i + (1 + r)·s_j), mixed (s copied;
  actions redrawn per neighbor interaction, the code; variant `per_round` draws one action per
  player per round, Fig. A1).
- Lattice: 100 × 100 Moore, periodic, N = 10⁴, payoff summed; κ = 0.1 as temperature (β = 10).
  Well-mixed: N = 10⁴ (variant `prose_n` = 500), expected payoffs from counts, β = 10^m.
- One round is N focal picks with replacement; only ordinary players act; one random neighbor or
  random other as model; 10⁴ rounds, the mean of the last 2,000, 100 runs. Ordinary players start
  uniform in [0, 1] (continuous, mixed) or as the code does (discrete). Bot fraction ρ is a
  per-node Bernoulli draw, as in the code. Cooperation is measured over ordinary players only.

### World `coloring`

- Graph: Barabási–Albert, 20 nodes, each new node with 2 links, regenerated per session; kept only
  if 3-colorable (rejection, ours; the paper says "by construction"). Three colors.
- Bot: decides every 1.5 s; with probability equal to its noise (0, 0.1, 0.3) picks a uniform color
  among all three; otherwise, if its color is not the least common among its neighbors, switches to
  the least common (ties broken uniformly at random, ours). Placement: 3 highest-degree, 3
  lowest-degree or 3 random nodes (degree ties by index).
- Humans (reconstruction, labeled): continuous-time greedy responders with exponential decision
  times of mean 1.59 s (Extended Data Fig. 1's median), errant rate e (default 0.041, Fig. 4's human
  mean) choosing a conflict-increasing color, otherwise greedy as the bot. Initial colors uniform
  (ours). Time limit 300 s; success is reaching a proper coloring.
- Exact: the chromatic polynomial at 3 by deletion–contraction (20 nodes is small).
- Extended Data Fig. 2 statistic: from a fixed initial coloring, a random node recolors to a color
  different from a random neighbor's until solved; mean steps over 100 repeats per graph.

## Decision rules

Fixed on 2026-10-17, before any measurement. "SE" is the standard error over independent runs;
"2 SE" for a difference uses the combined standard error.

### Booker

| ID | Claim | Holds if |
|---|---|---|
| B1 | Fig. 1, Samaritan: all-C weight switches on at M* | exact chain, N = 100, b = 2: the first M with weight ≥ 0.5 lies in [1, 8] at β = 0.1, in [50, 65] at β = 1, and no M ≤ 100 reaches 0.5 at β = 2.5 |
| B2 | Samaritan beats discriminatory at low β, the reverse at high β | the mean all-C weight over the Fig. 1 grid (M 0..100, b 1.1..3 by 0.1) is higher for Samaritan at β = 0.1 and higher for discriminatory at β = 2.5 |
| B3 | Figs. 2–3: lower β cooperates at lower b; x = 0 is a fixed point at α = 0.5 | judged on Eq 18 (Samaritan) as printed. Registered expectation: fails (curves identical across b; h(0) > 0). Eq 20 (discriminatory) is judged on the same two statements separately |
| B4 | Printed G_D and simplified α_c | errata: printed values differ from the direct product and from the root scan by more than 1e-9 relative; corrected forms agree within 1e-9 |
| B5 | Lattice: Samaritans raise cooperation at β = 0.1 as α grows and barely help at β = 5 | paper protocol: at β = 0.1, mean human cooperation ≥ 0.5 at α ∈ {0.25, 0.5} and ≤ 0.2 at α = 0.01; at β = 5, every α ≤ 0.5 gives cooperation ≤ 0.2 above the α = 0 run. Also reported under `sweep_steps`, as a variant row |
| B6 | Topology does not change the final state | at matched (α, β) with α ∈ {0.1, 0.2, 0.3, 0.4}, Barabási–Albert and lattice final cooperation differ by ≤ 0.1 |

### Guo

| ID | Claim | Holds if |
|---|---|---|
| G1 | Replicator phase diagrams (Figs. 3–4); snowdrift y_min = 0.4 | the numerical classification on a 101 × 101 grid of y ∈ [0, 5] and φ ∈ [0, 1] matches the closed-form thresholds at every grid point not within 1e-9 of a boundary; y_min = 0.4 within 1e-9 |
| G2 | Fig. 5, master equation | PD at K = 100: every stable root ≤ 0.05 for y ∈ [0, 5]; snowdrift at K = 100: the stable root rises with y at φ = 0.1 and falls with y at φ = 0.9 (monotone on a 0.05 grid, 1e-9 slack); K = 1: the φ = 0.9 root exceeds the φ = 0.1 root at every y ∈ (0, 5] in PD, snowdrift and stag hunt |
| G3 | Fig. 5 simulation dots | Algorithm I with N = 1000 agrees with the stable master-equation root within 0.05 at y ∈ {0.5, 1, 2, 5}, φ ∈ {0.1, 0.9}, K ∈ {1, 10, 100} for each game (single stable root cases; bistable cases started on each side) |
| G4 | Fig. 6 networks | slope of mean human cooperation against φ (φ ∈ {0, 0.2, …, 1}, least squares over the six means, 95% CI by bootstrap over runs): snowdrift negative on the lattice and positive on Erdős–Rényi and Barabási–Albert; PD positive on all three; each CI excludes 0 |
| G5 | Fig. 7: ten hub bots move cooperation, ten random bots do not | on Barabási–Albert over φ ∈ {0, 0.2, …, 1}: in PD and stag hunt, the ten-hub mean at φ = 1 exceeds the no-bot mean by more than 2 SE and the ten-hub slope against φ is positive (CI as in G4 excludes 0); in all three games the ten-random mean is within 2 SE of the no-bot mean at every φ |

### Si et al.

| ID | Claim | Holds if |
|---|---|---|
| S1 | Docking against the published code | at 27 lattice points (r ∈ {0.05, 0.1, 0.15} × ρ ∈ {0, 0.3, 0.5} × three frameworks) and 9 well-mixed points (r ∈ {0.05, 0.1, 0.15} × three frameworks, ρ = 0.3), our mean minus theirs is within 3 combined SE, or under 0.02 in absolute value |
| S2 | Fig. 1: bots raise well-mixed cooperation; continuous and mixed stay near θ up to r ≈ 0.1 at ρ = 0.5 | cooperation at each ρ ∈ {0.1, 0.3, 0.5} exceeds ρ = 0 at every r in {0, 0.04, …, 0.2}; at ρ = 0.5, continuous and mixed are ≥ 0.8 for every r ≤ 0.1 |
| S3 | Fig. 3: on the lattice, bots suppress cooperation in discrete and continuous for r < 0.13 and promote it in mixed at every r | the ρ = 0.5 mean is below ρ = 0 by more than 2 SE at every r < 0.13 on the grid for discrete and continuous, and above ρ = 0 by more than 2 SE at every r for mixed |
| S4 | Fig. 6: an interior optimum θ in mixed at r = 0.01 | at ρ = 0.3, the argmax over θ ∈ {0, 0.1, …, 1} lies strictly inside (0, 1) and exceeds the θ = 1 mean by more than 2 SE |

Evaluation rows (reported, not scored as the paper's claims): S3 under `per_round`; S2 under
`prose_n`.

### Coloring

| ID | Claim | Holds if |
|---|---|---|
| C1 | Extended Data Fig. 2: log chromatic polynomial and log mean convergence steps correlate at −0.990 | on 230 accepted graphs (seeded), Pearson r ≤ −0.9 |
| C2 | Mechanism sufficiency (reconstruction; not the human result) | e is fit on the control arm only (20 humans, 200 sessions per value, e ∈ {0, 0.005, …, 0.2}, the value whose solve fraction is nearest 0.67; reported beside the observed 0.041); the rule is judged at the fitted e and, as a second row, at e = 0.041. Holds if, at the fitted e, 10% central bots solve more often than 0% central and than 30% central, each by more than 2 SE, over 200 sessions per arm |

### Evaluation beyond the claims

- Guo's hub explanation: on Barabási–Albert snowdrift, compare random placement with summed and with
  mean payoffs; if the positive slope survives mean payoffs and random placement, degree
  heterogeneity in payoff accumulation, not bot hubs, carries it.
- Booker under the N-update step and to stationarity (B5 variant rows).
- The cited human results (Shirado's 17/20 against 20/30; Ishowo-Oloko's machine penalty;
  Fernández Domingos's 66.7% against 86.7% group success) are reported as human data beside the
  model results, never as reproduced.

## Architecture

- `crates/sugarscape-core/src/hybrid/`: `mod.rs` (exports), `config.rs` (schema, world enum,
  switches), `graph.rs` (lattice builders and placement on top of `crate::graph`), `engine.rs`
  (population, Fermi micro-update, MCS), `booker.rs`, `guo.rs`, `si.rs` (each the world's payoff
  rule and exact references), `coloring.rs` (game, bot, reconstructed human, chromatic
  polynomial), `stats.rs` (series and snapshot), `world.rs` (the `HybridWorld` the app drives),
  `presets.rs`, tests beside each file.
- Registered like milestone 43: `pub mod hybrid;` appended to `lib.rs`, a model kind, schema,
  presets and titles, sweep entries, native and WASM goldens, the caching identity fixture
  (`reviewedAdditions` gains `pub mod hybrid;`) and the core literal inventory if any
  `include_str!` lands outside `core/src`.
- Survey: `survey/` gains a `hybrid` report that runs every claim with these rules, multithreaded,
  and writes receipts; Si docking reads the reference CSVs from the scratch build.

## Configuration

One `HybridConfig` with `world` and per-world fields; the rules panel narrows the parameters per
world (the `schemaVariant` mechanism). Key fields: graph (complete, lattice, Erdős–Rényi,
Barabási–Albert; size; neighborhood; boundary), bots (count or fraction, placement, type or φ or θ),
game parameters (b; T and S; r), β, framework (si), step unit, payoff accumulation, and the named
variants listed above. Live changes rebuild the world from the current seed.

## Dynamics

Per tick, one MCS (or the world's paper step). Booker's chain, Guo's master equation and the
replicator equations are not simulated; they are drawn as references.

## Measurement and compute

- Series: human cooperation (fraction, or mean s), bot share, and, for coloring, conflicts and
  solved time.
- The survey runs every scored claim at the paper's sizes and protocol on the grid points the rules
  name. A pilot measures throughput first; if a protocol is still out of reach the run count is cut
  and the row is labeled reduced. Sizes and run lengths never change silently.
- Si docking: their code compiled as is, run on the S1 points with their run counts (or the reduced
  count used on our side, the same for both), CSVs kept.

## Playground

- The kind, with a world picker on the rules panel.
- A lattice or graph view: cooperators, defectors and bots in distinct colors, bots outlined;
  continuous strategies as a gradient; coloring shows the three game colors with bots marked.
- An inspector: node type, strategy, payoff, degree.
- A reference panel beside the live run: Booker's all-C weight and the replicator root for the
  current cell; Guo's phase and master-equation root; for coloring, the solution count.
- Presets with plain titles: Samaritan bots at low β; discriminatory bots at high β; snowdrift on a
  lattice against a scale-free graph; Si's mixed bots on the lattice; Shirado's central 10% bots.

## Testing

- Paper-supplied: Si's Fig. A1 worked example (both panels); Eq 2 equals the expectation of Eq 1;
  Booker's F and G closed forms against direct products (corrected G_D, printed G_D as an erratum
  test); Guo's thresholds against the classification; a θ = 1 mixed bot equals a discrete
  cooperating bot.
- Graphs: lattice degrees and wraparound or open edges; Barabási–Albert edge count and mean degree;
  Erdős–Rényi mean degree; hub and leaf placement order.
- Coloring: chromatic polynomial on paths, cycles, complete graphs and a wheel; the bot's
  least-common rule with seeded ties.
- Determinism, native and WASM goldens, the caching identity fixture, the literal inventory, the
  web tests for the rules panel and view, and the Pages build.

## Process

Plan, implementation, survey, a findings doc with per-claim verdicts, the reproducibility register
and a `docs/papers.md` queue update (catalog lines for the cited papers). An independent review of
rules and measurements before anything is called complete.
