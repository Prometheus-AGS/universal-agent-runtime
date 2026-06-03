# Reflection: wire-dead-metric-recorders

**Phase:** `wire-dead-metric-recorders` (single change closing PARTIAL goal **H8** from `uar-harness-parity`)
**Date:** 2026-06-03 · Merged `main`: `6583087` (PR #30)

## Outcome
**H8 PARTIAL → ✅ MET.** All 5 previously-dead recorders are wired: sandbox created/execution/error + active-count (process-global atomic, concurrent in-flight) at the single orchestrator chokepoint; `set_mcp_server_status` at MCP connect (stdio + http), false on failure.

## Quality
- cargo check clean (zero new warnings; `#[expect]` on small count/duration casts); 234 lib tests pass; clippy clean for touched code; strict-valid.
- Metrics-only, behavior-preserving, no new dependency, bounded labels. Only the 3 intended files changed (no fmt drift).

## Deferrals (documented)
- MCP status is **connect-time only** — an ongoing health/heartbeat loop is a follow-up.
- Sandbox metrics cover the **ephemeral execution path**; adopting the unused `SessionManager` for persistent sessions is out of scope.
- Live-env `/metrics` verification pending.

## Lesson
The HP2 "not cheap, defer" call was right at the time, but a re-grounded assessment found a **single orchestrator chokepoint** that made 4 of 5 trivially wireable and the 5th (active count) a small atomic — turning a deferred item into a quick, clean close. Re-assessing carried-over debt against current code beats assuming the old deferral still holds.

## Status
H8 MET. `uar-harness-parity` now fully resolved except the by-design deferral H7 (eval harness). Next: `uar-safety-and-evals` phase.
