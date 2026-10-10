# Informed excavation engineering verification

**Completed:** 2026-10-10. **Corrected runtime:** `56f711042d1f61914f384c63024382d076e8a73c`, tree `7b70a84e309aaaff73446366d12b2cf8dbe292cf`.

The user approved the [specification](../superpowers/specs/2026-10-09-foraging-informed-excavation-design.md) and [implementation plan](../superpowers/plans/2026-10-09-foraging-informed-excavation.md). All four subagent-driven stages received independent spec/quality review. The whole-plan review found one Important interaction, F1; one consolidated correction passed the final scoped review with no residual Critical, Important or Minor finding. Programme status, completed tracker and all plan-owned evidence are preserved before retiring only this plan's tracker/ignored workspace. At archival, branch `foraging-5-registration` and its linked worktree were retained; their subsequent verified local merge and cleanup are recorded in the [integration record](2026-10-10-foraging-integration.md).

This is engineering acceptance of an explicitly versioned controller, not a revised scientific comparison. The original F5 archive and report bytes are preserved.

## Result and retained boundary

`ExcavationPolicy::InformedCostV1` permits an empty informed Departing worker to compare a one-cell opening before exhausting open routes/frontiers. Three shared private fields cover known origin routes, known outlet routes, and optimistic site continuation. Both strict geometric improvement and full cost improvement are required; unavailable references are handled explicitly. Cost includes approach travel, Dig, both known spoil legs, disposal, entry and optimistic continuation. Other remembered solids block continuation. Paired minimum ties use stable Pos ordering and seeded selection, including singleton draws.

Loaded jobs, terminal arrivals, traffic holds and original search priorities retain precedence. Each eligible opportunity reconsiders; no tunnel commitment, peer map sharing, role allocation or relay is introduced. Cost rejection retains ordinary pursuit and exploration-first fallback; proactive Dig accounting distinguishes that fallback.

Existing constructor/runner and traffic APIs retain `exploration_first_v1`, with complete old episode bytes, work/computation counts and next RNG preserved. `new_with_policies`/`run_with_policies` select traffic and excavation independently. The new serialize-only `foraging-construction-policy-episode-v1` wrapper labels both policies and records base/traffic/excavation frames under a combined 64 MiB compact-byte cap. Diagnostic views are observational; historical assessment records do not require their old face still to be solid.

## Review and correction chronology

| Stage | Runtime commit | Review and verification |
| --- | --- | --- |
| 1: records/private estimator and pre-edit capture | `50aeb77` | Spec/quality approved, no findings; 211 construction units, six integration, format/Clippy. |
| 2: paid controller integration | `7b5ba0a` | Spec/quality approved, no findings; 239 construction units, six integration, format/Clippy. |
| 3: views/labeled recording | `b55b307`, corrected by `02d0cfe` | Behavior/spec approved; duplicated generic byte helper found and replaced with the existing shared helper. Scoped review closed it; 260 construction units and format/Clippy passed, with earlier six integration tests retained. |
| 4: privacy/public rollback/F5 acceptance | `621d39b` | Spec/quality approved; 13 supplemental tests initially GREEN, full workspace3266/0/105 and survey 436/0/0, native and saved compatibility passed. Superseded by F1 correction. |
| Final F1 correction | `56f7110` | Exact compiled behavioral RED, successful hold/public-continuation GREEN, fresh full gates/native/saved identity and scoped review approval. |

Task 1's two invalid geometry fixture preparations and Task 2's distinct supplemental fixture assumptions are retained as unexpected fixture failures, not feature RED. Task 3's preliminary formatter/test overlap is preserved and superseded by stable post-format gates. The final regression's one import collision is separately labeled scaffold failure; its next compiled run reproduces F1 before correction. No failure evidence was erased, test disabled, hook bypassed or dependency added.

### F1: peer opens selected face before a paid hold

The whole-plan reviewer confirmed a legal paid 7x10 U fixture: site (4,4), face (3,4), approach (2,4), approach/spoil/continuation moves1/1/1, pre-opening optimistic distance10, cost7 versus reference11. Two blocked selected waits precede yielding to (3,3). A later worker digs the face in that same ascending-ID tick. On the next opportunity fresh learning changes the selected face to KnownOpen, but the old hold path retains the active face. Public `World::step` repeatedly rejects `tick[44].agents[0]` for the active-face invariant and rolls back.

The eight-line production correction retires an invalid active face only during an informed-cost nonterminal hold. It keeps a still-valid face, the paid hold, goal/cargo/phase, historical assessment and excavation counts; no new observation, route computation, assessment, replacement selection or draw occurs. The invariant remains unchanged. The regression fails before the fix, then proves exact paid/learning deltas, zero hold draws, retained valid face and twenty subsequent valid public ticks. Legacy behavior is untouched.

Whole review SHA256: `d6733ce563ea5a112bdf9d30d459681686f5626f7b1fc1f859b1ac1b8880b70c`. Final scoped review SHA256: `43919948da2d6d508a846f36ecb79897372ebf213a16f0cf3f2b547fc8ea547d`. Corrective report SHA256: `786ca7a4bbe1ae732a68c5eb13ca8f0e84046b38f400ab47f900b3fd833aa781`. The embedded original reproduction and its extracted Rust block are archived with those reports; the temporary diagnostic harness is not required to preserve the finding.

## Fresh corrected-tree gates

All commands ran in `/Users/nathan/.config/superpowers/worktrees/SugarScape/foraging-5-registration`. Cargo used `CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target` and normal shared locks. Complete command/cwd/UTC timing/exit/output and before/after source receipts are archived.

| Command | Result | Duration |
| --- | --- | ---: |
| `cargo test --workspace --locked` | 3,267 passed, 0 failed, 105 existing ignored; 54 summaries | 837.258s |
| `cargo test --manifest-path survey/Cargo.toml --locked` | 436 passed, 0 failed, 0 ignored; seven summaries | 397.128s |
| `cargo fmt --all --check` | exit0 | 16.811s |
| `cargo fmt --manifest-path survey/Cargo.toml --all -- --check` | exit0 | 17.522s |
| `cargo clippy --locked -p sugarscape-core --all-targets -- -D warnings` | exit0, no warnings | 48.666s |
| `cargo clippy --manifest-path survey/Cargo.toml --locked --all-targets -- -D warnings` | exit0, no warnings | 28.804s |

Core unit totals are 2,979 passed plus six ignored, 2,985 cases; construction contains 274 passing unit tests. Ignored cases are not counted as passing. The focused final construction group passed 274/0/0 before the full gates. The remaining five independent gates were started while the existing workspace gate continued; the owned scheduler verified complete identical command/cwd/source/log receipts instead of running duplicates. No source changed during gates, no running suite was cancelled, and all completed primary outputs remain preserved.

## Frozen inputs and native binding

All 2,737 frozen checkout hashes/sizes and 855 selected native inputs match before/after gates, commit and saved analysis. All 278 literal includes and seven literal path-module inputs are audited. The existing cfg(test)-only protocol include stays outside the native source selector and is separately frozen; the existing generated OUT_DIR identity is checked in the native phase.

Checkout/committed/generated/embedded source identity is `f8f562754b798dcb279b74b61d3969bcf52e7531a77358e9e013c3a8fa889106`. Both generated constants are 107 bytes, SHA256 `70e0c196c24936af1923a934d5d9602d7446aec832627aa442c8a1f2628d53cf`. The clean tested runtime built with `cargo build --manifest-path survey/Cargo.toml --locked --bin survey`; all 855 selected committed blobs equal checkout bytes.

Private collector: `/tmp/sugarscape-foraging-informed-excavation-20261010-final-fix/collector-final`, 102,226,328 bytes, SHA256 `26b5a7076af3daeb497313ed7ebf366fd19af1a9756ee1fcd060befcc7a82e4c`. Exclusive creation and embedded identity checking protect the analyzed copy from later shared-target replacement. Completion environment: Rust/Cargo 1.98.1, host `aarch64-apple-darwin`; the retained native binary is arm64. No cross-platform identity is asserted.

The full pre-edit traffic wrapper fixture is 38,285 bytes, SHA256 `9956760d45b3955f09bd92fba5e593fabd3fb2d5715d460598701530b5366217`, next RNG `15278877981002486553`. The earlier ordinary fixture is 36,147 bytes, SHA256 `a9856df02addb8c008c0b9cbb4a6314128e00048a9f17fb08641f807962bc5c8`, with the same next RNG. Neither was regenerated after production edits; both complete regressions pass.

## Retained scientific archive compatibility

Only saved `--analyze` commands were run with the new private collector. Two fresh report pairs live in the final-fix root's `reanalysis-a` and `reanalysis-b`. Both match each other and both retained original report pairs exactly:

| File | Bytes | SHA256 |
| --- | ---: | --- |
| `analysis.json` | 13,976,411 | `7bba70c19802bb7b97abe69b84c592fd869bd1a09132912eabf508ab657ce527` |
| `results.md` | 11,864,047 | `2903e4c952e7974a13f17dcd209a0f70b5d9e7086b546f3756e50fc343591c46` |

The original index at `/Users/nathan/Projects/ndouglas/SugarScape/survey/out/foraging-shortcuts/2026-10-09-registered-600/index.json` remains SHA256 `faaa5acbed1397f3451cd670071b97d66fbcc39b1870308e6dc8f2dc97104fa4`; all 600 original raw hashes/sizes match, total 66,473,358 bytes. F5 production still calls ordinary `core::run`. Three immutable manifests, protocol, both historical construction report golden pairs and the cached seed7 legacy comparison remain exact. No scientific worlds, tuning, merge or push accompanied this increment.

The older `621d39b` evidence/native/reanalyses remain at `/tmp/sugarscape-foraging-informed-excavation-20261009/` and in the evidence archive as superseded proof. The correction independently verified all 172 superseded file identities and 104 new evidence/output identities. Parent completion documentation changes only prose/status and removes the archived completed tracker; selected runtime inputs remain the verified source identity above.

## Chronological rulings and rework costs

These are all eleven `Ruling:` entries from the plan ledger, in order. The final message also exposes them; archived `rulings.json` preserves their original wording.

1. Use `--lib` for focused construction units: filters selected no unrelated integration cases; retain explicit integration/all-target/final full gates. If wrong, missed interaction coverage costs rework.
2. Trust the existing private selector for SelectedTravel direction; validate current record/local action/traffic effect without a second BFS. If wrong, malformed internal Decisions need a stronger validation/certificate seam.
3. Enforce chronology/unique endpoints in the public runner; private aligned append accepts repeated/decreasing synthetic triples. If wrong, future internal callers need a chronology guard.
4. Review the entire new increment from approved-plan HEAD `1cb6184`, preserving completed older reviews and checking concrete interactions/compatibility. If wrong, an older cross-increment defect requires later rework.
5. D1: scientific efficacy/tuning is a separate authorized registration/campaign. If wrong, scientific design/evaluation work remains; no current efficacy claim is made.
6. D2: multi-cell planning, collective benefit, roles/relay, shared maps, backfilling and widening remain excluded. If wrong, those need separate design/implementation.
7. D3: portability and universal fairness/deadlock freedom remain unpromised; valid-history invariant failure F1 stays in scope. If wrong, broader portability/liveness work remains; F1 was corrected regardless.
8. D4: computation/enumeration and serialized byte bounds are verified, not absolute timing/RSS guarantees. If wrong, further measurement/bounds are needed.
9. D5: older unrelated F5/traffic behavior is not regraded wholesale; concrete new interactions and retained compatibility are reviewed. If wrong, older defects need cross-increment rework.
10. D6: final parent documents and retirement remain required after technical approval. If wrong, documentation/archival rework is needed before completion; those steps are now carried out.
11. Retire the freshly invalidated active face only in informed-cost hold, preserving paid hold/job/history/counters and zero extra work/draws; do not alter estimator/traffic rules or legacy behavior. If wrong, target reconciliation/scope needs integration rework. Paid intersection RED/GREEN and fresh complete gates cover this ruling.

## Evidence preservation and limits

This plan's complete ledger/briefs/reports/review packages, all old/fresh gate logs and source receipts, reproduction, original raw/report hash inventories, completed root tracker and all rulings are preserved in `/tmp/sugarscape-foraging-informed-excavation-20261010-final-fix/sdd/`, with the adjacent `sdd-evidence-hashes.json` inventory. The verified archive contains 289 files/23,057,460 bytes. Its adjacent inventory SHA256 is `7dc1d7a936c43fea99a716aec4cb8c9e7456238bd1d6fcdaec8ee2333681f6fd`; the completed tracker SHA256 is `d4a7aa947823e3f97ce18ccc3440c4735fd2c2aacc5b0fa46325cb6bf0e456e1`. Only this plan-owned ignored workspace and completed root tracker are retired after hash verification. The pinned collectors, original records, both sets of reanalyses, other plan archives and other worktrees remain untouched.

The approved limits remain: optimistic unknown continuation can fail; one-cell lookahead can miss multi-cell tunnels; conservative spoil charges can reject useful digs; reassessment can switch ties. No universal liveness, biological calibration, revised scientific efficacy, absolute runtime/RSS or cross-platform replay claim follows. Dedicated roles and relay remain future increments. The verified implementation and retained branch/worktree are ready for a later authorized integration decision.
