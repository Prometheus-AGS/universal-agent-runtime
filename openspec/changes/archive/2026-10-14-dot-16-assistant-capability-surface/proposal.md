# Proposal

## Why

Current HTTP/embedded method reachability is uneven and blanket parity claims would be misleading. This change supplies the D4 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Expose the six contracts through authenticated server/sidecar HTTP management and supported MCP/native assistant tools using the same host services. Publish per-operation capability descriptors for memory controls, schedule CRUD/pause/cancel, follow-up policy/inbox, tool outcomes, approval reads/decisions and checkpoint recovery. Postgres/in-memory/unattested remote and direct embedded Runtime must explicitly identify unsupported operations; no silent fallback. Provide API examples for consumers, not a new assistant UI.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `product-capability-support-matrix`: Assistant capability discovery matches reachable operations. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `src/uar/api/capabilities.rs`, `src/uar/api`, `src/uar/memory/mcp_server.rs`, `sdks/rust/src/client.rs`, `sdks/rust/src/runtime.rs`. Dependencies: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

