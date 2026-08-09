---
type: Reference
id: knowme-embedded-mobile-build-fixed-by-gating-auto-discovered-bins
title: KnowMe embedded-mobile build fixed by gating auto-discovered bins
tags:
- knowme
- embedded-mobile
- cargo-features
- universal-agent-runtime
- rust
- surreal-memory
links:
- knowme-embedded-mobile-cargo-gating-scope-assessment
sources:
- stdin
- manual:KnowMe/embedded-uar-offline-agents
timestamp: 2026-07-27T23:54:25.645892+00:00
created_at: 2026-07-27T23:54:25.645892+00:00
updated_at: 2026-07-27T23:54:25.645892+00:00
revision: 0
---

## Context

- Project: **KnowMe**
- Phase: `embedded-uar-offline-agents`
- KBD root: `/Users/gqadonis/Projects/know-me/know-me-system`
- Captured: `2026-07-27T23:42:16Z`
- Position: `embedded-uar-offline-agents › embed-uar-mobile-offline`
- Status: `implementation_ready`
- Progress: `changes 0/1`
- Commit state: `3302a5d` plus UAR `392eea1`

## Result

`cargo check --no-default-features --features embedded-mobile` now completes successfully:

- Duration: **5m52s**
- Errors: **0**
- Task **1.3** closed

The failure cause was narrower than the earlier audit suggested. The implemented fix matches the follow-up scope described in [KnowMe embedded-mobile Cargo gating scope assessment](/knowme-embedded-mobile-cargo-gating-scope-assessment.md).

## Root cause

Cargo auto-discovers `src/bin/*.rs` targets. Auto-discovered binaries were being compiled under every profile, including `embedded-mobile`.

Problem binaries:

- `stub-llm`
  - Opens a `TcpListener`.
  - Should be server-only.
- `test_db_setup`
  - Constructs a `PostgresProvider`.
  - Should be server-only.

`uar-sidecar` was already declared with `required-features`; `stub-llm` and `test_db_setup` were not declared at all. An auto-discovered binary cannot be excluded from a feature profile by ordinary `cfg` use elsewhere; it must be explicitly declared with `required-features`.

## Fix

All four relevant binaries now carry:

```toml
required-features = ["server"]
```

Validation:

- Binaries are excluded under `--no-default-features --features embedded-mobile`.
- Binaries remain present when the `server` feature is enabled.

## Deliberately declined audit recommendation

The audit headline recommendation was to gate `surreal-memory` so `axum` and `tonic` stop being pulled. This was investigated and intentionally not implemented.

Reason: `surreal-memory` is not merely an optional transport/server dependency in the current architecture. It supplies core memory domain types:

- `Memory`
- `MemoryScope`
- `MemoryType`
- `MemoryStorage`

Those types are re-exported as public API from:

- `uar/domain/memory.rs`
- `uar/memory/mod.rs`

Usage evidence:

- **19 files** use `surreal-memory`.
- **0** of those usages are `cfg`-gated.

Making `surreal-memory` optional would remove or restructure the memory subsystem for mobile. That is a product/API decision, not a narrow dependency fix. It would also require restructuring a vendored submodule that had just been stabilized onto one branch. The evidence was recorded in `tasks.md` rather than making the change as a side effect of the build task.

## Audit corrections

- The second observed `axum` path is through `axum-test`, a dev-dependency; it does not affect a `--lib` build.
- The stated success condition, “until Android and iOS targets compile,” was already met for the APK: the Android build was built and certified at HEAD during the session.

## Next work

Potential next priorities from phase tracking:

- **3.2 embedded admin surfaces**: skills, MCP, knowledge, and memory should return `[]` or `not_on_embedded` as appropriate. This is the largest remaining gap.
- **5.4 A2UI**: blocked on needing an artifact producer.

# Citations

1. stdin
2. manual:KnowMe/embedded-uar-offline-agents