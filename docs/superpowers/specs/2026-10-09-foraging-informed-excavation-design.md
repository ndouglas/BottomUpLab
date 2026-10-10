# Cost-aware informed excavation for construction-foraging

**Date:** 2026-10-09

**Status:** the user approved the written specification after commit `981c894`. The user approved the [implementation plan](../plans/2026-10-09-foraging-informed-excavation.md) on 2026-10-10; subagent-driven execution has started.

**Context:** [F5 diagnosis](../../studies/2026-10-09-foraging-5-diagnosis.md), [local-yielding design](2026-10-09-foraging-local-yielding-design.md), [verified traffic increment](../../studies/2026-10-09-foraging-local-yielding-verification.md), and [F4 construction design](2026-10-06-foraging-4-construction-design.md).

## 1. Intent and scope

Let an empty worker pursuing a privately remembered or recruited food site consider excavation before exhausting open exploration. Compare a candidate opening with the worker's existing travel alternative, include the paid cost of digging and spoil transport, and distinguish optimistic continuation from known geometry. This addresses the diagnosed exploration-first policy limitation; it does not assert a defect in the retained policy or promise better scientific delivery.

Use one-cell lookahead and reconsider on each eligible opportunity. Preserve private maps, local observations, tagged food/spoil conservation, two-worker capacity, one paid action per opportunity, seeded replay, and atomic ticks. Finish loaded jobs before assessing excavation. Role allocation, relay transport, shared maps, collective benefit, tunnel commitments, backfilling, widening, parameter fitting, and scientific execution are outside this increment.

The selected alternative to this design would permit only privately proven shortcuts; that would exclude many openings into unseen terrain. Random early digging would weaken the connection to learned geometry and transport costs. The approved design instead makes its uncertainty explicit and keeps its numerical choices fixed before implementation.

## 2. Explicit policy boundary and compatibility

Add `construction::ExcavationPolicy` with serialized identities `exploration_first_v1` and `informed_cost_v1`. Traffic and excavation policies are independent typed choices.

Add `World::new_with_policies(setup, seed, traffic_policy, excavation_policy)` and `run_with_policies(setup, seed, options, traffic_policy, excavation_policy)`. The runner returns a new serialize-only `ConstructionPolicyEpisode` with schema `foraging-construction-policy-episode-v1` and these fields:

- `schema`, `traffic_policy`, `excavation_policy`, and the existing `episode` payload;
- `traffic_summary`, `traffic_snapshots`, and `traffic_snapshot_bytes`;
- `excavation_summary`, `excavation_snapshots`, and `excavation_snapshot_bytes`.

Both policy identities appear in excavation summary/snapshot views. The existing traffic views retain their existing wire shape and traffic identity; the enclosing construction-policy wrapper provides excavation provenance. The base physical episode alone is not revised-policy provenance.

Keep `World::new`, `run`, `World::new_with_traffic`, and `run_with_traffic` on `exploration_first_v1`. Preserve their signatures and existing serialized output, decisions, RNG consumption, computation counts, and work totals. Do not change `Setup`, the five CPFA parameters, existing episode/view schemas, or `ComputeCounts`. In explicit legacy-excavation worlds the new excavation diagnostics remain zero/empty and consume no draws or searches.

For either traffic policy, `run_with_policies(..., exploration_first_v1)` must reproduce the corresponding existing runner's base episode, traffic views, and byte counts exactly. New diagnostics have their own serialization and accounting. Unknown serialized policy identities are errors, never implicit defaults.

The F5 collector continues calling ordinary `construction::run`. Retain all three immutable manifests, protocol, validators, historical report goldens, and saved-analysis behavior. No new scientific campaign, integration, push, CLI, browser, WASM interface, or generic policy registry is part of this work.

## 3. Eligibility and decision order

Keep fresh current/cardinal observation and private learning first. Preserve terminal food/nest/spoil actions and the existing nonterminal traffic hold before controller dispatch.

Assess informed excavation only inside the existing `Departing` branch when the worker has `site: Some(site)`, empty hands, and no active food find. A recruited label supplies a destination, not terrain classifications or knowledge of food still being present. Uninformed departure, ordinary searching, empty return, food return, and spoil hauling keep their current controller priorities.

Preserve pursuit's existing immediate rules before assessment: a worker already at its site enters searching, and a privately KnownSolid protected site abandons that target normally. An eligible assessment then occurs before pursuit selects a known open route, an exploration frontier, or its legacy excavation fallback.

If an improving candidate exists, request one ordinary route step to its selected approach, or one Dig if already at that approach and the fresh observation confirms the face is solid and diggable. Clear the old frontier and set the selected face for this opportunity. The action's validity, work category, and traffic handling use existing transactions. A locally blocked approach uses the selected traffic policy; congestion is not a reason to invent a different dig. Never combine travel, digging, disposal, or pickup in one opportunity.

If no candidate improves the estimate, execute the existing informed-pursuit logic. Its frontier/face commitments and exploration-first fallback remain available. Thus cost rejection does not globally prohibit the original fallback digging after exploration is exhausted; diagnostics distinguish proactive cost-selected digs from those legacy fallback digs.

Recompute the comparison on every eligible opportunity, even when a face was previously selected. There is no retained tunnel plan or new route reservation. Successful Dig still opens one cell without moving, creates exactly one carried spoil token, clears the face/frontier, and suspends the food intent. Spoil disposal retains the site and resumes that ordinary intent; the next eligible opportunity assesses the newly learned geometry at the worker's then-current position. It need not return to the old approach merely because the estimate included that return.

## 4. Private distance fields

All estimates use the worker's post-observation private classifications and supplied outlet coordinate. They use cardinal unit edges, ignore occupancy, and never read researcher terrain, access/connectivity, remote workers, hidden food, other maps, or peer intentions.

For an assessment, compute/reuse at most three grid-bounded distance fields:

1. **Known origin field:** shortest distances from the current position through KnownOpen cells.
2. **Known outlet field:** shortest distances from the outlet through KnownOpen cells. An Unknown outlet provides no known spoil route; a privately solid outlet is an invariant error under the validated setup.
3. **Optimistic site field:** shortest distances to the site through KnownOpen and Unknown cells. Every remembered solid cell, whether protected or diggable, blocks this field. If the site itself is remembered solid, the field is empty; the special candidate opening of that exact site is handled below.

Unknown cells in the third field are assumed open only for this estimate. Do not write those assumptions into the learned map or use an optimistic route for actual movement. A route through Unknown may fail when locally inspected and may need further paid excavation; those future digs are not predicted by this one-cell lookahead.

Enumerate each remembered diggable face `f` and each cardinal KnownOpen approach `a` reachable from the origin. A candidate pair is eligible only when `a` also has a finite known route to the outlet. Remembered protected faces, unknown faces, unreachable approaches, and unobserved eligibility never authorize excavation.

Let `o(f)` be zero when `f == site`; otherwise it is one plus the minimum finite optimistic site distance among `f`'s cardinal neighbors. If there is no such neighbor, the continuation is unavailable. This is the shortest optimistic continuation from the candidate face after opening that face alone. All other remembered solids remain blocked. It can be obtained from the shared site field, without a separate BFS for every face.

Require the opening itself to improve estimated onward geometry from its approach: `1 + o(f)` must be strictly less than the pre-opening optimistic site distance at `a`, or that pre-opening distance must be unavailable. Otherwise exclude the pair. This prevents crediting a cheap unknown continuation that was already available without the dig. It is an optimistic geometric improvement test; the full paid-cost test below is still required.

Occupancy affects only the actual next movement and its normal traffic selection. Remote occupancy must not change estimated costs or selected candidates.

## 5. Fixed comparison and tie rules

Use checked nonnegative integer costs in paid-opportunity units. For an eligible pair `(f, a)` with finite continuation:

```text
approach_moves    = known distance from current position to a
spoil_leg_moves   = known distance from a to outlet
continuation_moves = o(f)
estimated_cost   = approach_moves
                 + 1                        # Dig, without moving
                 + spoil_leg_moves          # carry spoil to outlet
                 + 1                        # DisposeSpoil
                 + spoil_leg_moves          # estimated return to approach
                 + 1                        # step from approach into opened face
                 + continuation_moves       # optimistic onward travel to site
```

The known spoil round trip is a conservative modeled charge using the pre-opening private map. The new opening might shorten it; that saving is not credited. Actual resumption may take a different route. The onward continuation is optimistic. Their sum is a heuristic comparison, not a proven lower/upper bound on total actual work.

The **reference cost** is the shortest KnownOpen origin-to-site distance when finite. Otherwise use the optimistic origin-to-site distance, when finite. Otherwise record no finite reference (`None`, meaning unavailable). This models existing known-route pursuit first, with optimistic exploration as the estimate when no known route exists; it does not reproduce the legacy frontier heuristic's exact future trajectory.

Pick the smallest finite candidate cost. Select it only if it is strictly less than a finite reference, or if the reference is unavailable. Equality retains ordinary pursuit. There is no multiplier, probability of digging, adaptive threshold, amortization over future trips, or benefit assigned to other workers. A costly dig can therefore lose even when it appears geometrically shorter.

Keep tied minimum-cost candidate pairs in lexicographic order using the repository's `Pos` ordering: face first, then approach. When an improvement is selected, choose uniformly with the existing checked choice convention, including a draw for a singleton. Consume no new selection draw when no improvement exists. Ordinary route-step draws, including singleton movement choices and local-yielding tickets, follow that selection draw. Estimates, rejection, and views consume no random draws. Invalid/exhausted draws are contextual errors and never fallback values.

The diagnostic best candidate is the first ordered minimum-cost pair, determined without draws. The selected candidate may be another tied pair and is recorded separately. No claimed winner is inferred from an unfavorable or favorable F5 result.

Example: after passing the geometric-improvement test, an adjacent face with a known three-move outlet leg and four optimistic onward moves costs `0 + 1 + 3 + 1 + 3 + 1 + 4 = 13`. It beats a known route of 20 moves, ties and loses against 13, and loses against 12. This explicitly charges digging, disposal, and spoil travel before valuing the shortcut.

## 6. Diagnostics and bounded recording

Keep new diagnostics in private tick-owned state. Add observational `excavation_summary()` and `excavation_snapshot()` methods that validate and clone state without sensing, learning, expiry, route work, or draws. Summary/snapshot worker rows are ascending-ID ordered and carry both policy identities at the enclosing view level.

Each worker has checked counters `assessments`, `faces_scanned`, `candidate_pairs`, `selected_opportunities`, `initiated_digs`, `distance_calls`, `distance_visits`, and `peak_queue`. Face scans count every remembered solid entry examined, including protected faces and faces without a reachable approach. A candidate pair counts once for each remembered diggable face/reachable KnownOpen approach examined, before outlet/continuation filtering. Distance calls/visits count actual new-policy field computations and dequeues, including initial seeds; queue peaks aggregate by maximum. Any reused field is charged once. Existing physical navigation continues reporting its work in existing `ComputeCounts`; never count the same field computation twice or hide an assessment BFS there.

`selected_opportunities` counts accepted proactive choices, including travel, blocked waits, or yielding toward the chosen approach. `initiated_digs` counts only successful paid Digs caused by those choices, not fallback Digs. Sum worker counters with checked addition, except peak queue by maximum. Enforce `initiated_digs <= selected_opportunities <= assessments <= work.opportunities`, `initiated_digs <= work.digs`, and zero diagnostics under `exploration_first_v1`.

Retain each worker's `last_assessment: Option<Assessment>` rather than an unbounded decision log. An assessment contains its one-based tick, site, optional reference cost, examined-pair count, optional best candidate, and optional selected candidate. A candidate records face, approach, the three move components above, optional pre-opening optimistic distance at the approach, and estimated cost. No candidate is selected if its cost fails the comparison. Selected and best costs match; the selected pair is an eligible tied minimum. A best candidate without a selected candidate records a rejected comparison; no best candidate records no finite eligible candidate. Later nonassessment opportunities retain this last record, with its original tick. Initial state has none. Validate current eligibility during selection; later views validate the historical record's bounds, tick, arithmetic, and comparison without requiring its old face still to be solid. Views do not expose assumed map classifications as learned facts.

Base, traffic, and excavation snapshots share every existing scheduled checkpoint, including initial/final frames once. Disabled recording emits all three snapshot lists empty and zero bytes. Count compact JSON separately for each list and enforce the checked combined base-plus-traffic-plus-excavation snapshot limit of 64 MiB. Preserve existing base/traffic byte definitions. Final summaries and wrapper metadata remain outside the snapshot sum; this is a serialized-byte bound, not a resident-memory or timing guarantee.

At most three fields of at most `N = width * height` entries and one queue are required per assessment, with at most four approach pairs per remembered face. Candidate enumeration is bounded by `4N`; retain only tied minima and the compact last assessment. Reuse existing bounded geometry conventions. No per-candidate BFS, full tunnel search, new dependency, new numeric configuration, or lowered existing world/run bound is required.

## 7. Atomicity and limitations

Clone excavation policy/state/counters/records with the candidate world and RNG. Any late worker error rolls back physical state, private maps, advice, both policy diagnostics, and all draws together. Validate cost arithmetic, policy/state consistency, candidate geometry and fresh Dig eligibility; return descriptive tick/worker errors. A recording-budget failure yields no successful partial episode.

The policy may reject useful multi-cell tunnels because other remembered solids block lookahead, or accept a bad opening because Unknown is optimistically traversable. Conservative spoil charging can miss beneficial openings. Reassessment can switch tied faces; no persistent excavation crew or completion guarantee exists. Local yielding still offers no universal fairness or deadlock freedom. These are explicit model limits, not reasons to tune on registered outcomes.

## 8. Engineering acceptance and handoff

1. Preserve all existing F4/F5 and traffic tests, the exact legacy engineering episode/RNG fixture, and historical report goldens. Compare both traffic policies under explicit legacy excavation with their current APIs. Capture a small complete local-yielding engineering baseline before changing production behavior if existing coverage does not already freeze its full episode and next RNG state.
2. Prove the three fields and the one-cell continuation on small private maps, including Unknown, remembered solids, a solid site opened by its own candidate, protected faces, disconnected cells, and invalid coordinates. Assert bounded scans and exact computation counts.
3. Demonstrate an informed legal fixture digs before exhausting reachable open frontiers and before reaching a known open site when its estimate wins. Use ordinary paid observations/transactions to establish knowledge; controlled intent fixtures are labeled engineering fixtures, not seeded scientific trajectories.
4. Reject higher and equal costs, reject openings that do not improve optimistic geometry over the already available continuation, charge both spoil legs and disposal, reject unknown/unreachable outlet routes, and check no-candidate/no-improvement draw counts. Assert ordered ties, singleton selection, and exact RNG consumption on acceptance.
5. Show that existing at-site/protected-site rules, searching pickup/phase priorities, terminal arrivals, traffic holds, and loaded food/spoil jobs precede or bypass assessment. Confirm fallback digging remains available and is not counted as proactive excavation.
6. Complete Dig, tagged spoil hauling/disposal, and resumed informed pursuit with exactly one paid action each; preserve conservation, departure/find/advice semantics, and capacity. Change a formerly selected face or optimistic continuation through fresh observations and prove reconsideration rather than stale commitment.
7. Change only unseen terrain, hidden food, remote occupancy, or another worker's map while preserving the decision worker's private state/local view; decisions, estimates, and draws must remain identical. Optimistic searches must not alter knowledge.
8. Exercise both traffic choices with proactive approach travel, congestion, yield/hold, and spoil delivery. Prove a late failure after an earlier selected action and real RNG consumption rolls back everything; include malformed draws, overflow, and diagnostic invariant errors.
9. Check new policy provenance, legacy wrapper equivalence, observational purity, replay on the supported platform, initial/final paired recording, disabled snapshots, and combined byte-budget boundaries.
10. Run fresh workspace and survey suites, their formatting checks, core/survey all-target Clippy, and the source-include/fingerprint audit after changed production inputs. Confirm immutable manifests/protocol and existing saved F5 reanalysis remain exact. Use scripted fixtures and existing engineering seeds 7/8; do not run scientific seeds 10001..10040 or create revised campaign results.

After written-spec approval, write a staged implementation plan and obtain its review before execution. Preserve the user's subagent-driven implementation preference, with independent task and final reviews. Use a new plan-specific ignored SDD workspace and retain this feature branch/worktree for subsequent authorized integration. Do not reopen completed traffic tasks or reuse their archived workspace as active execution state.
