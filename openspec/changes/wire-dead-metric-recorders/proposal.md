# wire-dead-metric-recorders

## Why

Closes PARTIAL goal **H8** from `uar-harness-parity`. Five metric recorders are defined in `src/uar/telemetry/metrics.rs` but never called (0 call sites outside that file): the four sandbox recorders (`record_sandbox_created`, `record_sandbox_execution`, `set_active_sandboxes`, `record_sandbox_error`) and `set_mcp_server_status`. They were deferred during HP2 as "not cheap"; the current code now offers clean wiring points, so they can be lit up. (`record_cache_tokens` + `set_active_sessions` were already wired earlier.)

## What Changes

- **Sandbox execution metrics** at the single orchestrator chokepoint (`src/llm/orchestrator.rs`, around the `runner.create → execute → destroy` site):
  - `record_sandbox_created(runner_type, language)` after a successful `create` (`runner_type` from `runner.capabilities().runner_type`; `language` from the request).
  - `record_sandbox_execution(language, exit_code_class, duration_secs)` on success (`exit_code_class` = `success` if `exit_code == 0` else `error`; `duration_secs` from `execution_time_ms`).
  - `record_sandbox_error(error_type)` on the `Err` branch, classifying `SandboxError`.
  - `set_active_sandboxes(count)` via a process-global `AtomicI64` incremented at `create` and decremented after `destroy` (reports concurrent in-flight sandbox executions).
- **MCP server status** in `src/mcp/registry.rs::from_config`: `set_mcp_server_status(name, true)` when a server connects (stdio + http), `false` on connection failure.

Out of scope (documented): an ongoing MCP health/heartbeat loop (status is connect-time only); adopting the unused `SessionManager` for persistent sandbox sessions; ML/extra metrics.

## Capabilities

### Modified Capabilities
- **`prometheus-metrics`** — delta `specs/prometheus-metrics/spec.md`. The previously-dead sandbox recorders and MCP-server-status gauge are now emitted (the existing spec already mentions an "Active session gauge"; this adds the sandbox + MCP-status series as fulfilled requirements). Existing series unchanged.

## Impact

- **Affected code:** `src/llm/orchestrator.rs` (sandbox chokepoint metrics + active counter), `src/mcp/registry.rs` (connect-time status), `src/uar/telemetry/metrics.rs` (optional small atomic-counter helpers for active sandboxes). No new dependency.
- **APIs:** no HTTP changes; new Prometheus series at the existing `/metrics` (`uar_sandbox_*`, `uar_mcp_server_status`).
- **Behavior preservation:** metrics-only; unconditional like existing recorders; no run/UX change.
- **Cardinality:** labels bounded — `runner_type`, `language`, `exit_code_class` (success/error), `error_type` (small set), `server_name` (configured MCP servers).
- **Security:** no sensitive data — only types/languages/durations/exit-classes/server names.
- **KBD workflow state:** YES — closes H8.
