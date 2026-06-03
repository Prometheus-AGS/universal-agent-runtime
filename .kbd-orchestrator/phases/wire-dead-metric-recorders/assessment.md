# Assessment: wire-dead-metric-recorders

**Phase:** `wire-dead-metric-recorders` (closes PARTIAL goal **H8** from `uar-harness-parity`)
**Date:** 2026-06-03 · Backend: OpenSpec · base `main` `10c0655`

## Goal

Wire the 5 metric recorders that are defined in `src/uar/telemetry/metrics.rs` but never called (confirmed: 0 call sites outside `metrics.rs`):
`record_sandbox_created`, `record_sandbox_execution`, `set_active_sandboxes`, `record_sandbox_error`, `set_mcp_server_status`. (`record_cache_tokens` + `set_active_sessions` were already wired in HP2/prior.)

## Findings (current `main`)

1. **Sandbox execution chokepoint — `src/llm/orchestrator.rs:~689-724`.** All sandbox code paths converge on one site: `runner.create(cfg).await` → `runner.execute(&handle, req).await` → `runner.destroy(handle)`. `ExecutionResult` has `exit_code: i32`, `execution_time_ms: u64`; `lang: Language` (`Bash|Python|Node|Rust`, serde lowercase) is in scope; `runner.capabilities().runner_type: RunnerType { MicroVm|Wasmtime|Remote }`. → `record_sandbox_execution` (success), `record_sandbox_error` (Err branch, classify `SandboxError`), and `record_sandbox_created` (after `create`) all wire here. **READY.**
2. **Active count — ephemeral model.** Runners are created→executed→destroyed per call (the `SessionManager` exists but is **not** wired to the orchestrator). There's no persistent "active" set. A meaningful gauge = **concurrent in-flight sandbox executions** across runs, achievable with a process-global atomic incremented at `create` / decremented after `destroy`, reported via `set_active_sandboxes`. Small, no architecture change. **READY (via atomic counter).**
3. **MCP status — connect-time only, `src/mcp/registry.rs:~67-122`.** `McpRegistry::from_config` connects each server once at startup (stdio ~line 92, http ~117); server name is the `mcp_servers` map key. No health/heartbeat loop. → `set_mcp_server_status(name, true)` on connect success, `false` on failure. **READY (connect-time; no ongoing health loop — documented).**
4. **Metrics are always-on** — no config gate; recorders no-op if the exporter didn't init. Existing recorders (`record_request`, `record_llm_tokens`) are called unconditionally. → wire unconditionally.

## Scope decision

Wire **all 5** recorders:
- `record_sandbox_execution` + `record_sandbox_error` + `record_sandbox_created` at the orchestrator sandbox chokepoint.
- `set_active_sandboxes` via a process-global `AtomicI64` (concurrent executions) inc/dec at create/destroy.
- `set_mcp_server_status` at MCP connect success/failure.

This moves **H8 PARTIAL → MET.** Honest caveat: MCP status reflects connect-time only (no live health loop — a follow-up if heartbeat monitoring is wanted); sandbox metrics cover the ephemeral execution path (the unused `SessionManager` adoption is out of scope).

## Complexity & risk

S–M, low risk. Additive metric calls at existing sites + one global atomic; no new dependency; behavior-preserving (metrics only). Wiring touches `orchestrator.rs`, `mcp/registry.rs`, `metrics.rs`.

## Ready for `/kbd-plan` → single change `wire-dead-metric-recorders`.
