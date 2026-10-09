# Private local yielding for construction-foraging

**Date:** 2026-10-09

**Status:** the user approved the traffic-first approach and requested this written specification. This draft awaits written-spec review; engineering implementation has not started.

**Context:** [F5 saved-only diagnosis](../../studies/2026-10-09-foraging-5-diagnosis.md), [registered results](../../studies/2026-10-09-foraging-5-full-run.md), [F4 construction design](2026-10-06-foraging-4-construction-design.md).

## 1. Intent and scope

Address sustained opposing traffic that arises when workers accept only immediately distance-decreasing steps and those cells are full. Preserve the private learned-map boundary, material conservation, two-worker cell capacity, one paid action per opportunity and atomic seeded ticks. Successful engineering evidence means a representative blocking fixture gains passage through legal local yielding; it does not mean universal deadlock freedom or a favorable scientific delivery result.

The user accepted two separate increments: traffic resolution first, then earlier informed excavation. This specification covers traffic only. Excavation remains exploration-first, and food/spoil job priorities remain unchanged except for the explicitly described travel pause. Dedicated roles, relay transport, shared maps, peer intentions, swaps, displacement, corridor widening, researcher connectivity and global occupancy-aware routing are outside scope.

The existing F4/F5 policy is a retained baseline. The revised behavior is explicitly selected and labeled; it must not silently replace the policy behind an existing manifest or archive.

## 2. Explicit policy identity and public boundary

Add `construction::TrafficPolicy` with two serialized identities: `shortest_only_v1` and `local_yielding_v1`. They are versioned supplied engineering policies, not biological calibrations.

Keep `World::new(setup, seed)` and `run(setup, seed, options)` on `shortest_only_v1`. Their signatures, existing `Setup`/`Parameters`, `Episode`/`Snapshot`/`Summary`/`WorkCounts` wire representations and emitted baseline bytes stay unchanged. The five CPFA parameters keep their present meanings. Baseline control decisions, RNG consumption, work/computation counts and observational sampling must remain identical.

Add explicit `World::new_with_traffic(setup, seed, policy)` and `run_with_traffic(setup, seed, options, policy)`. The latter returns a distinct `TrafficEpisode`, not an unlabeled substitute for `Episode`. Its schema is `foraging-traffic-episode-v1`; it contains policy identity, the existing normalized `Episode` payload, a final traffic summary and traffic snapshots aligned with its recorded base snapshots. A wrapper for `shortest_only_v1` supports labeled engineering comparisons; its inner episode must equal ordinary `run` exactly.

World traffic state and diagnostics are private tick-owned state. Observational `traffic_summary()` and `traffic_snapshot()` methods clone those diagnostics without sensing, drawing, learning or expiring advice. A revised world emits its policy identity in these views. The existing `summary()` and `snapshot()` remain physically compatible views, not standalone revised-policy provenance.

The F5 collector continues calling ordinary `construction::run`. Its three immutable manifests, saved resolver, schemas, strict validators and registered definitions remain unchanged. It must not accept the new wrapper as an F5 episode or strip its policy identity to collect revised results. A future scientific comparison needs its own exact policy-bound manifest and registration. This work adds no CLI, browser, WASM or generic model registry.

## 3. Fixed local-yielding choices

`local_yielding_v1` fixes these constants:

- two consecutive **paid blocked-route waits** before yielding is eligible;
- a yield-attempt probability of 0.25 on each subsequent eligible blocked opportunity;
- one paid hold opportunity immediately after a successful yielding move.

These values are supplied before implementation and are not fitted to F5 outcomes. There is no parameter sweep, adaptive tuning or exposed adjustable numeric parameter in this increment. Changing a constant creates a different policy version.

For each worker retain a blocked-wait streak, saturated at two, and a one-opportunity hold flag. Initialize them to zero/false without draws. The streak counts consecutive actual route-congestion waits, regardless of which route target produced them; it is not a claimed target-specific timer. Any move, hold, arrival, transaction, phase-transition wait or no-neighbor wait resets the streak. A normal paid route-congestion wait increments it up to two. Legacy workers never set a hold flag or execute yielding actions.

## 4. Decision order and allowed information

Every opportunity still starts with the ordinary fresh current/cardinal observation and private map learning. Terminal job actions take precedence: FoodReturning in the nest deposits food; EmptyReturning in the nest completes its normal empty arrival; SpoilHauling at the outlet disposes spoil. Clear pending traffic memory on these arrivals. Preserve their existing transactions, advice operations and draws.

If a nonterminal worker has a pending hold, clear it and spend exactly one opportunity on a traffic hold. Keep cargo, food phase, informed site and travel commitments. Consume no controller-phase, yield or step-choice draws on this opportunity. Learning still occurs. Pickup, phase transitions and route travel resume on the following opportunity; they are not combined with the hold.

Otherwise follow the existing food-trip, exploration, excavation and cargo dispatch. Apply the yielding rule only where that policy requests a route step and the private shortest route exists but every immediately distance-decreasing step is locally full. Ordinary free shortest steps use exactly the existing route selection. AtGoal and Unreachable retain their present handling. Wandering with no legal neighbor is not a yielding trigger. A geometrically reachable frontier remains an exploration target even when its next step is occupied; this increment does not turn congestion into permission to dig.

When a route is blocked and the worker already has two consecutive paid blocked waits:

1. Use the same private reverse-distance field for the current route goals. Do not use researcher terrain, remote occupancy, food truth or peer intentions.
2. Form yielding candidates from freshly observed, privately KnownOpen cardinal neighbors with fewer than two occupants and a finite private route distance to the same goals. Since normal decreasing steps are blocked, these candidates have nondecreasing distance.
3. Keep candidates with the smallest private goal distance among that yielding set. Keep north/south/east/west order for exact ties. Do not invent unknown routes or classify unseen cells.
4. If the set is empty, consume no yield or selection draw and retain the ordinary congestion wait. There is no legal capacity exception.
5. Otherwise draw the 0.25 Bernoulli ticket. A ticket strictly below 0.25 succeeds; equality fails. Failure is an ordinary paid congestion wait, with no step-selection draw.
6. On success, select uniformly among the tied candidates using the existing checked choice convention, including its draw for a singleton. Execute one ordinary legal Move, retaining the current cargo and intended destination. Clear the streak and set the hold flag.

The third consecutive blocked-route opportunity can therefore yield: the first two opportunities must actually have paid waits. The new draws occur after ordinary controller draws leading to that route request. No new draws occur on free shortest travel, during a hold, at an arrival, before the streak threshold or when no yielding candidate exists. Invalid or exhausted scripted draws produce contextual errors, never fallback random values.

All travel modes can yield: empty departure/search/return, FoodReturning and SpoilHauling. Loaded workers do not drop cargo, collect another item, dig, change destinations or publish advice by yielding. An existing commitment may later change under ordinary fresh observation or normal trip policy; yielding itself neither deletes it nor supplies new information.

## 5. Physical accounting and separate diagnostics

A yielding Move is one paid Move in its ordinary purpose category: departure, search, empty return, food transport or spoil transport. It is not free motion or a new food departure. A hold is one paid Wait in the matching existing congestion category. Thus the existing disjoint work equations and material transactions remain unchanged. Congestion totals under the revised policy include deliberate holds, so use the separate counters to distinguish those from forced waits.

`TrafficEpisode` contains `schema`, `policy`, `episode`, `traffic_summary`, `traffic_snapshots` and `traffic_snapshot_bytes`. Its traffic summary and snapshots carry the same policy identity. Worker traffic rows are in ascending worker-ID order. Each traffic snapshot has `completed_ticks` and those worker rows; its checkpoint must equal the corresponding base snapshot. The final traffic summary has aggregate counters and the final worker rows, including any pending hold.

Add checked aggregate/per-worker traffic counters:

- `blocked_route_opportunities`: route requests whose private decreasing steps were all locally full, including opportunities that subsequently yield;
- `yield_attempts`: Bernoulli tickets actually drawn;
- `yield_moves`: successful paid yielding moves;
- `hold_waits`: completed paid holds.

Count blocked route requests in both explicit policy modes; the legacy mode has zero attempts, yield moves and holds. Recording these diagnostics must not change baseline actions, RNG, existing compute counts or serialized base views. Aggregate counters equal checked sums of per-worker counters. Enforce `yield_moves <= yield_attempts <= blocked_route_opportunities`, `hold_waits <= yield_moves`, `blocked_route_opportunities <= work.opportunities`, `yield_moves <= work.moves`, and `hold_waits <=` the sum of the worker's three congestion-wait categories. The missing final hold of a yield on the last tick is a legal censored state, not an accounting error; terminal precedence can also consume a pending hold without paying one.

Traffic worker views retain worker identity, streak, hold flag and counters. Traffic snapshots align exactly with base completed-tick checkpoints; disabled recording emits neither set. They expose no private-map contents beyond the existing boundary and do not restore World state. Counts and geometry computations stay separate from paid actions. Reuse the existing route distance computation rather than silently performing an uncounted second BFS.

The base snapshot byte definition remains unchanged. The new runner also counts compact traffic snapshot JSON and enforces a checked combined base-plus-traffic snapshot limit of 64 MiB. Final summaries and input/wrapper metadata are outside that snapshot sum, as in the existing runner. Report both base `snapshot_bytes` and traffic snapshot bytes. Byte limits are not resident-memory guarantees. Preserve current grid, worker, tick, opportunity and material bounds; no new dependencies are needed.

## 6. Atomic ticks, errors and liveness limits

Keep ascending-ID scheduling and fresh observations after earlier workers' committed-in-candidate actions. A yielding action does not reserve capacity for other workers, move them or change their priorities. Capacity is checked again by the ordinary action transaction.

Clone traffic memory/counters together with world and RNG when preparing a tick. A late invalid draw, illegal action, accounting error or budget failure rolls back traffic state, physical state, maps, advice and RNG together. No successful partial tick or run is returned.

Validate policy identities at typed/serialization boundaries; do not silently map an unknown identity to legacy. Internal streaks above two, inconsistent hold/streak state, or invalid diagnostic sums are invariant errors with tick/worker context. A hold flag requires a zero streak. Legacy holds/attempts/yields are errors, while legacy blocked-route diagnostics are legal.

Seeded yielding may break a particular opposing queue, but offers no universal termination or fairness guarantee. A fully occupied region with no free local cell remains blocked. A worker may retreat, revisit a cell or encounter another queue. The policy leaves finite-horizon censoring and ascending-ID competitive bias explicit. Do not promise AlreadyOpen will win or require favorable F5 outcomes as acceptance.

## 7. Required engineering acceptance

1. **Baseline preservation:** retain all existing F4/F5 tests and report goldens. Add a complete engineering fixture captured before implementation, with exact serialized base episode bytes and RNG/action checks; default and explicit legacy runs reproduce it. Do not generate scientific F5 seed fixtures.
2. **Threshold and boundaries:** first two blocked opportunities pay waits; the third may attempt. Tickets below 0.25 yield and tickets equal to/above 0.25 wait. Assert exact draw counts, singleton selection and saturated streak behavior.
3. **No-candidate and normal travel:** full neighbors wait without yield draws; a free shortest step remains normal travel; unknown/solid/out-of-bounds candidates and unreachable goals are never escape steps. Preserve private-route invariants.
4. **Opposing queue with a free bay:** construct a small legal capacity-two fixture using ordinary observations and transactions. Script one worker's yield and hold so opposing traffic demonstrably advances. The legacy counterpart remains blocked over the same specified opportunities. Check every paid action, capacity and retained destination rather than only final food totals.
5. **Hold cadence and terminal precedence:** hold observes/learns but consumes no controller draws, movement or transaction. The next opportunity resumes ordinary dispatch. A yield reaching an applicable nest/outlet endpoint performs the appropriate terminal action next instead of holding. Arrival advice, disposal resumption and trip counts remain correct.
6. **All cargo modes:** food and spoil can yield/hold without losing identity or changing job intent. Empty workers can yield without a new departure, search entry or publication. Holds use the correct paid work category.
7. **Privacy:** changing remote occupancy, unobserved food or unobserved terrain while preserving the worker's private state and local view cannot affect its decision or draws. Fresh revisions remain local and paid Dig remains unchanged.
8. **Rollback:** inject a later worker failure after an earlier yielding move/hold and prove full state, diagnostics and RNG rollback. Include malformed yield draws and diagnostic invariant failures.
9. **Recording:** repeated seeds replay within the supported platform; sampling and observational traffic views consume no draws. Test initial/final alignment, disabled recording, combined snapshot-byte caps and the legal final-tick pending hold.
10. **Integration:** run applicable workspace and survey tests, formatting, core/survey all-target Clippy and source-include audit after changed core inputs. Confirm the F5 collector still selects legacy, all immutable manifest bytes and historical reports remain unchanged, and its source fingerprint covers every new compile input.

Use scripted deterministic fixtures and existing engineering seeds 7/8 for real F5 engineering worlds. The implementation follows the user's subagent-driven preference, with independent task and final reviews. No scientific seeds 10001..10040 are run or expanded by this engineering increment.

## 8. Subsequent construction increment

After traffic acceptance, design how an empty informed worker may consider excavation before exhausting open frontiers, using a remembered/recruited site label and private observed geometry. It must account for digging and spoil transport, preserve loaded-job completion and distinguish optimistic estimates from known route benefit. This document does not choose that excavation heuristic or authorize its implementation. Keep the original registered results intact and register any later scientific policy comparison separately.
