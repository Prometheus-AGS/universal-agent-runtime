PLAN: wire-dead-metric-recorders
Project: universal-agent-runtime · Date: 2026-06-03 · OpenSpec: YES · Changes: 1

Single change closes H8 (PARTIAL → MET). Wire all 5 dead recorders:
1. emit-runtime... no — `wire-dead-metric-recorders`:
   - sandbox created/execution/error at the orchestrator chokepoint
   - set_active_sandboxes via process-global AtomicI64 (concurrent in-flight)
   - set_mcp_server_status at MCP connect success/failure (connect-time)
   - Scope: ui n/a | api metrics | Complexity S–M | Model medium | Value MEDIUM (ops observability)
   - Depends on: NONE

EXECUTION: Round 1 — single change.
COMMANDS: /opsx:new wire-dead-metric-recorders (done)

Sycophancy self-check: grounded in file:line exploration; scope held to 5 recorders;
caveats surfaced (MCP connect-time only, ephemeral sandbox model, SessionManager out of scope).
PLAN COMPLETE
