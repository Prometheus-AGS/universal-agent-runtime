# Lifecycle lead integration handoff

Date: 2026-10-20. Phase: `function-first-release-certification`, Execute.
Change: `deterministic-lifecycle-management`. Verification: **CLOSED**.

This handoff records root integration source changes. Specialist files and exact
responsibilities are listed in their adjacent runtime/state/trust/console/A2A/
sandbox/startup/watcher/entrypoint handoffs. Source attribution excludes unrelated
pre-existing dirty edits. Saved per-file identities are in the renewed inventory;
they are not a passing gate result.

## File-by-file integration

| File or exact module | Root integration |
| --- | --- |
| `src/server.rs` | Retained startup/cleanup owner; physical provider and memory ownership; scoped domain access; service/task registration; committed-work forwarding; authorized snapshot route; owner session projection; HTTP lease through response frames/trailers; readiness receipts; one supervisor close path. Startup and watcher writers supplied the named follow-ups under explicit ownership. |
| `src/server/lifecycle_graph.rs` | Explicit service finalization dependencies; watcher writer corrects producer-first order. |
| `src/server/lifecycle_control.rs` | Trusted private snapshot/reconnect protocol; safe failure categories, unresolved IDs and retained close-report projection. |
| `src/server/lifecycle_snapshot.rs` | Frontend writer's authorized route reuse, integrated into the composition root. |
| `src/server/process_signals.rs` | OS subscriptions acquired before database initialization, scope-owned signal task and actual join; external Windows hosts retain their cancellation authority. |
| `src/main.rs` | One-shot commands do not start file watchers; normal/error/panic paths retain configuration and telemetry cleanup; return exit status after the runtime's owned work. |
| `src/server/windows_service.rs` | Real startup acknowledgement before Running, awaited cleanup despite status errors, explicit aggregate terminal errors. |
| `src/bin/uar-sidecar.rs` | Trust writer's owned inherited-input lifecycle integrated with the host control API; metrics startup remains inside the caught cleanup boundary. |
| `src/embedded.rs` | Retained initialization and cleanup, explicit owned/borrowed persistence/MCP/memory APIs, dependency metadata and scope-state retention; undelivered successful startup finalizes its runtime. |
| `src/uar/runtime/lifecycle.rs` | Runtime writer's supervisor primitives integrated with weak scopes and retained actual resource state. Added atomic `register_external` classification so shutdown closes external intake immediately while persistence remains available to finalizers. The supervisor may attach the acquired-prefix dependency graph after startup cancellation closes intake, while its registry lock still excludes finalization; a scope whose close has started rejects attachment. |
| `src/uar/persistence/scoped.rs`, `src/uar/persistence/mod.rs` | Admitted domain facade delegates persistence operations; mutation uncertainty is retained without treating read cancellation as a durable effect; owned session listing is delegated. |
| `src/uar/realtime/lifecycle.rs`, `src/uar/realtime/mod.rs` | Retained bus scope, generations, topic readiness, owned worker outcomes and explicit reconnect; fatal worker failures invalidate topics. |
| `src/uar/realtime/surreal_bus.rs` | Scoped Surreal subscriptions and original-session awaited closure; no residual sweeps or timed reconnect loop; separate memory bus. |
| `src/uar/realtime/postgres_bus.rs` | Dedicated listener ownership, transport gap state and joined local pool closure after unlisten. |
| `src/uar/api/live.rs` | Public-topic gate, versioned generation/gap invalidations, no raw shared-row exposure; readiness comes from the actual bus state. |
| `migrations/20261020000000_lifecycle_work_notifications.sql` | Committed PostgreSQL notifications for internal lifecycle work topics. |
| `src/uar/api/collaboration/team_execution.rs`, `src/uar/runtime/team_execution/controller.rs`, `src/uar/runtime/manager.rs` | Owner-authorized exact-attempt registration acknowledgement, retained registration/catalog/committed-work wakeups, no run-registration probe loop. |
| `src/uar/api/administration_capabilities.rs` | Truthful declarations for registration acknowledgement and authorized ingestion-status SSE. |
| `src/config.rs`, `src/uar/settings/manager.rs` | Hermetic skill-root input and accurate legacy admission/shutdown timeout descriptions; timeout expiry is not completion. |
| `example.config.yaml` | Exact advertised streaming peer interface paired with the intended compiler-agent identity. |
| `src/uar/rag/ingestion_worker.rs`, `src/uar/rag/ingest.rs` | Scope-owned dispatch/blocking work, event/due-based wakeups, retained results and temporary-file cleanup; watcher work is included in the producer-first follow-up. |
| `src/uar/runtime/observer/channel_authority.rs` | Removes an implicit ten-second effect-authority deadline; authoritative result/uncertainty remains the outcome boundary. |
| `Cargo.toml`, `Cargo.lock` | Local SurrealDB overlay integration, existing Tokio-util tracking feature, target-specific pipe dependencies, existing `http-body-util` 0.1.3 resolution promoted direct to preserve body trailers. |
| `versions.toml`, `vendor/git/README.md`, exported memory storage source | Pinned first-party host-managed memory lifecycle integration from local upstream commit `9e36b1cd8b83a657366eb1e3ef8cd837178336de`; no publication claim. |
| `vendor/patches/surrealdb-lifecycle/README.md` | Six-crate 3.3.0 source overlay provenance, maintenance ownership and upstream replacement criterion. |
| Overlay SDK `src/{lib,method/live,conn/{mod,driver_scope}}.rs` and remote router glue | Shared close authority, inert closed facades, original-session stream close, tracked driver ownership and aggregate errors. State/runtime/trust writers own their remaining overlay lanes. |
| Overlay core `src/{kvs/ds,idx/builder,dbs/tasks}.rs` | Explicit maintenance/index drain, aggregate physical close errors and native/WASM completion acknowledgements; no bootstrap retry sleeps as authority. |
| `src/server/tests.rs` | Extracted existing server tests for trust writer's controlled-process acceptance authoring; not executed. |
| `docs/perfect-lifecycle-management-architecture.md`, `docs/operations/lifecycle-*.md` | Architecture, resource ownership, first-stop watcher requirement, compatibility, handoffs, inventory, unverified limits and later exact-command proposals. |
| `openspec/changes/deterministic-lifecycle-management/` | Proposal/design, lifecycle/migration/realtime deltas and source/deferred-acceptance task separation. |
| Human KBD phase plan, supported canonical KBD/team state | Unified scope, compatibility decisions, follow-up tasks and closed-gate next work. Generated projections are written only by their supported tools. |
| `.prometheus/decisions.md`, `gotchas.md`, `session-log.md` | Append-only decisions, watcher-lifetime mistake, failed primary memory append fallback and current handoff. |

## State and evidence limits

The original eight implementation task completions were followed by 5.2
entrypoint and 5.3 watcher-order corrections. Completed canonical tasks are
immutable; their earlier status is history, not permission to ignore a newly
observed defect. The source inventory is renewed after the follow-ups.

No verification ran. Inspection used source reads/searches, read-only Git
metadata, dependency documentation and saved-file inventory authoring. Supported
KBD/team mutations recorded source status; they did not execute gates. Primary
memory observation append returned an error, so the follow-up is retained in
append-only Markdown. Nothing added requires a passing result to be fabricated.

The uncomfortable facts remain: all new integration is uncompiled and
unexecuted; the exact historical migration 503 body and individual sidecar
failure traces are absent; prepared images/customer payloads are stale; Windows
behavior and physical engine release are unproven. A permanent remote silence
can keep acknowledged cleanup pending. Timers do not resolve that uncertainty.

No unsolicited feature, unrelated dependency upgrade, public release, deploy,
push or destructive cleanup was performed. The isolated first-party memory
source commits are local and explicitly recorded. Guards follow the named
observed races and real principal/peer/command trust boundaries. The existing
dirty checkout and other writers' changes were preserved.
