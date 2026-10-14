# Paid Persistent Notes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user has already selected subagent-driven execution. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and verify a finite laboratory that separates the benefit of notes surviving their writer from information acquisition, transmission costs and private incentives.

**Architecture:** A new `persistent_notes` native module owns physical transactions, private observations, supplied routines and exact evaluation. A separately registered protocol and source-blind references precede implementation. CLI, selected browser episodes and Flump playback consume native records; scientific collection consists of one frozen diagnostic and one repeat.

**Tech Stack:** Existing Rust workspace, serde/serde_json/clap, Python standard library (`Fraction`, `json`, `hashlib`, `unittest`), the separate experiments-WASM crate, TypeScript/Vitest and existing episode worker. Reuse actual Studio Flump assets; add no dependencies or learning framework.

**Spec:** [Approved design](../specs/2026-10-14-paid-persistent-notes-design.md), approved at `e3d48044d7087c243a9044668195be42714b3f2b`.

## Global Constraints

- Preserve all existing scientific engine files, fixtures, archived native identities, retained reports, gallery chapters and original `survey/out/foraging-shortcuts/` evidence. Shared registration files may receive reviewed additive changes; record any resulting current delivery-identity change rather than relabeling an archived measurement.
- Use the existing isolated branch/worktree `paid-persistent-notes-design-20261014` under `~/.config/superpowers/worktrees/SugarScape/`. Recheck main and other worktrees before integration; do not assume they stayed at the inspected base.
- Never overwrite the repository-root `IMPLEMENTATION_PLAN.md`: it belongs to Key delegation. This plan owns `docs/superpowers/plans/2026-10-14-paid-persistent-notes/IMPLEMENTATION_PLAN.md` and its own SDD ledger.
- Two goods and one hidden request; 32 credits each; four Writer opportunities; one departure boundary; at most six Reader opportunities; Wait free; correct reward 12; wrong delivery reward zero; no second task.
- Grid s∈{1,4}, w∈{1,2,4,6}, r∈{1,2}, i∈{2,4,8}, b∈{0,3,6}; 144 strata, four arms, four routines, two equiprobable requests; 4,608 episode keys. The off-grid inspection tie is a unit witness, never a census key.
- At most 16 captured boundaries per episode and 73,728 per diagnostic. Capture committed boundaries rather than both BeforeAction and AfterAction frames; ephemeral controller inputs are not additional playback frames.
- One frozen first diagnostic and exactly one repeat, at most 9,216 production episode evaluations. No full production `diagnose` during implementation tests, no hidden retries, tuning, training, seed selection or extra campaigns.
- Per diagnostic caps: 120 seconds, 512 MiB measured peak memory, 512 MiB report; first/repeat reports ≤1 GiB. Task 1 also binds deterministic work/storage caps before implementation. Feasibility changes require a reviewed amendment.
- Source-blind reference derivation and its separate review see approved declarative inputs, not new production code or outputs. Keep role/access attestations and failures.
- Selected engineering episodes and saved-data checks are separately labeled/accounted; they do not substitute for collection. Saved validation does not invoke the full collector or optimize new policies.
- Only own program/routine, public settings, own state/history and permitted observations reach controllers. Arm, hidden request and peer histories stay researcher-only, except the labeled Oracle request input.
- Exact integer/rational values, duplicate-key and unknown-field rejection, checked arithmetic, atomic failure and no trusting saved success flags.
- No push, publication, remote CI deployment or author contact. Local commits follow normal hooks. Do not disable tests, bypass hooks or format unrelated files. Stop and reassess after three failed engineering attempts at an issue.

## Review Focus

1. A visible cache/board or import must not disclose the request/note before paid sensing; test native views, all adapter checkpoints and rendered Agent panes in Tasks 2, 5 and 6.
2. Passive key receipt must update possession without making a stale map current; test timestamps in Tasks 2 and 5.
3. Writer departure must preserve its ledger and a retained note but reject later Writer actions; test both H values, erased timing and failed-handoff inactive ownership in Task 2.
4. Blank-note fallback at a private-payoff tie must guess cheaply; test the off-grid decision helper in Task 3 without expanding the census.
5. A valid wrong delivery/deadline, unavailable operational result, malicious import and output/flush failure must remain distinct; test outcome/transport/CLI behavior in Tasks 2, 4 and 5.

## File map and ownership

New native files under `crates/sugarscape-core/src/persistent_notes/`:

- `mod.rs`, `types.rs`, `validation.rs`: exported types, strict wire and input/own-history checks.
- `kernel.rs`, `observation.rs`: atomic physical state, clock, departure, local sensing and capture.
- `controller.rs`: pure Writer/Reader choice using only a `View`.
- `evaluation.rs`, `metrics.rs`: ordered census, exact request-weighted rows, contrasts and routine envelope.
- `budget.rs`: deterministic work/storage counters and precommit bounds.
- `canonical.rs`, `sha256.rs`, `source_inputs.rs`: portable identities, domain-bound receipts and exact embedded inputs.
- `protocol.rs`, `references.rs`, `diagnostics.rs`: frozen schema/caps, sealed reference comparisons and saved-report validation/summary.
- `tests/mod.rs` and focused `tests/{validation,kernel,observation,controller,evaluation,budget,canonical,diagnostics}.rs`.
- Sealed `tests/fixtures/{kernel,evaluation,limits}-reference.json`.

New integrations: `crates/sugarscape-cli/src/persistent_notes.rs`; `crates/sugarscape-core/src/browser_experiments/notes/{mod,projection,duplicate}.rs`; `web/src/episodes/{notes-view,notes-projection,notes-comparison,notes-comic}.ts` with corresponding tests; provenance-bound Flump assets under `web/public/flumps/persistent-notes-v1/`. Generated source-bound selected fixtures and the recorded summary live beside existing browser fixtures.

Additive registrations: core `lib.rs`; CLI `main.rs`; browser `mod.rs`, `input.rs`, `catalog.rs`, `record.rs`, `equivalence.rs`, `wire.rs`, `recorded.rs`; web episode `catalog.ts`, `view.ts`, `recorded.ts` and existing styles. Refresh only affected compatibility/registration stamps through existing reviewed procedures. Root Cargo files/lockfiles, core build machinery and old engine directories remain unchanged.

New research documents: `docs/superpowers/protocols/2026-10-14-paid-persistent-notes.md`, final `docs/persistent-notes.md`, and a narrow entry in `docs/reproducibility.md`. No measured findings are written before collection.

## Shared interface contract

Task 1 fixes every serialized field, event order and cap before code. These names are the task boundaries; later tasks do not independently rename them:

```rust
pub enum Role { Writer, Reader }
pub enum Cell { P, O, C }
pub enum Good { Left, Right }
pub enum Note { Blank, NoteLeft, NoteRight }
pub enum Arm { Retained, Erased, NoNote, NoAcquireNoNote }
pub enum WriterProgram { ObserveWrite, ObserveSilent, Silent }
pub enum ReaderRoutine { BlindGuess, InspectFirst, ReadAdaptive, Oracle }
pub enum BlankChoice { Inspect, GuessLeft }
pub enum Error {
    InvalidInput(String), InvalidHistory(String), InvalidReport(String),
    ResourceLimit { resource: String, limit: u64 }, ArithmeticOverflow, Io(String),
}
impl Role { pub fn index(self) -> usize; } // Writer=0, Reader=1
impl Limits { pub fn registered() -> Result<Self, Error>; }
pub struct Ratio { pub numerator: i64, pub denominator: u64 }
pub struct Stratum {
    pub acquisition_cost: u64, pub write_cost: u64,
    pub note_read_cost: u64, pub manifest_inspection_cost: u64,
    pub writer_share: u64,
}
pub struct Ids { pub writer: String, pub reader: String }
pub struct Config {
    pub stratum: Stratum, pub arm: Arm, pub routine: ReaderRoutine,
    pub request: Good, pub ids: Ids,
}
// Defaults: s=1,w=1,r=1,i=4,b=0; IDs Agent-A/Agent-B.
impl Config { pub fn example(arm: Arm, routine: ReaderRoutine, request: Good) -> Self; }
pub fn config_from_json(text: &str) -> Result<Config, Error>;
pub fn run_episode(config: &Config) -> Result<Episode, Error>;
pub fn run_with_capture<F: FnMut(&Checkpoint) -> Result<(), Error>>(
    config: &Config, capture: F,
) -> Result<Episode, Error>;
pub fn choose(view: &View) -> Result<Primitive, Error>;
pub fn blank_fallback(inspection_cost: u64, reader_share: u64)
    -> Result<BlankChoice, Error>;
pub fn registered_strata() -> Vec<Stratum>;
pub fn evaluate(limits: &Limits) -> Result<Evaluation, Error>;
pub fn diagnose(limits: &Limits) -> Result<Report, Error>;
pub fn validate_report(report: &Report, limits: &Limits) -> Result<(), Error>;
pub fn summary_json(report: &Report) -> Result<String, Error>;
pub fn source_identity() -> Result<String, Error>;
pub fn reference_identity() -> Result<String, Error>;
```

`Episode` has `config`, `checkpoints`, `outcome`; `Outcome` exposes `correct: bool`, `spent: [u64;2]`, `reward: [u64;2]`, `net: [i64;2]`, `group_net: i64`, completion/deadline and categorized costs. `Checkpoint` exposes `clock`, public status, researcher `physical`, and `own: [View;2]`. `View` contains role, only its own program/routine, public prices/own share, clock, own cell/key/cargo/credits/reward, `known_request: Option<Good>`, its dated map, note-read record and own history. It never contains a `Config`, arm, peer private history or unobserved request.

`Primitive` is Wait, Move(Cell), InspectManifest, PassKey, WriteNote(Note), Pickup(Good), Deliver. Writer chooses the note from its own paid observation. Physical writing stores a raw nonblank symbol; all registered programs are truthful. False-note raw engineering behavior is not an extra judged condition. The protocol specifies valid blocks, insufficient-credit/inconsistent-history errors and every before/after transaction.

`Evaluation` retains all 4,608 request-wise episodes, 2,304 request-weighted arm/routine rows, all four fixed-routine contrasts (4,608 request-wise and 2,304 averaged contrast keys) and 576 routine-envelope keys with all ties. Oracle is excluded from the envelope. No array is sampled or pruned. `Report.rows` is the ordered `Vec<Episode>`; `Summary` preserves ordered means/contrasts/envelopes and source/reference receipts. Task 1 binds their other serialized fields and strict null/type rules.

---

### Task 1: Preserve baselines and register independently derived references

**Stage:** 1, Registration and references.

**Files:** Create the scoped stage tracker, protocol document and only the three native fixture JSON files listed above. Keep author scripts, full derivations, review reports and source snapshots in this plan's ignored SDD workspace. Production Rust files are not created in this task.

**Interfaces:** Consumes approved design and base identities. Produces protocol version 1, strict declarative schema, ordered 4,608 keys, complete independent reference tables/traces and fixed `Limits` constants for Tasks 2–4.

- [x] **Step 1: Initialize preservation and the scoped tracker.** Check actual HEAD/status/worktrees; record the existing key/shared-surface/native fixture receipts and old browser record identities. Mark this tracker Stage 1 In Progress and all other stages Not Started. Retain the six-comic ignored gallery without altering its index. Run the SDD workspace script with this plan path; never use another plan's ledger.
- [x] **Step 2: Write the declarative protocol before new production code.** Specify field names/types, required-null versus absent rules, canonical codec (UTF-8 recursively sorted keys, compact JSON, explicit exact fractions), normalized IDs, own-view allowlist and sensing clocks. Initial and own-action sensing may show occupants/key/good identities but never manifest or field symbols. Passive possession receipt never senses. Bind the supplied Writer programs and Oracle input explicitly. Define capture at initial state, four committed Writer actions, atomic departure/erasure, each committed Reader action, and final settlement marker; at most 13 ordinary frames, registered cap 16.
- [x] **Step 3: Register transaction witnesses and deterministic resource charges.** Define Writer/Reader clocks and costs, four physical Reader credits, correct/wrong delivery, deadline, inactive-key ownership and immutable departed ledger. Charge counters before each event/row append and before serialization buffers; define exact packed logical sizes and maximum work from the declarative schema. Freeze integer `Limits` values that cover the full census beneath the design's wall/RSS/bytes caps. Independently recompute these limits; do not invent constants during later implementation.
- [x] **Step 4: Obtain independent protocol and source-blind numerical review.** Use distinct protocol author/reviewer and numerical author/reviewer roles. Numerical derivation receives only approved declarative inputs, uses its own transitions/`Fraction` arithmetic and derives all request rows, complete traces, 2,304 means, four contrasts and envelope/ties. A separate numerical reviewer recomputes them, rather than approving just hashes. Each derivation is capped at 120 seconds/512 MiB/256 MiB output; archive failures. The protocol audit checks ordering, information channels, source receipts, caps and the negative/zero cases. No production collection is authorized by reference creation.
- [x] **Step 5: Seal fixtures and demonstrate reference completeness.** Use a standard-library checker such as:

```python
import json
from pathlib import Path
reference = json.loads(Path("complete-reference.json").read_text())
strata = [(s, w, r, i, b) for s in (1, 4) for w in (1, 2, 4, 6)
          for r in (1, 2) for i in (2, 4, 8) for b in (0, 3, 6)]
arms = ("Retained", "Erased", "NoNote", "NoAcquireNoNote")
routines = ("BlindGuess", "InspectFirst", "ReadAdaptive", "Oracle")
requests = ("Left", "Right")
rows = reference["rows"]
expected = [(*s, a, p, h) for s in strata for a in arms for p in routines for h in requests]
assert len(strata) == 144
assert len(expected) == 4608
assert [tuple(row["key"]) for row in rows] == expected
assert all(len(row["checkpoints"]) <= 16 for row in rows)
assert all(row["outcome"]["spent"][j] <= 32 for row in rows for j in range(2))
assert all(row["outcome"]["group_net"] == sum(row["outcome"]["net"]) for row in rows)
```

The independent full derivation is named `complete-reference.json`; its keys use the displayed flat schema-normalized tuples. The three fixtures retain raw receipts of the complete derivations. Verify every value/trace, not merely the displayed assertions. Record author isolation and both approvals. Commit only protocol, sealed fixtures and scoped tracker. Stage 1 completes only after those gates pass.

### Task 2: Implement atomic physical state and private observations

**Stage:** 2, Native world and routines (first part).

**Files:** Create native `mod.rs`, `types.rs`, `validation.rs`, `kernel.rs`, `observation.rs`, `tests/{mod,validation,kernel,observation}.rs`; add the core module export in `lib.rs`.

**Interfaces:** Consumes sealed Task 1 schemas/witnesses. Produces Config/View/Primitive/Checkpoint/Episode/Outcome and `config_from_json`, `run_episode`/`run_with_capture` entry points; the full routine dispatch is completed in Task 3. Raw step witnesses use the Task 1 transition schema rather than an extra public controller family.

- [x] **Step 1: Write red tests for role state, custody and local information.** Start with fixture-backed transaction tests and strict input rejection, not an unreviewed whole collector. Example initial-state assertion:

```rust
let config = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Right);
let initial = initial_checkpoint(&config)?; // private test helper constructs native initial state
assert_eq!(initial.own[Role::Reader.index()].known_request, None);
assert_eq!(initial.physical.key_owner, Role::Writer);
assert_eq!(initial.physical.field, Note::Blank);
```

Define the private `initial_checkpoint(&Config) -> Result<Checkpoint, Error>` helper in the kernel test module. Exhaustively compare initial unprivileged Reader views under Left/Right and across arms; include view serialization to catch an embedded Config. Test Oracle as a separate explicit exception. Run `cargo test -p sugarscape-core persistent_notes::tests::` and retain the expected missing-module/API failures.
- [x] **Step 2: Implement minimal strict types and atomic transactions.** Use the existing key lab as a pattern, not a dependency on its scientific implementation. Decode raw objects before maps erase duplicate keys; reject unknown fields, out-of-grid selected inputs, invalid IDs and noninteger/overflow atoms. Snapshot state, check phase/role/location/custody/cost, then commit exactly one transaction. Invalid input/arithmetic is operational; a valid blocked action charges its defined attempt cost. Registered spending never binds. An impossible insufficient-credit history is rejected before mutation, not partially charged or made negative.

Use this private kernel helper for candidate-state transactions; a valid Blocked outcome is an ordinary `Ok` result after its charge, while errors leave live state untouched:

```rust
fn commit_candidate<T: Clone, F>(live: &mut T, transaction: F) -> Result<(), Error>
where F: FnOnce(&mut T) -> Result<(), Error> {
    let mut candidate = live.clone();
    transaction(&mut candidate)?;
    *live = candidate;
    Ok(())
}
fn charge(credits: &mut u64, spent: &mut u64, cost: u64) -> Result<(), Error> {
    let next_credits = credits.checked_sub(cost)
        .ok_or_else(|| Error::InvalidHistory("insufficient credits".into()))?;
    let next_spent = spent.checked_add(cost).ok_or(Error::ArithmeticOverflow)?;
    *credits = next_credits;
    *spent = next_spent;
    Ok(())
}
```
- [x] **Step 3: Implement public departure and role-local sensing.** Derive the Writer program outside its controller from Arm; choose WriteNote from the Writer's own observed request. Depart at the fixed boundary, leave ledger/trace intact, erase only in Erased, reject Writer actions thereafter. In failed-handoff witnesses retain inactive Writer key ownership. Maps expose only allowed cells/occupants/key/good identities and sensing clocks. A key receipt updates own key without updating a map timestamp. A `ReadNote` outcome is the only unprivileged source of raw note content.
- [x] **Step 4: Pass every sealed transaction witness and mutation rejection.** Cover both H values, wrong-location paid blocks, opaque board/manifest sensing, stale passive map, inactive Writer, wrong delivery settlement, deadline, duplicate key, double delivery, clone/replay, IDs and overflow. For retained/erased boundary checks:

```rust
assert_eq!(retained_after_departure.physical.field, Note::NoteRight);
assert_eq!(erased_after_departure.physical.field, Note::Blank);
assert!(!retained_after_departure.physical.writer_active);
assert_eq!(retained_after_departure.own[Role::Reader.index()].known_request, None);
```

These values come from the sealed Right-request boundary witnesses; do not call `diagnose`. Run the focused native tests and `cargo clippy -p sugarscape-core --all-targets -- -D warnings`; format only changed Rust files with rustfmt child traversal disabled.
- [x] **Step 5: Self-review and commit the native world.** Check old key/shared-surface directory receipts are unchanged and no raw request/arm is reachable from unprivileged views. Refresh affected shared registration stamps now if the additive `lib.rs` export changes their actual dependency identity; record baseline/current receipts and preserve archived scientific fixtures. Use existing compatibility procedures rather than ignoring a failing identity test until Task 5. Fresh task review precedes Task 3. Keep Stage 2 In Progress.

### Task 3: Implement supplied routines and exact census evaluation

**Stage:** 2, Native world and routines (second part).

**Files:** Create `controller.rs`, `evaluation.rs`, `metrics.rs`, `budget.rs`, `tests/{controller,evaluation,budget}.rs`; complete native module exports and runner routine dispatch.

**Interfaces:** Consumes Task 2 local View/transaction runner and Task 1 ordered keys/limits. Produces `choose`, `blank_fallback`, `registered_strata`, `evaluate`, complete expected metrics/contrasts/envelope and typed resource counters. `choose` consumes only View; it cannot accept Config or researcher state.

- [ ] **Step 1: Write red tests for supported local choices and the off-grid tie.** Use integer comparison with checked multiplication, avoiding floating thresholds:

```rust
assert_eq!(blank_fallback(3, 6)?, BlankChoice::GuessLeft); // off-grid equality witness
assert_eq!(blank_fallback(2, 6)?, BlankChoice::Inspect);
assert_eq!(blank_fallback(8, 12)?, BlankChoice::GuessLeft);
assert_eq!(registered_strata().len(), 144);
```

Construct Writer views whose own program is ObserveWrite and own request is Left/Right after paid inspection; assert corresponding raw notes. A missing own observation must be rejected as invalid history, not filled from Config. Reader blank behavior depends on own share/prices; nonblank behavior uses only its read. Retain a genuine red test result before code.
- [ ] **Step 2: Implement the four finite routines and runner.** Writer always occupies the same four slots. Reader tracks only its own route progress/observations; ReadAdaptive consumes its supplied initial read, inspects iff `2*i < g` after Blank, picks L otherwise; direct inspection is forced for InspectFirst; Oracle receives its labeled initial target only. The runner captures committed boundaries and terminates at delivery/deadline, never charges unused suffixes. Keep archived exact event count per episode beneath 16.

The private note encoder and exported blank decision have these exact local-only bodies:

```rust
fn writer_note(view: &View) -> Result<Note, Error> {
    match view.known_request {
        Some(Good::Left) => Ok(Note::NoteLeft),
        Some(Good::Right) => Ok(Note::NoteRight),
        None => Err(Error::InvalidHistory("missing own manifest observation".into())),
    }
}
pub fn blank_fallback(i: u64, g: u64) -> Result<BlankChoice, Error> {
    let twice_cost = i.checked_mul(2).ok_or(Error::ArithmeticOverflow)?;
    Ok(if twice_cost < g { BlankChoice::Inspect } else { BlankChoice::GuessLeft })
}
```
- [ ] **Step 3: Write route/ledger regression tests and pass them.** For the defaults s=w=r=1,i=4,b=0 and H=Right:

```rust
let retained = run_episode(&Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Right))?;
let erased = run_episode(&Config::example(Arm::Erased, ReaderRoutine::ReadAdaptive, Good::Right))?;
assert_eq!(retained.outcome.spent, [4, 5]);
assert_eq!(erased.outcome.spent, [4, 9]);
assert_eq!(retained.outcome.net, [-4, 7]);
assert_eq!(erased.outcome.net, [-4, 3]);
assert!(retained.outcome.correct && erased.outcome.correct);
assert!(erased.checkpoints.len() <= 16);
```

Verify the six-action blank-inspection path, b=6/i=8 blank guessing, both H values, wrong BlindGuess, Oracle and all categorized costs against references. Test public history equality before H-dependent paid sensing and exact view-based replay.
- [ ] **Step 4: Implement exact census and comparison reducers.** Iterate the protocol order s,w,r,i,b, then Arm/Routine/Good. Use reduced integer ratios with checked cross-products. Contrast request-wise rows at fixed keys, then average with mass 1/2; no per-request privileged routine selection. Retain all 2,304 means, 4,608 request-wise contrasts, 2,304 mean contrasts and 576 envelope keys/all ties. Exclude Oracle from the envelope. Validate prospective formulas by comparing saved reference arrays, not using the formulas as the implementation's only oracle.

Use this private metrics helper for equally weighted request pairs; signed odd values remain exact:

```rust
fn mean_pair(left: i64, right: i64) -> Result<Ratio, Error> {
    let sum = left.checked_add(right).ok_or(Error::ArithmeticOverflow)?;
    Ok(if sum % 2 == 0 {
        Ratio { numerator: sum / 2, denominator: 1 }
    } else {
        Ratio { numerator: sum, denominator: 2 }
    })
}
```
- [ ] **Step 5: Run conformance without a production diagnostic.** Compare the complete evaluator output to sealed references in an explicitly labeled engineering test; record evaluated engineering keys separately from the two future collections. Verify all grid keys, means, signed outcomes and resource witnesses; inject low caps to require a typed ResourceLimit before append rather than a truncated successful array. Pass focused tests/clippy, check legacy receipts, commit, obtain fresh task review and mark Stage 2 Complete.

### Task 4: Add source-bound reports, saved validation and CLI

**Stage:** 3, Diagnostics and CLI.

**Files:** Create native `canonical.rs`, `sha256.rs`, `source_inputs.rs`, `protocol.rs`, `references.rs`, `diagnostics.rs`, `tests/{canonical,diagnostics}.rs`; create CLI `persistent_notes.rs`; add CLI registration in `main.rs` and binary tests in `crates/sugarscape-cli/tests/persistent_notes.rs`.

**Interfaces:** Consumes Task 3 Evaluation/limits, sealed reference receipts and immutable protocol. Produces Report version `persistent-notes-diagnostic-v1`, Summary version `persistent-notes-summary-v1`, protocol 1, `diagnose`, `validate_report`, `summary_json`, source/reference identities and the `persistent-notes` command family.

- [ ] **Step 1: Write red identity, tamper and I/O tests.** Use reference-derived engineering Report fixtures rather than calling the full collector. Exact tests reject missing/reordered/duplicated rows, altered signed costs, false passes, wrong private beliefs, hidden Oracle injection, denominator changes, mismatched receipts and exceeded limits. Require fresh recomputation on saved values:

```rust
let mut report = report_from_sealed_references()?; // test-only, not a collected report
report.rows[0].outcome.spent[0] += 1;
assert!(validate_report(&report, &Limits::registered()?).is_err());
```

Define `report_from_sealed_references() -> Result<Report, Error>` in the diagnostics test module, label its provenance as engineering. Add independent SHA known-answer tests for empty/abc and canonical UTF-8/control-string fixtures.
- [ ] **Step 2: Implement receipt binding and strict reports.** Embed new native source/protocol/dependency bytes in SOURCE_INPUTS, and the three separately sealed reference files in REFERENCE_INPUTS; receipts never contain their own computed hash. Do not change root Cargo dependencies/lockfiles or import mutable old scientific kernels. Include every expected row, mean, contrast, envelope, counter and failure check in deterministic order. Validator checks all saved events/views/ledgers and recomputes reductions without calling `diagnose` or selecting new policies. Summary freshly validates the retained report and preserves exact numerators/denominators and all negative/tied outcomes.
- [ ] **Step 3: Implement the bounded CLI.** Commands are `persistent-notes run --input <JSON-file> --out <new-file>`, `persistent-notes diagnose --out <new-file>`, `persistent-notes validate --input <report-file>`, and `persistent-notes summary --input <report-file> --out <new-file>`. Scientific settings are registered, with no diagnose tuning flags. File outputs use exclusive creation and precheck before expensive computation. Successful status is 0; invalid scientific payload is 2 with explicit validation failures; parse/resource/I/O/write/flush failures are 1. Preserve errors rather than emit a successful empty report. Unit-test BrokenWriter write/flush errors using the serializer; don't launch `diagnose` to force failure.
- [ ] **Step 4: Verify selected CLI cases and saved report reconstruction.** Test malformed JSON, duplicate keys, existing output rejection, Oracle labeling, default Retained Right ledger, strict summary and invalid saved checks. CLI selected `run` is an engineering episode, not a full collection. Validate the complete reference-derived report and stream-compare summary projections; verify deterministic report formatting/runtime exclusion. Never execute the two full production commands here.
- [ ] **Step 5: Pass gates and commit.** Run focused core/CLI tests, core all-target clippy and release build. Verify frozen old scientific files/reference hashes still match preservation baseline; account for additive shared CLI/registration dependencies in current delivery receipts/stamps now. A changed identity is recorded, not concealed by relabeling an archived measurement. Fresh task review precedes Stage 4; mark Stage 3 Complete.

### Task 5: Add bounded native/WASM episode and browser projections

**Stage:** 4, Browser and Flump playback (first part).

**Files:** Create `browser_experiments/notes/{mod,projection,duplicate}.rs`; add study/family/input/run/validate/wire registrations in the shared browser files listed above; add `web/src/episodes/{notes-projection,notes-comparison}.ts` and tests; add native adapter tests and `web/src/episodes/notes-parity.test.ts`.

**Interfaces:** Consumes native Config/Episode/Checkpoint/View and strict validation. Produces StudyId `persistent_notes`, family `notes`, source-bound EpisodeRecord with ≤16 native checkpoints, local/researcher projections, bounded selected-input controls and exact arm/routine comparisons. Selected request truth is researcher setup, never an Agent observation.

- [ ] **Step 1: Write red privacy and quota tests.** Selected input has exactly `study`, `stratum`, `arm`, `routine`, `request`, `ids`. No own-known-request, costs, flags, clocks or result fields are accepted from callers. Test 64 KiB input/link envelope, 16 MiB record, 4,096 shell checkpoint cap and stricter 16 study cap; reject before native allocation. Compare every unprivileged initial Reader projection under Left/Right and all arms; only its permitted read/inspection can reveal the request.
- [ ] **Step 2: Implement native projections and fresh import validation.** Choose one native episode without an exhaustive compilation. Store owned local snapshots separately; later observations/settlement must not rewrite an earlier view through shared mutable references. Researcher gets truth/field/lineage. Projection receives only shell-selected perspective, not complete input/physical state. Own possession may update on passive receipt while map timestamp stays stale. Wrong delivery/deadline remain real results; operational errors never masquerade as completed episodes. Match imported records to fresh native reconstruction at exact source identity; declare reviewed historical compatibility only when frozen legacy fixtures actually pass.
- [ ] **Step 3: Integrate transport and selected controls.** Preserve one worker request, fresh WASM state validation, cancellation and 60-second wall timeout; build the separate experiments crate, not the root engine bundle. Exact integer wire values use decimal strings as existing episode transport does; no new float tolerance. Input truth editors/import previews are labeled researcher setup and are not displayed as Agent-local information. Keep full result payloads researcher-only, including terminal/comparison checkpoints.
- [ ] **Step 4: Verify actual native/WASM parity.** Use `npm run wasm:experiments` from web and an actual Node-WASM test. Compare complete records for both requests across four arms/four routines (32 bounded engineering cases), plus passive receipt/Oracle/invalid-wire examples. Test matched comparisons preserve stratum, H and IDs while changing only Arm or Routine; require independent timeline availability when branch lengths differ. Run affected existing episode/compatibility tests and typecheck; do not regenerate legacy measured fixtures to hide a mismatch.
- [ ] **Step 5: Commit and review adapter compatibility.** Old native engine hashes/fixtures stay byte unchanged. Additive shared adapter sources may change current delivery metadata; record their actual byte deltas and use existing reviewed compatibility manifests/stamp procedures. Never claim a source identity stayed fixed when a dependency changed. Fresh task review precedes renderer work.

### Task 6: Render real episodes with the approved Flump vocabulary

**Stage:** 4, Browser and Flump playback (second part).

**Files:** Create `web/src/episodes/notes-view.ts`, `notes-comic.ts`, corresponding projection/DOM/browser tests, and versioned public Flump assets/receipt manifest; add new-family dispatch in `catalog.ts`/`view.ts` and scoped styles. Add a bounded native-source-driven preview generator under `studio/` only if the existing asset pipeline needs it; no Python alternative physics or evaluator.

**Interfaces:** Consumes perspective-projected native checkpoints and native costs/observations. Produces the live map/history viewer and a six-panel comic export for the selected native episode. Renderer functions accept projected Checkpoint data and a manifest of decorative assets, never Config or kernel state.

- [ ] **Step 1: Write red projection-driven rendering tests.** Construct initial, post-key-receipt, post-write, departure, post-read/inspection and terminal projected fixtures. In Agent mode, unknown request/note stay unavailable; peer finances/histories and actual field never appear by inspecting the board image, alt text, data attributes, tooltip, export or comparison. An Oracle record prominently labels supplied target knowledge. Distinguish all role/public clocks, not approximate animation time.
- [ ] **Step 2: Implement the map and local ledgers.** Show P/O/C, a carrier-only barrier, distinct goods Left/Right, departure and dated local notes/maps. Use the approved scale/grounded Flumps, one-hand carried good, pencil Write and magnifier Read/Inspect. Own-target inspection card must face its tool rather than imply a board read. Field is a single raw symbol; erasure clears it before the later read. Departed Writer disappears from active map while original-cohort ledger remains Researcher-visible. No invented mood, dialogue, discovery or automatic model repair.
- [ ] **Step 3: Implement selected-episode comic export.** Choose actual native boundaries for acquisition, handoff/write, departure, reader information action, pickup and settlement; if a boundary is absent say unavailable rather than invent an event. Export self-contained SVG/HTML and exact provenance/clock/cost labels. Agent exports contain only projected records/identity, not full-input truth, researcher payloads or hidden-state episode digests; Researcher export is separately labeled. Browser export may supply SVG directly; printable HTML is local. Prefer reused project-generated sprites; any new actual Studio Blender renders get source/settings/asset SHA receipts, no image-generation model. Derive captions from native records, not hand-coded case outcomes.
- [ ] **Step 4: Run actual browser acceptance.** Serve the final artifacts and inspect both H values, retained/erased controls, inexpensive-inspection and expensive-guess branches, wrong BlindGuess and Oracle. Test Agent/Researcher masks at every checkpoint, playback without rerunning, comparison joins, export/import, cancellation, keyboard focus, mobile overflow and reduced motion. For SVG export, assert corresponding captions/costs/clock identities and only authorized request/field symbols; hidden truth must not survive in embedded metadata for an Agent export.
- [ ] **Step 5: Pass final precollection gates and commit.** Run affected browser tests and `npm run typecheck`, then workspace tests, core all-target clippy and release build once for the final scientific tree. Build both WASM bundles and check existing bundle-size budgets; preserve the separate engine/experiments split. Full task/whole-branch review must pass before freeze. Stage 4 Complete requires actual browser evidence, not only screenshots of a mockup.

### Task 7: Freeze, collect exactly twice, independently validate and preserve

**Stage:** 5, Frozen collection and verification.

**Files:** Scientific outputs/source/executable/receipts/logs live in ignored `studio/out/paid-persistent-notes-20261014/` or the date of actual collection recorded before launch. Commit only final `docs/persistent-notes.md`, reproducibility entry, reviewed browser recorded-summary fixture/registration delivery delta and completed tracking changes. Source/configuration cannot be tuned after observing the results.

**Interfaces:** Consumes approved plan, sealed references, final reviewed native tree, feasible registered limits, actual executable and browser acceptance. Produces immutable first/repeat reports, saved-only independent verification, complete summary and original/stored artifact hash index. No publication or remote push is part of this task.

- [ ] **Step 1: Audit readiness before a collector launch.** Record exact commit/tree, executable SHA, native/reference/adapter identities, toolchain/target, 4,608 ordered keys, caps and approved command. Confirm actual report size/work bounds from engineering/reference data fit; no preflight third `diagnose`. Source-informed whole review and independent numeric approval are separate. Changes in rules require a reviewed protocol/reference amendment before launch. If approved collection cannot fit, retain the problem and stop for amendment.
- [ ] **Step 2: Execute the exclusive first and one repeat.** Use a process supervisor enforcing 120s wall/512 MiB RSS and native report-byte guard, retaining exit/stdout/stderr and peak measurements. Commands, under the frozen executable:

```text
<frozen executable> persistent-notes diagnose --out <evidence>/first.json
<same frozen executable> persistent-notes diagnose --out <evidence>/repeat.json
```

The supervisor is tested on tiny fake commands before launch; it never overwrites files or silently retries. Both have the same immutable source/config/reference bytes. A failed invocation is retained and is not replaced by an unapproved third launch. The runtime ledger records production versus engineering/reference evaluations separately.
- [ ] **Step 3: Independently validate saved data without new collection.** Check raw byte identity, all 4,608 keys/typed traces, costs, private views, 2,304 means, all contrasts/envelope ties, reference arrays and resource counters. A source-independent saved checker operates on the retained bytes and independently interpreted registered transitions; it never calls the native collector or searches/optimizes new policies. Run native saved validation and summary against first.json, then separately compare summary fields to the full first report. Preserve failures and all unavailable/negative outcomes.
- [ ] **Step 4: Write measured findings and delivery summary.** Clearly separate prospective formulas from observed verification; disclose supplied roles/programs/model and no equilibrium/volunteer/novelty/incident-fidelity finding. Report fixed-routine contrasts versus offline selected-routine envelope, shared-read cost versus whole production cost, original departed Writer versus living Reader, correctness versus cost savings, all rows and ties. Install a complete source-bound recorded summary as a separately identified postmeasurement delivery artifact; native scientific source/settings/first/repeat stay frozen. Recheck only affected summary/adapter tests after this data-only change, plus actual served recorded-result view.
- [ ] **Step 5: Verify preservation and finish locally.** Copy exact source/dependencies/executable/protocol/references/reviews/first/repeat/summary/stage tracker and ledger into ignored evidence, hash original/stored sizes/bytes, validate any compressed files by decompressed hash, exclude generated Cargo/browser caches. Preserve Git commit/bundle prerequisites if helper-only source is ever retired; never force-remove a dirty worktree. Recheck main and other worktrees, stage only owned delivery paths, respect concurrent changes and do not merge/push automatically. Mark all five stages Complete, archive then remove this plan's temporary tracker. Report measured findings, limitations, local commits and any outstanding integration work.

## Self-review and execution handoff

Coverage: source/scientific scope and preservation→Task 1/7; transactions/observations→Task 2; routines/algebra→Task 3; exact reports/caps/errors→Task 4; native/WASM/privacy/compatibility→Task 5; actual Flump viewer/exports→Task 6; frozen collection/saved verification/full costs→Task 7. Each Review Focus line is assigned above. Names and counts are shared by the contract; Task 1 freezes their strict serialized schemas rather than letting implementers choose them independently.

The approved design authorizes preparing this plan. The user reviews this concrete plan before implementation. After plan approval, preserve subagent-driven execution, enforce every reference/precollection gate, and conduct only the two registered local collections within the reviewed caps. Approval does not authorize later parameter variants, extra campaigns, push or publication. No execution-method menu is needed.
