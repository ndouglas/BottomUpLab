# Negotiable Assistance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user has already selected subagent-driven execution. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and independently verify the exact cake and priority-update foundation for reproduction queue item 41, without running a scientific survey or adding a world/UI.

**Architecture:** A focused core module uses checked rational arithmetic for cake policies, stochastic dominance and supplied-history priority updates. Separately authored Python references precede production code. Three new survey claims expose deterministic, bounded, source-bound records through the existing claim interface.

**Tech Stack:** Existing Rust/serde/serde_json core and survey, existing survey SHA-256, Python standard-library `fractions`, `json`, `hashlib` and `unittest`; no new dependencies.

**Spec:** [Approved design](../specs/2026-10-14-negotiable-assistance-design.md), commit `89823ae`; written approval: user's “LGTM!” on 2026-10-14.

**Status:** User-approved plan; engineering implementation, independently approved v2 references and required unit/integration/source/build checks are complete. Fresh Task 4 and whole-branch reviews remain pending. Scientific first/repeat collection and publication remain separately authorized work and have not run.

**Recorded integration rulings:** F5 selection is shared through `claims/mod.rs` with an equivalent import in its existing consumer; its validators are unchanged. The String-returning `build_record` convenience is test-only, while shipped callbacks preserve typed mathematical/operational failures. The current compatibility bridge updates only the two `lib.rs` byte/SHA receipts for the exact 21-byte module export; original JSON, historical identity fields and every other receipt are preserved. These narrow scope/interface adjustments are retained in the SDD ledger for fresh review.

## Global Constraints

- All registered scientific targets use reduced rational arithmetic and exact equality; decimal approximations are display-only.
- No new world kind, generic POMDP solver, dependency, UI, WASM bundle or shared rational-library refactor.
- Keep existing assistance games, defaults, outputs and claim identities unchanged.
- Retain all nine deterministic policy IDs, all maximizing ties and both `paper_weak` and `strong` labels; include stochastic mixtures through the whole exact payoff hull.
- Fixed cake: utilities `(30,0)`, `(20,20)`, `(0,30)`; Alice/Bob red likelihoods `9/10`, `1/10`; 25 weights and 225 objective cells.
- Fixed update domain: red likelihood pairs `(9/10,1/10)`, `(1/2,1/2)`, `(1,4/5)`; prior Alice weights `{0,1/2,1}`; all histories of lengths 0 through 8; 4599 cells.
- Zero total observation mass is explicitly impossible; zero prior weights cannot revive; no epsilon, NaN, random draws or favorable row deletion.
- Source-informed transcription and separate numerical reference approvals precede production implementation.
- Reject malformed probabilities, invalid weights, zero denominators, arithmetic overflow and overlong histories with context. Do not reuse the assistance module's unchecked private `Ratio`.
- Scientific reports, gridworlds, strategic demonstrations, novelty claims, publication and war-calibration reuse are outside execution scope.
- One product writer and one build owner at a time. Use the existing design worktree and preserve crowd/Gridworlds work, user files and frozen evidence.
- Do not overwrite the key-delegation root tracker. On execution, create the owned `.superpowers/negotiable-assistance/IMPLEMENTATION_PLAN.md` with the four stages below; update it and preserve its completed snapshot before removing it.
- Retain commands, logs, status, source identities, mutations and reviews under `.superpowers/negotiable-assistance/`. Never stage ignored evidence, PDFs or survey outputs. No bypassed hooks or weakened source guards.

## Review Focus

1. Zero prior and impossible history: impossibility stays explicit through suffixes; a reset would change the model. Task 2 pins this.
2. Mixed-policy improvement: deterministic-only comparisons can miss dominance; strict and non-strict coordinate conditions differ. Task 3 pins this.
3. Rational boundaries and duplicates: all tied action/policy IDs survive; identical values do not merge scientific IDs. Task 3 pins this.
4. Tampered source/reference data: compilation alone cannot certify current input bytes; stale runtime binding must become `Verdict::Error`. Task 4 pins this.
5. Oversized or partial output: predeclared record/byte bounds and serialization errors must fail explicitly, without presenting partial cells as a successful report. Task 4 pins this.

## Files and stage map

| Stage | Deliverable | Product ownership |
|---|---|---|
| 1: Independent registration | Sealed settings, rational references, source and numerical approvals | New `survey/negotiable_reference/` scripts; new core reference JSONs |
| 2: Exact priorities | Checked arithmetic and exhaustive supplied-history updates | New `core/src/negotiable/{mod,ratio,priority}.rs` and tests; one `lib.rs` export |
| 3: Cake and mixtures | Exact policy/weight evaluation and both hull classifications | New `core/src/negotiable/{cake,pareto}.rs` and tests |
| 4: Survey and verification | Three bounded source-bound claims, integration and independent review | New survey claim/helper/tests; narrow claim registration and F5 inventory additions |

`core/` in this table means `crates/sugarscape-core/`; all task paths below are explicit. Each stage's success criteria are its listed reference comparisons, error tests and independent review. The isolated branch currently starts at `32d920e`, with design commit `89823ae`; check the actual branch/head before execution and never replay a completed ledger task.

## Record contract shared by all tasks

Canonical fraction wire format is `{"num":"-1","den":"2"}`: reduced signed decimal numerator, positive decimal denominator, no leading zeros except `0`, no plus signs. Internal values use `i128`; JSON represents them as strings. Version is `negotiable-foundation-v1`.

Reference JSONs have exactly `schema`, `settings` and `rows` keys. Settings name source hashes, action/policy ordering, all weights, all likelihood pairs, priors, history horizon and Pareto definitions. No timings or generated timestamps enter mathematical reference records. Python writes canonical UTF-8 JSON with sorted object keys, compact separators and one final newline, using exclusive creation (`open(path,'xb')`). Hash files separately in a seal manifest stored in ignored evidence.

- `cake-reference.json`: nine `policies` embedded in settings, each ID `3*red_action+green_action`; 225 rows sorted by weight then policy ID, each with `weight`, `policy_id`, `expected:[A,B]`, `objective`, and complete `global_maximizers`/`local_maximizers` arrays.
- `pareto-reference.json`: nine rows sorted by policy ID, each with `policy_id`, `payoff`, `paper_weak` and `strong`. Each classification is `{"efficient":bool,"witness":null|Witness}`. `Witness` is `{"parts":[{"policy_id":id,"weight":Fraction},...],"payoff":[A,B]}` with at most three parts, nonnegative weights summing to one, ordered policy IDs. Different valid witness decompositions are allowed; acceptance compares classifications and independently verifies each production witness rather than requiring identical arbitrary witnesses.
- `priority-reference.json`: 4599 rows sorted by model ID, prior index, history length, then binary history. Each contains `model_id`, `prior_index`, `history` as an array of `0` red / `1` green, and `result`. Supported result is `{"status":"supported","weights":[A,B]}`; impossible result is `{"status":"impossible","first_impossible_prefix":k}` with one-based prefix length. Length-zero histories are supported.

Every fixture is at most 8 MiB; the three total at most 16 MiB. Validate strict keys, canonical fractions, ordering, counts, uniqueness and all settings before checking rows. Store generator and fixtures in different paths from production implementation. A changed seal or settings requires renewed reference review before comparison; never replace a reference merely to match production.

## Task 1: Independently derive and approve the references

**Files:** Create `survey/negotiable_reference/{__init__,reference,test_reference}.py`; create `crates/sugarscape-core/src/negotiable/tests/fixtures/{cake-reference,pareto-reference,priority-reference}.json`. Do not add Rust module files yet.

**Interfaces:** `reference.build_cake()`, `build_pareto()` and `build_priority()` return the three records above; `write_exclusive(record, path)` enforces the size cap and canonical bytes. `validate_reference(record, kind)` raises contextual `ValueError` for malformed or incomplete data. `decode_reference(data:bytes, kind:str)->dict` rejects inputs above 8 MiB before parsing, uses `json.loads` with a duplicate-rejecting `object_pairs_hook`, then invokes validation. No production imports, binary invocation or expected-production-output input is allowed.

- [ ] Preserve and verify the two PDF identities from the design in ignored evidence; preserve main and supplemental extracted text. Do not execute downloaded content. Read §3/equations 3–8 and supplement §1 before deriving any data.
- [ ] Write failing Python behavior tests:

```python
def test_source_cake_policy_uses_each_principals_prior(self):
    record = reference.build_cake()
    rows = [r for r in record['rows'] if r['policy_id'] == 2]
    self.assertTrue(all(r['expected'] == [wire(Fraction(27)), wire(Fraction(27))]
                        for r in rows))

def test_reference_rejects_missing_history(self):
    record = reference.build_priority()
    record['rows'].pop()
    with self.assertRaisesRegex(ValueError, '4599'):
        reference.validate_reference(record, 'priority')
```

Here `wire(q)` is the test-local helper `return {'num':str(q.numerator),'den':str(q.denominator)}` and `Fraction` is imported from `fractions`; test code must import `copy`, `unittest` and the reference module as used. Run `python3 -m unittest survey.negotiable_reference.test_reference -v`; retain the initial missing-module/function failure.
- [ ] Derive the cake from utilities and likelihoods, not the design's four boundary formulas. Compute `expected_A = p_A*U_A(red)+(1-p_A)*U_A(green)` and independently for B; compute each weighted objective. Derive local scores from unnormalized `w*p_A` and `(1-w)*p_B` masses and retain every argmax; Cartesian products give the local policy set. Derive the four boundaries by solving pairwise linear local-score equalities, then compare with the registered weights.
- [ ] For the reference dominance calculation, construct the rational monotone-chain convex hull of payoff pairs, preserving original policy IDs separately. Clip the hull to the candidate's northeast closed quadrant using exact line intersections. Strong domination exists if a retained feasible point improves either coordinate; weak domination exists if a feasible point improves both, or if the midpoint of two feasible points does. Track convex coefficients during clipping to produce independently checkable witnesses; if more than three positive coefficients remain, reduce them by rational affine dependence in two dimensions while preserving the payoff and nonnegative sum-one weights. Handle point/segment hulls without a polygon assumption. Test the synthetic points `(1,1),(3,0),(0,3)`: the first is improved only by a mixture. Test `(0,0),(0,1)`: the first is weakly efficient but strongly dominated. These are labeled mathematical unit fixtures, not modified cake scenarios.
- [ ] Generate each priority prefix directly from initial weights and products of each model's observation likelihoods using `Fraction`, identifying the first zero-mass prefix. Check a separately implemented sequential update against the direct product. Test all supported and impossible prefixes, including initial zero weights.
- [ ] Add generator rejection tests for duplicate keys/rows, stale settings, noncanonical fractions, missing rows, out-of-order histories, invalid witness weights and output oversize; add source-example and four-boundary tests. Run the Python suite, generate the three fixtures exclusively, re-open and validate every byte, and seal counts/hashes/settings/generator identity.
- [ ] Dispatch a fresh source-informed reviewer for transcription/scope and a separate numerical reviewer for the Python algorithms, full data census, hull witnesses and arithmetic. These are different seats from the reference author and later production implementers. Fix findings using the normal scoped review loop; record both approvals before Task 2. Commit only scripts and JSONs as `test(negotiable): seal independent rational references`.

## Task 2: Checked rational arithmetic and priority updates

**Files:** Create `crates/sugarscape-core/src/negotiable/{mod,ratio,priority}.rs` and `tests/{mod,ratio,priority}.rs`; modify `crates/sugarscape-core/src/lib.rs` only to add `pub mod negotiable;`.

**Interfaces:** Re-export `Ratio`, `Error`, `Observation`, `HistoryResult`, `history_weights`, `update_weights` and `WireRatio` from `negotiable`.

```rust
// Fields are private; constructors and arithmetic return Result.
pub struct Ratio { num: i128, den: i128 }
pub enum Error { InvalidDenominator, InvalidFraction(String), Overflow, InvalidProbability,
    InvalidWeights, HistoryTooLong, ImpossibleObservation,
    InvalidPolicy, InvalidFixture(String), RecordTooLarge }
pub struct WireRatio { pub num: String, pub den: String }
pub enum Observation { Red, Green }
pub enum HistoryResult { Supported([Ratio; 2]),
    Impossible { first_impossible_prefix: usize } }
pub fn update_weights(prior: [Ratio; 2], likelihood: [Ratio; 2])
    -> Result<[Ratio; 2], Error>;
pub fn history_weights(prior: [Ratio; 2], red: [Ratio; 2],
    history: &[Observation]) -> Result<HistoryResult, Error>;
```

`Ratio` derives `Clone,Copy,Debug,PartialEq,Eq`; `Error` and `HistoryResult` derive `Debug,PartialEq,Eq`, and `Error` implements `Display` and `std::error::Error`. `WireRatio` derives serde serialization/deserialization with unknown fields denied. `Ratio::new(num:i128,den:i128)->Result<Self,Error>` rejects nonpositive denominators and canonicalizes by GCD. Provide `num()`, `den()`, `wire()`, `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_cmp`, `from_wire`, all fallible except getters/wire. Comparison returns `Result<Ordering,Error>`; do not implement an infallible ordering using overflowing products.

- [ ] Write failing tests before module implementation:

```rust
#[test]
fn zero_mass_is_not_reset_to_an_equal_prior() {
    let r = |n,d| Ratio::new(n,d).unwrap();
    assert_eq!(update_weights([r(1,1),r(0,1)], [r(0,1),r(1,5)]),
               Err(Error::ImpossibleObservation));
}
#[test]
fn rejected_fraction_does_not_panic() {
    assert_eq!(Ratio::new(1,0), Err(Error::InvalidDenominator));
}
```

Run `cargo test -p sugarscape-core negotiable`; retain the red failure.
- [ ] Implement reduced checked arithmetic: use unsigned absolute magnitudes/GCD without negating `i128::MIN`; reject unrepresentable reduced results with `Overflow`. Cross-cancel numerator/denominator factors before multiplication, use denominator GCD for addition/subtraction and checked products for comparison. Parse canonical decimal strings with at most 40 characters and reject invalid or non-reduced wire fractions rather than silently repairing them. Tests cover zero division, invalid denominator, `i128::MIN`, sign cancellation, reduction, comparison overflow and maximum integer operation failures.
- [ ] Implement update validation before multiplication: priors nonnegative and sum exactly one; likelihoods in `[0,1]`. Normalize the two weighted masses; zero sum returns `ImpossibleObservation`. `history_weights` rejects length above 8 before iteration, validates initial inputs even for empty history, uses complement red likelihood for green, and retains the first impossible prefix rather than evaluating later suffixes as a fresh prior.
- [ ] Deserialize and validate all 4599 priority fixture cells with test-local strict DTOs; compare `history_weights` by keys/status/weights, never array position alone. Add explicit tests for red→`9/10`, green→`1/10`, common-belief preservation, eliminated models, zero priors, empty histories, negative/above-one probabilities and nine-step rejection.
- [ ] Document a numeric bound for the registered domain in `ratio.rs`: observation products have denominators at most `10^8`, initial weights at most 2, and normalized masses at most the sum of two such integers. Derive bounds for actual sequential operations, including their checked comparison intermediates; confirm the bound using the independent reference. Do not claim this is a proof for arbitrary public inputs; those remain fallible.
- [ ] Temporarily mutate impossible-observation handling to equal weights; confirm the named error/prefix tests fail, restore and re-run. Run `cargo test -p sugarscape-core negotiable`, edited-file rustfmt checks, and `cargo clippy -p sugarscape-core --all-targets -- -D warnings`. Record a clean independent task review and commit `feat(negotiable): add checked exact priority updates`.

## Task 3: Cake policies, complete ties and stochastic dominance

**Files:** Create `crates/sugarscape-core/src/negotiable/{cake,pareto}.rs` and `tests/{cake,pareto,reference}.rs`; extend the new `mod.rs` and `tests/mod.rs` only.

**Interfaces:** Re-export the following; all record DTOs serialize exact `WireRatio` values, not floats. `Policy` is an opaque ID with `Policy::from_id(u8)->Result<Policy,Error>`, `id()->u8`, `actions()->[u8;2]`; IDs 0–8 use the fixed action order. `CakeRow` has `weight`, `policy_id`, `expected`, `objective`, `global_maximizers` and `local_maximizers` from the record contract. `Point=[Ratio;2]`, `Witness` holds up to two production policy IDs/weights plus their payoff, and `Classification` contains `efficient:bool,witness:Option<Witness>`.

```rust
pub fn cake_payoff(policy: Policy) -> Result<[Ratio;2],Error>;
pub fn registered_weights() -> Result<Vec<Ratio>,Error>;
pub fn cake_rows() -> Result<Vec<CakeRow>,Error>;
pub enum ParetoKind { PaperWeak, Strong }
pub fn classify(points: &[Point], candidate: usize, kind: ParetoKind)
    -> Result<Classification,Error>;
```

`classify` accepts 1–9 points and a valid candidate index; it preserves original indices as witness IDs. Checked arithmetic errors propagate. Its public scope is this bounded two-coordinate set; no generic optimization engine.

- [ ] Write RED tests for policy ID 2→`(27,27)`, ID 4→`(20,20)`, nine distinct IDs, invalid ID 9, all four boundary ties and exact whole/local maximizer equality. Add this mixture test before implementation:

```rust
#[test]
fn a_mixture_can_dominate_without_a_dominating_vertex() {
    let r = |n| Ratio::new(n,1).unwrap();
    let points = [[r(1),r(1)],[r(3),r(0)],[r(0),r(3)]];
    let result = classify(&points,0,ParetoKind::PaperWeak).unwrap();
    assert!(!result.efficient);
    // The separate witness-verification test recomputes its payoff/weights.
}
```

Run `cargo test -p sugarscape-core negotiable` and retain red evidence.
- [ ] Compute both subjective payoffs independently from the policy's actions. Sort/deduplicate the union of 21 grid weights and four exact boundaries with fallible exact comparison. For each weight compute all nine objectives and return every maximum. Separately score red/green actions from unnormalized likelihood masses; take Cartesian products of complete maximizing action sets and compare with global maximizers. Do not use the design's boundary thresholds as a production lookup table.
- [ ] Implement production stochastic dominance by bounded segment enumeration, independent of the reference hull-clipping method. For every unordered pair of original payoff points, form `v(t)=p_i+t(p_j-p_i)` for `t∈[0,1]`. Breakpoints are 0, 1 and any coordinate crossing of the candidate within `[0,1]`. Evaluate every breakpoint and midpoint of each consecutive interval. A point dominates under `PaperWeak` iff both coordinates are strictly greater; under `Strong` iff both are at least equal and one strictly greater. Return the first valid witness in stable `(i,j,t)` order; otherwise mark efficient. Include degenerate/singleton segments. A northeast feasible region, if nonempty, reaches the upper hull boundary represented by a pair of original vertices; document why this procedure covers interior mixtures, not merely the sampled candidate segments.
- [ ] Check all 225 cake rows and nine Pareto labels against the sealed fixtures. Verify every witness's allowed indices, nonnegative rational weights, total weight one, reconstructed payoff and actual dominance; do not compare arbitrary witness bytes between methods. Test a singleton, collinear points, duplicate points with distinct IDs, invalid candidate, empty/ten-point input, boundary-only improvement and the synthetic `(0,0),(0,1)` weak/strong distinction. Do not require the cake classifications to differ merely because definitions differ.
- [ ] Mutate `>=` versus `>` in the strong/weak rule and retain-first argmax instead of retaining ties; confirm the named tests fail, restore and rerun. Run negotiable tests, `cargo test -p sugarscape-core assistance`, edited-file rustfmt and core Clippy. Fresh independent task review checks source equations, full-reference comparisons and dominance completeness. Commit `feat(negotiable): evaluate cake policies and stochastic efficiency`.

## Task 4: Bounded source-bound survey integration and final review

**Files:** Create `survey/src/claims/{negotiable,negotiable_tests,negotiable_records}.rs`; modify `survey/src/claims/mod.rs` to register the group and `survey/build_support/f5_source_identity.rs` to bind its three embedded fixture files. No change to the generic survey main/output handling or old claim implementation is planned.

**Interfaces:** `negotiable::claims()->Vec<Claim>` returns exactly `negotiable.cake`, `negotiable.pareto`, `negotiable.priority`, item `negotiable`. `negotiable_records::build_record(kind:RecordKind)->Result<Vec<u8>,String>` builds canonical compact records capped at 8 MiB each; kind variants are `Cake`, `Pareto`, `Priority`. Each record has `schema`, `scope`, `binding`, `settings`, `rows`; `scope` is `finite_exact_foundation`, never a theorem proof. `binding` contains the two PDF hashes, all three reference hashes, settings SHA-256 and current compiled-source SHA-256.

Runtime binding uses the existing F5 selector and generated `f5_source_identity.rs`: include its module from `survey/build_support/f5_source_identity.rs`, read the same `DIRECTORIES`/`FILES` as `survey/build.rs`, sort/deduplicate and recompute its existing fingerprint. Compare against `COMPILED_SOURCE_IDENTITY` before and after constructing any record; error on mismatch. Add the three fixture paths to `FILES`, because F5's automatic selector covers Rust files, not arbitrary JSON. W1/W1-A automatically walk the core subtree; do not add exclusions or weaken their validators.

- [ ] Write RED survey unit tests for exactly three unique registered IDs, all record counts, scope, exact fraction strings, complete ties, fixed settings and a changed-source binding error using a temporary engineering source tree. Verify error injection returns `Verdict::Error` with context instead of a successful partial report. Run `cargo test --manifest-path survey/Cargo.toml --bin survey negotiable` before implementation.
- [ ] Implement the three claim checks by evaluating the production functions, checking every row against the sealed references and independently checking generated witnesses. Paper-derived cake/priority identities use `Source::Book`; Pareto-convention comparison and finite census checks use `Source::App` and explicitly say they are engineering checks. Mathematical disagreement returns `Fails`; malformed references, stale bindings, arithmetic or encoding failure returns `Error`. Never print a general theorem verdict from finite instances.
- [ ] Store each deterministic record as the claim's `Outcome.measured` JSON string; use `Outcome.detail` for the finite-scope explanation and the exact design/source citation. Check total record bytes before returning, with a bounded writer that rejects growth before exceeding 8 MiB. Output only the registered rows, and check their count before serialization. For the fixed records, certify worst-case bytes from maximum canonical `i128` strings, keys, arrays and witness size, with at least the exact declared 4599-row priority census; if the proof exceeds 8 MiB, stop for a reviewed schema adjustment. Do not enlarge the cap after generating results.

```rust
#[test]
fn bounded_record_writer_does_not_publish_a_partial_success() {
    let mut writer = BoundedRecordWriter::new(3);
    assert!(std::io::Write::write_all(&mut writer, b"four").is_err());
    assert!(writer.finish().is_err());
}
```

`BoundedRecordWriter` is internal to `negotiable_records.rs`, with `new(limit:usize)`, `Write` and `finish()->Result<Vec<u8>,String>`; a sticky failure forbids finishing after any error. The production cap is fixed, while tests inject a smaller cap. Test Unicode error context, invalid fixture keys, stale reference hashes, partial serialization, fixture truncation and all missing/duplicate rows. Test helpers accept supplied fixture bytes/root paths internally; do not expose a new runtime input parser or command.
- [ ] Preserve baseline hashes for existing assistance sources/configs/tracked findings/output before edits, then verify those files remain byte-identical. Compare old catalog entries' IDs, item, source, citation and text before/after registration using a test that filters only `negotiable.*`; new registration must be additive. Never rerun the stochastic assistance campaign merely to obtain a baseline.
- [ ] Run the following checks serially under one build owner, preserving actual logs/status and any legacy report-generating test scope. New negotiable claim calls in unit tests are engineering checks, not scientific report collection:

```bash
python3 -m unittest survey.negotiable_reference.test_reference -v
cargo test -p sugarscape-core negotiable
cargo test -p sugarscape-core assistance
cargo test --manifest-path survey/Cargo.toml --bin survey negotiable
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --manifest-path survey/Cargo.toml --bin survey -- -D warnings
cargo test --workspace
cargo test --manifest-path survey/Cargo.toml --bin survey claims::burrow_tests
cargo test --manifest-path survey/Cargo.toml --features war-studies --bin war1-study source_tests
cargo test --manifest-path survey/Cargo.toml --features war-studies --test war1_study_cli metadata_reports_actual_binary_and_build_inputs_without_collection
cargo test --manifest-path survey/Cargo.toml --features war-studies --test war1_study_cli merged_core_literal_audit_matches_native_inventory
cargo test --manifest-path survey/Cargo.toml --features war-benchmarks --bin war1 source_tests
cargo build --manifest-path survey/Cargo.toml --release --bin survey
```

Check edited Rust files using `rustfmt --check --edition 2021 --config skip_children=true` and explicit actual file paths. Do not format unrelated modules. Verify the F5 runtime fingerprint equals the compiled stamp and all reference paths are present; W1/W1-A source tests and actual metadata must still agree with current bytes. Expected inventory hashes may change; no historical artifact is relabeled or rewritten. No Engine fixture collection or `survey --only negotiable` command is permitted by this plan.
- [ ] Fresh task review checks the diff and retained evidence, then fresh whole-branch review examines all production/reference code, mathematical scope, dependency/source inventories and recorded rulings. Follow the user rule: reassess after three failed attempts at an issue. Commit reviewed code as `feat(survey): register exact negotiable-assistance checks`. Preserve completed stage/SDD snapshots and evidence before removing owned temporary tracking.

## Report collection reserved for later authorization

Exclusive retained destinations, if separately authorized, are `.superpowers/negotiable-assistance/collection/first/` and `collection/repeat/`, with stdout, stderr, exit status, full original survey JSON, canonical mathematical records and a source/binary/toolchain/settings manifest in each. Reserve 32 MiB per attempt, 120 seconds elapsed and 512 MiB child peak RSS. An exceeded bound or failed attempt is retained and stops execution; no favorable retry. Independently certify packing and the operational collector before invoking it.

The existing survey CLI writes `survey/out/results-negotiable.json` and records elapsed `seconds` per claim, so it is neither create-only nor byte-deterministic as a whole. Do not claim two full outputs are byte-identical or silently overwrite an earlier result. A later collection proposal must specify exclusive staging/archival around that legacy path, preserve both raw timed outputs and compare canonical mathematical records separately. This plan deliberately does not activate that collector, modify generic CLI semantics, publish findings or push/merge.

## Self-review and handoff

Source/independent registration is Task 1; rational/support/error requirements are Task 2; exact cake/ties/stochastic Pareto are Task 3; record bounds, binding, additive registration and all affected verification are Task 4. Review Focus items each have an explicit owning task/test. Public types and function names above are the task boundaries; Rust implementations, independent Python algorithms and record formats may not silently diverge.

User review of this plan precedes implementation. Execution method remains subagent-driven; do not ask the user to select it again. Preserve the accepted design, source versions and original references. A source/math ambiguity discovered during implementation is a retained finding to resolve prospectively, not permission to adjust the model after viewing a result.
