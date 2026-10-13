# WASM bundle split: engine page and experiments bundle

Date: 2026-10-12. Status: design for review.

## Problem

The page ships one WebAssembly bundle, `web/src/wasm-pkg/sugarscape_bg.wasm`, at
23.6 MB raw and 4.16 MB gzipped. Measurement (twiggy on a named release build,
2026-10-12) attributes most of it to the Minds provenance cluster in
`sugarscape-core`: `browser_experiments`, `deduction`, `surface_discovery`,
`gate_assistance`, `key_delegation` (merged 2026-10-12, after the
measurement), `shared_surface`, `active_surface`, `burrow` and `foraging`.
The cluster embeds about 14 MB of JSON fixtures with `include_str!`
and `include_bytes!` and compiles to roughly a third of the code. The engine and
all 33 reproduction kinds, including `minds`, `hoard` and `image`, account for
the rest.

Every visitor downloads the cluster, but the page reaches it through exactly one
import: the episodes worker (`web/src/episodes/worker.ts`), created only when
someone opens an episode. The two burrow replay exports have no page consumer at
all; only vitest calls them.

## Goal

The default page load carries the engine and the reproduction kinds only. The
provenance cluster loads only inside the episodes worker. Nothing is deleted,
no model fingerprint or golden changes, and the episodes UI and the published
studies behave exactly as today.

## What the spike showed

A throwaway build of `sugarscape-wasm` with only the four `experiment_*`
exports and the two `burrow_*` exports removed, and no change to
`sugarscape-core`, produced a bundle of 6.58 MB raw and 2.10 MB gzipped. The
release profile's link-time optimization already discards the cluster once no
export references it. A Cargo feature gate on the cluster is therefore not
needed for size, and it would cost something real: the caching-parity vitest
authenticates `crates/sugarscape-core/src/lib.rs` as a frozen historical file
plus an exact appended string, so any `#[cfg]` attribute placed mid-file would
break that test and force a new authentication scheme. This design leaves core
untouched.

## Design

### Crates

`crates/sugarscape-wasm` keeps every engine export: `Sim`, sweeps and
`run_point`, schemas and presets, the protection, deception and behavior-tree
episode exports, and the helpers they share. It loses `mod experiment_view`,
its `pub use` of the four experiment exports, `burrow_replay_json`,
`burrow_access_replay_json` and `checked_burrow_u32`. Its file
`src/experiment_view.rs` and its test `tests/experiment_view_tests.rs` move out.

A new crate, `crates/sugarscape-experiments-wasm` (library name
`sugarscape_experiments_wasm`, `crate-type = ["cdylib", "rlib"]`), holds:

- `src/lib.rs`: the `#[wasm_bindgen(start)]` panic-hook setup, private copies of
  the three small helpers `field_errors`, `decimal_seed` and
  `checked_burrow_u32`, the two burrow exports moved verbatim, and
  `mod experiment_view` with its `pub use`.
- `src/experiment_view.rs`: moved unchanged apart from `crate::` paths.
- `tests/experiment_view_tests.rs`: moved, importing from
  `sugarscape_experiments_wasm`; its `#[path]` include of
  `crates/test-support/spatial_experiments.rs` resolves the same way.

The new crate depends on `sugarscape-core` with default features, exactly as
the engine crate does, plus `wasm-bindgen`, `console_error_panic_hook` and
`serde_json`, with `wasm-bindgen-test` as its dev-dependency. The moved code
uses nothing else. It must not depend on
`sugarscape-wasm`: an rlib dependency would carry every engine export into the
experiments bundle. The crate is its own Cargo workspace, as `survey/` is,
with its own committed lockfile and the root profiles repeated; it is not a
member of the root workspace (see the implementation note at the end).

The CLI crate and the survey crate already use the cluster natively and are
unchanged.

### Packages and the page

wasm-pack builds two packages:

| Package | Crate | Out dir | Out name | Files |
|---|---|---|---|---|
| engine | `sugarscape-wasm` | `web/src/wasm-pkg/` | `sugarscape` | `sugarscape.js`, `sugarscape_bg.wasm` |
| experiments | `sugarscape-experiments-wasm` | `web/src/experiments-pkg/` | `experiments` | `experiments.js`, `experiments_bg.wasm` |

`.gitignore` adds `web/src/experiments-pkg/` beside `web/src/wasm-pkg/`.

`web/package.json` scripts become:

- `wasm:engine`, `wasm:experiments`: the two release builds.
- `wasm:engine:dev`, `wasm:experiments:dev`: the two `--dev` builds.
- `wasm`: both release builds, then `wasm:budget`.
- `wasm:dev`: both dev builds.
- `wasm:budget`: fails when `web/src/wasm-pkg/sugarscape_bg.wasm` exceeds
  8,000,000 bytes. The spike measured 6.58 MB; the budget leaves room for new
  kinds and trips if a cluster reference ever creeps back into the engine
  crate. A one-line Node script in `web/scripts/` does the check.
- `dev` and `build` are unchanged in shape and pick up the new `wasm` and
  `wasm:dev`.

Page changes:

- `web/src/episodes/worker.ts` imports `init` and the four experiment exports
  from `../experiments-pkg/experiments.js` instead of the engine package. It
  is the only non-test page file that changes. The worker already calls
  `init()` lazily inside its first request, so the experiments bundle is
  fetched only when an episode runs.
- Vite needs no configuration change: the worker is already an ES-module
  worker and the generated glue finds its `.wasm` by `import.meta.url`. The
  build emits `experiments_bg-<hash>.wasm` as a second asset.

### Tests

- The twelve vitest files that import `experiment_*` or `burrow_*` switch
  their import and their `initSync` bytes to the experiments package.
  `caching-parity.test.ts` also calls `protection_episode_json` and
  `deception_episode_json`, which stay in the engine package, so it
  initializes both packages. Two instances in one Node process are
  independent and fine.
- A new vitest, `web/src/bundle-split.test.ts`, asserts that the engine
  package's module namespace has no `experiment_catalog_json`,
  `experiment_run_json`, `experiment_validate_json`,
  `experiment_recorded_json`, `burrow_replay_json` or
  `burrow_access_replay_json`, and that the experiments package has all six.
  TypeScript's `tsc --noEmit` separately rejects any page import of those
  names from the engine package, because its generated `.d.ts` no longer
  declares them.
- The root `cargo test --workspace` and `cargo clippy --all-targets` do not
  reach the new crate. CI runs `cargo fmt --check`, `cargo clippy` and
  `cargo test` for it by manifest path, and the `wasm` job adds
  `wasm-pack test --node crates/sugarscape-experiments-wasm`.
- Core's tests, the CLI's tests, the survey, the war-study literal inventory
  and the caching-parity authentication of `lib.rs` are untouched because
  core is untouched. The war-study inventory audits only
  `crates/sugarscape-core`; the moved files live elsewhere.

### CI and deployment

- `.github/workflows/ci.yml`: the `lint` and `test` jobs gain manifest-path
  steps for the crate, the `wasm` job gains the second `wasm-pack test --node`
  line, and the `rust-cache` steps of the jobs that build the crate (and in
  `pages.yml`) list its target directory. The `web` job and `pages.yml` run
  `npm run build`, which now builds both packages and checks the budget.
- Expected results after the change, to be recorded in the plan's final
  measurement step: engine bundle about 6.6 MB raw and 2.1 MB gzipped;
  experiments bundle whatever the cluster plus the engine code it references
  comes to, measured, not estimated. The sum may exceed today's single bundle
  slightly because shared code is compiled twice; that cost falls only on
  episode runs.

### Documentation

- README "Tests" section: add
  `wasm-pack test --node crates/sugarscape-experiments-wasm` and say that the
  web build produces two packages, the second loaded only by the episodes
  worker.
- The crate's `lib.rs` doc comment names what belongs there: exports that
  reach the provenance cluster, and nothing the playground or sweeps need.

## Out of scope

- A Cargo feature gating the cluster in core. Deferred for the reason above.
  If CI isolation of the cluster is still wanted later, it can be done with
  test selection or a core split, and the engine bundle's budget check will
  already be in place.
- Fetching fixtures at runtime instead of embedding them. The cluster's
  compiled-identity scheme assumes its reference bytes are compiled in.
- Renaming the repository and the queue pruning agreed in principle on
  2026-10-12. Each is its own piece of work after this one.

## Risks

- A future engine export that touches the cluster silently pulls it back into
  the engine bundle. The budget check and the export-absence vitest catch
  this at build and test time.
- Dev-mode builds (`--dev`) skip link-time optimization, so the dev engine
  bundle may stay larger than the release one. Only the release bundle, which
  is what deploys, is budgeted.
- Two wasm-pack builds lengthen `npm run build` and the Pages deploy. Because
  the crate is its own workspace with its own target directory, the second
  build compiles core again rather than reusing the engine build's artifacts;
  measured costs are in the implementation note below.

## Measured after implementation

Release builds with wasm-pack 0.15.0 and wasm-opt, 2026-10-12.

| Bundle | Raw bytes | Gzipped bytes |
|---|---|---|
| Engine, `sugarscape_bg.wasm` (every page load) | 6,581,596 | 2,098,853 |
| Experiments, `experiments_bg.wasm` (episodes worker only) | 36,437,274 | 3,108,125 |
| Single bundle before the split, built from the same base commit | 41,512,040 | 4,692,428 |
| Single bundle as shipped when this design was written | 23,623,911 | 4,155,336 |

The last row was a local artifact built before key delegation merged; key
delegation added about 8.5 MB of embedded fixtures, which is why the
like-for-like row above it is larger. The experiments bundle is almost
entirely data: four data segments hold about 32 MB of embedded JSON fixtures,
and its code is smaller than the engine's. The engine budget is 8,000,000 raw
bytes, checked by `web/scripts/wasm-budget.mjs`.

Implementation note: core compiles the root `Cargo.toml` and `Cargo.lock` in
as reviewed provenance inputs (`key_delegation::source_inputs`), and the key
registration metadata pins their digests, so adding a workspace member breaks
core's own tests. The experiments crate is therefore its own Cargo workspace,
as `survey/` already is: an empty `[workspace]` table, a committed lockfile of
its own, and the root profiles repeated. The root manifest and lockfile are
byte-identical to those before the split, and CI runs fmt, clippy and the
tests for the crate by manifest path.

Measured cost of the separate target directory, cold, on the development
laptop: `cargo clippy` 27 s, `cargo test` 2 min 11 s (core in the test
profile), `wasm-pack test --node` 3 min 23 s, and the release
`wasm-pack build` 1 min 03 s, the same as the engine's. In CI, `rust-cache`
keeps these warm until core changes. Pointing the crate at the root target
directory with `[build] target-dir` might let cargo reuse core's artifacts
across the two workspace roots; that is untested and left as a follow-up to
measure.
