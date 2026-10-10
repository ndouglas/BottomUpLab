# Foraging local integration — 2026-10-10

The user authorized local integration of the completed `foraging-5-registration` branch. Merge commit `f1309308b9b760adbfe303b43d53796eb31c27b8` has parents `c58e7af0828617c5c766c3ae943b06b8669fd60a` (current main, including concurrent browser/minds/war work) and `948d849a2451795fe691ab8e18e97d72323c74f1` (reviewed feature archive). The corrected feature runtime was `56f7110`.

## Integration changes and review

The code merged cleanly. The sole conflict in `docs/foraging.md` was resolved by retaining main's browser engineering section and the feature's registered-run, traffic and informed-excavation sections.

A compiled-input audit found four concurrently added native browser fixture inputs missing from the F5 source selector: `caching-identities.json`, `caching-scenes.json`, `spatial-identities.json`, and `spatial-scenes.json`. A subagent prepared the minimal selector correction in an isolated worktree. The strengthened existing regression proves that changing each current browser fixture's bytes changes the selected-input fingerprint. It first failed for an omitted fixture (RED), then passed with all four paths included (GREEN). Independent scoped review approved the correction with no Critical, Important or Minor findings. An audit of 287 literal includes found only two deliberately excluded test-only burrow examples remaining outside the native selector. Separate historical BT/W1 provenance scopes were retained.

No controller behavior, registration, scientific manifest or original record was changed by this integration correction. Earlier implementation and whole-plan reviews remain recorded in the [corrected runtime verification](2026-10-09-foraging-informed-excavation-verification.md).

## Fresh merged verification

Commands ran in `/Users/nathan/Projects/ndouglas/SugarScape` with normal Cargo locks, hooks and test settings. Workspace runtime inputs remained unchanged throughout its gate. The source-selector/test correction affects the separate survey workspace; its complete suite, formatter and Clippy were rerun after applying the reviewed patch.

| Command | Result |
| --- | --- |
| `cargo test --workspace --locked` | 3,452 passed, 0 failed, 106 existing ignored; 55 summaries |
| `cargo test --manifest-path survey/Cargo.toml --locked` | 468 passed, 0 failed, 0 ignored; nine summaries, corrected tree |
| `cargo fmt --all --check` | passed |
| `cargo fmt --manifest-path survey/Cargo.toml --all -- --check` | passed, corrected tree |
| `cargo clippy --locked -p sugarscape-core --all-targets -- -D warnings` | passed |
| `cargo clippy --manifest-path survey/Cargo.toml --locked --all-targets -- -D warnings` | passed, corrected tree |
| `cargo build --manifest-path survey/Cargo.toml --locked --bin survey` | passed |

Core units account for 3,146 passed and seven ignored, 3,153 total cases. Ignored tests are excluded from passing counts. The initial survey gate also passed 468/0/0 before the source-binding correction; it is retained separately from corrected evidence. No suite was cancelled or hook bypassed.

## Native identity and original science preservation

All 931 selected checkout inputs matched the staged bytes before merge, and the committed merged checkout retained the exact embedded source identity `3b323b94826881ad30cba6e5a379fb325e11f28ea1834f7d08bc2a6cdb385128`. The exclusively created private collector is `/tmp/sugarscape-integration-20261010/collector-integrated`, 106,754,248 bytes, SHA256 `4572ab4b94ebe08fab5c5c01f3b454a145e64e75a0bdbf6066d9888779a07183`.

Two saved-only `--analyze` commands generated fresh report pairs under the integration evidence root. Both pairs match both original retained pairs exactly:

| File | Bytes | SHA256 |
| --- | ---: | --- |
| `analysis.json` | 13,976,411 | `7bba70c19802bb7b97abe69b84c592fd869bd1a09132912eabf508ab657ce527` |
| `results.md` | 11,864,047 | `2903e4c952e7974a13f17dcd209a0f70b5d9e7086b546f3756e50fc343591c46` |

The original index remains SHA256 `faaa5acbed1397f3451cd670071b97d66fbcc39b1870308e6dc8f2dc97104fa4`. All 600 original raw hashes and sizes were verified before integration and again after saved analysis/merge, totaling 66,473,358 bytes. Main's untracked `survey/out/foraging-shortcuts/` is preserved. This integration performed no new scientific campaign and no push. Two premature analysis launches exited 127 before the private binary existed and produced no reports; their logs are retained, followed by both successful analyses.

## Cleanup, evidence and next step

After successful gates, committed-source checking and preservation verification, normal `git worktree remove` removed only the integrated feature worktree at `/Users/nathan/.config/superpowers/worktrees/SugarScape/foraging-5-registration`; `git branch -d` removed its fully merged branch. Before removal, all 16 remaining ignored files (164,360 bytes) were copied and hash-verified under the integration evidence root. The temporary correction helper's 67 modified/untracked files and build outputs were also preserved before its normal removal. Other branches, worktrees and campaign evidence remain untouched; no force or global prune was used.

Evidence: `/tmp/sugarscape-integration-20261010/`, including complete initial/corrected gate logs and exit receipts, `gates.json`, selector patch/audit/independent review, `native-receipt.json`, both reanalyses, before/after science receipts, feature remaining-file inventory/archive, and helper archive. The completed integration tracker is archived there before retirement.

The original F5 scientific findings remain the baseline. The next scientific increment would register a distinct comparison of explicit traffic and excavation policies, retaining the original controls and records. Roles, relay transport, multi-cell planning and biological calibration remain separate future work; no revised-policy delivery benefit is claimed here.
