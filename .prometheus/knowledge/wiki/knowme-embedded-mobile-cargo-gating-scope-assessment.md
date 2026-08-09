---
type: Reference
id: knowme-embedded-mobile-cargo-gating-scope-assessment
title: KnowMe embedded-mobile Cargo gating scope assessment
tags:
- knowme
- embedded-mobile
- cargo-features
- universal-agent-runtime
- surreal-memory
- rust
sources:
- stdin
- manual:KnowMe/embedded-uar-offline-agents
timestamp: 2026-07-27T23:21:24.347372+00:00
created_at: 2026-07-27T23:21:24.347372+00:00
updated_at: 2026-07-27T23:21:24.347372+00:00
revision: 0
---

## Context

- Project: **KnowMe**
- Phase: `embedded-uar-offline-agents`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T23:17:39Z`
- Position: `embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress: `changes 0/1`

## Completed fix: gate auto-discovered binaries

Cargo auto-discovers `src/bin/*.rs`, so the following binaries were being compiled under every profile, including `embedded-mobile`:

- `stub-llm.rs`
  - Opens a `TcpListener`.
  - Should be server-only.
- `test_db_setup.rs`
  - Requires `PostgresProvider`.
  - Should be server-only.

Fix applied: declare both binaries with `required-features = ["server"]` so Cargo excludes them unless the `server` feature is enabled.

This is the correct gating mechanism for auto-discovered binaries; ordinary feature-gating elsewhere does not prevent Cargo from discovering and attempting to compile `src/bin/*.rs` targets.

## Scope concern: `surreal-memory` is not a simple Cargo gate

The audit recommendation to gate `surreal-memory` so `embedded-mobile` stops pulling `axum` and `tonic` was assessed as materially larger than a dependency-list change.

Findings:

- `surreal-memory` is used by **19 files**.
- **0** of those usages are currently `cfg`-gated.
- It provides core domain/public API types:
  - `Memory`
  - `MemoryScope`
  - `MemoryType`
  - `MemoryStorage`
- Those types are re-exported from:
  - `uar/domain/memory.rs`
  - `uar/memory/mod.rs`

Making `surreal-memory` optional would require adding `#[cfg(feature = ...)]` gates across:

- the memory subsystem,
- the MCP registry,
- the tool layer,
- public API exports.

Additional risk: the affected code is in a vendored submodule whose pointer was recently stabilized onto a single branch. Restructuring it would be a product/API decision rather than a narrow build fix.

Expected functional result of fully gating `surreal-memory`: a mobile build without the memory subsystem.

## Dependency assessment

Other dependencies mentioned in the audit are cheaper to gate independently:

- `tokio` with `full` features,
- `reqwest`,
- `rmcp`.

However, these are not the source of the `axum` pull. The assessment identifies `surreal-memory` as the dependency that pulls `axum`.

## Status of task 1.3

Task wording: gate transport-only/server dependencies **until Android and iOS targets compile**.

Observed status:

- Android/iOS targets already compile and ship by another route.
- An APK was built and certified at HEAD during the session with all three persistence markers.
- Therefore, the task's success condition is already satisfied, even though the narrower claim that the `embedded-mobile` profile excludes all server dependencies remains false.

## Immediate next step

An `embedded-mobile` check was running after the binary gating fix.

Decision path:

- If the check passes: close task 1.3 with a documented exception/scope note for `surreal-memory`.
- If the check fails: obtain product/owner approval before restructuring the vendored memory subsystem to make `surreal-memory` optional.

# Citations

1. stdin
2. manual:KnowMe/embedded-uar-offline-agents