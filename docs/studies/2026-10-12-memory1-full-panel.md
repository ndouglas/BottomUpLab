# Memory1 pricing: full coverage repairs all eight saved pairs, with two endpoint regimes

**2026-10-12; bounded saved-table counterfactual, twelve new arms and four preserved arms reused.** Full coverage repairs the policy errors in all eight terminal Both sources against the relevant frozen rival maps. When both tables respond, three pairs become uniformly competitive and five retain higher-price steady outcomes from the reference state while passing the all-state fixed-map best-response checks. The competitive endpoints of the earlier two-case probe therefore do not generalize to the entire saved panel.

This extends the [two-case operator probe](2026-10-11-memory1-operator-probe.md), alongside a separately approved [native-history study](2026-10-12-memory1-native-history.md), under the [research method](../research-method.md). It does not rerun the component campaign, restart a native world or change the earlier evidence. The [constructed counterexample](2026-10-12-memory1-generality-counterexample.md) already rejected universal competitive collapse across arbitrary initial tables; this panel additionally observes both endpoint regimes among default-generated archived sources.

## Fixed design and exposure

Use all eight preserved terminal Both Q/R tables from the component experiment. Their source status remains five native-finished cases (2/3/4/5/6) and three horizon-censored cases (1/7/8). Reuse the completed frozen_full and coupled_full arms for cases 3/4 with their original snapshot/sample/audit hashes. Execute only cases 1/2/5/6/7/8 in those two modes: twelve new arms, 512 blocks of 225 ascending states, 115,200 microsteps per arm.

The unchanged reviewed operator uses native binary64 alpha .15 and gamma .95, fifteen actions, simultaneous prewrite targets for both firms, exact-lowest row maximizers and passive R tracking. Frozen mode uses the original terminal rival maps; coupled mode uses evolving maps. There is no world, RNG, exploration, native stopping or oracle value in learning. The two modes differ only in their declared rival inputs; this is a temporary deterministic table experiment, not faithful native training.

New exposure is exactly 1,382,400 microsteps and 41,472,000 Q writes, with equal R writes. The sixteen full arms combine 55,296,000 Q writes, of which 13,824,000 are previously completed evidence rather than new execution. No old local arms, new seeds, tuning, extensions or scientific retries were added. Six fixed readouts at blocks 0/1/8/32/128/512 yield 72 new and 96 combined snapshots; 1,539 samples per arm yield 18,468 new and 24,624 combined frames.

Primary population is all 225 states and both firms, equally across eight sources. Define `E=(1-native gamma)*mean(Vstar-Vpi)`. This all-state average differs from the earlier component experiment's learned-cycle average. Strict pass requires all required raw discounted-gain upper bounds<=1e-8 with inherited certification/agreement gates. State 16 and the original source cycle are explicitly selected secondary populations. Numerical failures and source censoring remain visible.

## All-state results and reference environments

| Mode and rival reference | Initial mean all-state E | Block 512 mean E | Strict pair passes |
| --- | ---: | ---: | ---: |
| Frozen full, original rivals (also current target rivals) | .00416231400844 | numerically zero | 8/8 |
| Coupled full, current learned rivals | .00416231400844 | numerically zero | 8/8 |
| Coupled full, original terminal rivals | .00416231400844 | .00971207342580 | 0/8 |

All initial pairs fail the all-state strict test. Against the fixed original rivals, full coverage supplies two independently optimized responses per source. Those frozen adaptations do not establish a mutually responding pair or a common adapted-pair cycle. Against the coupled endpoint maps, both firms pass from all 225 states. The current-reference repairs hold in both source strata (5/5 native-finished and 3/3 censored), without upgrading a censored source to native convergence.

Every coupled endpoint remains vulnerable when evaluated against its original terminal rivals; maximum raw discounted gain there is about 2.493296042. This reference dependence matters: changing both policies changes the game each policy is answering. Neither reference can be silently substituted for the other.

## Competitive and higher-price coupled endpoints

A constant competitive endpoint requires both complete maps to be constant and their joint action pair to be mutual lowest-argmax best responses in the native discrete stage game. Record the continuous competitive benchmark separately (~1.47293). This definition was fixed before measurement.

| Source | Source status | Complete coupled maps | Steady joint actions from state 16 | Steady prices |
| --- | --- | --- | --- | --- |
| 1 | censored | nonconstant | (5,4) | (1.621460714,1.582713571) |
| 2 | native-finished | nonconstant | (5,3) | (1.621460714,1.543966429) |
| 3 | native-finished | constant competitive | (1,1) | (1.466472143,1.466472143) |
| 4 | native-finished | constant competitive | (1,1) | (1.466472143,1.466472143) |
| 5 | native-finished | nonconstant | (6,5) | (1.660207857,1.621460714) |
| 6 | native-finished | constant competitive | (1,1) | (1.466472143,1.466472143) |
| 7 | censored | nonconstant | (4,4) | (1.582713571,1.582713571) |
| 8 | censored | nonconstant | (3,4) | (1.543966429,1.582713571) |

For cases 1/2/5/7/8, both steady prices exceed the continuous benchmark. These maps prescribe other actions off path; they are not high constant-price stage-game profiles. Their complete current-map audits pass for both firms in every state, giving stronger conditional fixed-map support than a cycle-only deterrence check. The state 16 path describes a selected starting history; it does not imply high prices from every state.

The new all-state certification belongs to these operator endpoints. The original seven cycle-passing pairs still fail their original shared reachable support and global tests. No earlier cycle result is retroactively relabeled as global optimality, and no elevated price is dismissed merely because another source repairs to competition.

## Interpretation and remaining boundaries

The bounded repair result extends across all eight archived Both tables. Competitive collapse does not: five coupled endpoints retain higher-price outcomes with complete frozen-map best responses. Coverage can remove decision errors without selecting a unique price regime. This separates imperfect value estimates from the possibility of supported higher-price strategies.

The result is conditional on these source tables, exact deterministic operator, state order, feedback rule, native payoff grid/discount and finite cap of 512 blocks. It does not identify typical basins, source/RNG/parameter generality, native exploratory learning convergence, real-market behavior or population prevalence. All-state checks judge committed greedy maps at a checkpoint, not continuing stochastic adaptation. No novelty is claimed; the prior study's primary-source literature comparison remains the context.

## Evidence, verification and lifecycle

The source/method/task gates passed before the root's single launch. New helper 27 synthetic tests passed; the unchanged operator 22 tests and archived numerical-method 27 tests passed in the explicit Python 3.13.5/NumPy 2.3.2 environment. Intermediate sampled Q/R rows, checkpoint boundaries, continuation maxima (including prewrite self successors), complete caches, repeated-row linkage and cumulative policy-change histories were verified without replaying learning.

Independent saved-data verification checks 96 snapshots, 24,624 sampled frames and 738,720 scalar update cells; all 192 audit references/384 firm readouts, 149 distinct saved certificate contexts, 596 candidate arrays and 134,100 exact Bellman residuals; all 32 paired 450-row comparisons and twelve source strata. Failed diagnostic assumptions in the independent checker are preserved separately from scientific outcomes. Two actual saved-only analyses are byte-identical across 145 scientific files; only runtime receipts differ.

Launch manifest SHA-256: `e704224328e90b9fb05623943fdf3d04ddf2c189e9ba8cf8d8f90a6c08c59eef`. Both analysis reports SHA-256: `144178654f028f017f8f4a915399cb8962ec00474d81a505cc9996d4262e9dea`. Executed helper source: `eee05493061bd54bb052edb87e68d03d86239327`, preserving public prerequisite `a82e02cbb54a2f1f3c59218ad4a8c7417f49d9f2` through the old private source 72b920c and panel commits.

The complete source, verified Git bundle, frozen manifest, input/reference identities, outputs, tests and independent reviews are indexed in `survey/out/collusion-memory1-full-panel-20261012/`. The bundle was restored and its complete tree and all nine retained source files matched the executed commit before cleanup. Only the owned clean panel worktree was removed normally, then its archived branch was retired. This helper was never merged into maintained code. Lifecycle receipts record preservation and cleanup; `/tmp` evidence, prior archives, other worktrees and original foraging evidence remain untouched. Documentation stays local; no push, publication, author contact or additional campaign is part of this study.
