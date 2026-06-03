# Tasks — wire-dead-metric-recorders

## 0. Bootstrap
- [ ] 0.1 Confirm seams: orchestrator sandbox chokepoint (`orchestrator.rs` create/execute/destroy), `registry.rs::from_config` connect sites, `metrics.rs` recorder signatures
- [ ] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. metrics.rs — active-sandbox counter helpers
- [ ] 1.1 Add private `static ACTIVE_SANDBOXES: AtomicI64` + `sandbox_active_inc()` / `sandbox_active_dec()` that update it and call `set_active_sandboxes(count as f64)` (clamped >= 0)

## 2. orchestrator.rs — sandbox chokepoint
- [ ] 2.1 After successful `create`: `record_sandbox_created(runner_type, language)` + `sandbox_active_inc()`
- [ ] 2.2 On success: `record_sandbox_execution(language, exit_code_class, duration_secs)` (`success` if exit_code==0 else `error`; duration from `execution_time_ms`)
- [ ] 2.3 On `Err`: `record_sandbox_error(error_type)` classifying `SandboxError`
- [ ] 2.4 After `destroy` (both success + error paths): `sandbox_active_dec()`
- [ ] 2.5 Helpers for `runner_type` (`RunnerType` match) + `language` (`Language` → lowercase &str)

## 3. registry.rs — MCP connect status
- [ ] 3.1 stdio connect: success → `set_mcp_server_status(name, true)`; failure → `set_mcp_server_status(name, false)` then propagate
- [ ] 3.2 http connect: same

## 4. Validation (gate)
- [ ] 4.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 4.2 `cargo clippy` — no new warnings in touched files
- [ ] 4.3 `cargo test --features postgres-backend --lib` — existing pass
- [ ] 4.4 Manual: `/metrics` shows `uar_sandbox_*` after a sandbox run + `uar_mcp_server_status` after boot (pending live env — document if not runnable here)
- [ ] 4.5 `openspec validate wire-dead-metric-recorders --strict`; update `.kbd-orchestrator` progress

## Notes
- All 5 dead recorders wired. MCP status is connect-time only (heartbeat loop = follow-up).
- Ephemeral sandbox model; active gauge = concurrent in-flight executions (global atomic). `SessionManager` adoption out of scope.
- Metrics-only, behavior-preserving, no new dependency. Labels bounded (no cardinality risk).
