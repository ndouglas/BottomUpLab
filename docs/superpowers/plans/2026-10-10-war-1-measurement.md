# W1-A Measurement Preparation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user already selected subagent-driven execution; do not ask again.

**Goal:** Deliver a tested, source-bound compact W1-A collector, independent numerical/statistical
analysis and a concrete prospective readiness packet, without collecting or publishing the
registered population.

**Architecture:** Python standard-library modules own the immutable contract, seed-manifest
preparation, independent rational/decimal references, saved-data analysis and handoff. A separately
gated native `war1-study` binary owns validated kernel execution, compact projection, durable
evidence and preconstruction activation checks. The existing engine and engineering `war1` CLI
retain their semantics; final declaration and approval receipts are outside their own compiled
source inventory.

**Tech Stack:** Existing Rust/serde/serde_json/sha2/core `war-benchmarks`; Python standard library
(`json`, `fractions`, `decimal`, `hashlib`, `os`, `resource`, `subprocess`, `unittest`); existing
Cargo/npm/yq tools. No new runtime or numerical dependency.

**Spec:** [Approved W1-A design](../specs/2026-10-10-war-1-measurement-design.md) and [exact
arm/endpoint/contrast roster](../specs/2026-10-10-war-1-measurement-proposal.json), approved by the
user with LGTM. Preserve their bytes. The roster is a design artifact, not an activation manifest.

## Global Constraints

- "Total: 66 arms, 52,224 runs." 54 comparative/mirror arms have n=512; 12 tiny-law arms have n=2048. M=924 scalar slots, C=198 CDF slots, 121 scalar-contrast groups, 46 CDF comparisons; do not select a smaller reporting family.
- "Both geometries use H=8 and observation times 1, 2, 4, 8." `dt={1/16,1/32,1/64}` and `max_steps={128,256,512}`. All counts/rates/configs come from the approved JSON, not reconstructed prose.
- "Retain duplicate values if they occur; keys are (arm ID, replicate index), so sampling with replacement and its denominator are preserved. Reject duplicate keys, not repeated values."
- Scientific seeds are independent OS-uniform u64 draws with replacement, frozen before outcomes. No counter/hash seed replacement or paired cross-arm analysis. Implementation tests use construction seeds 7/8 and synthetic inputs only.
- "Kernel FNV stream seeding, RNG algorithm, shuffle and casualty ordering stay unchanged." No World, Config, Snapshot, existing golden, portable math, browser product, old study or frozen engineering-contract edits.
- "General finite-state CTMC solution and claims of general quantitative dt bias are deferred." Independent exact-q tiny DTMC and one-versus-one continuous-time formulas are the numerical scope.
- "Any missing/invalid registered outcome makes primary ensemble inference unavailable; never shrink n or replace seeds." Reference failures have endpoint-specific availability and preserve finite outcomes.
- "No scientific engine construction or output creation precedes this preflight." A typed receipt cannot create permission from an approval string; no `--approved`, retry, restore/resume or override command.
- "The existing engineering `war1` CLI remains unchanged and unregistered." New study feature/bin/source selector are separate; existing F5/BT selectors and existing CI job semantics remain unchanged.
- "Compact non-audit rows and transcript hashes are explicitly not a lossless raw archive." Replicate index 0 of each arm is the fixed full-audit subset.
- Caps: compact outcome 32 KiB; durable start 1 KiB; audit frame 32 KiB; metadata/manifest file 16 MiB; campaign 4 GiB uncompressed including provenance/failure reserve. One worker, 2 GiB RSS, 12-hour collection wall cap. Bound bytes before allocation/write; compression is not budget credit.
- Worst-case scientific census: 17,563,648 settlements; 2,054,160,384 initial actor-steps; maximum case population 768 and work 393,216. Resource forecast and exact encoded size proof are preparation gates, not measured forecasts already established.
- α_scalar=α_CDF=0.025. Use exact rational empirical means and outward-enclosed radii/bounds. JSON radius decimals are display-only. Tiny oracle normalized interval width ≤1e-10; stored p, mathematical p* and realized q remain distinct.
- Preserve the original engineering design/plan/amendment, approved measurement design/roster and retained science assets. Keep evidence/worktrees until unique artifacts are archived. Main's unrelated work and untracked outputs are not owned by this plan.
- User rule: maximum three failed attempts per issue, then document, research alternatives and reassess. Use normal commits/hooks. Task-specific checks per task; broad checks once at final code, repeating only affected checks after changes.

## Review Focus

1. A plausible scientific manifest without genuine matching readiness/execution authorization must
fail before Engine construction or output creation — Task 3 gate and Task 5 real-binary test.
2. Repeated random seed values are valid, while duplicate/missing/aliased keys and wrong manifests
are not — Tasks 1 and 4 exact-census tests.
3. A write/flush/sync or resource failure after physics commits must leave truthful
semantic/observed/durable clocks and a complete prefix, never artifact success or successful-only
inference — Tasks 3 and 4 injected failures.
4. Early extinction and actual final-step extinction must remain distinguishable from the
administrative atom at H; absorbing observations are derived, not fake frames — Tasks 3 and 4 tests.
5. A numerically plausible oracle using p* instead of q, uncertified rounding, or unresolved
exponential/CFD values must not activate collection or enlarge thresholds — Tasks 2 and 5
certificate/gate tests.

## File map and execution stages

| Stage | Deliverable | Files / ownership |
|---|---|---|
| 1 | Immutable contract, manifest and construction fixture | `survey/war1_study/{contracts,manifest,io}.py`, package/tests, checked-in construction manifest |
| 2 | Independent exact numerical preparation and native no-simulation export | Python `numerics.py`, `oracle.py`; gated Rust entrypoint/local `wire.rs`, `numerics.rs`, `cli.rs`; Cargo feature/bin declarations |
| 3 | Source/permission-bound compact native collector | Rust `source.rs`, `gate.rs`, `observer.rs`, `archive.rs`, `io.rs`; gated study build helper/build.rs; Rust/unit/real-binary tests |
| 4 | Saved-evidence validation and complete inference | Python `archive.py`, `analysis.py`, tests; fixed scalar/CDF/contrast matrices |
| 5 | Noncircular preparation, resource proof, CI and reviewed handoff | Python `registration.py`, `preparation.py`, `supervisor.py`, `__main__.py`; native validate/metadata commands, integrated tests, CI/docs |

Create a five-stage `IMPLEMENTATION_PLAN.md` in the execution worktree; preserve its completed
snapshot then remove only this owned tracker. Keep this approved plan immutable; task
statuses/review evidence belong to the plan's ignored execution ledger. Every task has one fresh
implementer and a fresh spec/quality reviewer; one whole-branch review follows.

### Shared wire contracts (normative for every task)

`Json` means a strict JSON object with finite numbers and exact Python integers; schema validation
still uses the exact key/type/bound rules below. Runtime serde types deny unknown fields and
recursive duplicate keys. UTF-8 is mandatory; reject BOM, trailing input, nonfinite literals and
nonzero literals that round to binary64 zero. `bool` is not an integer seed.

Manifest v1 has exactly `schema`, `purpose`, `protocol_sha256`, `proposal_sha256`, `roster_sha256`,
`sampler`, `arms`, `cases`. Purposes are `construction_fixture` and `prospective_seed_population`.
Every case is `{ordinal, arm_id, replicate_index, seed}`; canonical ordinals follow arm order then
index. Arms are the approved JSON arm dictionaries for science; construction fixture arms below are
explicitly separate. `sampler` is `{method, bytes_drawn, seed_bytes_sha256, command, version}`.
Science method is `os.urandom-u64le-v1`; seed-byte hash is SHA256 of concatenated sampled eight-byte
little-endian values in ordinal order. No self-hash, source-inventory digest or binary hash is
inside the manifest. Hash raw committed manifest bytes externally. Its input/proposal hashes
identify complete approved files; `roster_sha256` hashes compact canonical JSON of its actual arms.

Canonical Python JSON for generated objects is UTF-8 `json.dumps(value, sort_keys=True,
separators=(',',':'), ensure_ascii=False, allow_nan=False)` plus one newline. Stored native semantic
transcript uses native serde struct order, explicitly not this Python recipe. Hashes are 64
lowercase hex; text ID lengths ≤192 UTF-8 bytes; error detail ≤4096 UTF-8 bytes. Limits are checked
before deserialization/allocation.

Default checked-in construction manifest: eight arms (both geometries for 1v1, 2v1, E/N64 and
C/N256), dt=1/64, H=8; two cases/arm with seeds 7 and 8. IDs start `fixture.` and never use a
registered key. Scientific generation is a preparation operation, not a test. Full-size synthetic
manifest/census tests never instantiate scientific Engines or draw OS entropy.

Each compact successful outcome has exactly: schema, key, seed, case_input_sha256, manifest_sha256,
completed_steps, observed_steps, active_steps, calendar_time, active_time, initial_counts,
final_counts, ending, observations, casualty_totals, contributed_exposure_totals, final_checkpoint,
transcript_sha256, audit_descriptor. `key` is the case wire key without seed. `observations` has
exactly four entries `{step, counts, origin, reason, reference, reference_error}` at requested
steps; origin is `native` or `derived_absorbing`. A derived entry retains the actual
end/capture-unavailable reason; it is not an Audit Frame. Native finite outcomes are not rejected
just because a reference comparison is unavailable. `audit_descriptor` is null for non-audit rows,
otherwise `{path,sha256,bytes,acknowledged_steps}`. Calendar/active clocks retain their core
meanings. No guessed resources, shots or individual killer.

Native ending and checkpoint wire values use existing core types. Checkpoint RNG JSON is a bounded
string, never a serde_json::Value/u128-to-f64 conversion. A failure receipt is separate from
successful outcomes and has
kind/detail/key/attempted_step/semantic_steps/observed_steps/durable_audit_steps/final_checkpoint
plus acknowledged archive prefix. Rate-zero engineering controls have unavailable biological
duration; they do not become zero-duration scientific observations.

Archive v1: `header.json`, exact `manifest.json`, a source/provenance packet,
`blocks/block-NNNNNN.jsonl`, `audits/<ordinal>.jsonl`, optional failure receipt, and `success.json`
only after full validation/synchronization. Each fixed 128-key block has ordered tagged `start` and
`outcome` records; starts sync before construction, outcomes sync before durable acknowledgment.
Closed blocks have separate immutable descriptor files. The last block may have fewer than 128 keys.
A start without a complete durable outcome is partial; no attempt/outcome is silently recreated.
Registered authorization and construction-fixture mode are separate paths.

---

### Task 1: Immutable contract, exact manifest and seed preparation

**Goal:** A strict, independently testable contract/factory and construction fixture.
**Success Criteria:** Exact arm/key census and genuine with-replacement seed semantics; no outcome
or activation creation.
**Tests:** Profile/mirror/matrix identity, u64 types, duplicates, quotas and entropy consumption.

**Files:** Create `survey/war1_study/__init__.py`, `contracts.py`, `manifest.py`, `io.py`,
`fixtures/construction-manifest.json`, `test_manifest.py`, `test_io.py`. Keep old registration
modules unchanged.

**Interfaces produced:**

```python
# contracts.py
from dataclasses import dataclass
from pathlib import Path
Json = dict[str, object]
@dataclass(frozen=True)
class Key:
    ordinal: int
    arm_id: str
    replicate_index: int
@dataclass(frozen=True)
class Case:
    key: Key
    seed: int
@dataclass(frozen=True)
class Manifest:
    raw: Json
    raw_bytes: bytes
    sha256: str
    purpose: str
    arms: tuple[Json, ...]
    cases: tuple[Case, ...]
# io.py
def strict_json(data: bytes, limit: int) -> Json: ...
def canonical_bytes(value: object) -> bytes: ...
def read_regular(path: Path, limit: int) -> bytes: ...
def create_bytes(path: Path, data: bytes, limit: int) -> None: ...
# manifest.py
from collections.abc import Callable
PROTOCOL = Path("docs/superpowers/specs/2026-10-10-war-1-measurement-design.md")
PROPOSAL = Path("docs/superpowers/specs/2026-10-10-war-1-measurement-proposal.json")
def proposal(root: Path) -> Json: ...
def validate_manifest(data: bytes, root: Path) -> Manifest: ...
def make_fixture(root: Path) -> bytes: ...
def sample_manifest(root: Path, entropy: Callable[[int], bytes], *,
    command: tuple[str, ...] = ("python3", "-m", "survey.war1_study", "prepare")) -> bytes: ...
```

The code sketches are API contracts, not empty bodies to commit. Implement the annotated functions
with real validation and error paths. `sample_manifest` is a pure injected-source factory; Task 5
alone exposes the OS-backed preparation CLI. Tests pass deterministic bytes, never call the
OS-backed path.

- [ ] **Step 1: RED factory behaviors.** Tests should exercise real parsing/factory output:

```python
import unittest, json
from pathlib import Path
from survey.war1_study import manifest
ROOT = Path(__file__).resolve().parents[2]
class ManifestTests(unittest.TestCase):
    def test_with_replacement_retains_equal_seed_values(self):
        data = manifest.sample_manifest(ROOT, lambda n: (7).to_bytes(8, 'little'))
        parsed = manifest.validate_manifest(data, ROOT)
        self.assertEqual((len(parsed.cases), len({c.key for c in parsed.cases})),
                         (52224, 52224))
        self.assertEqual({c.seed for c in parsed.cases}, {7})
    def test_boolean_seed_is_not_u64(self):
        raw = json.loads(manifest.make_fixture(ROOT))
        raw['cases'][0]['seed'] = True
        with self.assertRaises(ValueError):
            manifest.validate_manifest(json.dumps(raw).encode(), ROOT)
```

Pin 66/52,224/17,563,648/2,054,160,384; 121 groups/46 CDF comparisons/14 scalar/3 CDF IDs; exact L/S
balance, NC17:16, all mirrors, n per group, requested steps and core limits. Mutate one
profile/rate/cap/hash/key/order/ordinal/replicate and require refusal. Check u64::MAX,
negative/floating/bool/out-of-range seeds, nested duplicates, unknown fields, trailing JSON,
nonfinite/underflow literals, >16 MiB and nonregular/FIFO/symlink paths. Equal input seed/config
identities at different declared keys remain valid.

- [ ] **Step 2: Run RED.** `python3 -m unittest survey.war1_study.test_manifest survey.war1_study.test_io -v`; observed missing module/API or behavioral failure, not an invalid test path.
- [ ] **Step 3: Implement strict contract and IO.** `json.loads(..., object_pairs_hook=reject_duplicates, parse_int=int, parse_float=Decimal, parse_constant=reject_constant)` plus exact schema/type/field validation. Decode counts/seeds as integers (`type(x) is int`). Resolved floating config bits must match approved arms. Reject symlinks in existing ancestors and nonregular paths before opening; open/create exclusively, bounded read and write, flush/file sync/parent sync. Do not claim adversarial descriptor-relative race protection.

```python
def draw_u64(entropy: Callable[[int], bytes]) -> tuple[int, bytes]:
    block = entropy(8)
    if not isinstance(block, bytes) or len(block) != 8:
        raise ValueError("entropy source must return exactly eight bytes")
    return int.from_bytes(block, "little"), block
```

Generate every key from the immutable arm order, request exactly eight bytes per key, retain
duplicates, record draw-byte SHA; never reject/retry a value. Verify the entire approved roster, not
a user-edited subset. Fixture factory makes exactly the eight/16 construction population and marks
`construction_fixture`; no real sampled list is written in this task.
- [ ] **Step 4: GREEN.** Run the two modules, validate the generated checked-in fixture, and recompute every census/byte cap. Full-size synthetic manifest test has no Engine/subprocess/OS-entropy call. No fake fixture is described as genuine OS-sampled evidence.
- [ ] **Step 5: Review and commit.** Fresh task review for schema/census/entropy semantics. Commit `feat(war-study): define immutable manifests and construction fixtures`.

### Task 2: Independent exact numerics and no-simulation native export

**Goal:** Certified tiny-law/continuous-time predictions and native numeric input identity.
**Success Criteria:** Exact q-law, correct censoring atom and bounded real-exponential/radius
calculations without RNG/Engine use.
**Tests:** Literal 1v1/2v1 transitions, rational propagation, native p/dose/quantization
certificates and numeric failure refusal.

**Files:** Create Python `numerics.py`, `oracle.py`, `test_numerics.py`, `test_oracle.py`; Rust
`survey/src/bin/war1_study.rs`, `survey/src/war1_study/{mod,wire,numerics,cli,numerics_tests}.rs`;
`survey/tests/war1_study_cli.rs`. Modify `survey/Cargo.toml` only to add feature/bin. Add no
dependencies.

**Consumes:** Task 1 Manifest/strict IO; public core config/math types and locked libm. No
independent oracle calls Engagement, matching, RNG or native settlement.
**Produces:**

```python
# numerics.py; fixed-decimal wire intervals enclose rational/real values
from dataclasses import dataclass
from .contracts import Manifest, Json
from fractions import Fraction
from decimal import Decimal
@dataclass(frozen=True)
class Interval:
    lo: Fraction
    hi: Fraction
    def wire(self, scale: int | None = None) -> Json: ...
def enclose(value: Fraction, scale: int = 10**12) -> Interval: ...
def exp_interval(x: Fraction, precision: int = 100) -> Interval: ...
def probability_certificate(row: Json) -> Json: ...
def confidence_radius(slots: int, n: int, alpha: Fraction) -> Interval: ...
# oracle.py
@dataclass(frozen=True)
class Oracle:
    arm_id: str
    scalars: dict[str, Interval]
    cdfs: dict[str, tuple[tuple[Fraction, Interval], ...]]
    numerical_certificate: Json
    ctmc_one_vs_one: Json | None
def prepare_oracles(manifest: Manifest, numeric_export: Json) -> tuple[Oracle, ...]: ...
def geometric_sum(s: Fraction, k: int) -> Fraction: ...
def count_transition(blue: int, red: int, exposed: tuple[int,int],
                     q: tuple[Fraction,Fraction]) -> dict[tuple[int,int], Fraction]: ...
```

Native export schema `war1-study-numerics-v1`: purpose, manifest SHA, rules ID, binding_state,
compiled_inputs_sha256 (null until Task 3), and states. Each state is arm_id/B/R plus two target
`{hazard_bits,dose_bits,p_bits,q_numerator}` objects; bits are 16 lowercase hex digits, numerator
integer in [0,2^53]. Export all interior states of the twelve tiny approved descriptors, with no
seeds or constructors. Fixture mode may export these pure numeric cases; they are not histories.

```rust
// Native numerics.rs
#[derive(serde::Serialize)]
pub struct NumericTarget {
    pub hazard_bits: String, pub dose_bits: String, pub p_bits: String,
    pub q_numerator: u64,
}
#[derive(serde::Serialize)]
pub struct NumericState {
    pub arm_id: String, pub blue: u32, pub red: u32,
    pub targets: [NumericTarget; 2],
}
#[derive(serde::Serialize)]
pub struct NumericExport {
    pub schema: String, pub purpose: String, pub manifest_sha256: String,
    pub rules_id: String, pub binding_state: String,
    pub compiled_inputs_sha256: Option<String>, pub states: Vec<NumericState>,
}
pub fn export_numeric_inputs(manifest: &wire::Manifest) -> Result<NumericExport, String>;
// NumericExport and NumericState derive Serialize; exact wire fields as above.
// wire.rs
#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String, pub purpose: String,
    pub protocol_sha256: String, pub proposal_sha256: String,
    pub roster_sha256: String, pub sampler: Sampler,
    pub arms: Vec<Arm>, pub cases: Vec<Case>,
}
#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sampler {
    pub method: String, pub bytes_drawn: u64, pub seed_bytes_sha256: String,
    pub command: Vec<String>, pub version: String,
}
#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arm {
    pub arm_id: String, pub group: String, pub profile: String, pub scale: u32,
    pub mirror: bool, pub config: sugarscape_core::war::config::EngagementConfig,
    pub replicates: u32, pub seed_namespace: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key { pub ordinal: u64, pub arm_id: String, pub replicate_index: u32 }
#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub ordinal: u64, pub arm_id: String, pub replicate_index: u32, pub seed: u64,
}
pub fn decode_manifest(bytes: &[u8]) -> Result<Manifest, String>;
```

`Case::key(&self) -> Key` copies its ordinal/arm_id/replicate_index; add this method in wire.rs.
Sampler command is an argv array (at most32 items, each at most1024 UTF-8 bytes), version at most128
bytes. `sample_manifest` records the actual preparation CLI argv via an optional keyword `command:
tuple[str,...] = ("python3","-m","survey.war1_study","prepare")`; tests use an explicitly synthetic
command label. The standalone CLI supplies real argv. Keep RNG/checkpoint text as strings.
`strict_json` parses Decimal; `canonical_bytes` normalizes finite decimal config/display values to
binary64 after schema validation, retaining integer seeds, and rejects nonzero-to-zero/overflow.
Oracle/radius intervals serialize as rational endpoints, not Decimal numbers. Every numeric export
target is merged with its exact approved config, `blue`, `red`, and target_index0/1 before
`probability_certificate(row)`; the returned certificate binds those inputs/p bits/q numerator and
every exact-dose/ULP decision. `prepare_oracles` persists all registered tiny descriptors even in
fixture preparation; collection fixture IDs stay distinct.

- [ ] **Step 1: RED independent literal laws.** Use arbitrary rational q fixtures independent of libm:

```python
import unittest
from fractions import Fraction
from survey.war1_study import oracle, numerics
class OracleTests(unittest.TestCase):
    def test_geometric_one_step_categories(self):
        qb, qr = Fraction(1, 4), Fraction(1, 2)
        self.assertEqual(((1-qb)*qr, qb*(1-qr), qb*qr, (1-qb)*(1-qr)),
                         (Fraction(3,8), Fraction(1,8), Fraction(1,8), Fraction(3,8)))
        self.assertEqual(oracle.geometric_sum(Fraction(3,8), 2), Fraction(11,8))
        actual = oracle.count_transition(1, 1, (1,1), (qb,qr))
        self.assertEqual(actual, {(1,0): Fraction(3,8), (0,1): Fraction(1,8),
                                  (0,0): Fraction(1,8), (1,1): Fraction(3,8)})
    def test_interval_rounding_is_outward(self):
        x = Fraction(1, 3); bounds = numerics.enclose(x)
        self.assertLessEqual(bounds.lo, x)
        self.assertGreaterEqual(bounds.hi, x)
```

Add real `prepare_oracles` assertions for K=1/2, two-v-one product-binomial row masses, absorbing
states and final-step extinction, finite-duration CDF jump at1, s=1 mathematical limit (not native
rate-zero classification), input p/q mutation, q-threshold rounding, malformed/unresolved
certificates, nonfinite numeric fields and wrong manifest/rules hash. At least one actual oracle API
assertion must fail RED; arithmetic-only fixtures are not sufficient.
- [ ] **Step 2: Add native feature/test discovery and run RED.**

```toml
war-studies = ["war-benchmarks"]
# separate [[bin]]
name = "war1-study"
path = "src/bin/war1_study.rs"
required-features = ["war-studies"]
```

Start integration file with `#![cfg(feature = "war-studies")]`. Commands: Python numerical modules;
`cargo test --manifest-path survey/Cargo.toml --locked --offline --features war-studies --bin
war1-study --test war1_study_cli`. Observe missing API/assertion. No scientific collect command
exists in this task. Native numerical-case descriptors are the twelve approved tiny configurations,
derived from the approved roster independently of fixture collection IDs; exporting their arithmetic
never constructs their Engines.
- [ ] **Step 3: Implement native numeric export and engineering CLI.** Exact forms: help, `validate --manifest FILE`, `numerics --manifest FILE --out NEW_FILE`. Before Task 3, permit only byte-matching checked-in construction manifest and emit `engineering_unbound` with null compiled digest. Unknown commands including collect fail usage. Native validator rejects recursive duplicates/types/unknowns/bounds and obtains raw hash before resolution. Read fixture bytes with `include_bytes!` for engineering expected identity; Task 3 replaces this with the selected-manifest build context, never a fake stamp. Numeric state enumeration/hazard operator order must match source arithmetic; call only pure math for p, independently recompute q in Python from p bits.

The native entrypoint uses this exact Task2 CLI contract; implement both handlers with the bounded fixture/numeric behavior above. Task3/5 add their explicitly specified command forms, preserving these forms.

```rust
pub struct CliFailure { pub exit_code: i32, pub message: String }
pub fn validate_command(path: &str) -> Result<(), CliFailure>;
pub fn numerics_command(path: &str, output: &str) -> Result<(), CliFailure>;
pub fn main_args(args: &[String]) -> Result<(), CliFailure> {
    match args {
        [] => { println!("war1-study validate --manifest FILE | numerics --manifest FILE --out NEW_FILE"); Ok(()) }
        [flag] if flag == "--help" || flag == "-h" => {
            println!("war1-study validate --manifest FILE | numerics --manifest FILE --out NEW_FILE"); Ok(())
        }
        [cmd, flag, path] if cmd == "validate" && flag == "--manifest" => validate_command(path),
        [cmd, flag, path, outflag, output]
            if cmd == "numerics" && flag == "--manifest" && outflag == "--out" => numerics_command(path, output),
        _ => Err(CliFailure { exit_code: 2, message: "unsupported war1-study command or argument order".into() }),
    }
}
```

- [ ] **Step 4: Implement certified Python arithmetic.** Convert binary64 bits to Fraction exactly. For tiny states h is 1/2,1 or2 and dt is dyadic: assert exact native dose, no loose tolerance. Enclose Decimal.exp's nearest-half-even result with its half-ULP decimal rounding error plus subsequent arithmetic errors; directed context alone is not an interval exponential. Refine precision 100→200→400 deterministically; unresolved unique binary64 rounding at400 blocks preparation. Define ULP as max spacing to adjacent values around the uniquely rounded p* and require the complete error interval ≤4 ULPs.

```python
def enclose(value: Fraction, scale: int = 10**12) -> Interval:
    scaled = value * scale
    lower = scaled.numerator // scaled.denominator
    upper = -((-scaled.numerator) // scaled.denominator)
    return Interval(Fraction(lower, scale), Fraction(upper, scale))
```

Build binomial masses independently with `math.comb`, exact q Fractions and product transitions;
propagate all ≤6 states to every K. Compute four-time count means, five endings, U/H, CDFs and 1v1
CTMC comparison. Retain rational propagation internally; persist outward intervals on
denominator10^12, not gigantic decimal expansions of exact fractions. Normalized interval widths
≤1e-10, exact mass1. CDF support is authoritative, right-continuous; U/H at1 includes censored
cases, explicit censor probability stays separate. Store every dt CTMC criterion entry, pass/fail
margins at finest dt≤0.02; no global CTMC claim.

`Interval.wire(scale=None)` encodes exact rational lower/upper endpoints as signed numerator and positive denominator decimal strings (≤4096 digits each). High-precision exponential/ULP certificates must use this exact form; rounding p* to a10^-12 grid would destroy a four-ULP certificate. Exact DTMC intermediate fractions may exceed Python's decimal-digit conversion limit: never serialize them directly. Enclose final oracle quantities on the10^12 grid before wire conversion. Statistical report writers use `wire(scale=10**12)`, widening each lower/upper endpoint outward; the internal certified high-precision radius stays intact. This keeps CDF/report files bounded without narrowing confidence coverage.

- [ ] **Step 5: GREEN/review/commit.** Python four modules + native bin/integration tests, survey fmt and feature clippy. Verify numeric commands have no Engine/World/RNG construction path. Commit `feat(war-study): certify independent tiny-count references`.

### Task 3: Bound source/activation and compact durable collection

**Goal:** A functioning fixture collector and a scientific collector that cannot run before its exact activation.
**Success Criteria:** Pure observer equivalence; before-construction gate; stable compact/audit
evidence with truthful failures and budgets.
**Tests:** Exact direct-engine and runner receipts, full preflight refusals, partial writes,
accounting, absorption, quotas and source drift.

**Files:** Create Rust `source.rs`, `gate.rs`, `observer.rs`, `archive.rs`, `io.rs`, sibling
`*_tests.rs`; create `survey/build_support/war1_study_source_identity.rs`; extend local
wire/cli/mod, native integration test and `survey/build.rs` with separately gated study stamp.
Existing W1/F5/BT helper selection/logic remains unchanged.

**Consumes:** Task 1 manifest wire; Task 2 native structs/export; existing public
`Engagement::{new,step,counts,ending,checkpoint}` and independent `reference_at`.
**Produces:**

```rust
// gate.rs: fields private; callers cannot create a scientific authorization.
pub struct AuthorizedCampaign { manifest: wire::Manifest, binding: CampaignBinding }
pub struct CampaignBinding {
    pub manifest_sha256: String, pub compiled_inputs_sha256: String,
    pub executable_sha256: String, pub declaration_sha256: Option<String>,
    pub acceptance_sha256: Option<String>, pub execution_approval_sha256: Option<String>,
}
pub fn preflight(declaration: &std::path::Path, acceptance: &std::path::Path,
    execution: &std::path::Path) -> Result<AuthorizedCampaign, String>;
// observer.rs
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub schema: String, pub key: wire::Key, pub seed: u64,
    pub case_input_sha256: String, pub manifest_sha256: String,
    pub completed_steps: u64, pub observed_steps: u64, pub active_steps: u64,
    pub calendar_time: f64, pub active_time: f64,
    pub initial_counts: [u32;2], pub final_counts: [u32;2],
    pub ending: sugarscape_core::war::Ending, pub observations: Vec<Observation>,
    pub casualty_totals: [u32;2], pub contributed_exposure_totals: [f64;2],
    pub final_checkpoint: sugarscape_core::war::Checkpoint,
    pub transcript_sha256: String, pub audit_descriptor: Option<AuditDescriptor>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub step: u64, pub counts: [u32;2], pub origin: String, pub reason: Option<String>,
    pub reference: Option<ReferenceWire>, pub reference_error: Option<ReferenceErrorWire>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceWire {
    pub requested_time: f64, pub evaluated_time: f64, pub forces: [f64;2],
    pub regime: String, pub extinction_time: Option<f64>, pub at_boundary: bool,
    pub survivor: Option<sugarscape_core::war::config::Side>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceErrorWire { pub field: String, pub detail: String }
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditDescriptor {
    pub path: String, pub sha256: String, pub bytes: u64, pub acknowledged_steps: u64,
}
#[derive(serde::Serialize)]
pub struct CaseFailure {
    pub kind: String, pub detail: String, pub key: wire::Key,
    pub attempted_step: Option<u64>, pub semantic_steps: u64,
    pub observed_steps: u64, pub durable_audit_steps: u64,
    pub final_checkpoint: Option<sugarscape_core::war::Checkpoint>,
    pub acknowledged_archive_prefix: Option<archive::ArchiveReceipt>,
}
pub(crate) fn execute_case(case: &wire::Case, arm: &wire::Arm,
    audit: Option<&mut dyn FrameSink>) -> Result<Outcome, CaseFailure>;
pub trait FrameSink { fn frame(&mut self, frame: &sugarscape_core::war::Frame) -> Result<(), String>; }
// archive.rs
pub struct ArchiveReceipt { pub sha256: String, pub bytes: u64, pub acknowledged_cases: u64 }
pub struct CampaignFailure { pub detail: String, pub census: Census, pub receipt: ArchiveReceipt }
pub struct Census {
    pub planned: u64, pub attempted: u64, pub complete_valid: u64,
    pub failed: u64, pub invalid: u64, pub partial: u64,
    pub pending: u64, pub unstarted: u64,
}
pub fn collect(authorized: AuthorizedCampaign, out: &std::path::Path)
    -> Result<ArchiveReceipt, CampaignFailure>;
```

`Outcome`/`CaseFailure` use fully typed fields, not serde Values containing RNG. Copy core
ReferencePoint/Failure into the local strict wire types field-by-field, matching the four core
regime variants exactly; do not add Deserialize to the core just to read study outcomes. `Outcome`
is deserialize-capable because graph Checkpoint/Ending/Side already are; local reference DTOs close
the Serialize-only core reference boundary. Scientific bindings require all three Some receipt
hashes; fixture bindings have None and cannot activate science. Add `collect_fixture(manifest_path:
&Path, out: &Path) -> Result<ArchiveReceipt, CampaignFailure>` with fixed compiled-fixture
validation; it shares archive/observer logic but never produces a registered header. Actual
process-resource validation is mandatory for science; until Task5 supervisor support is complete,
collect returns a descriptive unready error before creating output. Census membership is a partition
over keys: unstarted; started pending/partial; closed complete-valid/failed/invalid. Also expose
attempted as derived total started. Avoid double counting partial among pending and failed. Include
last semantic/observed/audit clocks in per-key evidence. Scientific `execute_case` is private to the
collector module path guarded by an AuthorizedCampaign; expose its testing/fixture entry only
through explicit fixed-fixture authorization, not arbitrary public science simulation. No mutable
Engine/RNG handle reaches FrameSink.

- [ ] **Step 1: RED observer equivalence/gates.** Tests use seeds7/8 and construction configs; compare every raw Frame and final Checkpoint with direct engine and existing run_to. Include post-ending derived counts, final-step extinction, no repeated casualty IDs, source contributed-rate sums, final RNG strings, exact active/calendar clocks and observer toggles.

```rust
#[test]
fn valid_looking_declaration_without_execution_receipt_constructs_nothing() {
    // Real fixture paths; constructor counter is a test-only hook before new().
    let fixture = tests::gate_fixture();
    let before = tests::construction_count();
    assert!(gate::preflight(&fixture.declaration, &fixture.acceptance,
                           &fixture.missing_execution).is_err());
    assert_eq!(tests::construction_count(), before);
    assert!(!fixture.output.exists());
}
```

Create `gate_tests.rs` helpers `GateFixture { declaration: PathBuf, acceptance: PathBuf,
missing_execution: PathBuf, output: PathBuf }`, `gate_fixture() -> GateFixture`, and
`construction_count() -> usize`; the helper writes structurally valid synthetic bound receipts into
a canonical private temp root, except the intentionally absent execution file. The constructor hook
is thread-local Cell in cfg(test) and increments immediately before the production new call, never
an alternate settlement path. Read/validate required execution receipt before constructor/output
operations; assert the failure mentions that receipt. Keep synthetic receipt context explicitly
labeled as testing, never claimed as an actual readiness decision. Test wrong
source/binary/protocol/manifest hashes, unsupported schemas, missing readiness, self-labeled
approval text, prior histories, changed source/map, symlink/FIFO and >16 MiB before any output.
- [ ] **Step 2: Run RED.** Native feature bin/integration command from Task 2. Expected missing gate/observer/archive API or explicit behavior failure.
- [ ] **Step 3: Implement separately gated build binding.** `WAR1_STUDY_MANIFEST` selects exactly one committed manifest under `survey/war1_study/manifests/`, or defaults to the checked-in construction fixture. Emit rerun-if-env-changed and rerun-if-changed for exact selected inputs. Require canonical relative paths/no symlinks; fail missing inputs. Hash domain `war1-study-compiled-inputs-v1\0`, sorted UTF-8 relative paths, u64BE path/data lengths and raw bytes. Inputs: root/core/survey manifests/locks/build.rs, all core/src regular files and sugar-map, local native study sources/bin/helper, every Python study module, approved W1-A design/roster/this plan, original W1 design/plan/amendment and selected manifest. Final declarations/approvals/probes/outputs are explicit FILE exclusions, not recursive normative directories. No source/binary self-digest fields in the selected manifest. Remove the Task2 fixture-only include_bytes binding when replacing it with selected-manifest binding; a scientific build embeds only its selected manifest. Preserve existing stamp helpers.

Stamp fields: selected manifest relative path/raw SHA/purpose, compiled inputs SHA/path list, source
root, rustc-vV, features and inherited flags. Native invocation independently rehashes selected
current bytes and actual executable. Numeric exports now emit source-bound hashes; an
engineering-unbound certificate cannot activate collection.
- [ ] **Step 4: Implement exact preconstruction activation.** Final declaration/readiness/execution wire schemas are defined in Task 5; implement against that exact contract. Numeric/resource certificate digests and source/binary/manifest bindings must match. Require readiness accepted, independent named reviewer, zero registered histories and owner execution action `execute_exact_population`. Source-clean declared paths must match the frozen inventory. Fixture path `fixture --manifest FILE --out NEW_DIR` requires compiled fixture purpose, fixed fixture bytes and seeds7/8; refuses a scientific purpose. Science form is `collect --declaration FILE --acceptance FILE --execution FILE --out NEW_DIR`. No user string/flag sets permission. Validate all inputs and resource reservation before create-directory; sync starts before each constructor.
- [ ] **Step 5: Implement compact observer/transcript.** Canonical semantic transcript SHA256 domain `war1-study-transcript-v1\0`, then u64BE-length-prefixed native serde JSON bytes of `{config,seed}`, initial typed Checkpoint, every typed Frame in order, final typed Checkpoint. No ReferencePoint, Key or audit toggles in this semantic hash. Same input at repeated declared keys may have identical hashes. Validate Frame clocks/start/survivor/casualty accounting, bounded IDs, all floats and ordered state before observer commit. Track IDs in O(initial population) memory; every casualty appears once. Four observations keep counts/origin/reason and separate independent reference at t=1/2/4/8. Ended future entries are `derived_absorbing`, never native Frames. Reference evaluation uses original config/time, not observed survivor forces.
- [ ] **Step 6: Implement durable archive and resource failures.** Audit schema tagged header/initial/frame/terminal; frame carries fixed `benchmark_exposure`. Header retains `case_json` (native canonical `{config,seed}` JSON string); initial/terminal retain `checkpoint_json` (native canonical typed Checkpoint JSON string); frame retains `step` and `frame_json` (native canonical typed Frame JSON string). Deserialize these string payloads strictly for validation, but hash their exact UTF-8 bytes without Python reserialization. This preserves native float spelling/struct order and RNG text; no Python-vs-Rust JSON recipe ambiguity. Audit terminal carries compact counters without recursive captured Frames. Sync each audited frame before audit acknowledgment. Non-audit observation is in-memory; durable outcome acknowledgment happens only at successful row write/flush/sync. Fixed block128keys, at most two lines/key, close descriptor only after sync; never overwrite/rename across existing destinations. Bound all serialization, global bytes, error strings and arrays before allocation. Poison a failed writer; no retry.

Require start/outcome injection tests for short writes, partial writes, flush/sync/parent-sync,
final marker revocation and double failure-receipt errors. Task5's supervisor owns elapsed/RSS
checks at ≤0.1s and final individual child peak from os.wait4; its exact contract is below. In the
native loop, check its stop channel at each frame boundary and return the actual last semantic
checkpoint if signaled. Before Task5 integration scientific collect is unready. Native writes a
synchronized `candidate-index.json`, not artifact success; the supervisor validates peak/time/exit
and saved archive before exposing `success.json`. Fixture-only tests may finish through the same
supervisor or a deterministic injected monitor that is labeled construction mode. A resource stop
preserves actual completed state and is not `horizon`. Native candidate index rereads/validates
exact keys/rows/audits/source bindings and synchronizes all directory entries; Task5 parent performs
the final monitored success publication. Check checkpoint strings typed/bounded; never coerce u128
through Value.
- [ ] **Step 7: GREEN/review/commit.** All native study tests + unchanged W1 case integration/goldens; feature clippy/fmt. Failed permission/source inputs must leave constructor count/output absent; failure prefixes/clocks match injected effects. Commit `feat(war-study): collect compact source-bound engagement evidence`.

### Task 4: Saved-data-only validation and complete analysis

**Goal:** Complete reproducible summaries and all fixed comparisons from retained evidence.
**Success Criteria:** Exact census/accounting, certified numerical bounds and declared availability;
no simulation or successful-only denominator.
**Tests:** Full-size synthetic census, atomic horizon/censoring, every endpoint/map,
missing/torn/duplicate rows, tiny-law mismatch and interval enclosure.

**Files:** Create Python `archive.py`, `analysis.py`, `test_archive.py`, `test_analysis.py`,
`test_matrix.py`. Add no simulator subprocess to analysis.

**Consumes:** Task 1 Manifest/Key/strict IO; Task 2 Oracle/Interval; Task 3 archive wire.
**Produces:**

```python
from dataclasses import dataclass
from pathlib import Path
from .contracts import Manifest, Json
from .oracle import Oracle
from collections.abc import Callable, Iterator
@dataclass(frozen=True)
class ArchiveRead:
    header: Json
    outcomes: Callable[[], Iterator[Json]]
    census: Json
    audits: tuple[Json, ...]  # bounded descriptors, not all raw frames
    complete: bool
# archive.py
def read_campaign(root: Path, manifest: Manifest) -> ArchiveRead: ...
# analysis.py
@dataclass(frozen=True)
class Analysis:
    scalars: tuple[Json, ...]
    cdfs: tuple[Json, ...]
    contrasts: tuple[Json, ...]
    cdf_comparisons: tuple[Json, ...]
    oracle_checks: tuple[Json, ...]
    census: Json
    primary_available: bool
def analyze(manifest: Manifest, archive: ArchiveRead, oracles: tuple[Oracle, ...]) -> Analysis: ...
def save_analysis(out: Path, analysis: Analysis, binding: Json) -> None: ...
```

Emit one fixed row per arm/endpoint and per contrast group/endpoint, including unavailable rows.
Full science report:924 scalar rows,198 CDF descriptors with all exact support values,1694 scalar
contrasts (121×14),46 CDF comparisons, tiny-law14+3per12 arms, and all1v1 CTMC criteria. Fixture
reports explicitly `registered:false`; no construction output becomes research evidence.
Deterministic seeds7/8 have descriptive fixture summaries only, with sampling intervals/conformance
verdicts unavailable for reason `unregistered_construction_panel`. Full scientific family tests use
synthetic52,224-key inputs, clearly labeled synthetic, without engine calls. Fixture tiny
descriptors match numeric oracle descriptors by exact config bits; identifiers are not conflated
with scientific keys.

- [ ] **Step 1: RED authoritative endpoint/census tests.** Synthetic rows are generated deterministically without entropy/Engine. Pin five category partition and source K semantics:

```python
import unittest
from survey.war1_study import analysis, test_fixtures as fixtures
class AnalysisTests(unittest.TestCase):
    def test_actual_horizon_extinction_is_not_censored(self):
        final_extinction = fixtures.outcome(ending='one_side_extinction', step=512,
                                            winner='blue', censored=False)
        censored = fixtures.outcome(ending='horizon', step=512,
                                    winner=None, censored=True)
        self.assertEqual(analysis.endpoint(final_extinction, 'ending.horizon'), 0)
        self.assertEqual(analysis.endpoint(censored, 'ending.horizon'), 1)
        self.assertEqual(analysis.endpoint(final_extinction, 'duration.restricted_mean_over_H'),
                         analysis.endpoint(censored, 'duration.restricted_mean_over_H'))
```

Create `survey/war1_study/test_fixtures.py` containing `outcome(*, ending:str, step:int,
winner:str|None, censored:bool) -> Json`: a complete construction row for E/N64/dt1-64/seed7,
initial[64,64], final[1,0] if blue wins or [1,1] if horizon, casualty totals[63,64] or[63,63], valid
four count captures, fixture checkpoint/transcript/audit descriptors. The test-only checkpoint
numeric text and raw audit transcript are generated from deterministic synthetic structs, not Engine
outputs. Import this as fixtures. Define `endpoint(row:Json, endpoint_id:str)->Fraction|None` in
analysis; known unavailable returns None with emitted reason, unknown ID raises. Tests cover
duplicate keys vs repeated seeds, wrong ordinal/hash/config, missing/torn start/outcome, wrong
terminal clock, invalid RNG checkpoint text, forged audit digest/transcript, unavailable future
capture, nonfinite fields, normalized zero denominator, reference-only unavailable, no
simulation-import/subprocess path.
- [ ] **Step 2: Run RED.** `python3 -m unittest survey.war1_study.test_archive survey.war1_study.test_analysis survey.war1_study.test_matrix -v`.
- [ ] **Step 3: Implement bounded saved archive reader.** Strict path/digest/length/header/index/block checks, maximum256 lines/block and proven line quotas. Reconcile exact ordered key set and each compact row's seed/config/ending/count/deaths/time/final Checkpoint. Only parse RNG text with exact Python int, bounded <2^128/odd state; no simulator restore. For audits recompute transcript from the saved raw `case_json`, initial/terminal `checkpoint_json` and each `frame_json` UTF-8 string using the exact native length-prefix recipe; validate parsed fields separately and compare compact digest. Non-audit hash stays replay-checkable but unverified against full raw frames; label that limit. Source provenance/frozen declaration matched before inference. Incomplete archives may emit complete census/unavailability, never primary successful-only estimates.

`ArchiveRead.outcomes()` is a bounded fresh iterator over validated closed-block descriptors. It rechecks each block digest/length while consuming it, yields one at-most32KiB row, and retains no per-run checkpoint collection. Analysis keeps per-arm integer sums and finite-support histograms only; duplicate/key census memory is bounded by the52,224-key manifest, not raw rows. Stream audit verification one file at a time. If bytes change between validation/analysis, inference is unavailable. This avoids materializing a4GiB archive as an unbounded Python object graph.

- [ ] **Step 4: Implement exact estimates/outward bands.** Normalize integer observations and steps with Fraction; mean over planned n only when complete. Use Task2 outward radius intervals at M924/C198/α1/40. CDF thresholds are exact rational supports, right-continuous, including atom at1; censor fraction comes from ending flag. Difference intervals use certified arm interval arithmetic and correct left-minus-right mappings. Same-arm unchanged variables can cancel exactly; do not assume independent endpoints. Recompute radii, don't trust JSON display decimals.

```python
# Required interval logic, including unavailable handling outside this primitive.
def difference(a: Interval, b: Interval) -> Interval:
    return Interval(a.lo - b.hi, a.hi - b.lo)
```

`difference` is a public numerical helper in numerics.py added by this task; no later task invents a
competing formula. For CDF distance compare union support with outward bands and d_a+d_b.
`incompatible_with_ideal_word_law` only when observed/ideal interval separation exceeds the
prescribed confidence/numeric envelope; overlap is `not_rejected`, not equality. Reference/ideal
unavailability retains every slot and reason, never changes M/C. Print no post-selected headline
test or inferred historical duration.
- [ ] **Step 5: GREEN/review/commit.** All Python study tests; compare independent tiny-law means/CDFs to literal fixtures. Generate a complete52,224-key synthetic input and verify exact reporting cardinalities, byte-budget, deterministic reanalysis bytes and simulator-call absence. Commit `feat(war-study): analyze complete saved populations with certified bounds`.

### Task 5: Preparation orchestration, resource proof, CI and software handoff

**Goal:** Working preparation commands and verifiable no-history readiness artifacts.
**Success Criteria:** Noncircular source/binary packet, enforced gate tests, proven quotas/monitored
limits, full software checks and exact-head CI.
**Tests:** Real CLI prepare/validate/fixture refusal/analysis; fake entropy and synthetic payloads;
declaration/receipt mismatches; postbuild source/binary changes; size/monitor boundaries.

**Files:** Create Python `registration.py`, `preparation.py`, `__main__.py`, `test_registration.py`,
`test_preparation.py`, `test_cli.py`; native `metadata.rs` plus tests; finish local cli/integration
tests; modify CI with yq; add `docs/war1-study.md`; narrow living study next-step pointer. No
original approved source-bound doc or old scientific asset edits.

**Consumes:** Tasks1–4 contracts/Manifest, numeric export/Oracle, archive/Analysis and native binding.
**Produces (exact declaration/activation wire consumed by Task3):**

- Build receipt `war1-study-build-v1`: manifest/protocol/proposal/source-inventory/compiled-input SHA, selected path/purpose, binary path/SHA/size, resolved-roster SHA, rustc/cargo/platform/features/flags, exact build command and unchanged pre/post inventory descriptors.
- Numeric receipt `war1-study-numeric-v1`: manifest/source/binary/numeric-export/oracle packet SHA, certificate/mass/width/ULP/CTMC decisions and exact oracle IDs; all required valid, no unresolved bypass.
- Resource receipt `war1-study-resources-v1`: exact schema bounds/aggregate reserved byte proof; fixture probe descriptors/commands/source/binary hashes/time/RSS; one worker, 2 GiB/12h/4GiB limits; explicit conservative forecast method/inputs, no measured scientific output. Failure/unavailable blocks readiness.
- Declaration `war1-study-declaration-v1`: protocol/proposal/manifest/compiled-input/source-inventory/build/numeric/resource/resolved-roster/binary identities and artifact descriptors `{path,sha256,bytes}`; normative frozen revision and distinct declaration revision, purpose `prospective_frozen_not_blind`, `registered_histories:0`, exact census/budgets.
- Acceptance `war1-study-readiness-v1`: declaration SHA, independent reviewer identity/role, decision `accepted`, `registered_histories:0`, no open required finding; binds build/numeric/resource/binary/manifest digests. A label is not evidence of an actual review.
- Execution `war1-study-execution-v1`: declaration and acceptance SHA, owner identity, action `execute_exact_population`, exact census/budgets and quoted explicit user authorization with time; source/binary/manifest digests. This plan never manufactures that authorization from software approval.

```python
# registration.py; prepare draws input seeds only, never constructs an Engine.
from pathlib import Path
from .contracts import Manifest, Json
def prepare(root: Path, out_dir: Path) -> Path: ...
def freeze(root: Path, manifest_path: Path, out: Path) -> Json: ...
def build_receipt(root: Path, manifest: Manifest, inventory: Json,
                  binary: Path, build_command: tuple[str, ...], out: Path) -> Json: ...
def declare(build: Json, numeric: Json, resources: Json, out: Path) -> Json: ...
# preparation.py
def prepare_evidence(root: Path, fixture_binary: Path, scientific_binary: Path,
                     manifest: Manifest, out: Path) -> tuple[Json, Json]: ...
```

These are exact API contracts, not empty-function scaffolds. `prepare_evidence` runs only fixed
construction/synthetic fixtures and no-simulation validate/numerics on the scientific binary.
Registered case execution needs separate future activation. Artifact readers use Task1 bounded
strict IO; receipts use exact schema/field/key validation and raw-byte external hashes.

- [ ] **Step 1: RED noncircular/boundary integration.** Test entirely synthetic bindings/default fixture binary: declaration includes final binary hash yet is outside selected inventory; postbuild source/map/manifest/binary mutation refuses; missing certificate/forecast/wrong numeric envelope rejects; bad receipt or a supplied `--approved` flag leaves output absent. Mock entropy only in preparation tests and assert zero Engine constructors. Real default fixture-binary test with a plausible scientific manifest must fail before construction. Test current engineering `war1 --help/validate/run` forms and `registered:false` remain unchanged.
- [ ] **Step 2: Run RED.** All Python study tests plus real native study integration. Expected missing orchestration/metadata API or contract assertion.
- [ ] **Step 3: Implement exact CLI and supervisor.** Python entrypoint commands:

```text
python3 -m survey.war1_study prepare --root ROOT --out-dir DIR
python3 -m survey.war1_study freeze --root ROOT --manifest FILE --out FILE
python3 -m survey.war1_study build-receipt --root ROOT --manifest FILE --inventory FILE --binary FILE --command-file FILE --out FILE
python3 -m survey.war1_study declare --build FILE --numeric FILE --resources FILE --out FILE
python3 -m survey.war1_study analyze --campaign DIR --manifest FILE --oracles FILE --out NEW_DIR
```

Preparation uses `os.urandom(8)` and stores the full sampler/seed manifest create-only; no seed
repeats discarded. Output raw manifest has no own digest/path/source/binary self-reference. Hash
first, then choose `manifests/frozen-<rawsha>.json`; the content never contains that filename.
Freeze declared inputs after writers stop. `WAR1_STUDY_MANIFEST` must select this exact file for
scientific build. No preparation command runs collection. Native forms: help, validate, numerics,
metadata (all without Engine), fixture (only compiled construction purpose), collect (requires exact
three receipts). CLI exit2 for input/usage, exit1 execution/output, exit0 success. No
resume/retry/override flags.

Implement `supervise(command: tuple[str,...], out: Path, wall_seconds: int, rss_bytes: int) -> Json`
in preparation.py using Python stdlib only. Start native Popen with stdout/stderr bounded to files
and a bidirectional local UnixStream control socket (Python socket.AF_UNIX, Rust
std::os::unix::net::UnixStream with set_nonblocking). Use a private create-new directory below
canonical /tmp (Mac resolves to /private/tmp), socket path ≤99 bytes; no unsafe FD/ABI shim/new
library. Add native internal `--control-socket PATH` to collect/fixture invocations; only the
documented supervisor supplies it. After permission/source checks and before collection
output/constructor, handshake `war1-study-monitor-v1` binds manifest/binary hashes, child PID and
declared limits. Parent sends heartbeat every≤0.1s and stop messages; child drains bounded≤16KiB
messages at each frame, enforces its own monotonic12h limit, and stops if heartbeats stale>2s.
Progress carries key/semantic/observed/durable counts; parent retains last progress with a
timestamp, not a full unbounded message log. Do not call Popen.poll/wait/send_signal: collect the
individual child's status/peak with `os.wait4(pid, os.WNOHANG)` and set returncode after reaping.
Linux reads bounded `/proc/<pid>/status` VmRSS/VmHWM (KiB); macOS calls system `ps -o rss= -p PID`
(KiB) for active samples. Every ≤0.1s check monotonic elapsed/RSS. On completion validate individual
wait4 ru_maxrss (macOS bytes, Linux KiB) too, catching an unsampled burst. While child is unreaped
its PID cannot be recycled; stop with os.kill SIGTERM, allow at most5s, then SIGKILL and reap.
Missing/invalid sampler/monitor/platform errors fail closed. Record exact monitor/platform/units and
peak; never misattribute RUSAGE_CHILDREN's historical aggregate to this child. Read progress/prefix
after termination. If a cooperative native failure receipt exists, its actual clocks/checkpoint are
authoritative. After an abrupt kill without a final native receipt, outer failure has null
semantic/observed clocks and checkpoint with reason `unknown_after_termination`, plus separately
labeled last-progress values/time and exact durable prefix/census; never report a progress sample as
final physics. Return partial keys and both kill/receipt errors. Native `candidate-index.json` is
provisional; only successful exit plus source/archive/monitor validation permits parent
create-new/sync `success.json`. A peak/time violation after a native candidate still means outer
failure, not success or native horizon. Inject timer, sampler and wait4 results in tests; no 12-hour
wait or real large allocation. Actual science entry is through this supervisor; native gate requires
its control-channel handshake before directory/constructor operations, not a free-form monitor
claim.
- [ ] **Step 4: Prove schema and preparation resources.** Implement a byte-accounting certificate, not observed-size extrapolation. Text IDs192/error4096/sha64/RNG128bytes; arm population≤768, IDs≤767, horizons≤512, observations4. Bound the complete escaped audit frame line at ≤4096fixed bytes +32bytes×casualties≤28,672; independently confirm the actual field-name/scalar maximum proof and escaping overhead. Compact rows contain no repeated full Frames; 4 count/reference entries, ≤768 living IDs, two bounded RNG strings and fixed metadata must fit32KiB. Start ≤1KiB. Header/initial/terminal each ≤32KiB, no recursive captured Frames. Metadata/provenance file16MiB bound plus an explicit file-count/total reserve. Planned starts/outcomes1,764,753,408 bytes; exact66 audit horizons+3envelopes778,240,000; reserve1GiB gives3,616,735,232 <4GiB. Track all output bytes, including failed tails and receipts, against reserved subbudgets; reserve enough to write failure after a limit is reached. Bound full metadata/index files and object graph memory before allocation.

Fixed probes: the16 default construction cases/seeds7/8, plus synthetic worst-row/manifest/archive
workloads, run once in release with matching declared source. They are labeled engineering and
excluded from science. Forecast from measured maximum per-actor-step/projection/sync costs with
conservative margin and the full2,054,160,384 actor-step/52,224 start/outcome bound; document
limitations. If practical forecast/memory/encoded proof fails, do not sample science seeds or
activate; propose a separately reviewed budget/workload amendment. No tolerance, population or
ceiling change after scientific output.
- [ ] **Step 5: CI and broad software checks.** Add `war-study` CI job using existing checkout/toolchain/cache actions and yq; change no existing job semantics:

```sh
python3 -m unittest discover -s survey/war1_study -p 'test_*.py'
cargo test --manifest-path survey/Cargo.toml --locked --features war-studies --bin war1-study --test war1_study_cli
cargo clippy --manifest-path survey/Cargo.toml --features war-studies --bin war1-study --test war1_study_cli -- -D warnings
```

CI uses only the checked-in fixture manifest, no OS scientific sampling or activation. Temporary
names/standard library hash seeds are not scientific sampler draws; tests instrument the scientific
seed factory, not unrelated OS randomness. Local task-specific gates run `--locked --offline`. Once
at reviewed final software: root/survey fmt, root all-feature clippy, full workspace tests, core war
tests/goldens, engineeringW1 and study native bin/integration tests/clippy, default survey bins (no
study/war binaries), core wasm32 check with war feature excluded, web build before web tests, and
studio suite with byte-identical retained test fixtures if absent. Bind reused unaffected receipts
to matching actual inputs/toolchain/flags; don't copy whole targets or regenerate old scientific
assets.
- [ ] **Step 6: Documentation, review and commit.** Document separate CLI/purposes, frozen manifest/build selection, data/censor semantics, 10.5-point coarse precision, exact tiny-law criterion, compact/raw/transcript differences, receipts/quotas/limits and zero-history readiness. Status says tooling ready/prospective activation pending, not research results. Commit `feat(war-study): prepare prospective source-bound measurement handoff` after tests. Per-task independent review; whole-branch review after all five, resolve findings with affected checks. Integrate reviewed software normally into main preserving concurrent work; push and verify exact-SHA CI, as normal delivery. Preserve tracker/evidence and all historical identities.

## Post-software prospective handoff (part of Stage 5 completion)

After all software reviews/CI pass and no normative writers remain, prepare the actual OS-sampled
manifest once, commit it without constructing a scientific Engine, freeze its normative inputs, and
build a scientific `war1-study` executable with `WAR1_STUDY_MANIFEST` pointing to that exact
committed file. Keep the separate fixture binary for construction probes. Run scientific-binary
validate/numerics/metadata only; assemble certified oracle/resource/build receipts and external
declaration, commit declaration outside selected inputs, recheck inventory/bytes after build, and
retain both binaries/commands/artifacts. Distinguish source freeze revision from later
declaration/receipt commits; those extra commits may add only excluded provenance artifacts without
changing the bound normative inputs.

Readiness review checks the actual packet and zero-history census, not just these generic
instructions. Hand the user that concrete source/binary/manifest/resource/numeric declaration for
exact-workload execution approval. Approval of this implementation plan authorizes software
implementation and the post-review input/seed preparation, freeze/build, numerical/resource
verification and readiness packet above. It authorizes no scientific collection, analysis of
measured outcomes, publication or manufacturing of an execution receipt. All early task tests use
construction/synthetic data; the actual OS seed list is generated only once in the post-software
handoff. If source/method/manifest requires correction after freezing, issue a newly dated
identity/review rather than rewriting the frozen packet. Retain the execution worktree until its
unique artifacts are archived.

## Inline plan self-review / coverage map

| Approved requirement | Owner / check |
|---|---|
| Profiles/66arm exact census/seed population/duplicates | Task1 factory/strict contract; Task4 exact census |
| Native p vs q vs mathematical p*, exact dose/ULP/rational law | Task2 export/certificate/DTMC and no-constructor test |
| Censoring/absorption/active and calendar/durable clocks | Task3 observer/failure tests; Task4 atom-at1 tests |
| All 924/198/1694/46 estimates/maps and simultaneous bounds | Task4 matrix/interval/CDF tests |
| Separate compact collector/native boundaries/unchanged kernel | Tasks2/3 feature and observer equivalence; Task5 compatibility |
| Source/binary freeze/declaration/independent approval | Task3 private gate; Task5 noncircular actual packet |
| Full audit index0/semantic hash/non-lossless limit | Task3 transcript/audits; Task4 saved audit rehash |
| Bytes/work/RSS/wall/partial incomplete populations | Task3 injected failures; Task5 proven bound/monitor/forecast |
| Construction vs science vs publication scopes | Task1 fixture purpose; Task3 gate; Task5 zero-history handoff |

This plan builds preparation and guarded software. It leaves the approved52,224 scientific histories
unconstructed until the later explicit exact-packet execution approval.
