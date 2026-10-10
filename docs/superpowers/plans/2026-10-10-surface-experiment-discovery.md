# Surface Experiment Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user has selected fresh subagent implementers and independent reviews. Steps use checkbox syntax for tracking.

**Goal:** Compare optimal raw calibration actions with the supplied probe recipe and no probing, and visualize complete bounded episodes in the existing viewer.

**Architecture:** Add a sibling surface_discovery laboratory composed with frozen shared-surface physics. Its kernel exposes owned calibration decisions; an exact joint candidate planner and policy-conditioned replay preserve local information. A checked diagnostic evaluates the fixed census; a Rust browser adapter supplies chronological projections through the existing WASM worker and surface renderer.

**Tech Stack:** Existing Rust/serde/clap workspace, checked Probability/ScoreFraction, Python standard-library independent references, wasm-bindgen/wasm-pack, TypeScript/Vite/Vitest and plain DOM. No new dependency, optimizer, frontend physics or navigation shell.

**Spec:** [Approved written design](../specs/2026-10-08-surface-experiment-discovery-design.md), committed as 108b4aa and approved by the user “LGTM!” on October 10, 2026.

**Status:** Written plan awaiting user review. No product implementation or scientific collection has started. After plan review use the preserved subagent method; do not ask the user to select it again.

## Global Constraints

- Retain SharedPersistent, SharedResetting, PrivatePersistent and Inert. DataFlip is separately labeled transfer and absent from the Agent's catalog.
- Original slot ownership is A/B/B/A at slots 1/2/3/4. At most three purchased calibration rounds and four live trials; 48 initial credits per Agent; read/write/wait cost one, inspection four, correct prediction reward 12. Each purchased round costs each Agent two credits; skipped rounds incur no charge.
- Raw calibration offers Wait, Read and writes of Probe0, Ack0, Probe1, Ack1, Data0, Data1. Stop only at a free round boundary; no calibration inspection, blank write, half-round stop, arbitrary live action selection or new alphabet.
- Ties: Stop before Continue; Wait, Read, Write(Probe0), Write(Ack0), Write(Probe1), Write(Ack1), Write(Data0), Write(Data1); Inspect before Attempt; target tie predicts zero. Compare exact fractions numerically, never with float epsilon or structural rational ordering.
- Objective: expected investigator net task utility. Responder physical rules are identical across primary policies; target inference replays the declared policy. Report investigator, responder and group outcomes separately.
- Controller inputs: own prefix/prior/issued private bit/credits/public clock. Environment, peer history, future bits, reward feedback and lineage are evaluator-only. Own actions are interventions; public stopping is evidence; private decisions are not announced to the peer.
- Fixed census: 40 in-family settings × 256 = 10,240 episodes; eight DataFlip settings × 256 = 2,048; total 48 settings/12,288. Primary policies RawAdaptive, SuppliedAdaptive, NoProbe; controls KnownRaw and InspectOnly. DataFlip excludes KnownRaw. No postmeasurement seed, horizon, prior, price or policy sweep.
- Three primary paired contrasts in each of eight mechanism/role strata; uniform-prior expectations separately by role. No pooled roles, survivor means, sampled-seed confidence intervals or undefined terminal contrasts.
- Unsupported histories retain owner/first prefix, partial costs, available predictions and nominal beliefs. Unavailable terminal reward/net/accuracy stay null. Resource exhaustion is computational error, not contradiction. Never add DataFlip to inference or repair after failure.
- “Retain 64 KiB input/decompressed-link, 16 MiB episode, 4,096 checkpoint, one-worker request, 60-second timeout/cancellation, and two-retained-record limits.” Lossless integer transport and exact fresh reconstruction; no new float tolerance. Browser runs one sequence, not the census.
- Preserve legacy engines, frozen settings, measured artifacts, scientific guide prefixes and original source receipts. New versioned namespace. Common identity guards may change only with before/after inventories, unchanged original exports and contextual rejection of incompatible imports; never relabel historical measurements.
- Existing .claude/worktrees/crowd, branch crowd. Inspect Git before edits and preserve concurrent work. Never stage .claude/, papers/, survey/out/ or ignored evidence. No worktree recreation, stash/discard, force push or hook bypass. American spelling; Agent in code/docs.
- Fresh producer and independent spec/quality reviewers per implementation task, then independent precollection and whole-increment reviews. One product writer/build owner at a time; root owns orchestration, evidence and Git.
- Rustfmt only edited Rust files with child traversal disabled; never cargo fmt in survey. Workspace formatting checks are allowed. Stop/reassess after three failed attempts. Do not disable tests.
- Commit working increments; safely merge/push finished work and check exact-commit CI/Pages under standing authorization. Preserve first/repeat outputs exclusively; disclose postmeasurement revisions and retain originals as separate series.

## Review Focus

1. B's consecutive calibration slots and live boundary after an unobserved A action must preserve A/B/B/A chronology and private information — Tasks 2–4.
2. Equal model marginals with different retained fields/peer histories must not share a value cache; own action selection must not count as evidence — Task 3.
3. Data0/Data1 calibration can expose DataFlip earlier than the old recipe; stop at the first contradiction without another paid event or terminal zero — Tasks 4–6.
4. Forged decisions/fractions, missing settings/sequences, edited imports and input changes before blur must reject without replacing a valid retained record — Tasks 5–7.
5. Planner resource limits, cancellation/timeout and stale-source imports must fail contextually without shortening the horizon or adding a tolerance — Tasks 1, 3, 6–7.

## File Structure

All abbreviated laboratory paths below are beneath crates/sugarscape-core/src/surface_discovery/.

| Files | Responsibility / owner |
| --- | --- |
| docs/superpowers/protocols/2026-10-10-surface-experiment-discovery.md | Declarative protocol/census, pre-mortem and reviewed limits; Task 1, final source binding Task 8 |
| types.rs, kernel.rs, mod.rs, tests/kernel.rs | Local records and chronological transitions; Task 2 |
| belief.rs, planner.rs, budget.rs, tests/planner.rs | Original-candidate inference, exact backward induction and allocation limits; Task 3 |
| replay.rs, evaluation.rs, metrics.rs, tests/replay.rs, tests/evaluation.rs | Policy-conditioned replay, actual episodes, exact population metrics; Task 4 |
| diagnostics.rs, diagnostics/protocol.rs, diagnostics/references.rs, tests/diagnostics.rs | Compact reports and current-payload validation; Task 5 |
| tests/fixtures/{planning,evaluation,limits}-reference.json | Small independent exact fixtures/provenance; Task 1, consumed Tasks 3–5 |
| crates/sugarscape-cli/src/surface_discovery.rs, tests/surface_discovery.rs, src/main.rs | New surface-discovery diagnose and checked emitter; Task 5 |
| crates/sugarscape-core/src/browser_experiments/discovery/mod.rs, projection.rs | Native wrapper and allowlisted checkpoint projection; Task 6 |
| Browser mod.rs, input.rs, catalog.rs, tests/mod.rs, tests/discovery.rs, fixtures/discovery-examples.json | Additive registration, strict inputs, identity and examples; Task 6 |
| web/src/episodes/types.ts, controls.ts, comparison.ts, surface-view.ts, surface-projection.ts | New selection, matched comparisons and raw-action rendering; Task 7 |
| web/src/episodes/discovery-projection.test.ts, discovery-parity.test.ts, discovery.browser-test.ts | Projection/parity and browser acceptance; Task 7 |
| crates/sugarscape-wasm/tests/experiment_view_tests.rs | Actual WASM dispatch/reconstruction; Task 6 |
| docs/surface-discovery.md, browser recorded.rs, fixtures/discovery-recorded.json, appended guide links | Corrected findings and compact measured summary; Task 8 |

Core src/lib.rs receives only the additive export. Legacy active_surface/shared_surface remain unchanged. Existing browser_experiments/surfaces.rs remains the original adapter; new projection is a sibling. Common identity fixtures/tests change only where additive registration requires current identities.

Independent programs, engineering timings, raw reports, manifests and reviews live in ignored .superpowers/sdd/2026-10-10-surface-experiment-discovery/. Track compact reviewable fixtures, never large reports or ignored evidence.

Complete independently derived census projections remain in that ignored directory, sealed with path/size/hash receipts. Tracked planning/evaluation fixtures contain exact summary values for every setting plus bounded complete episode/prefix witnesses; they link by immutable hash to the full independent projections. Complete population coverage does not require checking large reference histories into Git.

## Stages and Evidence

After plan approval create an owned IMPLEMENTATION_PLAN.md with Goal, Success Criteria, Tests and Status for five stages: (1) protocol/reference feasibility; (2) engine/inference, Tasks 2–4; (3) diagnostic, Task 5; (4) browser/parity, Tasks 6–7; (5) frozen collection/findings/delivery, Task 8. All start Not Started. If a concurrent campaign owns the root tracker use docs/superpowers/plans/2026-10-10-surface-experiment-discovery/IMPLEMENTATION_PLAN.md and record ownership. Snapshot the completed owned tracker into evidence and remove only that tracker.

Preserve status/HEAD, original source inventories, Cargo.lock/manifests, guide prefix bytes, previous first report paths/hashes and original bounded active exports for all comparator settings/sequences before runtime edits. Do not rewrite previous evidence or unnecessarily copy multi-GB reports. Label engineering runs separately from scientific first/repeat outputs.

Tasks execute in order. The independent Task 1 numerical author receives only the spec and reviewed declarative protocol, with no production planner/source or measurement exposure. Root supplies physical/helper rules already verified against existing contracts; a separate declarative reviewer checks them before numerical handoff. Author records all inputs/read access. Numerical review precedes product implementation. Reviewers must not be producers reviewing their own results.

## Shared Contracts

Task 2 defines these laboratory types. Add structural Eq/Ord where used as keys, strict serde decoding and checked constructors. Re-export physical types from shared_surface without modifying that namespace.

~~~rust
pub use crate::shared_surface::{Action, Belief, Environment, EpisodeBits, Event,
    Ids, LocalEntry, Mechanism, OwnPrior, Phase, Position, Probability, Role,
    ScoreFraction, Symbol};
pub enum PolicyKind { RawAdaptive, SuppliedAdaptive, NoProbe, KnownRaw, InspectOnly }
pub enum Choice { Stop, Continue, Calibration(Action), Inspect, Attempt }
pub enum Boundary { Probe { completed: u8 }, Calibration(Position), Trial { trial: u8 } }
pub enum Checkpoint { Boundary(Boundary), BeforeSlot(Position), AfterSlot(Position),
    Prediction { trial: u8 }, Finished }
pub enum Entry { Physical(LocalEntry), OwnChoice { boundary: Boundary, choice: Choice },
    PublicProbeStop { completed: u8 } }
pub struct Protocol { pub investigator: Role, pub policy: PolicyKind, pub ids: Ids }
pub struct Prefix { pub role: Role, pub ids: Ids, pub own_prior: OwnPrior,
    pub checkpoint: Checkpoint, pub entries: Vec<Entry> }
pub struct View { pub prefix: Prefix, pub credits: u8,
    pub private_bit: Option<bool>, pub belief: Belief }
pub struct Limits { pub states: u64, pub branches: u64, pub allocation_bytes: u64 }
pub enum Error { InvalidProtocol(String), InvalidHistory(String),
    UnsupportedHistory { prefix: Prefix }, ResourceLimit { resource: String, limit: u64 },
    InvalidReport(String), ArithmeticOverflow, Physical(crate::shared_surface::Error) }
~~~

Task 2 exports:

~~~rust
impl Protocol {
    pub fn new(investigator: Role, policy: PolicyKind) -> Self;
    pub fn validate(&self) -> Result<(), Error>;
}
impl Prefix { pub fn validate(&self) -> Result<(), Error>; }
impl View { pub fn from_prefix(prefix: Prefix, belief: Belief) -> Result<Self, Error>; }
pub struct Step { pub histories: [Prefix; 2], pub events: Vec<(Role, Event)>,
    pub next: Option<Boundary>, pub spent_delta: [u8; 2] }
impl EpisodeState {
    pub fn new(protocol: &Protocol, environment: Environment, bits: EpisodeBits,
        priors: [OwnPrior; 2]) -> Result<Self, Error>;
    pub fn prefix(&self, role: Role) -> &Prefix;
    pub fn credits(&self, role: Role) -> u8;
}
pub fn select_choice(state: &mut EpisodeState, choice: Choice) -> Result<(), Error>;
pub fn advance_one(state: &mut EpisodeState) -> Result<Step, Error>;
~~~

The signatures above are interface declarations, not empty implementation bodies. Protocol defaults retain status-field/Agent-A/Agent-B. State owns World/bits/histories/credits/clock/round count/routine privately. Selection queues; advance returns after one paid event or explicit free/reset transition before later observations. Raw policies stop at each owned calibration slot for a private decision. Supplied controls retain original physical recipes without fictitious extra private decisions in legacy projections.

Task 3 exports:

~~~rust
pub struct ChoiceValue { pub choice: Choice, pub value: ScoreFraction }
pub struct Decision { pub choice: Choice, pub alternatives: Vec<ChoiceValue>,
    pub continuation_policy: PolicyKind }
pub struct SearchStats { pub candidate_worlds: u64, pub decision_states: u64,
    pub branches: u64, pub allocation_bytes: u64 }
impl Limits { pub fn registered() -> Result<Self, Error>; }
impl CompiledPolicy {
    pub fn build(protocol: &Protocol, prior: OwnPrior, limits: &Limits) -> Result<Self, Error>;
    pub fn infer(&self, prefix: &Prefix) -> Result<Belief, Error>;
    pub fn decide(&self, view: &View) -> Result<Decision, Error>;
    pub fn initial_value(&self) -> ScoreFraction;
    pub fn stats(&self) -> &SearchStats;
    pub fn protocol(&self) -> &Protocol;
    pub fn own_prior(&self) -> OwnPrior;
}
~~~

Limits loads Task 1 reviewed numeric ceilings and rejects zero. SuppliedAdaptive maps to frozen Adaptive, NoProbe to NoProbe, InspectOnly to InspectOnly; KnownRaw uses a point prior with raw opportunities, not the original Known action menu.

Task 4 exports Replay::build(&Protocol, &CompiledPolicy, &Limits) -> Result<Replay, Error>, Replay::infer(&Prefix) -> Result<Belief, Error>, run_episode(&Protocol, Environment, u16, &CompiledPolicy, &Replay) -> Result<Episode, Error>, evaluate_panel(&Protocol, Environment, &CompiledPolicy, &Replay) -> Result<Panel, Error>. New Episode/AgentMetrics/Failure/Panel mirror the complete field semantics of active_surface::Episode and associated records, using new Prefix/Decision types: both histories, decisions, predictions/beliefs, privileged paid steps/lineage, optional first failure, optional model projections, nominal discoveries; spent/action counts and optional reward/net/correct; failure role/prefix/last supported belief/spent; panel protocol/environment/all 256 episodes/exact aggregate. Legacy records remain unchanged.

Task 5 exports registered_settings() -> Vec<(Protocol, Environment)>, diagnose(&Limits) -> Result<DiagnosticReport, Error>, report_integrity(&DiagnosticReport, &Limits) -> Result<bool, Error>, reconstruct_episode(&DiagnosticReport, usize, u16) -> Result<Episode, Error>, summary_json(&DiagnosticReport) -> Result<String, Error>. Versions: surface-discovery-diagnostic-v1 / protocol 1. summary_json serializes a strict Summary DTO containing version, report/source hashes, all setting aggregates, all 24 primary stratum contrasts, both role expectations, transfer completion/unavailable mass and partial costs; it omits full episode histories. DiagnosticReport includes version, protocol_version, supplied_structure, interned histories, panels, exact comparisons/role expectations, independent checks and passed. Compact indexes/rationals are checked; current reconstruction and independent references validate the whole payload.

Task 6 adds StudyId::SurfaceDiscovery and Input::SurfaceDiscovery { protocol: surface_discovery::Protocol, environment: Environment, sequence: u16 }. Adapter exports validate_input(&Input) -> Result<(), Vec<FieldError>>, run(&Input) -> Result<EpisodeRecord, Vec<FieldError>>, descriptor() -> StudyDescriptor, rules_identity() -> Result<String, Vec<FieldError>>. Existing public WASM methods dispatch it; no additional WASM API.

## Task 1 / Stage 1: Independent Protocol, References and Feasibility

**Files:** Protocol document and three compact reference fixtures; ignored independent Python reference program/tests.
**Consumes:** Approved spec and declarative rules. **Produces:** Reviewed exact values/projections, identifiability derivations, equal-responder embedding and fixed numeric Limits.

- [ ] **Step 1: Preserve baseline and review declarative handoff.** Write original A/B/B/A schedule. Calibration: round 1 A Probe0/B Read/B Ack0 iff own read matched/A Read; round 2 A Read/B Read/B Probe1/A Read; round 3 A Ack1 iff previous round 2 read matched/B Read/B Wait/A Wait. Raw replaces only investigator-owned slots. Declare resets/prices/issue times. For a helper with nominal SP, role A sends at live round 1/slot 1 and reads at round 3/slot 1 iff it sent; role B reads at round 2/slot 2 and sends at round 2/slot 3; other helpers inspect at send opportunity and wait elsewhere. Investigator Attempt always follows these send/read slots; Inspect inspects at send opportunity and waits elsewhere. Helper predictions use local policy replay, never hidden targets. Freeze census order, metrics/null semantics, tie order and pre-mortem. Declarative reviewer checks handoff before numerical author begins.
- [ ] **Step 2: Independently test exact mechanics before implementing reference solver.** Author uses fractions.Fraction, explicit original worlds and separate inference/planning/replay; no production import/subprocess/fixture copying. Define independent World(mechanism), act(role, action, symbol, credits) returning outcome/remaining credits, reset() and finish_round(). Write these unittest cases first:

~~~python
def test_generic_paid_write_and_local_read(self):
    for model in ('SharedPersistent','SharedResetting','PrivatePersistent','Inert'):
        w = World(model)
        self.assertEqual(w.act('A','Write','Probe0',48), ('Accepted',47))
        expected = 'Probe0' if model.startswith('Shared') else 'Blank'
        self.assertEqual(w.act('B','Read',None,48)[0], expected)
def test_dataflip_only_inverts_data(self):
    w = World('DataFlip')
    w.act('A','Write','Data0',48)
    self.assertEqual(w.act('B','Read',None,48)[0], 'Data1')
    w.act('A','Write','Probe0',47)
    self.assertEqual(w.act('B','Read',None,47)[0], 'Probe0')
~~~

Add reset/PP separation, Inspect 9/Attempt 6, no calibration bit, exact tie/intervention tests. Run python3 -m unittest discover -s .superpowers/sdd/2026-10-10-surface-experiment-discovery/reference -v; retain genuine RED/GREEN logs.
- [ ] **Step 3: Derive exact references independently.** Enumerate four models × 256 sequences, IDs model-index× 256+sequence-index and original uniform or point mass. Partition on own chronological observations; optimize future investigator reward minus future costs. Derive full allowed-horizon equivalence classes for both roles separately from chosen-policy identification. Derive initial/continuation values and all declared setting/episode projections, including raw DataFlip likelihoods. Seal outputs before product exposure. The reviewed declarative handoff includes the complete reference wire schema (fields, variant names, rational encoding and chronological prefix records), derived from the Shared Contracts without showing the author runtime source. Include planning JSON initial_values rows with protocol/prior/value; evaluation JSON episodes rows with protocol/prior/environment/sequence/episode, plus complete aggregate/contrast projections. Prefix witnesses specify exact continuation values and original candidate support.
- [ ] **Step 4: Establish embedding and feasibility before production.** Independently embed supplied recipes into raw opportunities with equal helper physical actions/clocks/information; check exact Raw≥Supplied expected investigator objective for both roles, without responder/group/per-realization/transfer guarantees. Record branch/state counts, wall time, actual peak memory and serialized sizes for complete independent enumeration. Author records the reference counter and allocation-estimation formulas in limits-reference.json. Set numeric state/branch/allocation caps at twice the largest completed reference requirement, rounded upward with documented accounting; reviewer checks available host capacity. Task 3 can revise engineering caps before judged collection only through a retained, independently reviewed protocol revision with a demonstrated accounting reason; unchanged scientific actions/horizon and all old receipts remain preserved. Task 8 freezes the final caps. If enumeration cannot complete, retain failure and seek design amendment, never invent reference values or shorten horizon/actions. Browser feasibility remains Task 6.
- [ ] **Step 5: Review/freeze/commit.** Separate numerical reviewer recomputes reference checks and confirms author independence; fixtures include protocol SHA256, derivation receipts, exact rational pairs and all declared projection coverage. Commit only reviewed protocol/compact fixtures: docs(surface): register independent discovery protocol and references.

## Task 2 / Stage 2: Checked Chronological Kernel

**Files:** Laboratory mod.rs/types.rs/kernel.rs/tests/mod.rs/tests/kernel.rs; additive core lib.rs.
**Consumes:** Task 1 and frozen World/controller. **Produces:** Protocol/Prefix/View/Choice/State/Step contracts.

- [ ] **Step 1: Write RED local chronology/cost tests.**

~~~rust
#[test]
fn stop_zero_does_not_charge_or_issue_a_bit() {
    let p = Protocol::new(Role::A, PolicyKind::RawAdaptive);
    let mut s = EpisodeState::new(&p, Environment::InFamily(Mechanism::SharedPersistent),
        EpisodeBits::from_index(0).unwrap(),
        [OwnPrior::Uniform, OwnPrior::PointMass(Mechanism::SharedPersistent)]).unwrap();
    select_choice(&mut s, Choice::Stop).unwrap();
    assert_eq!([s.credits(Role::A), s.credits(Role::B)], [48,48]);
    assert!(s.prefix(Role::A).entries.iter().all(|e|
        !matches!(e, Entry::Physical(LocalEntry::PrivateBit { .. }))));
}
~~~

Run cargo test -p sugarscape-core surface_discovery::tests::kernel; retain missing API then genuine behavioral RED.
- [ ] **Step 2: Implement strict local records and queued transitions.** Check completed 0..3/trial 0..3, ownership/phase/choice, symbols, reconstructible budgets/bits/priors and missing/unknown wire fields. Use World for physics. Raw choice queues one paid slot privately; Continue commits entire round, no mid-round Stop. Public Stop appears once; no skipped waits, duplicate resets or early bits. Helper uses candidate-local original physical controller; privileged state is never View input.

Implement From<shared_surface::Error> for the new Error, preserving ArithmeticOverflow separately. The privileged paid-event seam is:

~~~rust
fn pay(state: &mut EpisodeState, role: Role, position: Position,
       action: Action) -> Result<Event, Error> {
    let target = match position.phase {
        Phase::Calibration => None,
        Phase::Live { trial } => {
            let (a, b) = state.bits.0[usize::from(trial)];
            Some(if role == Role::A { b } else { a })
        }
    };
    let event = state.world.apply(role, position, &action, target,
                                  state.credits[role.index()])?;
    state.credits[role.index()] = event.credits_after;
    state.histories[role.index()].entries
        .push(Entry::Physical(LocalEntry::Action(event.clone())));
    Ok(event)
}
~~~

The enclosing advance_one updates both public checkpoints only after this event and returns before the next reset/bit/slot. It captures evaluator read lineage at this event without giving it to a controller.
- [ ] **Step 3: GREEN boundary matrix.** Test purchased round costs [2,2], A/B/B/A, B consecutive slots and live boundary after unobserved A, mandatory stop at three, after-slot before reset/next bit, six legal symbols, generic acceptance, DataFlip effects, Inspect 9/Attempt 6 and maximum 42 credits. Wrong phase/ownership/insufficient credits leaves state unchanged. Serde tampered bit/credit/role/unknown fields, blank write and calibration inspection reject. Compare supplied physical transitions to original active kernel.
- [ ] **Step 4: Review and commit.** Focused tests, edited-file rustfmt --edition 2021 --config skip_children=true, core Clippy; independent spec/quality reviews and scoped fixes. Commit feat(surface): add owned calibration decision kernel.

## Task 3 / Stage 2: Joint Inference and Exact Planner

**Files:** belief.rs/planner.rs/budget.rs/tests/planner.rs and exports.
**Consumes:** Kernel and independent fixtures/limits. **Produces:** CompiledPolicy/Decision/SearchStats and budget enforcement.

- [ ] **Step 1: Write exact reference RED tests.**

~~~rust
let reference: serde_json::Value = serde_json::from_str(
    include_str!("fixtures/planning-reference.json")).unwrap();
for row in reference["initial_values"].as_array().unwrap() {
    let p: Protocol = serde_json::from_value(row["protocol"].clone()).unwrap();
    let prior: OwnPrior = serde_json::from_value(row["prior"].clone()).unwrap();
    let expected: ScoreFraction = serde_json::from_value(row["value"].clone()).unwrap();
    let policy = CompiledPolicy::build(&p, prior, &Limits::registered().unwrap()).unwrap();
    assert_eq!(policy.initial_value(), expected);
}
~~~

Run cargo test -p sugarscape-core surface_discovery::tests::planner and retain numerical RED.
- [ ] **Step 2: Implement joint conditioning.** Keep original IDs/full field/helper/history state; condition mass once. Replay own actions as interventions, future bits latent until issued, helper candidate point prior. Partition by local observations/public events, not actual peer trace. Cache full relevant local decision and candidate-support/state, never model marginals alone.
- [ ] **Step 3: Implement exact backward induction.** Raw slots enumerate eight paid options in tie order; round boundaries Stop/Continue up to 3; live Inspect/Attempt. SuppliedAdaptive restricts calibration recipe and optimizes live continuation; NoProbe/InspectOnly stop at zero, InspectOnly fixes inspection. KnownRaw has raw menu and matching point prior. Alternatives follow declared continuation. Investigator reward uses own exact target posterior. Numerical fraction ties preserve first declared option; all arithmetic is checked.

Use this numeric tie helper after constructing alternatives in declared order:

~~~rust
fn maximizing_choice(alternatives: &[ChoiceValue]) -> Result<Choice, Error> {
    let mut best = alternatives.first()
        .ok_or_else(|| Error::InvalidHistory("empty choice set".into()))?;
    for option in alternatives.iter().skip(1) {
        if option.value.checked_cmp(best.value)? == std::cmp::Ordering::Greater {
            best = option;
        }
    }
    Ok(best.choice)
}
~~~

The recurrence is explicit: for each action subtract all future investigator paid costs, partition original candidate mass by the next local observation, and weight each child optimal remaining value by its exact conditional mass. At prediction add 12 times the larger target posterior mass; at Finished return zero remaining value. Already incurred costs are not charged again. Forced-policy nodes select their prescribed option while reporting alternatives under that same downstream policy. Cache keys include full relevant state/support and public decision clock.
- [ ] **Step 4: Pin privacy/cache/limits/dominance.** Compare complete continuation witnesses. Equal marginals/different fields or helper histories must preserve different referenced values. Tampered View beliefs/credits/bits reject; own decisions add no likelihood, bits condition once, zero mass stays zero. Check signed rational ordering. Limits { states:1, branches:1, allocation_bytes:1 } must produce ResourceLimit before growth, never UnsupportedHistory. Enforce checked counters/allocation estimates before caches/branches allocate; measure actual memory separately. Check equal-helper embedding before both-role expected Raw≥Supplied; no per-mechanism or transfer assertion.
- [ ] **Step 5: GREEN engineering/review/commit.** Record compiler counts/time/peak RSS vs registered caps; label engineering, no scientific first output. Independent optimality/information review, focused tests/scoped fmt/core Clippy. Commit feat(surface): optimize raw probes over exact local beliefs.

## Task 4 / Stage 2: Policy Replay, Evaluation and Metrics

**Files:** replay.rs/evaluation.rs/metrics.rs and replay/evaluation tests.
**Consumes:** Compiled investigator policy. **Produces:** Replay/Episode/Panel and full-population metrics.

- [ ] **Step 1: Write independent episode/replay RED tests.** Parse each reviewed evaluation row into protocol/prior/environment/sequence and build registered limits:

~~~rust
let policy = CompiledPolicy::build(&protocol, prior, &limits).unwrap();
let replay = Replay::build(&protocol, &policy, &limits).unwrap();
let actual = run_episode(&protocol, environment, sequence, &policy, &replay).unwrap();
assert_eq!(serde_json::to_value(&actual).unwrap(), row["episode"]);
~~~

Run cargo test -p sugarscape-core surface_discovery::tests::replay and evaluation filters; include counterfactual investigator-inference prefixes, while responder replay rejects prefixes inconsistent with the declared policy. Do not substitute off-policy replay for declared-policy likelihoods.
- [ ] **Step 2: Implement candidate-local responder replay.** Re-execute original worlds under declared compiled policy, not actual peer history or fixed Ensemble likelihood. Deduplicate a world's contribution when private selection gives identical peer prefix. Public stopping informs; private selections do not. B boundary and issued bit chronology remain exact. Predictions use own posterior majority; rewards/lineage stay evaluator-only.

Task 4 defines the shared terminal prediction helper used by the host, never by physical transitions:

~~~rust
pub fn predict_target(belief: &Belief) -> Result<bool, Error> {
    let target = belief.target
        .ok_or_else(|| Error::InvalidHistory("prediction requires issued target".into()))?;
    Ok(target.checked_cmp(Probability::new(1, 2)?)?
        == std::cmp::Ordering::Greater)
}
~~~
- [ ] **Step 3: Implement actual chronological host.** Infer both histories after each paid observation/public transition before advancing. Halt at first unsupported owner/prefix with partial costs/last supported belief; no subsequent wait/reset/bit/prediction. DataFlip helper stays nominal SP. Resource errors propagate separately; completed wrong prediction is a valid supported outcome.
- [ ] **Step 4: GREEN whole-population and legacy equality.** Panels contain all 256 sequences once. Exact valid/failed/unavailable mass, spent and optional terminal reward/net/accuracy/group utility; three primary contrasts per eight strata and role-specific uniform expectations. No survivor means/unavailable paired terminal differences. Explicitly map new policy/choice names and compare SuppliedAdaptive full histories/clocks/actions/predictions/decisions/alternatives/costs to all original active comparator exports. Test first own/peer contradiction, earlier raw DataFlip, no mutation after halt, partial prediction retention, and supplied versus observed certainty.
- [ ] **Step 5: Independent review and commit.** Resolve reference/privacy/population discrepancies; focused tests/scoped fmt/core Clippy. Commit feat(surface): evaluate policy-conditioned discovery and transfer.

## Task 5 / Stage 3: Diagnostic and Checked CLI

**Files:** Diagnostic modules/tests, new CLI module/tests, additive main dispatch.
**Consumes:** Exact evaluation/reference projections. **Produces:** Fixed census, compact reconstruction/integrity/summary and CLI.

- [ ] **Step 1: Write census/tamper/emitter RED.** Assert 48 settings/40 in-family/eight transfer; five in-family policies per mechanism/role and four per transfer role, no KnownRaw transfer; sequences 0..255 once. A partial report cannot validate:

~~~rust
let mut report = incomplete_reference_report();
report.passed = true;
assert!(!report_integrity(&report, &limits).unwrap());
~~~

Define incomplete_reference_report() as test-only DiagnosticReport constructor using reviewed compact projections and missing census rows. Add write-error/late-flush-error writers. Run cargo test -p sugarscape-core surface_discovery::tests::diagnostics and cargo test -p sugarscape-cli surface_discovery, without judged collection.
- [ ] **Step 2: Implement compact authoritative records.** Intern complete histories/prefixes without losing choices/alternative values/failure observations. Check all indexes/rationals/IDs/census entries. Reconstruct episodes, recompute current source behavior and independent reference projections; cached passed flags never validate imports/reports. Summary contains complete declared comparisons and unavailable mass with source/report hashes.
- [ ] **Step 3: Implement deterministic CLI emission.** surface-discovery diagnose has no tuning flags, uses registered limits, deterministic JSON/newline and checked flush. Exit 0 requires current-payload/reference integrity; invalid report emits failed JSON then exit 2; write/flush failure exit 1. Resource/protocol failure remains contextual, never success with empty panels. Follow existing emitter pattern without editing it. Test parsing/help/emission/flush and rejected flags.

In the new CLI module define invalid(error: Error) -> crate::Failure using FieldError::new("surface_discovery", error.to_string()). The emitter owns a mutable DiagnosticReport and a std::io::Write sink:

~~~rust
report.passed = report_integrity(report, limits).map_err(invalid)?;
serde_json::to_writer(&mut *writer, report)
    .map_err(|e| crate::Failure::Io(e.to_string()))?;
writer.write_all(b"\n").map_err(|e| crate::Failure::Io(e.to_string()))?;
writer.flush().map_err(|e| crate::Failure::Io(e.to_string()))?;
if !report.passed {
    return Err(crate::Failure::Invalid(vec![FieldError::new(
        "surface_discovery", "diagnostic report failed current-payload validation")]));
}
~~~

Here limits is the registered Limits reference and writer implements Write; the enclosing emit returns Result<(), crate::Failure>. Validation failure clears check flags before serialization. Preserve malformed-report/resource errors contextually rather than converting them into episode evidence or policy outcomes.
- [ ] **Step 4: GREEN mutations/reconstruction.** Independently tamper observation, prefix, choice, alternative numerator, cost, role, environment, sequence, index and passed; missing/duplicate/reordered settings/sequences reject. Exact compact→complete equality to references. Successful full census CLI integration is the Task 8 gated invocation plus an evidence-side validation harness consuming its retained stdout/status; it does not add an ignored success test or run an unregistered extra scientific series.
- [ ] **Step 5: Review and commit.** Focused core/CLI tests/scoped fmt/workspace Clippy/release CLI build, independent integrity review. Commit feat(surface): add frozen discovery diagnostic and integrity checks.

## Task 6 / Stage 4: Rust Browser Adapter and WASM Feasibility

**Files:** Discovery adapter/projection/examples, common browser dispatch/codecs/catalog/tests, WASM experiment-view tests.
**Consumes:** Complete native Episode. **Produces:** Sixteenth catalog entry, bounded exact exports.

- [ ] **Step 1: Write strict input/native-payload RED.** Predetermined default:

~~~json
{"study":"surface_discovery","protocol":{"investigator":"A","policy":"RawAdaptive","ids":{"surface":"status-field","agents":["Agent-A","Agent-B"]}},"environment":{"InFamily":"SharedPersistent"},"sequence":0}
~~~

Reject sequence 256, unknown/missing fields, invalid IDs/policies/mechanisms, KnownRaw+DataFlip, oversized inputs before planner execution. Preserve valid opaque renaming. Compare payload to wire::lossless_value(&native), complete checkpoint reference and all 15 old catalog entries. Run cargo test -p sugarscape-core browser_experiments::tests::discovery.
- [ ] **Step 2: Implement single-episode/projection wrapper.** FamilySurface descriptor controls.settings lists 48 fixed settings and sequences 0..255. Compile/replay/run selected setting only. Public projection permits clock/reset/public stop/status; local permits own prefix/credits/issued bit/belief/inference status/decision/alternatives/prediction/last paid read/nominal discovery. Actual field/peer state/lineage only Researcher. No controller uses display data; after-slot precedes reset, private choice never peer pane; last read is historical rather than current storage knowledge.

Add dispatch arms in the existing common modules, keeping the public boundary unchanged:

~~~rust
// Input::study:
Self::SurfaceDiscovery { .. } => StudyId::SurfaceDiscovery,
// normalize_input validation:
Input::SurfaceDiscovery { .. } => super::discovery::validate_input(&input)?,
// browser_experiments::run:
Input::SurfaceDiscovery { .. } => discovery::run(&input),
// catalog::rules_identity:
StudyId::SurfaceDiscovery => super::discovery::rules_identity(),
~~~

Append discovery::descriptor() once in catalog(). The new adapter wraps native Episode as lossless payload, builds owned/public/researcher checkpoints with record::push_checkpoint, and calls record::check_bounds before returning. New records take the inherited EpisodeKind/EPISODE_VERSION/Semantics::Trajectory; their rules_identity names surface-discovery protocol 1 and hashes native engine dependencies, adapter schema and reviewed references separately from historical measurement hashes.
- [ ] **Step 3: GREEN bounds/wire/source reconstruction.** Unbounded integers/rationals use canonical decimal strings through existing wire helper; bounded selector sequence/checkpoint index keep inherited validated types. Preflight/count before retaining large components; final maximum 16 MiB/4,096 checkpoints. Same-source import reconstructs exact full export. Altered event/alternative/cost/identity rejects. Update necessary common source guards with before/after receipts and unchanged original payloads; preserve old source/report identities and contextual stale-import rejection.
- [ ] **Step 4: Actual WASM feasibility gate.** Existing experiment_run_json/validate_json dispatch tests cover all 48 settings at sequences 0/255 plus independent witnesses. Measure actual WASM time/memory and compiled policy/record sizes; browser worst-case compile/replay/serialization must fit 60 seconds/16 MiB/4,096 checkpoints. Test real pending worker cancellation. If infeasible retain evidence and review exact source-bound table delivery or explicit amendment; no shorter horizon/action pruning/approximation. Gate precedes collection.
- [ ] **Step 5: Independent review and commit.** Focused core/actual Node WASM tests and web WASM build; field provenance/native parity/source guard review. Commit feat(experiments): expose bounded surface discovery episodes.

## Task 7 / Stage 4: Viewer and Browser Acceptance

**Files:** Additive TS types/controls/comparison/surface rendering and discovery tests; CSS only for measured layout need.
**Consumes:** Task 6 descriptor/perspective-projected checkpoints. **Produces:** Existing Episodes shell with raw choices/matched comparisons.

- [ ] **Step 1: Write RED comparison/privacy/lifecycle tests.** Extend comparisonKey(input: Json):string to exclude discovery protocol.policy only. matchedInputs offers other primary policies, not controls as primary treatments. Define raw as exact Task 6 JSON, obj from presentation:

~~~ts
const supplied = structuredClone(raw);
obj(supplied.protocol).policy = 'SuppliedAdaptive';
expect(comparisonKey(raw)).toBe(comparisonKey(supplied));
const other = structuredClone(supplied);
other.sequence = 255;
expect(comparisonKey(raw)).not.toBe(comparisonKey(other));
~~~

Sentinel-check selected own checkpoint through existing perspective projection/surfaceDisplay: actual model/peer history/lineage/future bit absent. Test raw Calibration(Action) alternatives and declared continuation labels. Run npm test -- src/episodes/discovery-projection.test.ts src/episodes/comparison.test.ts in web.
- [ ] **Step 2: Implement viewer reuse.** Selector recognizes new study; controls show fixed setting/sequence with truthful supplied-helper/alphabet/live labels. surfaceSettingLabel reads investigator for discovery and experimenter for active; preserve old labels. Public stop applies active/discovery, old active copy remains correct. Show raw owned paid choices separately from round purchase. Reuse surfaceDisplay/renderSurface/timeline/perspectives/results. Default Agent, labeled Researcher. Match exact clock/stage; absent opportunity unavailable, never virtual wait. Two-record limit means three-policy comparison uses sequential pairs, never three retained records.
- [ ] **Step 3: GREEN input/worker/perspective boundaries.** Run/Share captures current edits before blur; invalid number/256 blocks without replacing successful record. Imports/links use native exact validation, no autorun. Actual pending cancel/timeout/disposal terminates worker; next request fresh worker. Distinguish computational failure/unsupported partial episode/supported wrong prediction. Repeated seek/perspective cannot change actions, reveal future observations or duplicate costs. Keyboard/focus/reduced motion and 390 px layout.
- [ ] **Step 4: Full parity and real browser.** New parity test invokes fresh native CLI through existing experiment command and actual Node WASM, comparing complete exports for all 48 settings×sequences 0/255 plus independent witnesses. Real browser covers reference-established stop at zero/raw read/write/reset/ambiguity/equivalence/transfer contradiction/wrong prediction. If a declared policy never exhibits an anticipated event, record absence instead of inventing a new policy. Capture actual worker input/output/source/WASM identity and displayed fields at every checkpoint vs native. Verify export/import/share, matched controls, real cancellation/60 seconds timeout, keyboard/narrow/reduced motion. Read browser skill at execution. Host-only WASM tests/mocks are not runtime parity.
- [ ] **Step 5: Review and commit.** npm run typecheck/focused tests/full web tests for integration changes, independent privacy/visual/parity review and producer fixes. Commit feat(episodes): visualize raw probes and matched discovery policies.

## Task 8 / Stage 5: Freeze, Collect, Report and Deliver

**Files:** Protocol source binding, product guide/compact summary/recorded registration, appended guide links and owned tracker closure. Large evidence remains ignored.
**Consumes:** Reviewed Tasks 1–7. **Produces:** Preserved scientific first/repeat, corrected findings and verified delivered increment.

- [ ] **Step 1: Fresh precollection review and source freeze.** Verify spec/plan coverage, numerical independence/references, equal helper embedding, both-role values, identifiability, transfer nulls, census/caps, native/WASM/browser feasibility and old source/export/guide preservation. Protocol registers census/priors/ties/metrics/caps/pre-mortem/full source inventory/toolchain/binary/WASM hashes before judged invocation. Resolve findings and refresh dependent evidence before passing gate. Commit frozen source/protocol binding before collection.
- [ ] **Step 2: Run project gates on recorded source.** One build owner; retain counts/exits/ignored/skipped/logs. Scoped formatting repairs, workspace fmt check-only:

~~~sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo build --release -p sugarscape-cli
~~~

In web: npm run build, npm run typecheck, npm test. Root: python3 -m unittest discover -s studio/tests -t studio -v. Browser receipts must bind current served source/actual WASM. Repeat gates only for dependent changes/failure/unresolved concern; disclose browser-only skips and packaging notices accurately.
- [ ] **Step 3: Exclusive first/repeat and independent validation.** Recheck frozen hashes immediately before invocation. Define evidence as owned evidence Path and binary as frozen absolute release CLI Path, with retained hashes; use pathlib/subprocess:

~~~python
with (evidence / 'first-surface-discovery.json').open('xb') as out:
    with (evidence / 'first-surface-discovery.stderr').open('xb') as err:
        result = subprocess.run([str(binary),'surface-discovery','diagnose'],
                                stdout=out,stderr=err,check=False)
assert result.returncode == 0
~~~

Record invocation status exclusively even on failure. Repeat once into exclusive repeat files with identical settings/source/binary; compare bytes and independently validate all 48 settings/12,288 episodes with streaming readers. Retain failed output; engineering invalidity needs separately named revised series, not overwritten first. Supported wrong/unsupported findings do not justify reruns. The CLI success validation harness consumes the retained first report and invocation status.
- [ ] **Step 4: Publish corrected findings and compact summary.** Report all 24 primary stratum contrasts, both-role expectations, controls, chosen action sequences/equivalence classes, exact costs/accuracy/net/helper/group outcomes, failures/unavailable mass and computation. Separate changed trace/confidence from benefit; state zero/adverse results. No learned language/unrestricted discovery/incident fidelity claim. Generate source/report-bound summary_json and verify against full report; browser summary separate from selected episode. Append guide links preserving old prefixes. No scientific engine/settings change after results; retain/disclose revisions and a new series if scientific source changes. Summary registration and product documentation are delivery changes: retain separate first-measurement and final-delivery source manifests, disclose the delta, and refresh affected browser/identity checks without relabeling the measurement source.
- [ ] **Step 5: Whole-increment review, commit and integration.** Fresh reviewer checks final diff/findings/report/summary/hash matches/limits; producer resolves findings with affected gates refreshed. Snapshot/remove only owned tracker, preserve evidence. Commit docs(surface): publish exact discovery and transfer findings. Inspect main/status/worktrees/remotes for concurrent work; normal merge incoming main into crowd if needed, resolve only owned conflicts, preserve foreign/untracked files. Safely merge finished crowd into main, push without force, check exact merged commit CI/Pages. Source-bearing incoming changes require dependent parity/source/browser refresh; documentation-only changes still require inventory/diff checks. Report actual final commit/results/limitations; queued CI is not completion.

## Self-review and Handoff

Coverage: Task 1 protocol/reference/limits; Task 2 clock/local records; Task 3 inference/optimal choices/interventions/cache; Task 4 replay/failures/metrics; Task 5 census/reconstruction/CLI; Task 6 bounded exports/WASM; Task 7 viewer/privacy/comparisons/lifecycle/parity; Task 8 registration/collection/preservation/findings/integration. Each Review Focus has tests in its owning tasks. No raw optimum or scientific outcome is invented.

Reference enumeration and browser feasibility may reject the design as computationally impractical. Retain evidence and review an explicit amendment rather than weaken the question silently. Unknown numerical results are outputs of reviewed derivation/registered measurement, not parameters chosen after observing production results.

Written plan review precedes implementation. Execution remains fresh subagent-driven with independent reviews under standing commit/integration authorization.
