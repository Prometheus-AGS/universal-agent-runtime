## Context

Five recorders in `metrics.rs` are dead (0 callers). The sandbox chokepoint is `orchestrator.rs` (`runner.create → runner.execute → runner.destroy`), with `ExecutionResult { exit_code: i32, execution_time_ms: u64 }`, `lang: Language`, and `runner.capabilities().runner_type: RunnerType`. Runners are ephemeral (no persistent session set; `SessionManager` exists but is unwired). MCP servers connect once at startup in `registry.rs::from_config` (stdio + http), keyed by name; no health loop. Metrics are always-on (no config gate).

## Goals / Non-Goals

**Goals:** emit all 5 recorders at clean, existing sites; keep it metrics-only and behavior-preserving; no new dependency.
**Non-Goals:** MCP heartbeat/health loop (connect-time only); persistent sandbox sessions via `SessionManager`; per-run sandbox attribution beyond labels.

## Decisions

### D1 — Single sandbox chokepoint
Wire `record_sandbox_created` (after `create`), `record_sandbox_execution` (success), and `record_sandbox_error` (Err) at the one orchestrator site, rather than in the 3 runner impls. `language` → lowercase label via a small match; `runner_type` → match on `RunnerType`; `exit_code_class` = `success` if `exit_code == 0` else `error`; `duration_secs = execution_time_ms as f64 / 1000.0`. `SandboxError` classified into a small `error_type` set.

### D2 — Concurrent-active gauge via a process-global atomic
Add a private `static ACTIVE_SANDBOXES: AtomicI64` in `metrics.rs` with `sandbox_active_inc()` / `sandbox_active_dec()` helpers that update it and call `set_active_sandboxes(count)`. The orchestrator calls inc right after `create` and dec after `destroy` (both success and error paths). Reports true concurrent in-flight executions across runs. Avoids the unused `SessionManager`.

### D3 — MCP status at connect time
In `from_config`, wrap each server connect in a match: success → `set_mcp_server_status(name, true)` then insert; failure → `set_mcp_server_status(name, false)` then propagate the error. Connect-time only; documented.

### D4 — Unconditional, label-bounded
Call recorders unconditionally (matches existing recorders; no-op if exporter uninitialized). All labels are bounded enums/known names — no cardinality risk.

## Risks / Trade-offs

- **[MCP status staleness]** a server that dies after startup still shows healthy → Mitigation: documented as connect-time; a heartbeat loop is a follow-up.
- **[Active gauge underflow]** mismatched inc/dec could drift → Mitigation: dec on BOTH success and error paths after `destroy`; inc exactly once after `create`; the atomic is saturating/clamped at >= 0 when reported.
- **[Error classification coarse]** `SandboxError` → a few buckets → Mitigation: acceptable; a small explicit match, default `"other"`.

## Migration Plan
1. Add `sandbox_active_inc/dec` helpers (+ atomic) to `metrics.rs`.
2. Wire create/execution/error + inc/dec at the orchestrator chokepoint.
3. Wire MCP connect success/failure in `registry.rs`.
4. `cargo check`/`clippy`/tests; manual `/metrics` check pending live env.
- Rollback: additive metrics; revert restores prior state.

## Open Questions
- MCP heartbeat loop now or follow-up? (Follow-up — keep this change bounded.)
