## Why

An installed UAR candidate reconnects an MCP transport after a failed call, but stores the replacement only in the request's disposable filtered registry. The next independent request rebuilds its view from the stale global handle and fails immediately, so crash and timeout recovery cannot be certified truthfully.

## What Changes

- Preserve each configured MCP server's replaceable service identity across global, filtered, and merged registry views without sharing their authorization maps.
- Keep the failed call fail-closed and non-replayed while making the replacement transport available to later independent requests.
- Prove crash and timeout behavior with streamed tool-result evidence and fixture-side process identifiers in the local installed-artifact certifier.
- Add focused ownership, recovery, non-replay, and authorization-isolation tests.

## Capabilities

### New Capabilities

- `mcp-reconnect-recovery`: Defines how UAR preserves an MCP server's replacement transport across authorized registry views after a failed call without replaying the failed operation.

### Modified Capabilities

- None.

## Impact

- Runtime code: private MCP registry ownership and replacement logic in `src/mcp/registry.rs`; no public API or protocol change.
- Runtime UX and realtime state: later requests recover without restarting UAR, while the failing request continues to emit one unsuccessful normalized tool-result event.
- Provider compatibility: LLM provider and model routing are unchanged; the behavior applies only after a provider-selected run invokes an MCP tool.
- Certification: local installed-artifact evidence gains process-boundary tracing and fail-closed negative controls. No product testing moves to GitHub Actions.
- Dependencies: none added or changed.
- KBD: the child change is tracked under `fix-mcp-reconnect-shared-service-state`; successful reflection resumes the parent `certify-operational-resilience` change and invalidates the prior immutable candidate.
