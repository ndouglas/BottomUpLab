# Key Delegation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user has already selected fresh implementers and independent reviewers. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build, measure, and visualize the exact carrier-only key delegation game, separating physical access, private incentives, and coordination.

**Architecture:** A sibling native `key_delegation` laboratory owns strict types, atomic ownership, local histories, the complete schedule game, and source-bound reporting. A bounded native adapter supplies the existing episode worker; TypeScript projects and renders records without implementing physics. Independent protocol and numerical registration precede production implementation; feasibility and precollection review precede the exclusive first/repeat measurement.

**Tech Stack:** Existing Rust/serde/clap workspace, Python standard-library independent references, portable SHA-256, wasm-bindgen/wasm-pack, TypeScript/Vite/Vitest and DOM/SVG. No new dependency or generic world/planner abstraction.

**Spec:** [Key delegation design](../specs/2026-10-11-key-delegation-design.md), commit `1d42e54485d9918ef937398c44ecfbccba95e8e5`, approved by the user's “LGTM!” on 2026-10-12. This plan does not alter its physical rules.

**Status:** Execution approved by the user on 2026-10-12 (“Great! Let’s proceed.”). Task 1 preservation and independent registration are complete, with reviewed protocol and exactly three independently approved reference fixtures. Tasks 2–3 native kernel, exhaustive game, metrics, and preallocation budgets are complete and independently approved. Task 4 source-bound diagnostic and CLI are next; Tasks 5–7 have not started. No scientific measurement has begun. Feasibility and precollection gates remain binding.

## Global Constraints

- Fixed P–O–C graph. P–O is always traversable; O–C requires the moving Agent's key. No global open state, Hold, key copying, dropping, remote passing, return, theft, or postterminal phase. One key has exactly one owner.
- Receiver starts O. Handoff distance 0 puts Giver O; distance 1 puts Giver P. Main census starts with Giver possession. These distance labels differ from the historical plate experiment.
- Six rounds, Giver then Receiver, 32 credits each. Wait costs zero; Approach and Advance cost one; Pass costs `t`, even when blocked. Success requires permission, possession, and co-location. Sender pays; receipt costs the Receiver no action or credits.
- Advance executes exactly one primitive: Move O→C, Pickup, Move C→O, Deliver according to own position/cargo. Both crossings require own possession. Reward total 12 splits `b` / `12-b`, once after delivery. Unexecuted slots are uncharged; deadline grants zero reward with spending retained.
- Grid order: permission disabled/enabled, price 1/2/4, share 0/3/6, distance 0/1, Giver ID, Receiver ID. 36 strata × 729 × 64 = 1,679,616 raw pairs. No sampling, strategy pruning, outcome-based merging, or adaptive grid.
- Giver digits Wait/Approach/Pass = 0/1/2; Receiver Wait/Advance = 0/1, most-significant-round first. Main pure membership uses only private payoff maxima and retains every tie. Team ordering is group net descending, spending ascending, delivery round ascending (none after six), Giver ID, Receiver ID.
- Six controls per stratum, 216 availability/outcome records. InitiallyEquipped changes starting ownership and has no main-game classification. 84 contrasts: 18 permission, 24 adjacent-price, 24 adjacent-share, 18 distance. Preserve fixed-word invariants separately from reselected-control differences.
- Only own word, own state/feedback/history and public clock/termination enter execution. Sense initially and before/after own actions. A successful transfer gives both participants a timestamped own-ownership receipt; it never refreshes the passive map or reveals peer finances. A failed transfer does not notify the Receiver.
- Team and individual controls have identical public initial information and local execution interfaces. Read-only capture/replay creates no observations. Unknown Agent masks remain empty of peer/evaluator data.
- Existing crowd worktree/branch; inspect Git before edits. One product writer/build owner; root owns orchestration, tracker, evidence and Git. Never recreate, stash, discard, force, bypass hooks, or touch another worktree's work.
- Preserve frozen Gate Assistance sources, first/repeat bytes, settings, records, defaults, and evidence. Never stage `.claude/`, papers, survey output, or ignored evidence. Narrow shared registration/source-guard changes require exact before/after receipts and independent review; no blanket identity exception.
- Fresh implementer and independent review per task, separate source-informed protocol and source-blind numerical seats, fresh precollection and whole-increment review. Preserve the worktree and all evidence after completion.
- Rustfmt only edited files with child traversal disabled; never `cargo fmt` in survey. Stop after three failed attempts and reassess. American spelling and “Agent” in code/docs.
- Commit working increments; safely integrate/push finished work and verify exact CI/Pages. Register/freeze settings and caps before collection. Exactly one first diagnostic and one repeat; tests must not launch extra successful key diagnostics. Scientific changes after results require a new registered series.

## Review Focus

1. Successful Pass changes passive own possession without refreshing its dated map; failed Pass sends no receipt. Task 2 pins atomic receipt chronology; Tasks 5–6 pin all-checkpoint Agent rendering.
2. InitiallyEquipped has changed initial ownership and must never inherit main-game regrets/equilibrium membership. Tasks 3–6 reject forged classification and visibly label this control.
3. Terminal suffixes may become executed under unilateral deviation. Task 3 uses Giver IDs 486/487 and Receiver IDs 60/15 to retain the distinguishing suffix; canonical hashes still include complete words.
4. Strict nested duplicate keys, wrong numeric atoms, null/absent selection, stale receipts, and edited text fields must reject without replacing a retained episode. Tasks 2, 4, 5 and 6 own explicit rejection tests.
5. Adding a study changes shared registration bytes but cannot silently invalidate historical Gate records or accept unknown identities. Tasks 1 and 5–7 bind complete same-target before/after exports and the exact permitted registration delta.

---

## File map and five owned stages

Stages are: (1) preservation/registration, (2) kernel/game, (3) diagnostic/CLI, (4) adapter/viewer, (5) frozen collection/delivery. Tasks 1–7 below map to these stages; root creates `IMPLEMENTATION_PLAN.md` only after plan approval, snapshots it when complete, and removes only that owned tracker after successful integration verification.

New native files live under `crates/sugarscape-core/src/key_delegation/`: `types.rs` (domain), `validation.rs` (strict wire), `controller.rs` (own-state choice), `observation.rs` (sensing), `kernel.rs` (transactions/capture), `canonical.rs` (trace codec), `sha256.rs` (raw portable hashing), `budget.rs` (precommit caps), `game.rs` (census/best responses), `metrics.rs` (controls/contrasts), `diagnostics.rs` (report integrity/summary), `source_inputs.rs` (ordered embedded closure), `protocol.rs` (registration), `references.rs` (sealed fixture decoding), and focused tests plus exactly three reference JSON files.

New browser adapter: `crates/sugarscape-core/src/browser_experiments/key/{mod.rs,projection.rs,duplicate.rs}`. Shared registration files: `browser_experiments/{mod.rs,input.rs,catalog.rs,record.rs,equivalence.rs,wire.rs,recorded.rs}` and core `lib.rs`. New CLI: `crates/sugarscape-cli/src/key_delegation.rs`, dispatch in `main.rs`.

New web modules: `web/src/episodes/key-{projection,view,comparison}.ts`, corresponding focused tests, `key-input.test.ts`, `key-parity.test.ts`, and `key.browser-test.ts`. Integrate through existing `controls.ts`, `view.ts`, `projection.ts`, `types.ts`, `comparison.ts`, `recorded.ts`; avoid unrelated renderer refactoring.

New documentation: `docs/superpowers/protocols/2026-10-12-key-delegation.md`, `docs/key-delegation.md`; final links/claim records in `docs/{roadmap,papers,reproducibility}.md`. New evidence workspace: `.superpowers/sdd/2026-10-12-key-delegation/`; never reuse or modify the completed gate ledger.

The shared core `build.rs` and `build_support/surface_discovery.rs` remain byte-unchanged: they are frozen Gate source inputs. New raw receipt hashing is implemented locally in `sha256.rs`, adapting the existing portable algorithm without editing its original module. Bind that implementation in the key closure and test independent known answers. This avoids a host-generator dependency and a historical scientific-source change.

## Interfaces fixed for task handoffs

Public domain: `Stratum::new(e: bool, t: u8, b: u8, d: u8) -> Result<Stratum, Error>`; `Ids::new(giver: String, receiver: String) -> Result<Ids, Error>`; `InitialOwner::{Giver, Receiver}`; `Config::new(stratum: Stratum, ids: Ids, initial_owner: InitialOwner) -> Result<Config, Error>`; `Profile::new(giver: u16, receiver: u8) -> Result<Profile, Error>`. These private-field validated types expose read-only accessors; strict wire rejects arrays/unknown fields/duplicates and noninteger IDs.

Execution: `run_episode(&Config, Profile) -> Result<Episode, Error>`; `run_with_capture(&Config, Profile, &mut dyn FnMut(&CapturePoint) -> Result<(), Error>) -> Result<Episode, Error>`; `choose(&View) -> Result<Intent, Error>`; `trace_hash(&Episode) -> Result<String, Error>`. `Outcome` exposes exact `spent`, `reward`, `net: [i16; 2]`, `group_net`, optional delivery round, attempts/blocked counts, and transfer count. `Episode` retains config, both raw words/profile, all events, histories and outcome. `View` contains only the own-state/local interface; `CapturePoint` separately holds both views and researcher physical state.

Game: `Limits::registered() -> Result<Limits, Error>`; `compile(Stratum, &Limits) -> Result<GameTable, Error>` (main Giver-owner game only); `classify(&GameTable, Profile) -> Result<Classification, Error>`; `Selector::{SoloAttempt, SuppliedHandoff, TeamOptimal, NashLex, NashBestGroup, InitiallyEquipped}`; `resolve(&GameTable, Selector) -> Result<Option<Selection>, Error>`. `Selection` contains initial owner, profile and optional main classification; InitiallyEquipped always has classification null, explicitly labeled changed-initial-condition control.

Reporting: `diagnose(&Limits) -> Result<Report, Error>`; `report_integrity(&Report, &Limits) -> Result<bool, Error>`; `summary_json(&Report, &Limits) -> Result<String, Error>`. Integrity reconstructs current-source complete evidence, never trusts a stored passed bit, supplied hash, or cached classification. `Report`, `Summary`, `Receipt`, `CompileStats`, event/observation and control wire schemas are finalized in Task 1 before consumers, with the named interfaces above retained.

Input uses `study: "key_delegation"`, `stratum: {transfer_enabled, transfer_cost, giver_share, distance}`, `ids: {giver, receiver}` and exactly one `selector` or `profile: {giver, receiver}`. Selectors serialize with their listed PascalCase names. Domain versions are `key-delegation-diagnostic-v1`, `key-delegation-summary-v1` and trace receipt path `key-delegation-trace-v1`. Browser identity is `key-delegation:protocol-1:browser-key-1:native-source-sha256:{native}:adapter-source-sha256:{adapter}:reviewed-reference-sha256:{references}` with actual component digests. For InitiallyEquipped, payload fields explicitly contain `classification: null` and `classification_scope: "changed_initial_control"`; normal main-game episodes use `classification_scope: "main"`. Public raw input cannot set initial owner.

`SOURCE_INPUTS` embeds the 15 native files listed in the file map, root `Cargo.toml`, core `Cargo.toml`, and `Cargo.lock`, in declared order. `REFERENCE_INPUTS` embeds the three fixtures in kernel/game/limits order. References are authenticated separately rather than omitted from dependency closure. Record toolchain and native/WASM artifacts independently; no derived receipt file is included in its own hash. The browser identity additionally binds all adapter files and shared registration dependencies actually used.

### Task 1: Preservation baseline and independent registration

**Files:** Create the declarative protocol; create three fixtures under `key_delegation/tests/fixtures/{kernel-reference,game-reference,limits-reference}.json` only after independent comparison; write all author/reviewer derivations and receipts under the new ignored workspace. Read existing native laboratory, browser transport, collectors and source selectors without changing production.

**Interfaces:** Consumes approved spec and existing selected-episode APIs. Produces reviewed exact wire/chronology, sealed source-blind references for every pair/control/contrast, independently derived logical work/payload caps, and preservation receipts. No production key module is compiled in this task.

- [ ] Capture Git/worktree state; root opens only this new plan's ledger and five-stage tracker. Bind the approved spec/plan identities and record ownership. Archive named old source/settings/first/repeat receipts without modifying prior ledgers. Capture actual existing release native/WASM artifacts before any rebuild. Export representative episodes for every current study to exclusive native and actual Node WASM paths; preserve each target separately because known legacy float differences exist.
- [ ] A source-informed protocol author fixes exact clock phases, event order, atomic Pass/receipts, postaction sensing, settlement/termination, all required/null fields, numeric domains, control labels, source/trace codecs, cap charge units and error semantics. A fresh reviewer approves this protocol before numerical handoff. Successful Pass must yield two own-transfer receipts before the acting Giver's postaction observation; neither creates Receiver sensing. Delivery must preserve cargo consumption/reward settlement/termination ordering and both own ledgers.
- [ ] A source-blind numerical author receives only the reviewed protocol, not key production source. Derive all 36 complete row arrays, best responses/pure sets, selectors, 216 controls, 84 contrasts, chronology witnesses and cap counts. A separate reviewer seals its own independent derivation before reading the author's output; compare every value, not only aggregate hashes. Keep first failures and revisions. Root accepts exactly three reviewed fixture JSONs; no fourth generated fixture.
- [ ] Run deterministic standard-library checks with explicit arithmetic:

```python
assert 729 * 64 == 46656
assert 2 * 3 * 3 * 2 * 46656 == 1679616
assert 36 * 6 == 216
assert 18 + 24 + 24 + 18 == 84
for t in (1, 2, 4):
    for b in (0, 3, 6):
        for d in (0, 1):
            assert (b-t-d) + (8-b) == 8-t-d
```

These are arithmetic checks, not a substitute for the independent full derivations. Numerical author and reviewer choose distinct implementations from protocol prose; production tests may not generate their own oracle.
- [ ] Root checks protocol/spec consistency, all fixtures and raw receipts, preservation scope and independently derived caps. Any behavioral ambiguity or feasibility-driven scope change requires a reviewed amendment before production. Commit only protocol and three fixtures, with `docs(key): register independently derived key game`; independent evidence remains ignored. No `key-delegation diagnose` invocation.

### Task 2: Strict domain, own-state kernel and canonical traces

**Files:** Create native `mod.rs`, `types.rs`, `validation.rs`, `controller.rs`, `kernel.rs`, `observation.rs`, `canonical.rs`, `sha256.rs`; focused `tests/{mod,kernel,validation,canonical}.rs`. Modify core `lib.rs` to export only the sibling laboratory. Do not edit old laboratory files or shared build script.

**Interfaces:** Consumes Task 1 protocol/fixtures; produces domain/execution/hash interfaces above. Add the future public game/report exports only when their task is implemented; every intermediate commit compiles.

- [ ] Write RED tests for enabled near/far handoff, disabled attempt costs, missing possession/co-location, repeat Pass, both authorized crossings, unsolicited receipt, failed-pass silence, passive map timestamp, settlement and deadline. Include the exact minimal physical witness:

```rust
#[test]
fn supplied_near_handoff_charges_sender_and_delivers_once() {
    let s = Stratum::new(true, 1, 3, 0).unwrap();
    let ids = Ids::new("Agent-A".into(), "Agent-B".into()).unwrap();
    let c = Config::new(s, ids, InitialOwner::Giver).unwrap();
    let e = run_episode(&c, Profile::new(486, 60).unwrap()).unwrap();
    assert_eq!((e.outcome.spent, e.outcome.net, e.outcome.delivery_round),
               ([1, 4], [2, 5], Some(4)));
}
```

- [ ] Run `cargo test -p sugarscape-core key_delegation::tests -- --nocapture`; record the expected absent-module/API failure. Implement the minimal reviewed state machine using own-state choice and atomic validate/charge/mutate/receipt/sense ordering. `choose` must have no peer-word/world parameter. Own possession changes through the receipt without passive map replacement.
- [ ] Add actual strict-wire tests for duplicated nested ownership keys, Boolean/float/string/negative/overflow IDs, unknown roles/cells/tokens/fields, missing required nullable fields, impossible two-owner states and forged receipt history. Complete Episode validation reconstructs the native trace; structural validity alone is insufficient.
- [ ] Implement local raw SHA using the existing portable algorithm with new-module visibility, and registered sorted-key exact-integer trace codec. Verify independent SHA-256 answers for empty bytes, `abc`, multiblock input and streamed/chunked boundaries; verify every registered kernel witness and renamed-Agent metamorphic result. Hash full words even when suffixes did not execute.
- [ ] Run focused Rust tests and `cargo clippy -p sugarscape-core --all-targets -- -D warnings`; format only edited Rust paths with `rustfmt --edition 2021 --config skip_children=true`. Root reviews the source receipts and fresh independent task review before committing `feat(key): execute atomic carrier-only delegation`.

### Task 3: Complete game, controls, contrasts and precommit budgets

**Files:** Create `game.rs`, `metrics.rs`, `budget.rs`, `tests/{game,metrics,full_reference}.rs`; extend native exports and scoped kernel allocation hooks.

**Interfaces:** Consumes exact Episode/Outcome/trace codec and three fixtures; produces `Limits`, `CompileStats`, `GameTable`, `Classification`, `Selector`, `Selection`, controls and contrasts. Main compile always initializes Giver possession; InitiallyEquipped uses a separately executed Receiver-owner config.

- [ ] Write RED synthetic payoff-matrix tests for ties, negative values, multiple/no pure sets and distinct team selection ordering. Add real suffix activation, initially-equipped exclusion, blocked costs and all 84 ordered contrasts. Pin the distinction using executed projections rather than full hashes:

```rust
let config = Config::new(
    Stratum::new(true, 1, 3, 0).unwrap(),
    Ids::new("Agent-A".into(), "Agent-B".into()).unwrap(),
    InitialOwner::Giver,
).unwrap();
let a = run_episode(&config, Profile::new(486, 60).unwrap()).unwrap();
let b = run_episode(&config, Profile::new(487, 60).unwrap()).unwrap();
assert_eq!(a.events, b.events); // final Approach suffix did not execute
assert_ne!(trace_hash(&a).unwrap(), trace_hash(&b).unwrap());
let late_a = run_episode(&config, Profile::new(486, 15).unwrap()).unwrap();
let late_b = run_episode(&config, Profile::new(487, 15).unwrap()).unwrap();
assert_eq!(late_b.outcome.net[0], late_a.outcome.net[0] - 1);
```

If canonical events contain embedded complete profiles, compare their registered executed-action projections instead; do not weaken the suffix assertion or merge strategies.
- [ ] Run the focused tests to RED. Implement exhaustive ordered row generation and both independent payoff maxima scans, retaining raw suffixes and all private ties. `classify` rejects wrong-stratum/length tables and out-of-range profiles. `resolve(InitiallyEquipped)` returns initial owner Receiver, profile `(0,60)`, and null classification. Its physical outcome is never indexed from the main table.
- [ ] Implement controls `(0,63)`, near `(486,60)`, far `(405,30)`, team and pure selectors; keep absence explicit. Implement permission/price/share/distance contrasts and fixed-word share invariants. Control records include initial ownership and classification applicability rather than zero-filled fabricated regrets.
- [ ] Apply independently registered caps before each event/history/row/response retention, before allocation/state commit. Add zero, one-below, exact-boundary and one-above tests for each counter; `Limits::registered` rejects malformed sealed limits. Keep logical payload units distinct from RSS. Internal trusted hash paths may avoid redundant reconstruction only after a validated kernel result, with tamper tests on every public entry point.
- [ ] Run all 36 production table comparisons against independently sealed row outcomes/trace hashes and pure sets. This is engineering reference verification, not scientific collection. Measure native/actual-WASM selected-run feasibility and report retention estimates; preserve failures. No diagnostic report is emitted. Run focused tests and all-target Clippy, obtain fresh independent review, then commit `feat(key): enumerate private and team schedule outcomes`.

### Task 4: Source-bound diagnostic, strict CLI and streaming validator

**Files:** Create `diagnostics.rs`, `source_inputs.rs`, `protocol.rs`, `references.rs`, `tests/diagnostics.rs`; create CLI `key_delegation.rs`, modify CLI `main.rs`. Create ignored standalone `stream-validator/{validator.py,test_validator.py,precollection-seal.json}` through the original source-blind numerical author and independent reviewer.

**Interfaces:** Consumes complete game/metrics/caps and fixtures; produces reporting interfaces, ordered embedded raw receipts and CLI `key-delegation diagnose`. Source inventory covers every behavioral/hash/reference-decoder/diagnostic dependency actually used, including portable hash code; reference inventory covers exactly the three fixtures. There is no generated self-hashing metadata or shared `build.rs` edit.

- [ ] Write RED tests for reordered/missing/duplicated strata or rows, changed ownership, forged classification on InitiallyEquipped, changed controls/contrasts/caps/receipts, stale source identity, false passed bit, trace corruption and reconstruction errors. A hand-built incomplete report is sufficient for transport tests; no test calls a successful whole `diagnose`.
- [ ] Implement the registered compact report with all 1,679,616 raw outcome/trace entries, complete maxima/pure membership, 216 controls, 84 contrasts, counters and provenance. `report_integrity` freshly verifies current-source semantics and all entries; `summary_json` validates first, then losslessly projects the compact measured summary. Distinguish raw file SHA from domain-bound trace/report identities.
- [ ] Implement strict CLI dispatch and tested write/newline/flush handling using the existing pattern:

```rust
fn write_report<W: std::io::Write>(report: &Report, out: &mut W)
    -> Result<(), Failure>
{
    let bytes = serde_json::to_vec(report).map_err(|e| Failure::Io(e.to_string()))?;
    out.write_all(&bytes).map_err(|e| Failure::Io(e.to_string()))?;
    out.write_all(b"\n").map_err(|e| Failure::Io(e.to_string()))?;
    out.flush().map_err(|e| Failure::Io(e.to_string()))
}
```

Operational errors must not be emitted as valid deadline episodes. Scientific validation failure retains its output and failure status. CLI success paths use retained reports only after Task 7 collection; precollection tests cover argument rejection and synthetic emission.
- [ ] The source-blind validator author implements a bounded streaming parser/comparator for complete outcomes/hashes, pure sets, controls/history, contrasts, caps and source receipts. Exact scientific integer atoms reject floats/Booleans; operational status parsing separately permits finite elapsed-time numbers and requires integer exit. Test malformed/truncated reports, every array order, changed initial ownership and field omission with tiny synthetic streams. Independently review and seal before reading any actual scientific report.
- [ ] Run validator unit tests with `python3 -m unittest discover -s .superpowers/sdd/2026-10-12-key-delegation/stream-validator -p 'test_*.py' -v`. Its tests compare synthetic streams to locally constructed expected atoms without importing Rust, invoking the CLI, or reading a measured key report. Both numerical seats retain separate programs and independently sealed output receipts in `numerical-author/` and `numerical-reviewer/` before comparison.
- [ ] Extend only actual affected collector source closures, found by inspecting F5, W1 and W1-A selectors/build stamps, to account for the new core/CLI source files. Record exact additions and preserve each collector's frozen behavioral inputs. Run their named source tests, Python metadata/manifest checks and actual build-stamp checks. Formatting touches only edited survey Rust files. Run focused Rust/CLI tests, release CLI build and Clippy; fresh review then commit `feat(key): bind exhaustive reporting to current source`.

### Task 5: Bounded native browser adapter and historical compatibility

**Files:** Create browser `key/{mod,projection,duplicate}.rs`, native/browser tests and small independent episode witnesses; modify the shared registration files listed above. Modify WASM tests only to add this study. No change to old Gate scientific files/build script or previously recorded fixtures.

**Interfaces:** New `StudyId::KeyDelegation`, `StudyFamily::Key`, and Input variant accept registered stratum, opaque Giver/Receiver IDs and exactly one selector or raw profile. Raw profiles always use Giver initial possession; InitiallyEquipped is reachable only through its explicit selector. Default is SuppliedHandoff, enabled, price 1, share 3, distance 0, clearly labeled supplied. Public `experiment_run_json` and record import remain the existing APIs.

- [ ] Write RED tests for native default/near/far/disabled/raw/InitiallyEquipped records, receipt checkpoint order, private masks, terminal/deadline results, all input quotas and strict null/duplicate rejection. Add forged initially-equipped main classification and wrong owner tests. Use one compile/request; no cross-request cache.

```rust
#[test]
fn raw_key_input_cannot_claim_initial_receiver_ownership() {
    let input = r#"{"study":"key_delegation",
        "stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},
        "ids":{"giver":"Agent-A","receiver":"Agent-B"},
        "profile":{"giver":0,"receiver":60},"initial_owner":"Receiver"}"#;
    assert!(crate::browser_experiments::normalize_input(input).is_err());
}
```
- [ ] Implement projection entirely from `run_with_capture`, never a second physics model. Local panes combine committed sensing and own transfer/settlement receipts without synthesizing observations. Researcher-only payload contains classifications/tables; initially-equipped payload says changed initial condition and classification unavailable. Record import re-executes native input and compares every typed field, with exact key equality.
- [ ] Enforce 64 KiB input/link, 16 MiB record, 4,096 checkpoints, one outstanding request, existing 60-second worker timeout, and two retained records. Verify quota/serialization failure rolls back capture without changing the scientific Episode. Test per-request timeout budgets; do not batch several fresh compiles under a single setup deadline.
- [ ] Capture actual native and Node WASM outputs across all 36 strata and required controls; compare every field/key/order and selected raw boundary profiles to sealed references/native captures. InitiallyEquipped must be included at enabled/disabled and near/far. Register exact mask checks for every emitted checkpoint and unknown Agent IDs.
- [ ] Re-export every preserved prior study on each target and compare complete typed bodies against Task 1 snapshots. Catalog grows by precisely the new study. For old adapter rules identities changed solely by shared registration, retain exact old/new source receipts and explicit historical bindings: authenticate unchanged old behavioral closure and inspect the new-study-only diff before admitting old identity with complete re-execution parity. Unknown identities and any unrelated behavioral/source changes reject. Never normalize whole rules fields or broadly ignore changed paths. Source-guard compatibility must receive independent scoped approval.
- [ ] Run native browser tests, `wasm-pack test --node crates/sugarscape-wasm`, actual web-target build and Node parity checks. Retain inherited browser-only skips precisely. Fresh independent review precedes `feat(key): export bounded private delegation episodes`.

### Task 6: Web controls, receipt-aware playback and comparisons

**Files:** Create key projection/view/comparison modules and focused Vitest/DOM/real-browser tests; integrate shared web files listed above. Preserve old source-guard fixtures and renderer semantics; separately authenticate necessary registration delta.

**Interfaces:** Import `Checkpoint`, `Json`, `EpisodeRecord`, `StudyDescriptor` from existing `types.ts` and `RenderDescriptor` from existing `catalog.ts`. Export `KeyAgent` with `id: string`, `role: string`, `own: Json`, `observation: Json`, `receipts: Json[]`; export `KeyDisplay` with `clock: Json`, `status: Json`, `agents: KeyAgent[]`, `researcher: Json`. Produce `keyDisplay(projected: Checkpoint): KeyDisplay`, `renderKey(projected: Checkpoint, descriptor: RenderDescriptor): HTMLElement`, `renderKeyResult(payload: Json): HTMLElement`, `keyComparisonKey(input: Json): string`, `keyMatchedInputs(input: Json, descriptor: StudyDescriptor): Json[]`, `keySettingLabel(input: Json): string`, and `matchKeyCheckpoint(primary: EpisodeRecord, secondary: EpisodeRecord, index: number): number | null`. Renderer view models project registered fields only; they do not add sensed state.

- [ ] Write RED DOM tests for enabled/disabled controls, selector/raw exclusivity, canonical ASCII decimal IDs (no coercion of empty/Boolean/hex/fractional text), opaque names, initially-equipped label, ownership/receipt timeline, stale passive map, private/evaluator masks and terminal comparison. Editing a field must validate its current text rather than replay a stale cached number. Rejection retains both existing records.
- [ ] Add actual WASM projection tests in `key-projection.test.ts`. Initialize the generated web WASM once using the established `readFileSync` / `initSync({module: ...})` pattern; construct each record in a separate 60-second `beforeAll` hook. Import `projectCheckpoint`/`resultPayload`, `keyDisplay`, `EpisodeRecord`, and Vitest normally. For the enabled near case:

```typescript
let record: EpisodeRecord;
beforeAll(() => {
  record = JSON.parse(experiment_run_json(JSON.stringify({
    study: 'key_delegation',
    stratum: { transfer_enabled: true, transfer_cost: 1, giver_share: 3, distance: 0 },
    ids: { giver: 'Agent-A', receiver: 'Agent-B' },
    selector: 'SuppliedHandoff',
  }))) as EpisodeRecord;
}, 60_000);
it('keeps evaluator data outside every Agent projection', () => {
  for (const at of record.checkpoints) for (const agent of ['Agent-A', 'Agent-B', 'public-only']) {
    const projected = projectCheckpoint(record, at.index, { kind: 'agent', agent });
    expect(Object.keys(projected.local)).toEqual([agent]);
    expect(resultPayload(record, at.index, { kind: 'agent', agent })).toBeNull();
    expect(keyDisplay(projected).researcher).toBeNull();
  }
});
```
- [ ] Implement key-specific controls/view/result using actual native records, dated observations and explicit own-transfer receipts. Render possession separately from global openness; label carrier-only access. InitiallyEquipped shows changed starting ownership and unavailable main-game regrets/equilibrium classification. Comparison matches exact public clock/phase/role; unavailable matching remains unavailable rather than guessing.
- [ ] Drive all native/WASM checkpoint masks through the DOM, including peer/public-only identities, transferred ownership before Receiver sensing, deadline, early termination, replay backwards, input errors, cancellation, worker termination and concurrency. Assert researcher payload and peer private words/finances are absent from Agent result panes.
- [ ] Run `npm --prefix web run typecheck`, targeted key Vitest files, and `npm --prefix web test`; build web WASM/Vite and use the supported browser skill for actual worker-backed acceptance. Bind final served renderer/helper/client/worker/glue/WASM/harness HTTP bytes before/after; capture actual errors and cleanup only owned tabs/server/workers. A separate supported-browser test harness observes unchanged production timers; do not use standalone Playwright or soften timeout/cancellation behavior.
- [ ] Retain complete old-study renderer/record parity and affected collector source-stamp proofs. Fresh independent task review then commit `feat(web): visualize carrier-only key delegation`. Recorded measured summary remains absent until collection, never populated with predicted numbers.

### Task 7: Frozen first/repeat, independent validation and publication

**Files:** Add measured fixture `browser_experiments/fixtures/key-recorded.json`, native recorded registration and complete recorded-summary web tests/view; create `docs/key-delegation.md`, update claim/roadmap/paper links. Engineering collectors, validators, release/source/toolchain/first/repeat/status/manifest files live only under the new ignored evidence workspace. Root owns tracker/plan closure and Git.

**Interfaces:** Consumes independently approved Tasks 1–6, sealed validator and references, complete feasibility proof and frozen source. Produces retained first/repeat report, independently accepted complete validation, native/browser lossless summary, exact final delivery manifest and CI/Pages proof.

- [ ] Run precollection source closure, all reference comparisons and feasibility checks; do not emit a diagnostic. Full required gates: edited-file rustfmt checks; workspace all-target/all-feature Clippy; `cargo test --workspace`; release CLI; actual Node WASM; web WASM/Vite build and TypeScript; complete web and studio tests; affected F5/W1/W1-A source/metadata/stamp gates. Existing regression tests genuinely invoke some legacy reports: explicitly inventory that scope, preserve their logs/status/source/artifacts and available outputs, and disclose deleted/in-memory outputs rather than claiming complete archival. They must not generate an extra key report.

Run sequentially under one build owner, retaining actual command/log/status/source receipts:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release -p sugarscape-cli
wasm-pack test --node crates/sugarscape-wasm
npm --prefix web run build
npm --prefix web run typecheck
npm --prefix web test
python3 -m unittest discover -s studio/tests -t studio -v
```

Formatting checks list actual edited Rust paths using `rustfmt --check --edition 2021 --config skip_children=true`; do not mutate an entire workspace. F5/W1/W1-A source checks use their existing `survey` manifest tests and normal actual build stamps, without scientific collector runs. Register exact affected test filters/metadata commands in the Task 4 report before execution and carry that receipt into precollection review. W1-A `metadata` must declare zero histories and have every inventory entry checked against actual current bytes.
- [ ] Before collection, source-blind author/reviewer close validator readiness and numerical/reference/cap agreement. Fresh precollection review approves a complete freeze manifest, actual release executable/toolchain/source/reference identities, registered settings, all caps and exclusive output paths. Root commits the frozen binding. Keep operational manifest/authorization metadata outside any self-hashing source fixed point; freeze after the final authorized engineering merge.
- [ ] Root invokes only the reviewed frozen executable, with exclusive first paths and one repeat. The collector verifies before/after source/reference/settings/executable bytes, records stdout/stderr/exits including failures, and never overwrites either output. The diagnostic invocation is:

```bash
/absolute/path/to/frozen/sugarscape key-delegation diagnose
```

The actual absolute path/hash and exclusive files are taken from the reviewed freeze manifest, not a rebuilt default target. Invoke it exactly twice; a failure remains a result pending investigation, not permission to rerun until favorable.
- [ ] Run the sealed independent streaming validator on the retained first with actual operational status and source/reference receipts. Check every raw outcome/trace, maxima/pure membership, all 216 control records, 84 contrasts and caps. Compare first/repeat full bytes. Retain parser limits in their actual units (bytes versus Unicode characters) and measured RSS separately. Any validator tooling defect uses a new retained version with focused tests/scoped review before an explicitly authorized data-only retry; never revise physics to fit outcomes.
- [ ] Archive the actual measurement binary/WASM/source/build artifacts before any delivery rebuild. Generate compact native summary from the retained first through fresh integrity, without `diagnose`; original numerical author compares every summary field against independently derived expected projection sealed before reading it. A fresh reviewer accepts that actual evidence. Preserve measurement executable identity distinctly from delivery binaries.
- [ ] Add RED native/DOM/recorded export tests; embed exactly the native summary bytes. Render all 36 strata, 216 controls/unavailability records and 84 contrasts with decimal-string integers and exact provenance. Run recorded source selectors/build stamps, affected local gates and final complete web tests. Actual supported-browser acceptance verifies recorded summary/export equals native before/after live runs, all masks, initially-equipped ownership, lifecycle bounds and owned cleanup, on final served artifacts.
- [ ] Publish every stratum, equilibria/ranges/refusal, adverse/zero/indifferent outcomes, all controls/contrasts, budgets/costs and unavailable examples. Register separate KEY-1 physical correctness, KEY-2 bounded incentives, KEY-3 coordination claims with primary-source limits. Preserve predictions separately from observations; state no learned cooperation or incident fidelity. Postmeasurement delivery-only changes have a separate artifact/source delta manifest with unchanged scientific inputs/settings/first/repeat; scientific changes require a new registered series.
- [ ] Fresh whole-increment independent review receives all handwritten source/docs/tests, exact generated-data receipts, named full files, complete validation evidence and every parked/deferred ruling. One producer-owned fix wave and scoped re-review if needed; never controller code fixes. Root commits only reviewed delivery files, inspects current main/remote/concurrent work, safely integrates without force/discard, pushes and checks exact CI plus actual Pages checkout/deploy for that commit. Concurrency cancellation may justify rerunning the same Pages event, not claiming success from another head.
- [ ] Only after actual successful integration CI/Pages, mark owned five stages Complete, snapshot/remove the owned tracker, close plan metadata and commit/push the final closure. Check that new exact commit's CI/Pages too. Then write completion only to ignored ledger/handoff with no tracked bookkeeping loop. Preserve all worktree/evidence; final response reports every chronological `Ruling:` and cost if wrong. No runtime experiment beyond the registered first/repeat.

## Plan self-review and handoff

Coverage maps ownership/chronology/privacy to Task 2; exhaustive equilibria/control distinctions/caps to Task 3; integrity/receipts/CLI/streaming validation to Task 4; strict native/WASM transport and old-record preservation to Task 5; user-facing controls/masks/lifecycle to Task 6; measured summary, scientific judgments and exact publication closure to Task 7. Task 1 closes all normative registration and independent-oracle requirements before production.

This plan uses the user's standing subagent-driven method without asking them to select it again. Review this written plan before implementation. Open five-stage tracking and the new SDD ledger only after approval; do not restart the completed Gate Assistance task, reuse its ledger, or adopt the separate Assistance Games worktree.
