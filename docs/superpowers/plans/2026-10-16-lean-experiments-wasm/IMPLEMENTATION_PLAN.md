# Lean experiments WASM — owned tracker

## Stage 1: Diagnose and scope
**Goal**: Trace embedded bytes and preserve baseline.
**Success Criteria**: Actual317MB payload cause and approved delivery boundaries established.
**Tests**: Read-only payload audit and builder check.
**Status**: Complete

## Stage 2: Lean selected delivery
**Goal**: Compact verified receipts and on-demand complete recorded data.
**Success Criteria**: Native validation/parity preserved; experiments WASM at most8,000,000bytes.
**Tests**: Receipt/asset/fresh-import/parity tests and actual release build.
**Status**: Complete

## Stage 3: Budget enforcement and final verification
**Goal**: Hard regression gates and actual browser evidence, reviewed local delivery.
**Success Criteria**: Both budgets enforce; full data/export and old evidence preserved.
**Tests**: Budget negatives, actual worker/network checks, source/fixture hashes and independent reviews.
**Status**: In Progress

## Verified checkpoint — 2026-10-16

Implementation and code review are complete through `1f8538e`; Task1 and its scoped fix review passed, and Task2/whole-branch reviewers found no source defects. Actual release experiments WASM is6,165,352bytes; engine6,836,271bytes. Both independent8,000,000byte gates and the normal production build pass. Complete recorded transport remains9,641,132bytes and byte-identical to original delivery. Scientific fixtures and original artifacts remain unchanged; only authorized compatibility receipt metadata advanced.

Stage3 remains **In Progress**. Actual served UI/catalog, Notes run/import/masks/playback/cancel, no-early-recorded-fetch sequence, missing/tampered asset recovery/full export and timings are not accepted. Supported Browser inspections repeatedly hit30second tool resets, including documented recovery; no app defect or passing UI result is inferred. Browser attempts stopped and a reconnection request is pending.

Evidence/reviews/current runtime preservation checkpoint are under ignored `studio/out/wasm-slim-20261016/`; the active plan ledger remains in `.superpowers/sdd/2026-10-16-lean-experiments-wasm/` and is also archived there. Keep this tracker, branch and worktree while the acceptance gate is open. The owned test server was stopped and its scratch fault mode restored to normal; restart with `node studio/out/wasm-slim-20261016/task2-serve.mjs` after reconnecting Browser. No merge, push, publication or scientific campaign occurred.

## Resumed browser acceptance

After the user reconnected Browser, the production19-study catalog, selected Notes worker, complete selected export/native equality, fresh import, Reader/researcher masks, playback/seek, hostile-import rejection, demand-only recorded loading, missing/tampered asset errors, cancellation, observed60second deadline and intact retry all passed. The selected worker completed361ms; import observation was3.140seconds including tool round trips. The scoped independent acceptance review supports those checks. No implementation or build changed.

Only downloaded complete retained-results and Notes-summary UI export equality remain open. Browser controls stalled again before tab attachment; an independent controller attempt stopped at session naming before a fresh page action. No application defect or successful export is inferred. The user was asked to perform the two downloads manually; no reply or files are inferred from elapsed time. The owned server is intentionally left running in normal mode at `http://127.0.0.1:4178` for that handoff. New receipts and review are preserved with the existing evidence; Stage3 remains **In Progress**.
