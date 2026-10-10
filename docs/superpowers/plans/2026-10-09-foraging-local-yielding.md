# Private Local Yielding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user has already selected subagent-driven execution; do not ask again.

**Goal:** Add an explicitly selected, seeded local-yielding traffic policy that can relieve an opposing queue while retaining the F4/F5 baseline and private learned maps.

**Architecture:** Factor private route planning from random step selection, then apply the new policy within the existing atomic construction world. Record traffic memory/counters in tick-owned state and expose a separate labeled episode wrapper. Keep the ordinary constructor, runner and F5 collector on the original policy.

**Tech Stack:** Existing Rust workspace, serde/serde_json, seeded PCG and scripted unit-test draws; no new dependencies.

**Spec:** [Approved private local-yielding design](../specs/2026-10-09-foraging-local-yielding-design.md), approved by the user on 2026-10-09 after commit `59e1036`.

**Execution status:** approved by the user on 2026-10-09; subagent-driven execution starting.

## Global Constraints

- `TrafficPolicy` identities are exactly `shortest_only_v1` and `local_yielding_v1`. The latter fixes two consecutive paid blocked-route waits, yield probability 0.25 and one paid hold. The third blocked opportunity may yield; do not tune these choices.
- Preserve ordinary `World::new`/`run`, existing Setup/Parameters/Episode/Snapshot/Summary/WorkCounts serialization, baseline decisions/RNG/work/computation/sampling and all F5 manifest/history/report bytes.
- Every opportunity learns a fresh current/cardinal observation. Yield candidates are privately KnownOpen, locally free cardinal cells with a finite private distance to the same goals. Minimize that distance within the yielding set and retain north/south/east/west tie order.
- Terminal food/empty-nest arrivals and spoil disposal override holds. A nonterminal hold learns but pays one congestion wait without phase/yield/selection draws; it preserves cargo and trip intent.
- All travel modes can yield. No swaps, displacement, capacity exceptions, peer intentions, global occupancy, shared maps, roles, relay, widening, excavation changes or new biological claims.
- Keep two-worker capacity, ascending-ID scheduling, one paid action per opportunity, food/spoil conservation, full horizon and atomic world/traffic/RNG rollback.
- Traffic counters are checked aggregate/per-worker blocked_route_opportunities, yield_attempts, yield_moves and hold_waits. Distinguish paid holds from forced blocking; preserve base work equations.
- Dimensions 3–125, workers 1–256, food 0–256; ticks 1–7200 and at most 1,000,000 opportunities. New recording caps combined compact base-plus-traffic snapshot JSON at 64 MiB; old snapshot-byte semantics stay unchanged.
- Keep F5 collection on ordinary legacy `run`; no revised-policy F5 archive, generic registry, CLI/browser/WASM change or dependency installation. A later scientific policy comparison requires separate registration.
- Real F5 engineering episodes use seeds 7/8 only. Never generate worlds for scientific seeds 10001..10040; saved-only analysis of the existing complete archive is permitted.
- Execute in the existing clean linked worktree `/Users/nathan/.config/superpowers/worktrees/SugarScape/foraging-5-registration`, branch `foraging-5-registration`. Do not create another worktree or change main. Preserve concurrent work, existing raw evidence and all old archives. No merge/push or new scientific run is part of this plan.

## Review Focus

1. A held worker reaches a terminal destination: complete its arrival transaction and original advice/disposal cadence rather than delaying or replaying it. Task 2 pins all three terminal cases.
2. A threshold-eligible worker has no free escape, a free ordinary shortest step, or an unreachable private goal: consume no yield ticket and preserve existing navigation behavior. Tasks 1/2 pin all cases and exact draws.
3. Loaded travel yields into a cell containing food or a diggable neighbor: preserve the tagged cargo and suspended job without pickup/dig/publication. Task 2 pins food/spoil and empty modes.
4. A later worker fails after an earlier worker yielded: roll back physical state, traffic memory/counters, maps/advice and RNG together. Task 4 injects a late failure.
5. A recording cutoff ends on a yield, or combined snapshot bytes exceed the cap although each component fits: retain a legal pending hold, or return a bounded error without a successful partial episode. Task 3 pins both and checked overflow.

## Execution and four-stage tracker

Create root `IMPLEMENTATION_PLAN.md` only after this plan's review, with four stages matching Tasks 1–4 and the user's Goal/Success Criteria/Tests/Status fields. Implementers mark their stage In Progress; the parent marks Complete after review. Remove it after whole-branch acceptance and hash-verified evidence archival.

Use this plan's own ignored `.superpowers/sdd/2026-10-09-foraging-local-yielding/` workspace, resolved by the SDD skill. Check ledger identity before dispatch; never redispatch completed tasks or alter another plan's workspace. Fresh implementer and combined spec/quality reviewer per task; parent coordinates fixes and bookkeeping, with a most-capable final whole-branch review. Workers do not dispatch agents. Preserve rulings and rework costs chronologically.

Run actual RED, inspect completed output, implement minimally, run focused GREEN, self-review and commit. Missing-API compilation failures are disclosed as scaffold REDs; do not claim behavioral RED without a compiled failing assertion. Capture the pre-change compatibility fixture before the first production edit; that capture is GREEN baseline evidence, not a feature RED. Preserve unexpected failures and stop/reassess after three attempts on an issue. Never bypass hooks or disable tests.

Use `CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target` and Cargo's locks. Freeze task inputs before gates; preserve exact commands/cwd/times/exits/logs/hashes. Run focused construction tests, root formatting and core all-target Clippy before each runtime commit. Task 4 owns fresh full workspace/survey tests and survey format/Clippy once on the final frozen tree. Broaden earlier only for a specific new failure or concern; unchanged documentation does not justify repeated suites.

## File map

| File | Responsibility | Owner |
| --- | --- | --- |
| construction/traffic.rs | Policy identity, checked counters, private traffic memory and public traffic views | 1; integration 2 |
| construction/navigation.rs | Draw-free private route plan and unchanged legacy step adapter | 1 |
| construction/traffic_navigation.rs | Explicit policy step selection using the existing route plan | 2 |
| construction/state.rs, world.rs | Per-agent traffic state, tick-owned policy, explicit constructor | 2 |
| construction/decision.rs, actions.rs, controller.rs | Hold/terminal order, route effect, paid commit, invariant checks | 2 |
| construction/traffic_view.rs | Read-only summary/snapshot projection | 3 |
| construction/traffic_runner.rs | Labeled wrapper and combined snapshot budget | 3 |
| construction/runner.rs | Share option validation/counting helpers without changing old recording | 3 |
| construction/mod.rs | Export new public types/methods; retain existing exports | 1/2/3 |
| construction/tests/traffic_baseline.rs, fixtures/local_yielding_baseline.json | Exact pre-change episode/RNG fixture | 1; final comparison 4 |
| construction/tests/traffic_types.rs, traffic_navigation.rs | Policy/counter/private-plan tests | 1/2 |
| construction/tests/traffic_support.rs, traffic_controller.rs | Paid fixture preparation, scripted transitions, cargo/hold tests | 2 |
| construction/tests/traffic_runner.rs, traffic_acceptance.rs | Recording and complete acceptance/privacy/rollback | 3/4 |
| construction/tests/mod.rs | Register owned test modules | each |
| survey/build_support/f5_source_identity.rs | Bind the new literal baseline fixture input | 1; audit 4 |
| survey/src/claims/foraging_shortcuts/tests/report.rs | Explicit proof that the F5 collection path remains legacy | 4 |
| docs/foraging.md, approved spec/plan, root tracker | Accurate implementation/evidence status | 4/parent |

All construction paths above are relative to `crates/sugarscape-core/src/foraging/`. Preserve existing file organization; do not reorganize unrelated modules.

## Shared contracts

```rust
// Task 1: traffic.rs; public types are Clone/Debug/PartialEq and Serialize.
// Policy additionally Copy/Eq/Deserialize; explicit serde rename per variant.
pub enum TrafficPolicy { ShortestOnlyV1, LocalYieldingV1 }
pub struct TrafficCounts {
    pub blocked_route_opportunities: u64,
    pub yield_attempts: u64,
    pub yield_moves: u64,
    pub hold_waits: u64,
}
pub struct TrafficAgentView {
    pub id: u32,
    pub blocked_wait_streak: u8,
    pub hold_pending: bool,
    pub counts: TrafficCounts,
}
pub struct TrafficSummary {
    pub policy: TrafficPolicy,
    pub completed_ticks: u32,
    pub counts: TrafficCounts,
    pub agents: Vec<TrafficAgentView>,
}
pub struct TrafficSnapshot {
    pub policy: TrafficPolicy,
    pub completed_ticks: u32,
    pub agents: Vec<TrafficAgentView>,
}
// Task 1: private state is Clone/Debug/Default/PartialEq/Eq.
pub(super) struct TrafficState {
    pub(super) blocked_wait_streak: u8,
    pub(super) hold_pending: bool,
    pub(super) counts: TrafficCounts,
}
pub(super) enum TrafficEffect { Normal, BlockedWait, YieldMove, Hold }
impl TrafficCounts {
    pub(super) fn checked_include(&mut self, delta: &Self) -> Checked<()>;
}
impl TrafficState {
    pub(super) fn commit(&mut self, effect: TrafficEffect) -> Checked<()>;
    pub(super) fn check(&self, policy: TrafficPolicy, work: &WorkCounts) -> Checked<()>;
}
// Task 1: navigation.rs; all planning is draw-free, one distance BFS.
pub(super) enum RoutePlan {
    AtGoal,
    Unreachable { counts: ComputeCounts },
    Reachable {
        free_shortest: Vec<Pos>,
        yield_candidates: Vec<Pos>,
        counts: ComputeCounts,
    },
}
pub(super) fn plan_route(
    knowledge: &Knowledge, origin: Pos, goals: &[Pos], observation: &Observation,
) -> Checked<RoutePlan>;
// Existing route_step signature stays unchanged and adapts this plan.

// Task 2: traffic_navigation.rs; records request/ticket counters, not paid commits.
pub(super) fn select_route(
    policy: TrafficPolicy, traffic: &mut TrafficState,
    knowledge: &Knowledge, origin: Pos, goals: &[Pos],
    observation: &Observation, draws: &mut impl DrawSource,
) -> Checked<(Navigation, ComputeCounts, TrafficEffect)>;
// Task 2: decision.rs; retain the existing legacy decide wrapper.
pub(super) fn decide_with_traffic(
    policy: &Policy, agent: &Agent, observation: &Observation,
    traffic_policy: TrafficPolicy, draws: &mut impl DrawSource,
) -> Checked<Decision>;
// Decision gains traffic_effect; ordinary manually scripted actions use Normal.
// Agent gains traffic: TrafficState; State gains traffic_policy: TrafficPolicy.
impl World {
    pub fn new_with_traffic(setup: Setup, seed: u64, policy: TrafficPolicy) -> Checked<Self>;
    // Task 3: views validate state, then clone; they consume no draws.
    pub fn traffic_summary(&self) -> Checked<TrafficSummary>;
    pub fn traffic_snapshot(&self) -> Checked<TrafficSnapshot>;
}
// Task 3: traffic_runner.rs; Serialize only, not a World-restoration format.
pub struct TrafficEpisode {
    pub schema: &'static str,
    pub policy: TrafficPolicy,
    pub episode: Episode,
    pub traffic_summary: TrafficSummary,
    pub traffic_snapshots: Vec<TrafficSnapshot>,
    pub traffic_snapshot_bytes: u64,
}
pub fn run_with_traffic(
    setup: Setup, seed: u64, options: RunOptions, policy: TrafficPolicy,
) -> Checked<TrafficEpisode>;
// Task 3: private paired recording, used by the runner and cutoff/cap tests.
pub(super) struct TrafficRecording {
    pub(super) base_snapshots: Vec<Snapshot>,
    pub(super) traffic_snapshots: Vec<TrafficSnapshot>,
    pub(super) base_bytes: u64,
    pub(super) traffic_bytes: u64,
}
impl TrafficRecording {
    pub(super) fn append(&mut self, base: Snapshot, traffic: TrafficSnapshot,
                         limit: u64) -> Checked<()>;
}
pub(super) fn checked_snapshot_total(base: u64, traffic: u64, limit: u64) -> Checked<u64>;
// Private lower-limit injection supports deterministic cap tests.
pub(super) fn run_with_traffic_limit(
    setup: Setup, seed: u64, options: RunOptions, policy: TrafficPolicy, limit: u64,
) -> Checked<TrafficEpisode>;
```

`Checked` is the existing private alias. `TrafficCounts` also derives Default/Eq; TrafficEffect derives Copy/Clone/Debug/PartialEq/Eq. Counts update with checked arithmetic on a candidate, so overflow leaves the receiver unchanged. `commit(Normal)` clears streak/hold; `BlockedWait` saturates the streak at two; `YieldMove` clears streak, sets hold and increments yield_moves; `Hold` clears streak/hold and increments hold_waits. Only `select_route` increments blocked_route_opportunities/yield_attempts. The action transaction performs the paid-effect commit exactly once. State validation uses the inequalities and legacy restrictions from spec section 5/6.

### Task 1: Checked policy data and draw-free private route planning

**Files:** Create traffic.rs, tests/traffic_types.rs, tests/traffic_baseline.rs and the exact fixture; modify navigation.rs, mod.rs, tests/mod.rs and the one source-selector fixture entry.

**Interfaces:** Consume current Knowledge/Observation/ComputeCounts and route_step; produce the Task 1 contracts above. Task 2 consumes RoutePlan/TrafficState/TrafficEffect; Task 3 consumes public views.

- [ ] **Step 1: Capture the actual baseline before production edits.** Temporarily register a capture test using the existing `tests::setup()`, seed 7 and `RunOptions { ticks: 40, sample_every: 7, snapshots: true }`. Serialize the complete base episode directly to a String, and independently step a World 40 times then capture its next `rng.gen::<u64>()`. Store a test-only JSON record containing that exact episode string and next RNG value through exclusive create into the task-owned capture directory. Do not deserialize a core Episode or reconstruct its floats through Value. Await the passing capture command and hash the output before changing production.

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct BaselineRecord { episode_json: String, next_rng: u64 }
let options = RunOptions { ticks: 40, sample_every: 7, snapshots: true };
let episode_json = serde_json::to_string(&run(setup(), 7, options).unwrap()).unwrap();
let mut world = World::new(setup(), 7).unwrap();
for _ in 0..40 { world.step().unwrap(); }
let record = BaselineRecord { episode_json, next_rng: world.rng.gen::<u64>() };
let destination = std::path::PathBuf::from(
    std::env::var("SUGARSCAPE_TRAFFIC_CAPTURE_DIR").unwrap(),
).join("local_yielding_baseline.json");
let file = std::fs::OpenOptions::new().write(true).create_new(true)
    .open(destination).unwrap();
serde_json::to_writer_pretty(file, &record).unwrap();
```

Run the temporary capture test with `SUGARSCAPE_TRAFFIC_CAPTURE_DIR` set to this task's existing owned capture directory and `cargo test --locked -p sugarscape-core capture_current_traffic_baseline -- --nocapture`. Copy the captured bytes to the exact fixtures path, hash-verify, remove the temporary capture test and environment-writing code, and retain a permanent read-only golden test. Add that literal JSON path to the maintained F5 source selector; it is the only new non-Rust compile input in this task.

- [ ] **Step 2: Add failing feature tests and inspect RED.** Register traffic_types and plan tests. Cover policy strings/unknown identity, checked overflow, illegal legacy yield/hold, streak/hold invariants, and private candidate ranking/free-shortest/AtGoal/Unreachable. Scaffold compile failures are recorded honestly.

```rust
#[test]
fn policy_identity_is_explicit_and_unknown_is_rejected() {
    assert_eq!(serde_json::to_string(&TrafficPolicy::LocalYieldingV1).unwrap(),
               "\"local_yielding_v1\"");
    assert!(serde_json::from_str::<TrafficPolicy>("\"future_policy\"").is_err());
}
#[test]
fn overflow_leaves_traffic_counts_unchanged() {
    let mut c = TrafficCounts { yield_attempts: u64::MAX, ..Default::default() };
    let before = c.clone();
    assert!(c.checked_include(&TrafficCounts { yield_attempts: 1, ..Default::default() }).is_err());
    assert_eq!(c, before);
}
#[test]
fn a_pending_hold_requires_zero_streak() {
    let s = TrafficState { blocked_wait_streak: 1, hold_pending: true, ..Default::default() };
    assert!(s.check(TrafficPolicy::LocalYieldingV1, &WorkCounts::default()).is_err());
}
```

For private-plan tests reuse navigation.rs's complete ordinary-observation `fresh`/`learned` fixture pattern in its current test module. The existing corridor fixture pins a full nearest goal plus a longer free escape: the plan must have no free shortest step but a finite yielding candidate. Add a branch with multiple equal ranked free cells to pin north/south/east/west ordering. A goal already at origin has zero route calls; an unreachable goal has one BFS but no candidates. Existing invalid observation/unknown-goal checks still fail contextually.

- [ ] **Step 3: Implement minimal types and plan extraction.** Reuse the exact reverse BFS and validation; do not query physical geometry or add another BFS. Keep legacy selection as the following adapter:

```rust
match plan_route(knowledge, origin, goals, observation)? {
    RoutePlan::AtGoal => Ok((Navigation::AtGoal, ComputeCounts::default())),
    RoutePlan::Unreachable { counts } => Ok((Navigation::Unreachable, counts)),
    RoutePlan::Reachable { free_shortest, counts, .. } => Ok((
        choose(&free_shortest, draws)?.map_or(Navigation::Blocked, Navigation::Move), counts,
    )),
}
```

Implement checked traffic data and export public types. A temporary narrow dead-code allowance may name TrafficState/TrafficEffect and their Task 2 consumers; remove it in Task 2. Do not enable revised World behavior yet.

- [ ] **Step 4: Verify GREEN and baseline.** Run `cargo test --locked -p sugarscape-core foraging::construction::`, `cargo fmt --all --check`, and `cargo clippy --locked -p sugarscape-core --all-targets -- -D warnings`. Confirm complete golden bytes/RNG and old route draw/count tests. Preserve capture, actual RED and final gate receipts.
- [ ] **Step 5: Self-review and commit.** Commit only the named task files with message `feat(foraging): add traffic policy data and private route plans`. The task reviewer checks counters, private inputs, byte identity and single-BFS legacy behavior.

### Task 2: Seeded yielding, holds and paid controller integration

**Files:** Create traffic_navigation.rs, tests/traffic_support.rs, tests/traffic_navigation.rs and tests/traffic_controller.rs; modify state.rs/world.rs/decision.rs/actions.rs/controller.rs/traffic.rs/mod.rs/tests/mod.rs. Update existing test Decision literals with `traffic_effect: TrafficEffect::Normal`, preserving their semantics.

**Interfaces:** Consume Task 1 contracts; produce select_route, decide_with_traffic and World::new_with_traffic. Keep `advance(setup, state, draws)` unchanged: it reads state.traffic_policy. Keep the legacy decide wrapper for current tests. Test support exports `queue_setup() -> Setup`, `opposing_queue(policy) -> World`, and `advance_scripted(world: &mut World, draws: &mut Scripted) -> Checked<()>` for Tasks 3/4. Test support also exports `QUEUE_THIRD_TICK_SCRIPT: &[f64] = &[0.5, 0.0, 0.5, 0.5, 0.9, 0.0, 0.9]` and `QUEUE_FOURTH_TICK_SCRIPT: &[f64] = &[0.5, 0.0, 0.5, 0.5, 0.5, 0.0]`, with the exact per-worker interpretation below. The advance helper clones state/draws and commits both only on success, following the existing controller test helper. Tests inspect Scripted.next for actual consumed draws. Also provide `held_worker(mode: Mode) -> (World, usize, Observation)` through paid fixture actions: actual PickupFood/Dig for cargo, a validated yielding Move for the pending hold, and a fresh local view. This helper must not fabricate ledger assignments.

- [ ] **Step 1: Write failing selector/hold tests.** Use Task 1 private plans and Scripted draws. Supply a pre-existing streak of two with valid paid counts for selector tests; standalone plan tests do not claim a naturally occurring episode. After scaffolding the new constructor/decision interfaces, run compiled assertions before implementing yielding; the third blocked step must still fail by returning the old wait.

```rust
let mut scene = setup();
scene.open = (0..5).map(|x| pos(x, 0)).collect();
scene.nest = vec![pos(0,0), pos(1,0)];
scene.waste = pos(4,0);
scene.workers = vec![pos(0,0)];
scene.food.clear(); scene.diggable.clear();
let terrain = Terrain::new(&scene).unwrap();
let mut known = Knowledge::new(scene.width, scene.height).unwrap();
for cell in &scene.open {
    known.learn(&observe(&terrain, *cell, &BTreeMap::new(), &BTreeSet::new()).unwrap()).unwrap();
}
let origin = pos(2,0);
let goals = [pos(1,0), pos(4,0)];
let observation = observe(&terrain, origin,
    &BTreeMap::from([(pos(1,0), 2)]), &BTreeSet::new()).unwrap();
let mut traffic = TrafficState { blocked_wait_streak: 2,
    counts: TrafficCounts { blocked_route_opportunities: 2, ..Default::default() },
    ..Default::default() };
let mut draws = Scripted::new(&[0.249, 0.0]);
let (step, _, effect) = select_route(TrafficPolicy::LocalYieldingV1, &mut traffic,
    &known, origin, &goals, &observation, &mut draws).unwrap();
assert!(matches!(step, Navigation::Move(_)));
assert_eq!(effect, TrafficEffect::YieldMove);
assert_eq!(draws.next, 2);
```

Repeat with ticket 0.25: Blocked/BlockedWait, one ticket, no choice draw. With streak zero/one or no free candidate expect zero tickets. With a free shortest step expect only its ordinary choice draw and Normal. AtGoal/Unreachable have no traffic tickets/counters. Assert the minimum-ranked tied candidate and singleton choice convention. Invalid NaN/1.0/negative/exhausted tickets must error.

- [ ] **Step 2: Build paid queue support.** Use this fixed fixture, not a favorable F5 seed:

```rust
fn queue_setup() -> Setup {
    Setup {
        width: 5, height: 4,
        open: vec![pos(1,2), pos(1,3), pos(2,2), pos(2,3),
                   pos(0,3), pos(2,1), pos(2,0), pos(3,2)],
        diggable: vec![],
        nest: vec![pos(1,2), pos(1,3), pos(2,2), pos(2,3)],
        waste: pos(0,3),
        workers: vec![pos(2,2), pos(2,2), pos(1,2), pos(1,2)],
        food: vec![],
        parameters: Parameters { p_search: 1.0, p_return: 0.0,
            lambda_fidelity: 0.0, lambda_publish: 0.0, lambda_waypoint: 0.0 },
    }
}
```

Construct with seed 7 and the requested policy. Prepare positions through ordinary fresh observations and paid apply transactions: worker 0 moves to (3,2); worker 2 travels (2,2) then (2,1); worker 3 takes that same path; worker 0 returns to (2,2). Never assign positions/cargo/maps by teleporting. Preserve complete-tick opportunity accounting with the existing fixture_tick pattern in tests/runner.rs. Fixture-controlled phase/target transitions are labeled test controls and checked through Decision/apply, not claimed stochastic trajectories: workers 0/1 become Searching with retained frontier (2,1), workers 2/3 become empty Returning. Their maps must arise from actual preparation observations and retain a known home route. Streaks start zero; no yield occurs during preparation. Validate the complete state before the tested sequence.

Use cloned fixtures for legacy and revised comparisons. Two blocked ticks pay waits and consume only the two Searching give-up draws per tick. On the third revised tick script worker 0's successful ticket and east-bay choice, worker 1's failed ticket, worker 2's ordinary nest step and worker 3's failed ticket. On the next tick worker 0 holds while worker 1 moves north and the returners use normal arrival/travel. The third-tick script has worker 0 give-up/ticket/bay-choice (0.5/0.0/0.5), worker 1 give-up/failed-ticket (0.5/0.9), worker 2 ordinary step (0.0), and worker 3 failed-ticket (0.9). The fourth has no draw for worker 0 hold, worker 1 give-up/step (0.5/0.0), worker 2 empty-arrival draws (0.5/0.5/0.5), and worker 3 ordinary step (0.0). Pin consumed lengths 7 and 6 and the actual north/south/east/west tie lists. Each legacy comparison tick consumes only [0.5, 0.5]. Verify both traffic directions cross the boundary, every capacity stays <=2, and legacy workers remain blocked over the comparison window.

- [ ] **Step 3: Implement controller integration.** All ordinary routes use select_route with the explicit profile and one shared private plan. A private selected-action result carries Action plus TrafficEffect through pursue/frontier/face/return paths; Decision exposes that effect to the transaction. Terminal actions select Normal and clear pending memory; otherwise hold selects its dedicated internal wait reason before phase draws. Ordinary route-blocked waits select BlockedWait; all other ordinary actions select Normal.

```rust
// Wrapper contract; the normal constructor keeps exact legacy selection.
pub fn new(setup: Setup, seed: u64) -> Checked<Self> {
    Self::new_with_traffic(setup, seed, TrafficPolicy::ShortestOnlyV1)
}
// Selection updates request/ticket diagnostics; paid commit happens once in apply.
if free_shortest.is_empty() {
    traffic.counts.checked_include(&TrafficCounts {
        blocked_route_opportunities: 1, ..Default::default()
    })?;
    if policy == TrafficPolicy::LocalYieldingV1
        && traffic.blocked_wait_streak == 2 && !yield_candidates.is_empty()
    {
        traffic.counts.checked_include(&TrafficCounts {
            yield_attempts: 1, ..Default::default()
        })?;
        if checked_uniform(draws)? < 0.25 {
            let target = choose(&yield_candidates, draws)?.expect("nonempty yielding set");
            return Ok((Navigation::Move(target), counts, TrafficEffect::YieldMove));
        }
    }
    return Ok((Navigation::Blocked, counts, TrafficEffect::BlockedWait));
}
```

At transaction application validate the effect matches the physical action: YieldMove requires Move and the revised profile; Hold requires the internal hold wait and revised profile; BlockedWait requires a route congestion wait; Normal cannot create a traffic hold. Pay the existing Move/Wait category, then commit traffic memory/counters. A hold contributes the mode-appropriate congestion count, and takes no arrival/server/phase draws. Check traffic invariants in controller::check, including checked aggregate sums and legacy restrictions. Remove all Task 1 future-use allowances.

- [ ] **Step 4: Cover hold/cargo/terminal behavior and verify GREEN.** Use paid fixture preparation to obtain food and spoil through PickupFood/Dig, never fabricate ledger ownership. For each mode check exact cargo/phase/site/frontier/face retention, one hold, observation increments and no extra trip/publication/transaction. Food arrival at nest, empty arrival at nest and spoil arrival at outlet each override a pending hold and retain their exact original draw cadence. Holds on a food-bearing cell defer pickup by one opportunity. A normal transition or no-neighbor wait clears the streak. Legacy holds/attempts/yields and hold-plus-streak corruption fail contextual checks.

```rust
let (world, worker, fresh_view) = held_worker(Mode::FoodReturning);
let before = world.state.agents[worker].clone();
let mut draws = Scripted::new(&[]);
let d = decide_with_traffic(&Policy::from(&world.setup), &before,
    &fresh_view, TrafficPolicy::LocalYieldingV1, &mut draws).unwrap();
assert_eq!(d.traffic_effect, TrafficEffect::Hold);
assert_eq!(d.agent.cargo, before.cargo);
assert_eq!(d.agent.phase, before.phase);
assert_eq!(d.agent.site, before.site);
assert_eq!(draws.next, 0);
```

Run construction tests, root formatter and core Clippy as in Task 1; inspect final output and self-review all route consumers. Commit `feat(foraging): add seeded local yielding and paid traffic holds`. Review gates focus on actual paid threshold, information limits, draw order and all cargo modes.

### Task 3: Observational traffic views and bounded labeled recording

**Files:** Create traffic_view.rs, traffic_runner.rs and tests/traffic_runner.rs; modify runner.rs helper visibility, mod.rs exports and tests/mod.rs.

**Interfaces:** Consume traffic state/profile and World::new_with_traffic; produce traffic_summary/traffic_snapshot/TrafficEpisode/run_with_traffic/run_with_traffic_limit with the signatures above. Existing run and base snapshot helper behavior remain intact.

- [ ] **Step 1: Write failing public boundary tests.** Pin wrapper identity, default-versus-explicit legacy episode equality, aligned checkpoints, disabled recording and read-only views. Record missing-API RED honestly.

```rust
#[test]
fn labeled_legacy_runner_preserves_the_complete_base_episode() {
    let options = RunOptions { ticks: 40, sample_every: 7, snapshots: true };
    let base = run(setup(), 7, options.clone()).unwrap();
    let labeled = run_with_traffic(setup(), 7, options, TrafficPolicy::ShortestOnlyV1).unwrap();
    assert_eq!(labeled.schema, "foraging-traffic-episode-v1");
    assert_eq!(labeled.episode, base);
    assert_eq!(labeled.traffic_summary.policy, labeled.policy);
    assert_eq!(labeled.traffic_snapshots.len(), labeled.episode.snapshots.len());
}
#[test]
fn traffic_views_do_not_mutate_world_or_rng() {
    let w = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    let before = w.state.clone();
    let mut expected_rng = w.rng.clone();
    let _ = w.traffic_summary().unwrap();
    let _ = w.traffic_snapshot().unwrap();
    assert_eq!(w.state, before);
    let mut actual_rng = w.rng.clone();
    assert_eq!(actual_rng.gen::<u64>(), expected_rng.gen::<u64>());
}
```

- [ ] **Step 2: Implement views and runner.** Validate state then clone worker rows in ID order; sum counters with checked arithmetic. Reuse existing option validation and CountingWriter semantics. Run the full horizon; append paired base/traffic snapshots at initial, scheduled and final checkpoints exactly once. Count compact base and traffic bytes separately, then check their addition against the supplied limit before accepting a pair. TrafficRecording.append validates matching checkpoint/policy/worker rows, calculates both next counts before mutating its vectors, and keeps the previous recording intact on a failed append. Disable both sets when snapshots=false. Keep wrapper metadata outside snapshot bytes. Do not construct traffic snapshots for ordinary legacy run.

```rust
let next_total = base_bytes.checked_add(traffic_bytes)
    .filter(|total| *total <= limit)
    .ok_or_else(|| vec![FieldError::new("traffic_snapshot_bytes",
        "combined snapshot byte budget exceeded")])?;
// Return next_total from checked_snapshot_total; append only after success.
```

Expose only the existing counting helper(s) needed by the sibling runner. No whole-file restructure or new archive parser. The typed policy enum rejects unknown deserialized identities; other new views/wrappers remain Serialize-only.

- [ ] **Step 3: Pin byte caps/cutoff/observational behavior.** Compute actual compact serialized sums for the successful tiny fixture, assert both recorded byte fields, then supply exact-total limit (success) and exact-total-minus-one (failure). Also choose a cap above each component but below their sum; it must fail. Directly test checked_snapshot_total(u64::MAX, 1, u64::MAX) overflow without allocating a huge record. Advance a prepared queue through a scripted yielding move, then feed its real base/traffic snapshots to TrafficRecording.append: the recorded pending hold and hold_waits<yield_moves must remain legal. Continuing one opportunity pays a hold unless terminal. This is a labeled synthetic fixture checkpoint, not a new seeded Episode or a public restoration API; do not add a prepared-world runner that disguises its history.

```rust
let options = RunOptions { ticks: 40, sample_every: 7, snapshots: true };
let e = run_with_traffic(setup(), 7, options.clone(), TrafficPolicy::LocalYieldingV1).unwrap();
let total = e.episode.snapshot_bytes.checked_add(e.traffic_snapshot_bytes).unwrap();
assert!(run_with_traffic_limit(setup(), 7, options.clone(),
    TrafficPolicy::LocalYieldingV1, total).is_ok());
assert!(run_with_traffic_limit(setup(), 7, options,
    TrafficPolicy::LocalYieldingV1, total - 1).is_err());
```

Check repeated seed replay, sample_every>ticks, sample_every dividing ticks, zero interval rejection, full horizons after depletion, disabled recording and identical control/traffic/RNG behavior across sampling options. Ensure baseline pre-change golden stays exact.

- [ ] **Step 4: Verify and commit.** Run construction tests, root formatter/core Clippy; preserve RED/GREEN and cap evidence. Commit `feat(foraging): record labeled traffic episodes with bounded snapshots`. Reviewer checks provenance labeling, observational APIs, combined bounds and cutoff censoring.

### Task 4: Complete queue, privacy, rollback and F5 compatibility acceptance

**Files:** Create tests/traffic_acceptance.rs; extend traffic_baseline.rs/traffic_controller.rs/traffic_runner.rs/support and survey F5 report tests as needed; modify docs/foraging.md and spec/plan status only after acceptance. Source-selector additions are restricted to genuinely new literal compile data revealed by the audit.

**Interfaces:** Consume all public APIs and Task 2 queue support. No new model policy, scientific workload or reporting schema is introduced.

- [ ] **Step 1: Add missing behavioral acceptance before corrections.** Assert complete legacy baseline preservation and exact raw golden RNG, both-direction queue progress/capacity/work sums, isolated malformed diagnostic inequalities and profile-label consistency. Added passing supplemental coverage is disclosed, not represented as an earlier failure.

```rust
let mut a = World::new(setup(), 7).unwrap();
let mut b = World::new_with_traffic(setup(), 7, TrafficPolicy::ShortestOnlyV1).unwrap();
for _ in 0..40 {
    a.step().unwrap(); b.step().unwrap();
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
}
assert_eq!(a.rng.gen::<u64>(), b.rng.gen::<u64>());
```

Add an isolated test for each counter inequality with all other counters/physical inventories consistent; do not repeat the existing M1 pattern of multiple simultaneous inconsistencies. The legacy wrapper has zero yield attempts/moves/holds despite legal blocked-route counts.

- [ ] **Step 2: Prove information and rollback boundaries.** Compare decisions/draw counts from identical private agent state and identical local observations when remote physical occupancy/food/diggable state differs. Use real valid worlds and fresh observations; no researcher state enters decision signatures. Test changed local capacity affects only legal local steps. For rollback, prepare the queue at the yielding threshold, then use a DrawSource that errors at a later worker after an earlier yield. Capture complete State and the scripted source before the cloned-tick helper; both must remain identical on error, including cargo/maps/advice/traffic state.

```rust
let before = world.state.clone();
// Worker 0 yields, worker 1 waits, then worker 2 exhausts its step draw.
let mut scripted = Scripted::new(&QUEUE_THIRD_TICK_SCRIPT[..5]);
let before_draws = scripted.clone();
assert!(advance_scripted(&mut world, &mut scripted).is_err());
assert_eq!(world.state, before);
assert_eq!(scripted.next, before_draws.next);
```

Additionally use a separate raw advance on cloned scratch solely to establish that an earlier candidate worker actually reached YieldMove before that later error; otherwise the fault test does not cover post-yield rollback. The rollback assertion must invoke the shared cloned-tick helper, not merely compare an untouched source against an uncommitted scratch variable.

Also exercise actual public World::step: start from the paid queue fixture after a yielding move, so the first worker has a pending hold. Set a later worker's observations counter to u64::MAX, which passes the initial invariants but errors when that worker learns its next observation. Prove the first candidate hold occurred using a separate scratch advance, then call World::step and compare the complete source State and cloned RNG continuation against before. Restore the injected counter in the source and an untouched control, step both successfully and compare state and the next RNG value. This tests production clone/commit wiring without adding a fault-injection API or relying on a lucky seeded yield ticket.

- [ ] **Step 3: Freeze and run final gates once.** Await and inspect each completed command; save complete logs/exit codes and aggregates. Both workspace and independent survey depend on changed core inputs, so neither full suite can be retained solely by the old input proof.

```bash
 CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo test --workspace --locked
 CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo test --manifest-path survey/Cargo.toml --locked
 cargo fmt --all --check
 cargo fmt --manifest-path survey/Cargo.toml --all -- --check
 CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo clippy --locked -p sugarscape-core --all-targets -- -D warnings
 CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo clippy --manifest-path survey/Cargo.toml --locked --all-targets -- -D warnings
```

Audit every literal include against the maintained source selector, including the baseline fixture. Verify selected checkout/committed/embedded source fingerprints on the final runtime commit. Remove temporary capture code and future-use allowances. Preserve every original manifest SHA and both historical construction report goldens. In the F5 report tests explicitly compare its cached seed-7 episode to run_with_traffic(...ShortestOnlyV1).episode, while inspecting that production collect still calls ordinary run. Default/help/preflight checks stay pure; no scientific collection test reaches a World.

- [ ] **Step 4: Saved-only historical science check.** After the final native binary is rebuilt from the clean runtime commit, analyze the existing completed index twice into fresh owned directories `/tmp/sugarscape-foraging-local-yielding-20261009/reanalysis-a` and `reanalysis-b`. Never overwrite an existing directory or rerun its 600 worlds. Use the copied verified executable with `--foraging-shortcuts --analyze /Users/nathan/Projects/ndouglas/SugarScape/survey/out/foraging-shortcuts/2026-10-09-registered-600/index.json --out /tmp/sugarscape-foraging-local-yielding-20261009/reanalysis-a`, then the same command with the reanalysis-b path. Pin a private executable copy from the clean final build before invoking it, to avoid replacement by concurrent shared-target builds.

Both report pairs must match each other and original JSON SHA `7bba70c19802bb7b97abe69b84c592fd869bd1a09132912eabf508ab657ce527` / Markdown SHA `2903e4c952e7974a13f17dcd209a0f70b5d9e7086b546f3756e50fc343591c46`. Recheck the completed index SHA `faaa5acbed1397f3451cd670071b97d66fbcc39b1870308e6dc8f2dc97104fa4` and all 600 original raw hashes/sizes before and after. This verifies old saved compatibility, not revised scientific efficacy.

- [ ] **Step 5: Commit, whole review and archival handoff.** Commit acceptance code as `test(foraging): verify traffic queue relief and retained F5 compatibility`; parent coordinates the final whole-branch review across all four tasks. Any correction goes through a consolidated implementer and scoped review, with covering verification. Then record runtime commit, actual gate totals, queue fixture evidence, limitations and source identities in programme docs. Hash-preserve this plan's evidence and all chronological rulings before retiring only its completed ignored workspace/tracker. Commit documentation separately; do not repeat unchanged-source suites. Keep the branch/worktree for a later integration decision. The next design is earlier informed excavation; no scientific performance claim follows from engineering queue acceptance.

## Plan self-review coverage

Spec 1/2: policy separation and default/F5 compatibility in Tasks 1–4. Spec 3/4: memory, exact threshold/draws, terminal/hold order, all modes and private ranking in Tasks 1/2. Spec 5: counters/views/wrapper/combined limits in Tasks 1–3. Spec 6: complete-tick rollback/invariants/explicit liveness limits in Tasks 2/4. All ten engineering acceptance classes appear in the owning tests above. Spec 8 remains a separate later design, with no excavation changes in this plan. The five Review Focus cases have named owning tests; shared signatures, fixture responsibility, source binding, legacy RNG and no-scientific-World boundary are explicit.
