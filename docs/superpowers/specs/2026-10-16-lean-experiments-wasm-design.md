# Lean experiments WASM packaging

The user approved fixing the 317 MB experiments download on 2026-10-16 after the concrete description: keep full references in native verification, give the browser a lean selected-episode module, and preserve validation guarantees. This is delivery work; no scientific collection or training is authorized. Technical packaging choices below are the builder's calls within that approved goal.

## Outcome

The browser continues to run and freshly validate the same bounded selected episodes using native Rust semantics, strict input parsing, owned private projections and source-bound identities. The raw release experiments WASM must be at most **8,000,000 bytes**, matching the existing engine ceiling in magnitude; this new experiments budget is my call. The existing engine budget remains unchanged. Measure raw bytes, not just gzip transfer. Keep all currently offered studies available. Recorded measurements remain complete, exact and available on demand, with source/size/hash verification; they must not enter the selected-episode module's initial download.

## Packaging boundary

Remove runtime retention of large reference/recorded JSON solely for calculating identities. Generate compact receipts from actual raw inputs at build time; do not substitute hand-entered unverified hashes or change a preimage silently. Native diagnostic/reference validation retains full raw fixtures and its checks. Browser selected-record import must still strictly parse, freshly rerun and compare the entire native record; a successful flag or digest alone is never authority. Any browser-only unavailable exhaustive operation must fail explicitly, never return empty/valid data. Bind generated identity machinery into current delivery provenance and test native/WASM receipt parity. Preserve order/domain conventions of existing identities and honestly record any current identity changes.

Deliver recorded results separately, lazily, using the existing exact lossless wire data. Keep complete rows, values, ties and frozen scientific source/report identities. Reject stale, missing, oversized or hash-mismatched assets visibly. Do not weaken the existing 16 MiB transport bound, worker cancellation or timeout, privacy masks, numeric precision, or source validation. Avoid a new server, third-party service, compression dependency or scientific evaluator. Prefer existing static Vite assets and build tools. The exact implementation structure is a builder choice after tracing all large reachable payloads, including duplicate copies and other study references.

## Preservation and integration

Start from completed local Notes HEAD26bcde0 in owned worktree wasm-slim-20261016. Leave the original Notes worktree, archived executable/WASM/reports/reference files, and its evidence index untouched. Do not edit frozen scientific fixtures or protocol to make tests fit. Current source/adapter identities may advance with actual code changes; frozen reports retain their original identities and are served as those original observations. In particular, a retained summary must authenticate against its frozen provenance, not be relabeled as a current-source measurement. No diagnose, training, collector census, publication, push, or automatic merge. Main checkout is detached with an unrelated unresolved conflict; never mutate it or its index.

## Verification

Verify generated receipts against raw files and domains; keep native full-reference validation intact. Compare actual native/WASM selected records and hostile imports, all role masks and chosen boundary clocks. Verify on-demand recorded output is exactly the prior complete wire output and original summary, with tamper/size/path failures and no extra episode execution. Build actual release artifacts, enforce both budgets, inspect actual browser loading/network behavior and selected-record import. Preserve before/after sizes, raw hashes, build tools, receipt closures, tests and failures. Local commits and independent review complete delivery.
