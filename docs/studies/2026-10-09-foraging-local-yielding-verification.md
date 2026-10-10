# Private local-yielding verification — 2026-10-09

The user approved the [traffic-first specification](../superpowers/specs/2026-10-09-foraging-local-yielding-design.md) and [implementation plan](../superpowers/plans/2026-10-09-foraging-local-yielding.md). Runtime `a45d2f6832ebc3c62c9c8a63ec35705e5ca4c666` implements explicit `shortest_only_v1` and `local_yielding_v1` policies; the original constructor/runner/F5 collector retain the baseline. Revised travel allows a seeded 0.25 yielding attempt only after two paid blocked waits, then one paid hold. Fresh private/current-cardinal information, cargo ownership, capacity two, one paid action per opportunity and whole-tick State/RNG rollback are preserved.

The opposing-queue engineering fixture proves legal passage in both directions with exact third/fourth tick draw counts 7/6, compared with continued baseline blocking. All five travel modes preserve cargo and intent. Defensive terminal precedence, paused pickup, private-information perturbations, isolated counter bounds, and actual public-step late-worker failure after prior seeded draws are covered. This does not establish universal fairness/deadlock freedom or revised scientific delivery efficacy. Earlier informed excavation remains the next separate design.

New public APIs are `World::new_with_traffic`, `traffic_summary`, `traffic_snapshot` and `run_with_traffic`; the separate `foraging-traffic-episode-v1` wrapper carries policy identity and diagnostics without changing the base episode wire. Recording enforces checked combined compact base/traffic snapshot bytes under 64 MiB; sampled snapshots are not restoration or complete controller trajectories.

## Final verification

- `cargo test --workspace --locked`: 3184 passed, zero failed, 105 existing ignored.
- `cargo test --manifest-path survey/Cargo.toml --locked`: 435 passed, zero failed/ignored.
- Both formatting checks and both core/survey all-target Clippy checks with warnings denied: exit zero.
- Final frozen mapping: 2721 files, SHA-256 `2630ad9d47aebe50cc677de399a8c07b39157f2f2095e3236e7cd6ec9042bc4a`.
- Selected native inputs: 842; checkout/committed/generated/embedded source fingerprint `f6ce7c03248c3ccdd63b7405a209c722a5a4b768315e9a629d7fdf300c9be2c4`.
- Literal audit: 278 occurrences; zero unresolved/unbound production inputs. The unchanged protocol Markdown include is a documented cfg(test)-only exception, hashed in the full test freeze and separately bound by protocol preflight.
- Pinned native executable SHA-256 `e8fc4edc3a38f2a43635405e67e211bfa5cd746b6f0bf0fa453e1bd1f0373ec7`.
- Original pre-change full engineering episode/RNG fixture SHA-256 `a9856df02addb8c008c0b9cbb4a6314128e00048a9f17fb08641f807962bc5c8`; next RNG word `15278877981002486553` remains exact.

Final gates ran after the narrow public-step fault test was strengthened to worker 3, proving earlier seeded RNG consumption. Earlier complete F0 gate receipts are retained as superseded evidence, not substituted for final F1 source gates. No production correction was needed during acceptance; fixture expectation/setup corrections and API versus compiled assertion REDs are disclosed in task reports. Passing supplemental acceptance assertions are not described as behavioral RED.

## F5 preservation

All three manifest identities and both locks remain unchanged. Four historical 30-record Construction report fixture goldens pass; these are fixture compatibility evidence, not an audit of an independently supplied production archive. Existing 600-episode scientific index SHA-256 `faaa5acbed1397f3451cd670071b97d66fbcc39b1870308e6dc8f2dc97104fa4` and all raw sizes/hashes remain exact (66473358 raw bytes). Two saved-only analyses using the verified new executable reproduce original JSON SHA-256 `7bba70c19802bb7b97abe69b84c592fd869bd1a09132912eabf508ab657ce527` and Markdown SHA-256 `2903e4c952e7974a13f17dcd209a0f70b5d9e7086b546f3756e50fc343591c46`. Original provenance remains runtime/protocol `a129f06` and original collector SHA `6c120a2a82fbc17d31aaef7d60d6e1821a41f7e7f708e9cb590e064ca356040c`; it is never relabeled as new-policy evidence. No new scientific worlds or revised scientific campaign ran.

## Reviews and archival

All four implementation tasks have independent spec/quality approval with zero findings. The final whole-plan architectural review also approved spec/quality with zero Critical, Important or Minor findings. It independently verified final gate/source/native/archive identities. No correction wave or deferred finding remained.

This plan's complete ignored evidence is hash-preserved at `/tmp/sugarscape-foraging-local-yielding-20261009/sdd/` before removing only its completed tracker/workspace. The pinned collector and both new saved analysis directories stay in that parent directory. Original scientific data and verified preservation copy remain at their existing locations. Local evidence is not a remote backup. The approved plan retains branch `foraging-5-registration` and its worktree; no merge or push accompanies implementation.

Task 1's inventory includes a parent-owned future Task 2 brief that later received append-only handoff/ruling text. The exact original 23492-byte version is retained as `task-2-brief-at-task1.md`, SHA-256 `c3e9ed9800260c26af699d5cbaf65ca3325452d217eb30171c70aeb254c98cca`; `task-1-evidence-version-map.json` records lineage. The original inventory is not rewritten.

## Rulings I made (chronological)

1. Ruling: Terminal pending-hold precedence tests may overlay internally consistent traffic memory on a destination reached by legal physical transactions, explicitly labeling it a defensive synthetic fixture — under current private goal routing, a free goal neighbor is an ordinary shortest step rather than a yielding candidate, but the approved spec still requires terminal precedence — if wrong, rework fixture reachability coverage; production order and paid physical/cargo rules remain unchanged.
2. Ruling: Interpret Task1's "unknown-goal checks fail contextually" as invalid/out-of-bounds coordinates, not valid Unknown/Solid/empty goal sets — the unchanged goal_equality_and_absent_open_goals_use_no_draws test and binding baseline-preservation spec require those sets to return Unreachable with original counts/no draws — if wrong, rework error-boundary coverage; preserve current valid-label semantics.
3. Ruling: Allow Task2 to gate the unchanged private legacy route_step adapter with cfg(test) after all production routes move to select_route/plan_route — parent call-site search confirms only navigation tests still consume it, while unified production selection records diagnostics without duplicating legacy branching — if wrong, restore a production adapter for a newly identified private consumer; public API/old behavior remain unchanged.
4. Ruling: Retain the preexisting cfg(test)-only F5 protocol Markdown include as an explicit native-selector exception and hash it in the full test freeze, rather than adding it to native source identity — parent verified cfg(test) reachability and prior approved audit; Task4 permits only genuinely new literal inputs, while protocol bytes are separately bound — if wrong, extend native selection if this input becomes production-reachable and rebuild/reverify source identity.
5. Ruling: Strengthen the public World::step fault to worker3 and prove scratch seeded RNG advanced before the late error, retaining old gates then refreezing/reverifying the narrow test change — the worker1 failure follows a draw-free hold and would not catch mistaken production RNG borrowing — if wrong, this costs extra verification time; no production policy or scientific inputs change.
6. Ruling: Leave revised scientific efficacy unjudged — no new policy-bound scientific comparison was authorized or run — if wrong, a separately registered campaign is needed before making efficacy claims.
7. Ruling: Keep universal fairness/deadlock freedom outside the guarantee — the approved local stochastic policy needs free capacity and retains finite-horizon/scheduler limits — if wrong, queues may remain blocked and require another controller design.
8. Ruling: Keep earlier informed excavation as the next separate design — this approved increment changes traffic only — if wrong, addressing construction timing takes an additional design/implementation increment.
9. Ruling: Keep CLI/browser/WASM extensions outside this Rust API increment — the approved spec excludes those integration surfaces — if wrong, users of those surfaces need separate adapter work.
10. Ruling: Claim compact snapshot-byte bounds, not resident-memory or performance guarantees — only serialized recording limits were specified/tested — if wrong, memory/runtime require measurement and possible optimization.
11. Ruling: Retain earlier F5 campaign reviews and audit only preserved compatibility here — completed campaign work was not redispatched, and all600saved bytes/reports remain exact — if wrong, a separate earlier-campaign audit may be needed.
12. Ruling: Keep branch/worktree without merge, push or removal — the approved plan explicitly preserves them for later integration — if wrong, integration/cleanup remains an additional authorized step.
13. Ruling: Limit bitwise replay evidence to the recorded supported platform — final receipts cover aarch64-apple-darwin/Rust1.98.1, not cross-platform identity — if wrong, other-platform results need validation and may diverge.
