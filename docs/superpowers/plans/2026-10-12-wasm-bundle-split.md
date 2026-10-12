# WASM Bundle Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the playground as a 6–7 MB engine bundle and load the Minds provenance cluster only inside the episodes worker, through a second WebAssembly package.

**Architecture:** A new crate `crates/sugarscape-experiments-wasm` takes the six exports that reach the provenance cluster (four `experiment_*`, two `burrow_*`) out of `crates/sugarscape-wasm`. Link-time optimization then drops the cluster from the engine bundle with no change to `sugarscape-core`. wasm-pack builds two packages into `web/src/wasm-pkg/` and `web/src/experiments-pkg/`; the episodes worker and twelve vitest files import the second one; a budget script fails the web build if the engine bundle grows past 8,000,000 bytes.

**Tech Stack:** Rust 2021 workspace, wasm-bindgen 0.2, wasm-pack 0.15.0, Vite 8 with ES-module workers, vitest 5, Node 22.

**Spec:** `docs/superpowers/specs/2026-10-12-wasm-bundle-split-design.md`

## Global Constraints

- `crates/sugarscape-core` is not modified in any way. Its `lib.rs` is authenticated byte-for-byte by `web/src/episodes/caching-parity.test.ts`.
- The new crate must not depend on `sugarscape-wasm`; an rlib dependency would carry every engine export into the experiments bundle.
- Package layout, verbatim from the spec: engine package in `web/src/wasm-pkg/` with out-name `sugarscape`; experiments package in `web/src/experiments-pkg/` with out-name `experiments`.
- Engine bundle budget: `web/src/wasm-pkg/sugarscape_bg.wasm` at most 8,000,000 bytes after `--release`.
- No model fingerprint, golden, or recorded fixture changes. `cargo test --workspace` and `npm test` must pass unchanged apart from the files this plan names.
- American spelling. Say "agent", never "Flump", in code and docs.
- Run `cargo fmt --all` before every Rust commit; CI runs `cargo clippy --all-targets -- -D warnings`.
- Commits end with the session attribution line given in the executing session's instructions.

## Review Focus

Failure modes the spec implies that no task's unit test would otherwise exercise; each line names the task whose steps pin it.

1. The deployed page must still run an episode: the built site must contain the experiments wasm as a separate asset and only the episodes worker chunk may reference it. Task 4 greps `web/dist` for this.
2. The budget script must fail, not pass, when the engine wasm is missing. Task 2 runs it against a missing file and expects a non-zero exit.
3. A page file importing a moved export from the engine package must be rejected at type-check time. Task 3 confirms the generated `sugarscape.d.ts` no longer declares the six names, so `tsc --noEmit` is the guard.
4. Two wasm instances in one Node process (caching-parity needs engine and experiments together) must both initialize. Task 3 keeps that test green with both `initSync` calls.
5. The dev server must still build both packages so `npm run dev` can open an episode. Task 2 wires `wasm:dev` to both dev builds and runs it once.

---

### Task 1: The experiments crate

**Files:**
- Create: `crates/sugarscape-experiments-wasm/Cargo.toml`
- Create: `crates/sugarscape-experiments-wasm/src/lib.rs`
- Move: `crates/sugarscape-wasm/src/experiment_view.rs` → `crates/sugarscape-experiments-wasm/src/experiment_view.rs`
- Move: `crates/sugarscape-wasm/tests/experiment_view_tests.rs` → `crates/sugarscape-experiments-wasm/tests/experiment_view_tests.rs`
- Modify: `crates/sugarscape-wasm/src/lib.rs` (remove lines 4–8 and the burrow block at lines 164–233)
- Modify: `Cargo.toml` (workspace members)

**Interfaces:**
- Consumes: `sugarscape_core::browser_experiments::{catalog, normalize_input, run, episode_json, validate_episode, recorded_results_json, FieldError}`, `sugarscape_core::burrow::{LabConfig, AccessConfig, run_episode, run_access_episode, RunOptions}`, `sugarscape_core::config::FieldError`.
- Produces: the wasm-bindgen exports `experiment_catalog_json() -> Result<String, JsValue>`, `experiment_run_json(input_json: &str) -> Result<String, JsValue>`, `experiment_validate_json(record_json: &str) -> Result<String, JsValue>`, `experiment_recorded_json() -> Result<String, JsValue>`, `burrow_replay_json(config_json: &str, seed: &str, ticks: f64, sample_every: f64) -> Result<String, JsValue>`, `burrow_access_replay_json(config_json: &str, seed: &str, ticks: f64, sample_every: f64) -> Result<String, JsValue>`, from library `sugarscape_experiments_wasm`. Task 2 builds this crate; Task 3 imports these names from `web/src/experiments-pkg/experiments.js`.

- [ ] **Step 1: Create the crate manifest**

```toml
[package]
name = "sugarscape-experiments-wasm"
description = "WASM bindings for the Minds provenance experiments, loaded only by the episodes worker"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
sugarscape-core = { path = "../sugarscape-core" }
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
serde_json.workspace = true

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

- [ ] **Step 2: Add the crate to the workspace**

In the root `Cargo.toml`, change

```toml
members = ["crates/sugarscape-core", "crates/sugarscape-wasm", "crates/sugarscape-cli"]
```

to

```toml
members = [
    "crates/sugarscape-core",
    "crates/sugarscape-wasm",
    "crates/sugarscape-experiments-wasm",
    "crates/sugarscape-cli",
]
```

- [ ] **Step 3: Move the two source files and the test with git**

```bash
mkdir -p crates/sugarscape-experiments-wasm/src crates/sugarscape-experiments-wasm/tests
git mv crates/sugarscape-wasm/src/experiment_view.rs crates/sugarscape-experiments-wasm/src/experiment_view.rs
git mv crates/sugarscape-wasm/tests/experiment_view_tests.rs crates/sugarscape-experiments-wasm/tests/experiment_view_tests.rs
```

`experiment_view.rs` needs no edit: it calls `crate::field_errors`, which Step 4 defines in the new crate.

- [ ] **Step 4: Write the new crate's `src/lib.rs`**

The burrow functions and `checked_burrow_u32` are the current `crates/sugarscape-wasm/src/lib.rs` lines 164–233 verbatim; `field_errors` and `decimal_seed` are copied from that file.

```rust
//! JavaScript-facing exports that reach the Minds provenance cluster
//! (`browser_experiments` and `burrow`). The episodes worker is the only page
//! consumer. Nothing the playground or the sweeps need belongs here: an export
//! added to this crate never loads on the default page, and an export added to
//! `sugarscape-wasm` that touches the cluster pulls the cluster back into the
//! engine bundle. Errors cross the boundary as JSON strings of
//! `[{ field, message }]`.

mod experiment_view;
pub use experiment_view::{
    experiment_catalog_json, experiment_recorded_json, experiment_run_json,
    experiment_validate_json,
};

use sugarscape_core::config::FieldError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn field_errors(errors: Vec<FieldError>) -> JsValue {
    JsValue::from_str(&serde_json::to_string(&errors).expect("errors serialize"))
}

/// Checked standalone excavation replay; serializes the same core record as the native CLI.
#[wasm_bindgen]
pub fn burrow_replay_json(
    config_json: &str,
    seed: &str,
    ticks: f64,
    sample_every: f64,
) -> Result<String, JsValue> {
    let config: sugarscape_core::burrow::LabConfig = serde_json::from_str(config_json)
        .map_err(|error| field_errors(vec![FieldError::new("burrow_config", error.to_string())]))?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let ticks = checked_burrow_u32(ticks, "ticks", 0)?;
    let sample_every = checked_burrow_u32(sample_every, "sample_every", 1)?;
    let episode = sugarscape_core::burrow::run_episode(
        config,
        seed,
        sugarscape_core::burrow::RunOptions {
            ticks,
            sample_every,
        },
    )
    .map_err(field_errors)?;
    Ok(serde_json::to_string(&episode).expect("episode serializes"))
}

/// Checked resource-access replay; returns the complete shared outer record.
#[wasm_bindgen]
pub fn burrow_access_replay_json(
    config_json: &str,
    seed: &str,
    ticks: f64,
    sample_every: f64,
) -> Result<String, JsValue> {
    let config: sugarscape_core::burrow::AccessConfig =
        serde_json::from_str(config_json).map_err(|error| {
            field_errors(vec![FieldError::new(
                "burrow_access_config",
                error.to_string(),
            )])
        })?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let ticks = checked_burrow_u32(ticks, "ticks", 0)?;
    let sample_every = checked_burrow_u32(sample_every, "sample_every", 1)?;
    let episode = sugarscape_core::burrow::run_access_episode(
        config,
        seed,
        sugarscape_core::burrow::RunOptions {
            ticks,
            sample_every,
        },
    )
    .map_err(field_errors)?;
    Ok(serde_json::to_string(&episode).expect("access episode serializes"))
}

fn checked_burrow_u32(value: f64, field: &str, minimum: u32) -> Result<u32, JsValue> {
    if !value.is_finite()
        || value.fract() != 0.0
        || value < f64::from(minimum)
        || value > f64::from(u32::MAX)
    {
        return Err(field_errors(vec![FieldError::new(
            field,
            format!("expected finite integer in {minimum}..={}", u32::MAX),
        )]));
    }
    Ok(value as u32)
}

fn decimal_seed(seed: &str) -> Result<u64, &'static str> {
    if seed.is_empty() || !seed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("expected unsigned decimal u64");
    }
    seed.parse().map_err(|_| "seed exceeds u64")
}

#[cfg(test)]
mod tests {
    #[test]
    fn decimal_seed_is_strict_and_preserves_full_u64_range() {
        assert_eq!(super::decimal_seed("18446744073709551615"), Ok(u64::MAX));
        assert!(super::decimal_seed("18446744073709551616").is_err());
        assert!(super::decimal_seed("").is_err());
        assert!(super::decimal_seed("+1").is_err());
        assert!(super::decimal_seed("1.0").is_err());
    }

    #[test]
    fn burrow_counts_reject_non_integers_and_values_below_minimum() {
        assert!(super::checked_burrow_u32(1.5, "ticks", 0).is_err());
        assert!(super::checked_burrow_u32(f64::NAN, "ticks", 0).is_err());
        assert!(super::checked_burrow_u32(0.0, "sample_every", 1).is_err());
        assert_eq!(super::checked_burrow_u32(0.0, "ticks", 0).unwrap(), 0);
        assert_eq!(
            super::checked_burrow_u32(f64::from(u32::MAX), "ticks", 0).unwrap(),
            u32::MAX
        );
    }
}
```

Before copying the burrow block, confirm the lines with `sed -n 164,233p crates/sugarscape-wasm/src/lib.rs`: it starts at the doc comment `/// Checked standalone excavation replay` and ends with the closing brace of `checked_burrow_u32`. No test in the engine crate references `checked_burrow_u32`, so nothing else moves.

- [ ] **Step 5: Point the moved test at the new library**

In `crates/sugarscape-experiments-wasm/tests/experiment_view_tests.rs`, line 4, change

```rust
use sugarscape_wasm::{
```

to

```rust
use sugarscape_experiments_wasm::{
```

Line 73's `#[path = "../../test-support/spatial_experiments.rs"]` resolves to `crates/test-support/spatial_experiments.rs` from the new location exactly as before; leave it.

- [ ] **Step 6: Remove the moved exports from the engine crate**

In `crates/sugarscape-wasm/src/lib.rs`:

1. Delete lines 4–8:

```rust
mod experiment_view;
pub use experiment_view::{
    experiment_catalog_json, experiment_recorded_json, experiment_run_json,
    experiment_validate_json,
};
```

2. Delete the block from the line `/// Checked standalone excavation replay; serializes the same core record as the native CLI.` through the closing `}` of `fn checked_burrow_u32` (currently lines 164–233), leaving `fn decimal_seed` in place because the engine still uses it.

Afterwards this must print nothing:

```bash
grep -nE "experiment_view|burrow|browser_experiments" crates/sugarscape-wasm/src/lib.rs
```

- [ ] **Step 7: Format, build and test both crates natively**

```bash
cargo fmt --all
cargo test -p sugarscape-experiments-wasm
cargo test -p sugarscape-wasm
cargo clippy --all-targets -- -D warnings
```

Expected: both test runs pass, including the moved `experiment_view_tests` under the new crate; clippy clean. If clippy reports an unused import of `FieldError` or `JsCast` in the engine crate, remove only that import.

- [ ] **Step 8: Run the Node tests of both crates under wasm-pack**

```bash
wasm-pack test --node crates/sugarscape-wasm
wasm-pack test --node crates/sugarscape-experiments-wasm
```

Expected: both pass. The second run exercises the moved file with the key, gate, discovery and spatial reproductions.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock crates/sugarscape-experiments-wasm crates/sugarscape-wasm
git commit -m "refactor(wasm): move provenance-cluster exports into sugarscape-experiments-wasm"
```

---

### Task 2: Two packages, one budget

**Files:**
- Modify: `web/package.json` (scripts)
- Create: `web/scripts/wasm-budget.mjs`
- Modify: `.gitignore` (one line after `web/src/wasm-pkg/`)

**Interfaces:**
- Consumes: the crate from Task 1.
- Produces: `web/src/experiments-pkg/experiments.js`, `experiments_bg.wasm`, `experiments.d.ts` (generated, ignored by git); npm scripts `wasm`, `wasm:dev`, `wasm:engine`, `wasm:experiments`, `wasm:engine:dev`, `wasm:experiments:dev`, `wasm:budget`. Task 3 imports from `./experiments-pkg/experiments.js`.

- [ ] **Step 1: Write the budget script**

`web/scripts/wasm-budget.mjs`:

```js
// Fails the web build when the engine bundle grows past its budget. The spec
// measured 6.58 MB after the split; the headroom is for new model kinds, not
// for the provenance cluster, which belongs in the experiments package.
import { statSync } from 'node:fs';

const budget = 8_000_000;
const path = process.argv[2] ?? new URL('../src/wasm-pkg/sugarscape_bg.wasm', import.meta.url);
const size = statSync(path).size;
if (size > budget) {
  console.error(`engine bundle is ${size} bytes, over the ${budget} byte budget; a cluster export has leaked into sugarscape-wasm`);
  process.exit(1);
}
console.log(`engine bundle ${size} bytes, within the ${budget} byte budget`);
```

- [ ] **Step 2: Check the script's two failure paths before wiring it in**

```bash
cd web
node scripts/wasm-budget.mjs /nonexistent.wasm; echo "exit $?"
head -c 8000001 /dev/zero > /tmp/oversize.wasm && node scripts/wasm-budget.mjs /tmp/oversize.wasm; echo "exit $?"; rm /tmp/oversize.wasm
```

Expected: the first prints an `ENOENT` error and `exit 1`; the second prints the over-budget message and `exit 1`.

- [ ] **Step 3: Replace the scripts block in `web/package.json`**

```json
  "scripts": {
    "pretest": "cargo build --release -p sugarscape-cli --manifest-path ../Cargo.toml",
    "test": "vitest run",
    "wasm:engine": "wasm-pack build ../crates/sugarscape-wasm --target web --release --out-dir ../../web/src/wasm-pkg --out-name sugarscape",
    "wasm:experiments": "wasm-pack build ../crates/sugarscape-experiments-wasm --target web --release --out-dir ../../web/src/experiments-pkg --out-name experiments",
    "wasm:engine:dev": "wasm-pack build ../crates/sugarscape-wasm --target web --dev --out-dir ../../web/src/wasm-pkg --out-name sugarscape",
    "wasm:experiments:dev": "wasm-pack build ../crates/sugarscape-experiments-wasm --target web --dev --out-dir ../../web/src/experiments-pkg --out-name experiments",
    "wasm:budget": "node scripts/wasm-budget.mjs",
    "wasm": "npm run wasm:engine && npm run wasm:experiments && npm run wasm:budget",
    "wasm:dev": "npm run wasm:engine:dev && npm run wasm:experiments:dev",
    "dev": "npm run wasm:dev && vite",
    "build": "npm run wasm && tsc --noEmit && vite build",
    "typecheck": "tsc --noEmit"
  },
```

- [ ] **Step 4: Ignore the generated package**

In `.gitignore`, after the line `web/src/wasm-pkg/`, add:

```
web/src/experiments-pkg/
```

- [ ] **Step 5: Build both release packages and record the sizes**

```bash
cd web && npm run wasm
ls -l src/wasm-pkg/sugarscape_bg.wasm src/experiments-pkg/experiments_bg.wasm
gzip -9 -c src/wasm-pkg/sugarscape_bg.wasm | wc -c
gzip -9 -c src/experiments-pkg/experiments_bg.wasm | wc -c
git status --short
```

Expected: the budget line prints "within"; `sugarscape_bg.wasm` is under 8,000,000 bytes (the spike gave 6,583,629); `git status` shows no files under `web/src/experiments-pkg/`. Write the four numbers into the implementer report; Task 4 copies them into the spec.

- [ ] **Step 6: Run the dev builds once**

```bash
cd web && npm run wasm:dev && ls src/experiments-pkg/experiments.js src/wasm-pkg/sugarscape.js
```

Expected: both files exist. Then rebuild release with `npm run wasm` so Task 3's tests run against release packages.

- [ ] **Step 7: Commit**

```bash
git add web/package.json web/scripts/wasm-budget.mjs .gitignore
git commit -m "build(web): build engine and experiments packages separately with an engine budget"
```

---

### Task 3: The page and the tests use the experiments package

**Files:**
- Modify: `web/src/episodes/worker.ts:2`
- Modify: `web/src/burrow-parity.test.ts:6,8`
- Modify: `web/src/burrow-access-parity.test.ts:6,8`
- Modify: `web/src/episodes/parity.test.ts:6,9`
- Modify: `web/src/episodes/recorded-parity.test.ts:5,8`
- Modify: `web/src/episodes/spatial-parity.test.ts:7,16`
- Modify: `web/src/episodes/gate-comparison.test.ts:7,8`
- Modify: `web/src/episodes/surface-projection.test.ts:3,8`
- Modify: `web/src/episodes/discovery-projection.test.ts:3,8`
- Modify: `web/src/episodes/gate-projection.test.ts:6,7`
- Modify: `web/src/episodes/discovery-parity.test.ts:6,10`
- Modify: `web/src/episodes/gate-parity.test.ts:6,8`
- Modify: `web/src/episodes/caching-parity.test.ts:7,15-16`
- Create: `web/src/bundle-split.test.ts`

**Interfaces:**
- Consumes: `web/src/experiments-pkg/experiments.js` from Task 2, exporting `default` (`init`), `initSync`, and the six functions named in Task 1.
- Produces: nothing new for later tasks.

- [ ] **Step 1: Write the failing bundle-split test**

`web/src/bundle-split.test.ts`:

```ts
// The engine package must stay free of the provenance cluster; the six exports
// that reach it live only in the experiments package the episodes worker loads.
import { describe, expect, it } from 'vitest';
import * as engine from './wasm-pkg/sugarscape.js';
import * as experiments from './experiments-pkg/experiments.js';

const moved = [
  'experiment_catalog_json',
  'experiment_run_json',
  'experiment_validate_json',
  'experiment_recorded_json',
  'burrow_replay_json',
  'burrow_access_replay_json',
] as const;

describe('bundle split', () => {
  for (const name of moved) {
    it(`${name} is exported by the experiments package only`, () => {
      expect(name in engine, `${name} leaked into the engine package`).toBe(false);
      expect(typeof (experiments as Record<string, unknown>)[name]).toBe('function');
    });
  }
  it('the engine package keeps the playground exports', () => {
    for (const name of ['Sim', 'run_point', 'presets_json', 'model_schemas_json']) {
      expect(typeof (engine as Record<string, unknown>)[name], name).toBe('function');
    }
  });
});
```

- [ ] **Step 2: Run it**

```bash
cd web && npx vitest run src/bundle-split.test.ts
```

Expected: PASS already, because Task 1 removed the exports and Task 2 built both packages. If any `moved` name is still `in engine`, Task 1 was incomplete; stop and fix it there. (This test is the regression guard for the future, so it must be committed even though it passes first time.)

- [ ] **Step 3: Switch the episodes worker**

`web/src/episodes/worker.ts` line 2, change

```ts
import init, { experiment_catalog_json, experiment_recorded_json, experiment_run_json, experiment_validate_json } from '../wasm-pkg/sugarscape.js';
```

to

```ts
import init, { experiment_catalog_json, experiment_recorded_json, experiment_run_json, experiment_validate_json } from '../experiments-pkg/experiments.js';
```

- [ ] **Step 4: Switch the eleven single-package tests**

In each file below, change the import path and the `initSync` bytes path. The import path changes `wasm-pkg/sugarscape.js` to `experiments-pkg/experiments.js`; the bytes path changes `wasm-pkg/sugarscape_bg.wasm` to `experiments-pkg/experiments_bg.wasm`. Keep the relative prefix (`./` at `web/src/`, `../` under `web/src/episodes/`) and every imported name as it is.

| File | Import line | Bytes line |
|---|---|---|
| `web/src/burrow-parity.test.ts` | 6 | 8 |
| `web/src/burrow-access-parity.test.ts` | 6 | 8 |
| `web/src/episodes/parity.test.ts` | 6 | 9 |
| `web/src/episodes/recorded-parity.test.ts` | 5 | 8 |
| `web/src/episodes/spatial-parity.test.ts` | 7 | 16 |
| `web/src/episodes/gate-comparison.test.ts` | 7 | 8 |
| `web/src/episodes/surface-projection.test.ts` | 3 | 8 |
| `web/src/episodes/discovery-projection.test.ts` | 3 | 8 |
| `web/src/episodes/gate-projection.test.ts` | 6 | 7 |
| `web/src/episodes/discovery-parity.test.ts` | 6 | 10 |
| `web/src/episodes/gate-parity.test.ts` | 6 | 8 |

For example, `web/src/episodes/gate-parity.test.ts` lines 6 and 8 become:

```ts
import { initSync, experiment_run_json, experiment_validate_json } from '../experiments-pkg/experiments.js';
```

```ts
initSync({ module: readFileSync(new URL('../experiments-pkg/experiments_bg.wasm', import.meta.url)) });
```

`spatial-parity.test.ts` line 115 hashes `wasmBytes` into its evidence record as `wasm_sha256`; after this change that is the experiments bundle's hash, which is the bundle that produced the record. Leave line 115 alone.

Afterwards this must print nothing:

```bash
grep -nE "import .*(experiment_|burrow_).*wasm-pkg" web/src -r
```

- [ ] **Step 5: Switch caching-parity to both packages**

`web/src/episodes/caching-parity.test.ts` line 7 currently imports six names from the engine package. Replace it with two imports:

```ts
import { initSync as initEngine, protection_episode_json, deception_episode_json } from '../wasm-pkg/sugarscape.js';
import { initSync, experiment_catalog_json, experiment_run_json, experiment_validate_json } from '../experiments-pkg/experiments.js';
```

Replace lines 15–16

```ts
const wasmBytes = readFileSync(new URL(`file://${root}/web/src/wasm-pkg/sugarscape_bg.wasm`));
initSync({ module: wasmBytes });
```

with

```ts
const wasmBytes = readFileSync(new URL(`file://${root}/web/src/experiments-pkg/experiments_bg.wasm`));
initSync({ module: wasmBytes });
initEngine({ module: readFileSync(new URL(`file://${root}/web/src/wasm-pkg/sugarscape_bg.wasm`)) });
```

Check with `grep -n wasmBytes web/src/episodes/caching-parity.test.ts` that every remaining use of `wasmBytes` is for the experiment exports' evidence; if one hashes bytes alongside a `protection_episode_json` or `deception_episode_json` result, give that use the engine bytes instead.

- [ ] **Step 6: Confirm the type-level guard**

```bash
cd web && grep -c "experiment_run_json\|burrow_replay_json" src/wasm-pkg/sugarscape.d.ts; grep -c "experiment_run_json" src/experiments-pkg/experiments.d.ts
```

Expected: `0` then `1` or more. Then:

```bash
cd web && npx tsc --noEmit
```

Expected: clean. A stale import of a moved name from the engine package would fail here.

- [ ] **Step 7: Run the whole web suite**

```bash
cd web && npm test
```

Expected: all files pass, including the twelve switched files and `bundle-split.test.ts`. The `pretest` step builds the native CLI the parity tests compare against.

- [ ] **Step 8: Commit**

```bash
git add web/src/episodes/worker.ts web/src/bundle-split.test.ts web/src/*.test.ts web/src/episodes/*.test.ts
git commit -m "web: load the experiments package only in the episodes worker and its tests"
```

---

### Task 4: CI, README, built-site check, and the measured result

**Files:**
- Modify: `.github/workflows/ci.yml` (wasm job)
- Modify: `README.md:3763-3768` (Tests section)
- Modify: `docs/superpowers/specs/2026-10-12-wasm-bundle-split-design.md` (append a Measured section)

**Interfaces:**
- Consumes: the sizes recorded in Task 2's report.
- Produces: nothing.

- [ ] **Step 1: Add the second Node test to the CI wasm job**

In `.github/workflows/ci.yml`, after the line

```yaml
      - run: wasm-pack test --node crates/sugarscape-wasm
```

add

```yaml
      - run: wasm-pack test --node crates/sugarscape-experiments-wasm
```

No other workflow change: the `web` job and `pages.yml` run `npm run build`, which now builds both packages and runs the budget.

- [ ] **Step 2: Update the README's Tests section**

Replace the block

```
    cargo test                                                      # whole workspace, incl. the CLI
    cargo test -p sugarscape-core                                   # unit + property tests
    cargo test -p sugarscape-core --release --test book -- --ignored # book reproductions
    wasm-pack test --node crates/sugarscape-wasm                    # bindings
    cd web && npm run build && npm test                             # front-end logic (build first: the engine tests load the real WASM package)
```

with

```
    cargo test                                                      # whole workspace, incl. the CLI
    cargo test -p sugarscape-core                                   # unit + property tests
    cargo test -p sugarscape-core --release --test book -- --ignored # book reproductions
    wasm-pack test --node crates/sugarscape-wasm                    # engine bindings
    wasm-pack test --node crates/sugarscape-experiments-wasm        # episode (provenance) bindings
    cd web && npm run build && npm test                             # front-end logic (build first: the tests load the real WASM packages)

The web build produces two WebAssembly packages: the engine (`web/src/wasm-pkg/`,
the playground and sweeps, budgeted at 8 MB raw) and the experiments package
(`web/src/experiments-pkg/`), which only the episodes worker loads.
```

- [ ] **Step 3: Build the site and check the asset split**

```bash
cd web && npm run build
ls dist/assets | grep -E "_bg-.*\.wasm$"
grep -l "experiments_bg-" dist/assets/*.js
grep -L "experiments_bg-" dist/assets/index-*.js
```

Expected: two wasm assets, `sugarscape_bg-<hash>.wasm` and `experiments_bg-<hash>.wasm`; exactly one JS chunk references `experiments_bg-`, the episodes worker chunk (its name contains `worker`); the `index-*.js` chunk is listed by `grep -L`, meaning it does not reference the experiments wasm. If `index-*.js` references it, the worker import was bundled into the page: stop and report.

- [ ] **Step 4: Append the measured result to the spec**

Append to `docs/superpowers/specs/2026-10-12-wasm-bundle-split-design.md`, filling in Task 2's numbers:

```markdown
## Measured after implementation

| Bundle | Raw bytes | Gzipped bytes |
|---|---|---|
| Engine, `sugarscape_bg.wasm` | <raw> | <gz> |
| Experiments, `experiments_bg.wasm` | <raw> | <gz> |
| Shipped before the split | 23,623,911 | 4,155,336 |

The engine budget is 8,000,000 raw bytes, checked by `web/scripts/wasm-budget.mjs`.
```

- [ ] **Step 5: Run the full native suite once on the branch**

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

Expected: all green. Core's tests are unchanged, which this run confirms.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/ci.yml README.md docs/superpowers/specs/2026-10-12-wasm-bundle-split-design.md
git commit -m "ci,docs: test the experiments crate in CI and record the bundle sizes"
```
