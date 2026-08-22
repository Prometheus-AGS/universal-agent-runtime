## Why

The 1.0 operational-resilience certification proved that an idle non-root UAR container waits the entire configured 30-second shutdown timeout before Axum begins draining, allowing the container orchestrator to send `SIGKILL` and produce exit code 137. This violates the existing one-second idle-shutdown requirement and blocks release certification on a real deployment boundary.

## What Changes

- Initiate HTTP connection draining immediately after shutdown begins instead of treating `shutdown_timeout_secs` as a mandatory pre-drain delay.
- Enforce `shutdown_timeout_secs` as the maximum graceful-drain window for the complete shutdown sequence, including cleanup and connection draining, then begin forced process termination at expiry.
- Distinguish normal cleanup completion from deadline-enforced exit and make the forced outcome synchronously observable.
- Add focused timing and real process-boundary coverage for idle shutdown, active-stream completion, listener refusal, caller-owned sidecar cancellation, SIGINT, and deadline expiry with held work.
- Keep the non-root container certification's exit-code-zero assertion while giving the external orchestrator a bounded margin beyond UAR's internal deadline.
- Preserve the existing run cancellation, ingestion-pool cleanup, A2A shutdown, dual-listener behavior, provider compatibility, and public APIs while explicitly closing configured MCP transports and joining UAR-owned embedded-database background tasks on normal shutdown.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `graceful-shutdown`: Clarify immediate drain, the configured maximum graceful-drain window, bounded deadline enforcement, sidecar cancellation isolation, and normal-versus-forced cleanup guarantees.

## Impact

- Runtime: shutdown supervision in `src/server.rs`, crate-private MCP lifecycle support in `src/mcp/registry.rs`, crate-private live-query task shutdown in `src/uar/realtime/surreal_bus.rs`, and focused process-boundary tests.
- Release certification: the local non-root container lifecycle in `scripts/certify-release-candidate.sh`; GitHub Actions remain deployment-only.
- Runtime UX: after shutdown begins, clients can no longer establish new HTTP or SSE work; active work receives only the remaining configured drain window.
- Realtime state: no event schema or persistence behavior changes; existing live connections close as part of the bounded drain.
- Provider compatibility: no provider routing, model, authentication, dependency, or public API changes.
- Profile boundary: `server-full` owns embedded SurrealDB and configured MCP services; it does not activate SQLx/Postgres and owns no Redis client, so no SQLx or Redis cleanup claim is made.
- KBD: the controlled child phase and parent certification waypoint must advance only after the corrected candidate passes its local evidence gates.
