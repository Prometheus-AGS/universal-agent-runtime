# Lifecycle producer and trust continuation

Date: 2026-10-20. Active phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`, Execute. Verification remains **CLOSED**. The waypoint read during this continuation was revision 3144, plan revision 68, 137/153 implementation items. This document is a writer handoff, not an implementation-complete inventory or acceptance result.

## Observed source failures and implementation

| Source-grounded reproducer | Authored correction |
| --- | --- |
| Cancel a registry shutdown waiter after a consuming `RunningService::cancel()` has taken a service owner. The old future owned the only join operation, and a later caller could find an empty collection. | Registry, admission, individual service slots, and retired replacement batches retain one owned cleanup operation and its result. Waiters await the stored join handle by mutable reference. Canceling a waiter leaves cleanup running and repeated callers retain failures. |
| Drop `McpInteractionRunGuard` during a run failure. Its former Drop implementation spawned database-dependent settlement outside the resource scope. | Register the interaction finalizer before returning the guard. Drop requests that registered finalizer; `finish()` observes its retained acknowledgement. Transport retirement and durable interaction settlement both contribute failures. |
| Cancel a tool caller or let the 30-second wrapper expire while a native/MCP future is accepted. The former timeout dropped that future. | Registry admission registers an owned operation before execution. Host task scope propagates to its descendants. Caller/scope cancellation requests the selected MCP connection to close, and the request remains owned until SDK completion. Dispatched transport loss is outcome unknown and enters the host scope's unresolved effects. Native tools without cancellation acknowledgement remain outstanding until their actual result. |
| Initialize/reconnect a legacy configured stdio MCP server. These paths used a raw child transport rather than the existing process supervisor. | Both paths now use `StdioProcessSupervisor`. A failed handshake awaits a per-child reap receipt; forced termination remains identified separately. A reaper panic produces an explicit unconfirmed-cleanup failure. |
| Cancel `from_config`, or fail schema/catalog construction after earlier services have opened. Cancel snapshot discovery while its local service owner is live. | An owned acquisition operation registers the host lease before starting initialization, immediately publishes acquired services into its cleanup owner, and retains the initializer when the recipient disappears. Failure, panic, canceled delivery, and canceled scope all finalize the acquired prefix. Stdio and HTTP snapshot handshakes/discovery use this owner. Administrator upserts also retain their accepted operation independently of the request waiter. |
| A failed binding cleanup is removed from the retirement queue, or the interaction helper discards its result. | Only successfully released bindings leave the queue. Interaction retirement returns and aggregates cleanup errors. |
| A Surreal transaction reports contention even though another identical migration may have committed its journal. | Perform one authoritative journal lookup after contention. Validate destination, source identity, approval and precondition through the existing replay contract. A matching receipt returns replay; missing/unavailable evidence remains typed contention with the stable operation identity. There is no repeated read or sleep. |
| PostgreSQL rejects a destination/precondition after beginning a migration transaction, or preparation panics. The former early return relied on transaction Drop. | Preparation errors and caught preparation panic await rollback. Receipt replay also awaits rollback. Actual mutation still commits destination and receipt atomically. Commit-response loss remains outcome unknown; rollback failure remains unavailable rather than successful cleanup. |
| Sidecar native input is acquired, then telemetry initialization fails or panics before the control reader owns it. | Await `AsyncInput::close()` on both paths and retain an input-close failure alongside the initialization failure. |

## Exact changed paths in this continuation

- `src/mcp/registry.rs`: retained cleanup and startup ownership; registered invocation/upsert ownership; configured child supervision; cancellation/result separation; unknown-outcome scope reporting.
- `src/mcp/stdio_process.rs`: retained reaper-panic failure and per-child completion receipt for failed handshakes.
- `src/mcp/runtime.rs`: registered interaction finalizer, retained acknowledgement, and Drop as a request only.
- `src/mcp/interaction.rs`: crate-private lifecycle scope accessor for that finalizer.
- `src/mcp/binding_cache.rs`: fallible interaction retirement and retention of failed cleanup obligations.
- `src/uar/security/credentials/principal_store.rs`: one-read Surreal receipt reconciliation and acknowledged PostgreSQL rejection/replay rollback.
- `src/bin/uar-sidecar.rs`: input finalization on telemetry initialization error/panic.
- `docs/operations/lifecycle-producer-trust-continuation.md`: this handoff.

No release script, workflow, dependency manifest, SDK overlay, server file, fixture, or test was changed in this continuation.

## Interfaces and dependency order

Public registry tool/upsert/configuration signatures remain compatible. The new operation owners use existing Tokio, Tokio-util task tracking, futures panic capture, and host `ResourceScope` contracts; no package is introduced. `retire_interaction` now returns an internal `anyhow::Result<()>`. `StdioProcessReceipt` is crate-private and reports actual exit acknowledgement and forced-process count.

The host must keep persistence usable until the run interaction finalizer and registered invocation/acquisition leases finish. The lead confirmed source integration of independently retained common server finalization, caught initialization/serving panic, startup task-local scope, and acquired-prefix dependency edges. That is a source handoff, not runtime proof.

The credential coordinator's existing same-runtime join remains in place. Each caller still authorizes independently in the provider service before joining. The existing fingerprint, source-key journal identity, PostgreSQL source arbitration, atomic effect/receipt boundary, and Surreal NONE conversion are preserved. No migration retry loop was added.

MCP annotations and descriptions remain untrusted metadata. These changes preserve the existing administrator approval list, verified owner checks, per-run credential lease and definition reauthorization, frozen connection generation, delegated native policy checks, and child environment allowlist. Errors added by cleanup/operation owners use fixed categories and do not include tool arguments or credential material.

## Remaining production limits and phase obligations

The uncomfortable case is deliberate: an accepted native tool without a cancellation interface, or an SDK handshake that never completes, can keep its registered obligation pending indefinitely. The implementation retains the actual future and does not report termination based on elapsed time. Adding cooperative cancellation to a specific native implementation requires an actual adapter acknowledgement; the generic `NativeTool` trait cannot manufacture it.

Local MCP connection closure cannot prove remote mutation rollback. An uncertain dispatched result remains outcome unknown even after local transport and child cleanup. The scope retains that uncertainty for the terminal host report.

The unused `src/mcp/stdio_client.rs` still contains older detached I/O and timeout code. Source searches found no module declaration or live use; it was not modified as an adjacent inactive implementation. Reintroducing it into the module graph requires the same lifecycle contract first.

The historical concurrent-migration 503 response body and individual old sidecar failures remain absent from evidence. These changes do not assign an invented historical cause or claim those cases are fixed in runtime execution.

Whole-phase obligations outside this writer packet remain with the lead's canonical ledger, including complete SDK connection-type coverage, server/resource dependency integration, fixture/consumer coverage, and final exact-source inventory. This handoff does not close task 5.4 or authorize a gate.

## Evidence and deferred acceptance

Only source inspection (`rg`, `sed`, `cat`) and authorized `apply_patch` authoring were used. No test, build, compiler, formatting, lint, policy validator, review gate, import probe, Docker action, or verification command ran. There is no passing runtime or compilation result to report.

After the whole-phase inventory and exact-command operator authorization, production acceptance must exercise: canceled shutdown waiters; panic/error during prefix acquisition; canceled snapshot discovery; failed stdio handshake reaping; caller cancellation during a tool call; unknown remote effect after local closure; interaction settlement while persistence is live; Surreal matching-receipt contention; and PostgreSQL rejection/rollback failures. Exact commands belong to the lead's proposed gate, not this source-only handoff.

All guards added here trace to the named source failure scenarios above or the existing trusted-host execution boundary. No unrelated feature was added. Compilation, runtime ordering, failure injection, Windows native-input closure, and historical reproducer outcomes remain unverified because the operator gate is closed.

## Follow-up: execution budgets cannot drop accepted requests

Additional source tracing found that `PreparedMcpServer::call_projected` still passed an owning dispatch future into `interaction_timeout`. Although registry invocation ownership survived that outer waiter's disappearance, the outer method could return before the selected connection's cleanup acknowledgement. The helper also imposed an implicit extra 600-second human-input cap.

This follow-up changes `src/mcp/runtime.rs`, `src/mcp/registry.rs`, and `src/mcp/interaction.rs`:

- Readiness is awaited through its existing acknowledgement contract. The runtime passes its explicitly configured call budget into the retained registry invocation instead of placing a timer around readiness and dispatch.
- `await_interaction_budget` accepts only `Pin<&mut Future>`: it cannot own or discard the actual request. Its expiry result is only a cancellation request to the enclosing owner. Human input pauses the execution budget; the hidden 600-second cap is removed. Durable elicitation expiry remains a separate business-record contract.
- Budget expiry, caller cancellation, or scope cancellation requests the selected connection to close, then awaits both the actual SDK request result and retained service-slot cleanup. A known successful result remains known even if cancellation raced it. Missing dispatched results remain outcome unknown. Cleanup errors are recorded separately in registry and host-scope final reports.
- Direct legacy registry calls no longer install an implicit 30-second budget. Explicit runtime execution budgets remain supported. A tool or transport that cannot acknowledge cancellation can keep the admitted operation pending; elapsed time cannot release it.

No verification was executed for this follow-up. The gate remains **CLOSED**. The remaining required evidence includes budget/response races, a caller dropped after dispatch, human-input pause/resume, and a canceled connection whose physical close stays pending.
