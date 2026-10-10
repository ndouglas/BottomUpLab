## Stage 1: Policy data and private route plans
**Goal**: Checked traffic types and draw-free route planning with pinned baseline.
**Success Criteria**: Baseline episode/RNG exact; identities/counter invariants/private plans pass.
**Tests**: Construction suite, policy/counter/plan tests, fmt/core Clippy.
**Status**: Complete

## Stage 2: Seeded yielding and paid holds
**Goal**: Explicit traffic policy integrated with atomic controller.
**Success Criteria**: Exact threshold/draws, all cargo modes and terminal precedence; legal queue passage.
**Tests**: Scripted selector/controller/queue tests, construction suite, fmt/core Clippy.
**Status**: Complete

## Stage 3: Labeled bounded recording
**Goal**: Observational traffic views and separate episode wrapper.
**Success Criteria**: Aligned snapshots, unchanged base output, checked combined64MiB cap.
**Tests**: Views/RNG/bytes/cutoff/replay tests, construction suite, fmt/core Clippy.
**Status**: Complete

## Stage 4: Complete acceptance and compatibility
**Goal**: Prove privacy/rollback and retain F5 saved evidence through whole review.
**Success Criteria**: Both full packages/style/lint pass; source binding and identical saved analysis verified.
**Tests**: Queue/cargo/privacy/fault tests, cargo test --workspace, full survey, both fmt/Clippy, saved-only reanalysis.
**Status**: Not Started
