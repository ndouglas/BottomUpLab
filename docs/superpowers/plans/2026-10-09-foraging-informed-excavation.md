# Cost-Aware Informed Excavation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user has already selected subagent-driven execution; do not ask again.

**Goal:** Let an empty informed construction worker compare early excavation with ordinary pursuit using private geometry and explicitly charged spoil transport, while retaining the original excavation policy.

**Architecture:** Build a draw-free estimator from three bounded private distance fields, then integrate its choices before ordinary informed pursuit. Keep traffic as an independent policy, physical actions in existing transactions, and new diagnostics in tick-owned state. A separate labeled runner records all three snapshot streams under one byte cap.

**Tech Stack:** Existing Rust workspace, serde/serde_json, seeded PCG, scripted unit-test draws, and existing Cargo/survey tooling; no new dependencies.

**Spec:** [Cost-aware informed excavation](../specs/2026-10-09-foraging-informed-excavation-design.md), written at `981c894` and approved by the user before this plan was written.

**Status:** the user approved this plan on 2026-10-10. Subagent-driven execution has started; the root tracker and plan-owned ledger record stage progress.

## Global Constraints

- Excavation identities are exactly `exploration_first_v1` and `informed_cost_v1`; traffic choices remain `shortest_only_v1` and `local_yielding_v1` independently.
- Preserve `World::new`, `run`, `World::new_with_traffic`, and `run_with_traffic` on `exploration_first_v1`, including serialized output, decisions, RNG consumption, computation counts, and work totals. Do not change Setup, the five CPFA parameters, existing schemas, or ComputeCounts.
- Assess only empty, find-free Departing workers with a site, after fresh learning and existing terminal/hold/at-site/protected-site handling. Loaded jobs and other modes keep their current priorities.
- Use at most three grid-bounded fields: KnownOpen origin, KnownOpen outlet, and KnownOpen-or-Unknown site. All remembered solids block the optimistic field; only the proposed face is hypothetically opened.
- Require both strict optimistic geometric improvement from the approach and strict full-cost improvement over the reference, unless the corresponding reference is unavailable. Charge approach travel, one Dig, both known spoil legs, one DisposeSpoil, one entry move, and optimistic continuation.
- No finite candidate/no improvement consumes no new draw. Accepted ties use face-then-approach Pos order and the checked choice convention, including a singleton draw. Reassess each eligible opportunity; no tunnel commitment, shared map, global occupancy, peer intention, role allocation, relay, fitting, or amortized collective benefit.
- Keep tagged food/spoil conservation, two-worker capacity, ascending-ID scheduling, one paid action per opportunity, full horizon, and complete physical/private/diagnostic/RNG rollback.
- New schema is exactly `foraging-construction-policy-episode-v1`. Base, traffic, and excavation checkpoints align; compact snapshot bytes across all three streams share a 64 MiB cap. Summaries/wrapper metadata remain outside that sum.
- Retain dimensions 3–125, workers 1–256, food 0–256, ticks 1–7,200, and at most 1,000,000 opportunities. An assessment scans at most N remembered cells and 4N approach pairs; no per-candidate BFS.
- F5 production collection remains ordinary construction::run. Preserve all three immutable manifests, protocol, saved-analysis identities, historical report goldens, and original raw records. Engineering worlds use scripted fixtures or seeds 7/8; do not execute scientific seeds 10001..10040.
- Work in `/Users/nathan/.config/superpowers/worktrees/SugarScape/foraging-5-registration`, branch `foraging-5-registration`; verify actual clean state and heads before execution. Preserve concurrent work and old evidence. No merge, push, new worktree, scientific run, CLI/browser/WASM change, or dependency installation belongs to this plan.

## Review Focus

1. An unknown continuation already exists without opening the face: reject credit for that continuation unless the opening strictly improves its optimistic distance. Task 1 pins this causal test separately from full cost.
2. The remembered site itself is a diggable solid: the ordinary optimistic field has no traversable seed, but that exact candidate has zero onward continuation. Task 1 pins this exception without opening other solids.
3. A rejected comparison leaves exploration-first fallback available: do not accidentally prohibit all digging or count a fallback Dig as proactive. Task 2 pins rejection, fallback, and exact draws.
4. A historical assessment describes a face another worker has since opened: views retain the old assessment tick and validate its arithmetic without requiring the old face still to be solid. Task 3 pins historical-record validation and observational purity.
5. A late worker fails after an earlier cost-selected action and actual PCG consumption: public World::step rolls back both policies, diagnostics, physical state, private maps, advice, and RNG. Task 4 proves scratch progress and source rollback separately.

---

## Execution discipline and four-stage tracker

After plan approval, read the SDD and worktree skills, verify this retained worktree rather than replacing it, and create root `IMPLEMENTATION_PLAN.md` with four stages matching Tasks 1–4 and Goal/Success Criteria/Tests/Status fields. Use a new ignored `.superpowers/sdd/2026-10-09-foraging-informed-excavation/` workspace, resolved through the SDD skill. Check plan identity and task ledger before dispatch. Never redispatch completed traffic/F5 tasks or reuse their archived workspace.

Use a fresh implementer and independent combined spec/quality reviewer per task, then the most-capable whole-plan reviewer under the SDD skill. The parent coordinates review findings, rulings, verification, and bookkeeping; workers do not dispatch agents. Preserve chronological evidence and corrections. Stop and reassess after three unexpected failures on the same issue. Never bypass hooks or disable tests.

Each implementer records actual RED output, makes the minimum implementation, records GREEN, self-reviews, and commits working code. Compilation failure from a missing API is scaffold RED, not behavioral RED. Passing compatibility captures and supplemental coverage are labeled GREEN evidence. Every runtime task runs focused construction tests, root formatting, and core all-target Clippy; Task 4 owns fresh full workspace/survey gates. Use `CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target` and Cargo locks. Save commands, cwd, times, completed output, exits, input hashes, and review reports under this plan's own evidence workspace.

## File map

Construction paths in this table are relative to `crates/sugarscape-core/src/foraging/construction/`.

| File | Responsibility | Task |
| --- | --- | --- |
| excavation.rs | Public policy, counts, candidate/assessment, worker/summary/snapshot types | 1 |
| excavation_state.rs | Checked private diagnostics, historical-record validation, paid effects | 1/2 |
| excavation_navigation.rs | Draw-free fields, geometric/cost filters, ordered tied candidates | 1 |
| state.rs, world.rs | Agent diagnostics, world excavation policy, explicit constructor | 2 |
| decision.rs, actions.rs, controller.rs | Eligibility/order, selection, paid commit, invariant integration | 2 |
| excavation_view.rs | Observational summary and snapshot projection | 3 |
| policy_runner.rs | ConstructionPolicyEpisode, triple recording, checked combined cap | 3 |
| runner.rs | Reuse existing option/counting helpers with unchanged old runners | 3 |
| mod.rs, tests/mod.rs | Focused exports/module registration | each |
| tests/excavation_baseline.rs, fixtures/informed_excavation_baseline.json | Pre-edit labeled traffic episode and next RNG baseline | 1 |
| tests/excavation_types.rs, excavation_navigation.rs | Checked types and private estimator acceptance | 1 |
| tests/excavation_support.rs, excavation_controller.rs | Paid fixture preparation and controller/cargo/traffic tests | 2 |
| tests/excavation_runner.rs | Views, recording, provenance and byte caps | 3 |
| tests/excavation_acceptance.rs | Privacy, actual public rollback, full acceptance | 4 |
| tests/controller.rs, runner.rs, traffic_support.rs | Existing private Decision literals adapted with Ordinary excavation effect | 2 |
| tests/material_access.rs | Existing private Agent factory initialized with default excavation diagnostics | 2 |
| survey/build_support/f5_source_identity.rs | Bind new literal baseline JSON input and audit compile inputs | 1/4 |
| survey/src/claims/foraging_shortcuts/tests/report.rs | Explicit default/F5 policy compatibility | 4 |
| docs/foraging.md, approved spec/plan, verification study, root tracker | Accurate implementation/evidence status | 4/parent |

Do not restructure unrelated geometry, traffic, or survey modules. Existing helpers can be reused with narrow visibility changes; neither optimistic distances nor researcher terrain enter physical route selection.

## Shared contracts

These signatures are the interfaces between tasks. Public diagnostic records are Clone/Debug/PartialEq/Eq/Serialize; policy additionally Copy/Deserialize with explicit serde identities, counts additionally Copy/Default. ConstructionPolicyEpisode is Clone/Debug/PartialEq/Serialize, as its base Episode does not implement Eq. New output wrappers remain Serialize-only.

```rust
// Task 1: excavation.rs.
pub enum ExcavationPolicy { ExplorationFirstV1, InformedCostV1 }
pub struct ExcavationCounts {
    pub assessments: u64,
    pub faces_scanned: u64,
    pub candidate_pairs: u64,
    pub selected_opportunities: u64,
    pub initiated_digs: u64,
    pub distance_calls: u64,
    pub distance_visits: u64,
    pub peak_queue: u64,
}
pub struct ExcavationCandidate {
    pub face: Pos,
    pub approach: Pos,
    pub approach_moves: u64,
    pub spoil_leg_moves: u64,
    pub continuation_moves: u64,
    pub pre_opening_distance: Option<u64>,
    pub estimated_cost: u64,
}
pub struct ExcavationAssessment {
    pub tick: u32, // one-based; old material event ticks stay unchanged
    pub site: Pos,
    pub reference_cost: Option<u64>,
    pub candidate_pairs: u64,
    pub best_candidate: Option<ExcavationCandidate>,
    pub selected_candidate: Option<ExcavationCandidate>,
}
pub struct ExcavationAgentView {
    pub id: u32,
    pub counts: ExcavationCounts,
    pub last_assessment: Option<ExcavationAssessment>,
}
pub struct ExcavationSummary {
    pub traffic_policy: TrafficPolicy,
    pub excavation_policy: ExcavationPolicy,
    pub completed_ticks: u32,
    pub counts: ExcavationCounts,
    pub agents: Vec<ExcavationAgentView>,
}
pub struct ExcavationSnapshot {
    pub traffic_policy: TrafficPolicy,
    pub excavation_policy: ExcavationPolicy,
    pub completed_ticks: u32,
    pub agents: Vec<ExcavationAgentView>,
}
// Task 1: excavation_navigation.rs; never consumes draws or mutates knowledge.
pub(super) struct AssessmentPlan {
    pub(super) assessment: ExcavationAssessment,
    pub(super) tied_candidates: Vec<ExcavationCandidate>,
    pub(super) counts: ExcavationCounts,
}
pub(super) fn assess(
    knowledge: &Knowledge, origin: Pos, site: Pos, outlet: Pos, tick: u32,
) -> Checked<AssessmentPlan>;
pub(super) fn estimated_cost(approach: u64, spoil_leg: u64, onward: u64)
    -> Checked<u64>;
// Task 1/2: excavation_state.rs.
pub(super) struct ExcavationState {
    pub(super) counts: ExcavationCounts,
    pub(super) last_assessment: Option<ExcavationAssessment>,
}
pub(super) enum ExcavationEffect { Ordinary, SelectedTravel, SelectedDig }
impl ExcavationState {
    pub(super) fn include_assessment(&mut self, plan: &AssessmentPlan,
        selected: Option<ExcavationCandidate>) -> Checked<()>;
    pub(super) fn commit(&mut self, effect: ExcavationEffect) -> Checked<()>;
    pub(super) fn check(&self, policy: ExcavationPolicy, work: &WorkCounts,
        width: u32, height: u32, completed_ticks: u32) -> Checked<()>;
}
pub(super) fn include_counts(total: &mut ExcavationCounts, delta: &ExcavationCounts)
    -> Checked<()>; // checked sums; peak_queue is max
// Task 2: decision.rs; existing test adapters retain legacy excavation.
pub(super) fn decide_with_policies(policy: &Policy, agent: &Agent,
    observation: &Observation, traffic_policy: TrafficPolicy,
    excavation_policy: ExcavationPolicy, opportunity_tick: u32,
    draws: &mut impl DrawSource) -> Checked<Decision>;
// Task 2/3: public World/runner interfaces.
impl World {
    pub fn new_with_policies(setup: Setup, seed: u64,
        traffic_policy: TrafficPolicy, excavation_policy: ExcavationPolicy) -> Checked<Self>;
    pub fn excavation_summary(&self) -> Checked<ExcavationSummary>;
    pub fn excavation_snapshot(&self) -> Checked<ExcavationSnapshot>;
}
pub fn run_with_policies(setup: Setup, seed: u64, options: RunOptions,
    traffic_policy: TrafficPolicy, excavation_policy: ExcavationPolicy)
    -> Checked<ConstructionPolicyEpisode>;
// Task 3's private runner takes the same arguments, plus limit: u64.
pub(super) fn run_with_policies_limit(setup: Setup, seed: u64, options: RunOptions,
    traffic_policy: TrafficPolicy, excavation_policy: ExcavationPolicy, limit: u64)
    -> Checked<ConstructionPolicyEpisode>;
pub(super) fn checked_policy_snapshot_total(base: u64, traffic: u64,
    excavation: u64, limit: u64) -> Checked<u64>;
```

Task 1 compiles the new private estimator/state modules under `cfg(test)` while its public record types are exported normally. Task 2 removes those two temporary module gates when production consumers exist; do not add broad dead-code allowances or expose private algorithms as public APIs just to satisfy Clippy. Task 1 unit tests exercise every private interface above; Task 2 wires them into paid world behavior.

## Task 1: Policy records and draw-free private estimator

**Files:** Create excavation.rs, excavation_state.rs, excavation_navigation.rs, tests/excavation_types.rs, tests/excavation_navigation.rs, tests/excavation_baseline.rs, and fixtures/informed_excavation_baseline.json. Modify mod.rs, tests/mod.rs, and survey/build_support/f5_source_identity.rs.

**Interfaces:** Consume Knowledge classifications/dimensions/known(), setup::neighbors, checked errors, and existing public traffic runner. Produce the Task 1 records, assess, estimated_cost, and checked state helpers above. No production controller behavior changes in this task.

- [ ] **Step 1: Capture the current complete traffic baseline before production edits.** Add a temporary test that writes a JSON record with `traffic_episode_json: String` and `next_rng: u64`, using the existing tests::setup(), seed 7, ticks 40, sample_every 7, snapshots true, and LocalYieldingV1. Capture the wrapper via serde_json::to_string; obtain next_rng by independently stepping World::new_with_traffic 40 times. Keep the exact captured bytes in the new fixture, remove the writer test, and pin them with the test below. Preserve capture log, original HEAD, fixture size/hash, and next RNG value; do not regenerate after production changes. Add the fixture's exact path to the source selector FILES list because .json is not selected by the Rust directory rule.

```rust
#[derive(serde::Deserialize)]
struct TrafficBaseline { traffic_episode_json: String, next_rng: u64 }
#[test]
fn retained_traffic_episode_and_rng_match_pre_excavation_bytes() {
    use rand::Rng;
    let golden: TrafficBaseline = serde_json::from_str(
        include_str!("fixtures/informed_excavation_baseline.json")).unwrap();
    let options = RunOptions { ticks: 40, sample_every: 7, snapshots: true };
    let episode = run_with_traffic(setup(), 7, options, TrafficPolicy::LocalYieldingV1).unwrap();
    assert_eq!(serde_json::to_string(&episode).unwrap(), golden.traffic_episode_json);
    let mut world = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
    for _ in 0..40 { world.step().unwrap(); }
    assert_eq!(world.rng.gen::<u64>(), golden.next_rng);
}
```

- [ ] **Step 2: Write estimator/type RED tests and inspect their failure.** Tests must cover explicit policy spellings/unknown identities, cost arithmetic, unavailable reference, strict equality rejection, geometric causality, protected/disconnected faces, and the diggable-site exception. Use complete fresh observations of real Terrain to prepare geometry-only Knowledge; label these as private-map unit fixtures rather than paid World trajectories. Run `cargo test --locked -p sugarscape-core foraging::construction::tests::excavation` with the shared target; disclose missing-module/API compilation errors as scaffold RED.

```rust
#[test]
fn cost_charges_dig_disposal_entry_and_both_spoil_legs() {
    assert_eq!(estimated_cost(0, 3, 4).unwrap(), 13);
}
#[test]
fn cost_overflow_is_an_error() {
    assert!(estimated_cost(0, u64::MAX, 0).is_err());
}
#[test]
fn excavation_identity_is_explicit() {
    assert_eq!(serde_json::to_string(&ExcavationPolicy::InformedCostV1).unwrap(),
        "\"informed_cost_v1\"");
    assert!(serde_json::from_str::<ExcavationPolicy>("\"informed_cost_v2\"").is_err());
}
```

- [ ] **Step 3: Implement bounded fields and candidate ranking.** Index by y*width+x after dimensions/origin/site/outlet validation. KnownOpen origin is required. A remembered solid outlet is an error; an Unknown outlet has an empty known field. The optimistic site field traverses KnownOpen/Unknown, blocks every KnownSolid, and is empty for a solid site. Count calls/dequeues/queue maxima for actual field computations, including initial seeds. Reuse fields within this assessment rather than calling existing per-face BFS helpers.

```rust
// One shared site field; f itself is remembered solid and excluded.
let onward = if face == site { Some(0) } else {
    neighbors(face, width, height).into_iter()
        .filter_map(|p| optimistic_site[index(p)])
        .min().map(|distance| u64::from(distance) + 1)
};
// Apply to every reachable KnownOpen approach with a known outlet route.
let improves_geometry = match (onward, optimistic_site[index(approach)]) {
    (Some(o), Some(before)) => o + 1 < u64::from(before),
    (Some(_), None) => true,
    (None, _) => false,
};
// Reference: known origin-to-site first; otherwise optimistic site-to-origin.
let reference = known_origin[index(site)].or(optimistic_site[index(origin)])
    .map(u64::from);
let improves_cost = reference.is_none_or(|before| candidate.estimated_cost < before);
```

Scan all remembered solid entries, including protected/unreachable ones, into faces_scanned. Count every diggable-face/reachable-approach pair before filtering outlet/continuation/geometry. Keep finite geometric candidates, ordered by face then approach; retain only minimum-cost ties. The diagnostic best is first tied minimum even when its full cost loses; tied_candidates is empty on full-cost rejection. Set assessment.tick/site/reference/candidate_pairs/best, with selected_candidate initially None; delta assessments is one, selected/dig counts zero. All arithmetic is checked; estimated_cost is `approach + 2*spoil_leg + onward + 3` with checked operations.

- [ ] **Step 4: Implement checked state helpers and add behavioral field tests.** include_assessment applies computation/candidate deltas and stores the selection/record without incrementing paid selected/dig counts. commit(SelectedTravel) increments selected_opportunities; commit(SelectedDig) increments selected_opportunities and initiated_digs; Ordinary changes neither. Build replacements before assigning on overflow. check enforces the spec inequalities, zero/None legacy diagnostics, coordinates, tick in 1..=completed_ticks, cost formula, strict geometry improvement for both best/selected, strict reference improvement for selected only, best/selected equal costs, and selected implies best. A finite best that beats its reference must be selected in a committed record; a rejected best need not beat the reference. Historical validation does not compare the face with today's map. Aggregate counts use checked sums and peak maximum.

Pin a private U corridor with open route `(2,1)..(2,5)..(4,5)..(4,1)`, outlet `(1,1)`, diggable face `(3,1)`, and KnownOpen site `(4,1)`. Learn complete observations of all open cells. Assert reference 10, winning origin approach `(2,1)`, onward 1, spoil leg 1, cost 6, and geometric pre-opening distance 10. A separate map where unknown continuation is already as short must reject opening credit. A remembered-solid site candidate has onward zero while other solid cells still block. Unknown outlet, full protected barrier, and no known origin route produce the specified empty candidates/errors without changing Knowledge. At 125x125 assert distance_calls<=3, distance_visits<=3N, peak_queue<=N, faces_scanned<=N, candidate_pairs<=4N; no per-face search.

```rust
let before = knowledge.clone();
let plan = assess(&knowledge, pos(2,1), pos(4,1), pos(1,1), 1).unwrap();
assert_eq!(plan.assessment.reference_cost, Some(10));
assert_eq!(plan.assessment.best_candidate.as_ref().unwrap().estimated_cost, 6);
assert_eq!(knowledge, before);
assert!(plan.counts.distance_calls <= 3);
```

- [ ] **Step 5: Verify GREEN, self-review, and commit.** Run all construction tests, `cargo fmt --all --check`, and `cargo clippy --locked -p sugarscape-core --all-targets -- -D warnings` with the shared target for Cargo. Confirm both baseline fixtures remain exact. Commit `feat(foraging): add bounded private informed-excavation estimates`. Independent review checks all filters, bounded fields, exact diagnostics, and baseline capture before Task 2.

## Task 2: Informed selection and atomic paid controller integration

**Files:** Modify state.rs, world.rs, decision.rs, actions.rs, controller.rs, mod.rs, excavation_state.rs, the Agent factory in tests/material_access.rs, and existing private fixture Decision literals in tests/controller.rs, runner.rs, traffic_support.rs. Create tests/excavation_support.rs and excavation_controller.rs; register them in tests/mod.rs. Remove Task 1's private-module cfg(test) gates.

**Interfaces:** Consume AssessmentPlan/ExcavationState/ExcavationEffect; produce World::new_with_policies and decide_with_policies above. Extend Decision and its internal selected-action carrier with excavation_effect; ordinary conversion always supplies Ordinary. Keep the existing decide/decide_with_traffic test adapters on legacy excavation.

- [ ] **Step 1: Prepare a paid informed fixture and write failing decision tests.** Define `informed_world(traffic_policy, excavation_policy) -> World` in excavation_support.rs using this exact setup:

```rust
fn informed_setup() -> Setup {
    let mut open = (1..=5).map(|y| pos(2,y)).collect::<Vec<_>>();
    open.push(pos(3,5));
    open.extend((1..=5).map(|y| pos(4,y)));
    open.push(pos(1,1));
    Setup { width: 7, height: 7, open, diggable: vec![pos(3,1)],
        nest: vec![pos(2,1), pos(2,2)], waste: pos(1,1),
        workers: vec![pos(2,1)], food: vec![Resource { id: 42, pos: pos(4,1) }],
        parameters: Parameters { p_search: 0.0, p_return: 0.0,
            lambda_fidelity: 0.0, lambda_publish: 0.0, lambda_waypoint: 0.0 } }
}
```

Use traffic_support::fixture_tick/controlled_tick after adding Ordinary to their Decision literals. Travel from spawn down the U to `(4,1)` and back along the same U to `(2,1)`, then visit outlet `(1,1)` and return. All moves/learning/opportunities are paid, no ledger/map/position teleporting, food remains Available because the fixture keeps Departing. Use one controlled NoNeighbor wait to set site `(4,1)`, Departing, frontier/face None; disclose this intent as a fixture control rather than a seeded advice history. Validate the completed state. For a separate frontier variant, append open cells `(1,3)` and `(0,3)` to this setup, travel from `(2,3)` to `(1,3)` and back during preparation, and leave `(0,3)` unvisited. It is observed KnownOpen with Unknown neighbors `(0,2)`/`(0,4)`, so a genuinely reachable Unknown frontier remains; verify that early Dig still wins while that frontier exists.

```rust
let mut world = informed_world(TrafficPolicy::ShortestOnlyV1, ExcavationPolicy::InformedCostV1);
let observation = traffic_support::view(&world, 0);
let before_digs = world.state.agents[0].work.digs;
let mut draws = Scripted::new(&[0.0]); // singleton proactive candidate; no movement draw
let d = decide_with_policies(&Policy::from(&world.setup), &world.state.agents[0],
    &observation, world.state.traffic_policy, world.state.excavation_policy,
    world.state.tick + 1, &mut draws).unwrap();
assert_eq!(d.action, Action::Dig(pos(3,1)));
assert_eq!(d.excavation_effect, ExcavationEffect::SelectedDig);
assert_eq!(draws.next, 1);
assert_eq!(world.state.agents[0].work.digs, before_digs); // decision is not paid yet
```

Run the focused test and record scaffold/behavioral RED accurately. Compare the same prepared legacy-excavation world: it requests ordinary route travel instead of early digging.

- [ ] **Step 2: Wire world policy and controller order without changing legacy draws.** Store excavation_policy in State and ExcavationState in Agent, initialized without draws. new/new_with_traffic delegate to new_with_policies with legacy excavation. In controller::advance pass its already checked `next` tick as opportunity_tick; do not change zero-based material/advice event clocks. Rename the production decision entry point to decide_with_policies; retain test adapters as needed.

Preserve observation, terminal/hold, all cargo branches, at-site/protected-site checks, then enter assessment only for informed Departing. Use draw_index on the ordered pair list (choose currently accepts Pos only); do not alter choose or flatten pair ties into duplicate face choices. include_assessment receives the selected candidate or None. Set frontier None/face Some only on selection; route to exactly its selected approach with the existing traffic selector, or Dig its freshly confirmed adjacent face. Rejection delegates to the unchanged pursuit body without an extra choice draw.

```rust
let plan = assess(&agent.map, agent.pos, site, policy.waste, opportunity_tick)?;
let selected = if plan.tied_candidates.is_empty() { None } else {
    let length = u32::try_from(plan.tied_candidates.len())
        .map_err(|_| vec![FieldError::new("excavation.candidates", "exceeds u32")])?;
    Some(plan.tied_candidates[draw_index(draws, length)? as usize].clone())
};
agent.excavation.include_assessment(&plan, selected.clone())?;
// Selected travel retains its TrafficEffect; selected Dig has TrafficEffect::Normal.
```

- [ ] **Step 3: Commit paid effects exactly once and validate invariants.** Ordinary cannot increment selected/dig counts and cannot accompany a newly selected candidate for this opportunity; an older retained selected record remains legal. SelectedDig requires revised policy, informed empty Departing, matching recorded selected face, current approach, and Action::Dig(face). SelectedTravel requires that same eligibility/record and either an ordinary route Move or congestion Wait toward the approach, including an allowed yielding move; TrafficHold is Ordinary and bypasses assessment. Run existing physical transaction first, then commit excavation effect with traffic/work accounting. Controller checks each worker and checked aggregates at completed-tick boundaries. Private fixtures constructing Decision explicitly use Ordinary unless exercising a legitimate selected decision.

Test each diagnostic inequality and counter overflow in isolation with otherwise valid physical/work state. Preserve tick/worker error prefixes; invalid effect/action combinations fail rather than silently relabeling a Dig. No general fault-injection API, role system, optimistic physical movement, or additional observation is introduced.

- [ ] **Step 4: Cover priorities, reconsideration, fallback, and traffic.** Script singleton/tied selections and ordinary route draws precisely. Pin acceptance versus equal/higher cost rejection without new rejection draws; unavailable known outlet; at-site/search transition; protected-site abandonment; uninformed/search/return bypass; loaded food/spoil bypass; terminal disposal/arrival and pending hold precedence. Use existing paid held_worker support for all modes rather than inventing cargo ownership.

Drive the winning fixture through Dig, its tagged spoil Move to outlet, DisposeSpoil, and resumed site pursuit. Assert one token/one dig, one disposal, unchanged food intent/trip/publication counts, and no forced return to the old approach. A next-opportunity fresh observation that opens/invalidates a selected face removes it from the comparison; a previously optimistic continuation becoming remembered solid changes the choice. Use legal paid actions/observations for these changes. For rejection followed by exhausted frontiers, demonstrate an ordinary fallback Dig increments work.digs but not initiated_digs. For distant approaches demonstrate normal congestion and both traffic choices; hold opportunities retain last assessment but do not increment assessments or select/dig counts.

- [ ] **Step 5: Verify GREEN and commit.** Run all construction tests, formatter, and core all-target Clippy. Confirm both complete baseline fixtures, zero/None excavation diagnostics under existing constructors, all previous queue/hold tests, and no temporary cfg(test) module gates remain on production estimator/state modules. Commit `feat(foraging): select early informed digs with paid spoil costs`. Independent review checks exact dispatch/draw order, action/effect validation, retained fallback, conservation, and both policies before Task 3.

## Task 3: Observational excavation views and bounded labeled episodes

**Files:** Create excavation_view.rs, policy_runner.rs, tests/excavation_runner.rs. Modify mod.rs exports and tests/mod.rs; runner.rs only if existing helper visibility needs narrowing/sharing. Keep traffic_runner.rs behavior/wire shape intact.

**Interfaces:** Consume new World constructor and diagnostic state. Produce World excavation views, run_with_policies/run_with_policies_limit, and checked_policy_snapshot_total above. Define ConstructionPolicyEpisode with exactly the fields/types in spec section 2: schema &'static str, both policy enums, Episode, TrafficSummary, Vec<TrafficSnapshot>, u64 traffic bytes, ExcavationSummary, Vec<ExcavationSnapshot>, u64 excavation bytes.

- [ ] **Step 1: Write public-runner RED tests.** Pin both policy identities and exact schema, serialize-only wrapper, legacy-equivalent inner episode/traffic output for both traffic choices, and read-only excavation views.

```rust
let options = RunOptions { ticks: 40, sample_every: 7, snapshots: true };
for traffic in [TrafficPolicy::ShortestOnlyV1, TrafficPolicy::LocalYieldingV1] {
    let old = run_with_traffic(setup(), 7, options.clone(), traffic).unwrap();
    let new = run_with_policies(setup(), 7, options.clone(), traffic,
        ExcavationPolicy::ExplorationFirstV1).unwrap();
    assert_eq!(new.schema, "foraging-construction-policy-episode-v1");
    assert_eq!(new.episode, old.episode);
    assert_eq!(new.traffic_summary, old.traffic_summary);
    assert_eq!(new.traffic_snapshots, old.traffic_snapshots);
    assert_eq!(new.traffic_snapshot_bytes, old.traffic_snapshot_bytes);
    assert_eq!(new.excavation_summary.counts, ExcavationCounts::default());
}
```

Run the focused test, inspect its complete failure, then implement views/recording.

- [ ] **Step 2: Implement checked read-only views.** Call controller::check and clone worker rows in ID order; sums use include_counts with peak max. Validate historical assessment structural fields against its original tick/arithmetic rather than current terrain. Use a paid fixture after its successful recorded Dig to prove the face is now KnownOpen yet its last_assessment remains valid. Subsequent spoil/hold/nonassessment ticks retain that record. Clone State and RNG before calling both views repeatedly; compare complete state and next RNG afterward.

- [ ] **Step 3: Implement full-horizon triple recording.** Add private PolicyRecording with base/traffic/excavation vectors and three byte totals. Its `append(base: Snapshot, traffic: TrafficSnapshot, excavation: ExcavationSnapshot, limit: u64) -> Checked<()>` validates checkpoint alignment, both policy identities across frames, and matching ascending worker IDs across all three streams. Calculate prospective compact byte totals with the existing bounded CountingWriter and check their combined sum before mutating vectors/totals. Ordinary runners must not build excavation frames. Sample initial, scheduled, and final checkpoints once; disabled recording constructs no snapshot views.

```rust
fn checked_policy_snapshot_total(base: u64, traffic: u64, excavation: u64,
    limit: u64) -> Checked<u64> {
    base.checked_add(traffic).and_then(|n| n.checked_add(excavation))
        .filter(|n| *n <= limit)
        .ok_or_else(|| vec![FieldError::new("excavation_snapshot_bytes",
            "combined snapshot byte budget exceeded")])
}
```

- [ ] **Step 4: Pin caps, chronology, censoring, and sampling.** Count serialized stream sizes independently and compare reported bytes. Run with exact total (success) and total-minus-one (error); a limit larger than each individual stream but smaller than their sum still fails. Directly test u64 overflow without allocation. A failed append preserves all previous vectors/totals. Mismatched policies/IDs/checkpoints error. A legal final snapshot may retain pending traffic hold or carried spoil and a prior assessment. Use real fixture snapshots in direct append tests; do not disguise prepared history as a new seeded public episode.

```rust
let e = run_with_policies(setup(), 7, options.clone(), TrafficPolicy::LocalYieldingV1,
    ExcavationPolicy::InformedCostV1).unwrap();
let total = e.episode.snapshot_bytes + e.traffic_snapshot_bytes + e.excavation_snapshot_bytes;
assert!(run_with_policies_limit(setup(), 7, options.clone(), TrafficPolicy::LocalYieldingV1,
    ExcavationPolicy::InformedCostV1, total).is_ok());
assert!(run_with_policies_limit(setup(), 7, options, TrafficPolicy::LocalYieldingV1,
    ExcavationPolicy::InformedCostV1, total - 1).is_err());
assert!(checked_policy_snapshot_total(u64::MAX, 1, 0, u64::MAX).is_err());
```

Test disabled snapshots, interval zero, interval greater than ticks, exact-divisor final checkpoint, full horizon after depletion, repeated seeded runs, and equal State/RNG/diagnostics across sampling settings. View calls consume no searches or draws. Reject corrupt historical cost/geometry/reference/tick records individually; accept valid historical records after actual face opening.

- [ ] **Step 5: Verify and commit.** Run construction tests, formatter, and core all-target Clippy; keep exact old fixtures and traffic records unchanged. Commit `feat(foraging): record labeled informed-excavation episodes`. Independent review checks provenance, observational purity, historical validation, triple bounds, and errors before Task 4.

## Task 4: Privacy, production rollback, and retained F5 acceptance

**Files:** Create tests/excavation_acceptance.rs; extend owned tests/support above for missing cases. Modify survey F5 report tests only to pin retained default collection; source-selector changes only for newly discovered literal compile inputs. Parent updates docs/foraging.md, this spec/plan, and `docs/studies/2026-10-09-foraging-informed-excavation-verification.md` after runtime acceptance.

**Interfaces:** Consume all prior public/private APIs and paid support. No new policy rule, parameter, scientific world, or report schema is added.

- [ ] **Step 1: Audit acceptance coverage and add missing tests before any correction.** Map all ten spec acceptance classes to actual tests. Preserve passing supplemental coverage as GREEN evidence; require behavioral RED for a corrective fix. Compare explicit legacy excavation against both old APIs per tick, including exact snapshots and next RNG, and preserve complete pre-edit serialized fixtures. Check all diagnostic inequalities independently rather than triggering several at once.

```rust
use rand::Rng;
let mut old = World::new_with_traffic(setup(), 7, TrafficPolicy::LocalYieldingV1).unwrap();
let mut explicit = World::new_with_policies(setup(), 7, TrafficPolicy::LocalYieldingV1,
    ExcavationPolicy::ExplorationFirstV1).unwrap();
for _ in 0..40 {
    old.step().unwrap(); explicit.step().unwrap();
    assert_eq!(old.snapshot().unwrap(), explicit.snapshot().unwrap());
    assert_eq!(old.traffic_snapshot().unwrap(), explicit.traffic_snapshot().unwrap());
}
assert_eq!(old.rng.gen::<u64>(), explicit.rng.gen::<u64>());
```

- [ ] **Step 2: Prove privacy using independent remote-state changes.** Hold a worker's private agent state and local observation equal while changing valid unseen terrain, hidden/remote food, remote occupancy, or another worker's map. Compare AssessmentPlan, selected Decision, both diagnostics, and consumed draws exactly. Fresh local capacity may affect the next route step but not costs/tied candidates. Unknown optimistic search must leave Knowledge identical. Read the production function signatures to confirm no terrain/access/server/global occupancy leaks into assessment; compare actual valid worlds where practical, not only identical function calls.

- [ ] **Step 3: Prove actual public clone/commit rollback.** Extend the paid informed setup with a second worker at the other nest cell. Prepare both workers through paid moves/waits so worker 0 is eligible for a guaranteed singleton proactive Dig and worker 1 has no cargo. Validate the prepared state. Set only worker 1's compute.observations to u64::MAX, preserving other physical invariants; this passes initial check but fails during its fresh learning.

Advance a scratch State using PcgDraws on a cloned RNG to prove worker 0 completed the cost-selected Dig, created spoil, recorded the selection, and consumed a real random value before worker 1's error. Do not commit scratch. Then invoke real World::step on the original injected state and compare complete State and next RNG with their saved copies. Repair the injected counter in source and an untouched control, step both, and compare successful state/RNG. Preserve all diagnostic and material assertions; add a separate malformed scripted selection draw test with no source-state commit.

```rust
let before = world.state.clone();
let mut expected_rng = world.rng.clone();
assert!(world.step().is_err());
assert_eq!(world.state, before);
let mut actual_rng = world.rng.clone();
assert_eq!(actual_rng.gen::<u64>(), expected_rng.gen::<u64>());
```

- [ ] **Step 4: Freeze the final runtime tree and run full gates.** Record the exact tree/input hashes and all outputs; await every command to completion and inspect exits/counts. Stop mutating code while gates run. Run all six commands below; do not reuse prior traffic verification as new-policy acceptance.

```bash
CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo test --workspace --locked
CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo test --manifest-path survey/Cargo.toml --locked
cargo fmt --all --check
cargo fmt --manifest-path survey/Cargo.toml --all -- --check
CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo clippy --locked -p sugarscape-core --all-targets -- -D warnings
CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo clippy --manifest-path survey/Cargo.toml --locked --all-targets -- -D warnings
```

Audit every literal include against selected native inputs, including the new fixture. Preserve the existing separately frozen cfg(test)-only protocol include exception rather than changing production fingerprint scope. Verify checkout/committed/embedded source identity on the clean final runtime commit. Pin a private native collector executable copy before use to avoid replacement by concurrent shared-target builds. Check F5 production still calls ordinary run and an existing cached seed-7 engineering episode equals the legacy-excavation explicit-policy base episode. Preserve immutable manifest/protocol hashes and both historical construction report golden pairs; tests must not collect scientific seeds.

Commit the tested runtime acceptance tree as `test(foraging): verify informed excavation privacy and retained F5 compatibility` after all full gates and audits pass. Record its SHA and ensure the worktree is clean before the native build in Step 5. If that build or saved compatibility reveals a correction, preserve the failed evidence, fix/review the affected scope, and rerun only checks whose inputs or conclusions changed.

- [ ] **Step 5: Verify saved-only compatibility without rerunning worlds.** Build the native executable from that clean runtime commit, then create an exclusive private copy and verify its embedded source identity against the committed selected inputs. The source and private copy hashes must match; do not overwrite a previous collector.

```bash
CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target cargo build --manifest-path survey/Cargo.toml --locked --bin survey
```

```python
from pathlib import Path
import hashlib
import shutil
source = Path("/Users/nathan/Projects/ndouglas/SugarScape/target/debug/survey")
private = Path("/tmp/sugarscape-foraging-informed-excavation-20261009/collector-final")
private.parent.mkdir(parents=True, exist_ok=True)
with source.open("rb") as incoming, private.open("xb") as outgoing:
    shutil.copyfileobj(incoming, outgoing)
shutil.copymode(source, private)
def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()
assert digest(source) == digest(private)
print(digest(private))
```

Reanalyze the already completed 600-record index twice into fresh owned directories `/tmp/sugarscape-foraging-informed-excavation-20261009/reanalysis-a` and `reanalysis-b`, using the pinned clean native collector:

```bash
/tmp/sugarscape-foraging-informed-excavation-20261009/collector-final --foraging-shortcuts --analyze /Users/nathan/Projects/ndouglas/SugarScape/survey/out/foraging-shortcuts/2026-10-09-registered-600/index.json --out /tmp/sugarscape-foraging-informed-excavation-20261009/reanalysis-a
/tmp/sugarscape-foraging-informed-excavation-20261009/collector-final --foraging-shortcuts --analyze /Users/nathan/Projects/ndouglas/SugarScape/survey/out/foraging-shortcuts/2026-10-09-registered-600/index.json --out /tmp/sugarscape-foraging-informed-excavation-20261009/reanalysis-b
```

Do not overwrite existing directories. Before and after, verify index SHA `faaa5acbed1397f3451cd670071b97d66fbcc39b1870308e6dc8f2dc97104fa4` and all original raw hashes/sizes. Both report pairs must match each other and original JSON SHA `7bba70c19802bb7b97abe69b84c592fd869bd1a09132912eabf508ab657ce527` / Markdown SHA `2903e4c952e7974a13f17dcd209a0f70b5d9e7086b546f3756e50fc343591c46`. This is retained archive compatibility, not a scientific evaluation of the revised policy.

- [ ] **Step 6: Obtain task/whole-plan review and archive evidence.** Preserve the runtime acceptance commit from Step 4 and the clean native/saved-only evidence from Step 5. After independent Task 4 spec/quality approval, the parent requests final whole-plan review covering all four tasks. Consolidate any corrections through a fresh implementer/scoped review and covering verification. Record actual gate totals, runtime commit, estimator/draw/paid fixture evidence, limits, source/binary identities, and all rulings in the verification study and programme docs. Mark the specification/plan implemented only after acceptance. Hash-preserve this plan's completed SDD evidence and completed tracker before removing only those plan-owned files; commit documentation/tracker cleanup separately. Retain branch/worktree. No scientific benefit, merge, or push is inferred from engineering acceptance.

## Plan self-review coverage

Spec sections 1/2: explicit independent policies, retained APIs/F5 identities, and scope in Tasks 1–4. Section 3: eligibility, all priorities, reconsideration, paid effects, and fallback in Task 2. Sections 4/5: three private fields, solid-site exception, causal geometry, full cost, unavailable references, and exact tie/draw rules in Tasks 1/2. Section 6: separate counters/records, historical validation, pure views, bounds, and triple recording in Tasks 1–3. Section 7: checked failures, both-policy rollback, and declared model limits in Tasks 2/4. Section 8: all ten acceptance classes and retained SDD handoff are assigned above. Each Review Focus item has an owning test step. Shared type/field/function names match their consumers; no scientific workload or previously completed implementation is redispatched.
