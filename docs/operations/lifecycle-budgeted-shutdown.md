# Budgeted shutdown observation

The operator approved this additive host contract on2026-10-21. It limits how
long a caller waits for acknowledgement while the existing supervisor retains
cleanup ownership. It never uses elapsed time to establish resource release.

## Host API

`RuntimeSupervisor`, `RuntimeHandle` and `EmbeddedRuntime` expose
`shutdown_with_budget(Duration) -> ShutdownObservation`. The budget is supplied
by the caller; there is no default duration. Existing `shutdown() -> CloseReport`
continues to await the actual retained result without an observation budget.

For an embedded application, use `EmbeddedRuntime::shutdown_with_budget`: it
cancels the existing run-root token before requesting the supervisor's cleanup.
A bare `RuntimeHandle` is the supervisor capability and does not own that
embedded root token. Normal server/process shutdown continues through its
existing owner. This change adds no sidecar wire command or launcher behavior.

| Observation | Meaning and required caller behavior |
| --- | --- |
| `Pending { shutdown_id, outstanding_resources }` | The caller's observation budget expired. Preserve the identity for correlation; work remains owned. Do not destroy dependencies, assume rollback or report graceful termination. |
| `SnapshotPending` inventory | The retained owner has not yet acquired the resource registry. Inventory is unavailable; this is not an empty resource list or a clean result. |
| `Resources(Vec<ResourceSnapshot>)` inventory | Current snapshots of registered obligations that have not acknowledged successful release. Failed/unresolved resources remain represented alongside active work. |
| `Completed { shutdown_id, report }` | The retained finalization attempt has finished. Inspect `report.released`, failures, unresolved effects and child reports before claiming successful cleanup. Completion can contain failed or unreleased resources. |

An empty resource vector does not change Pending into Completed: the supervisor
must still publish its actual result. Each supervisor has one shutdown identity
shared by repeated, concurrent, budgeted and canceled callers. The identity is
for correlation, not authentication or a receipt for a remote durable effect.

After Pending, a host can await `shutdown()` for the same retained completion
report. It may also explicitly request another budgeted observation on the same
handle; that does not create another finalizer or reset cleanup. Do not implement
a status-polling loop around this API. Existing retained state and completion
notifications remain the event-driven observation mechanism.

## Ownership and ordering

Shutdown initiation registers one retained owner before any registry-lock await
or timed observation. An initiating caller that times out or disappears cannot
cancel that accepted request. The owner subsequently closes admission, captures
the graph and follows its existing dependency order. Watcher and producer joins
remain prerequisites of dependent teardown. Independent branches can finish while
a blocked branch retains the resources it still needs.

The observation timer races only the caller's wait. It does not own the graph
worker, abort a task, kill a child, drop an accepted database operation or release
an engine. An already retained completion report takes precedence at the return
boundary. Outstanding inventory uses retained scope capabilities and does not
await the async resource registry after expiry. Failed release is not removed
from the inventory as if it were successful.

## Limits and evidence

The uncomfortable case remains an uncooperative operation. A budgeted call can
return Pending while the operation continues indefinitely. The caller must keep
the async host alive for retained cleanup to finish. A timer is not a hard
real-time scheduling guarantee; blocking the executor, destroying it or killing
the process cannot produce a graceful release acknowledgement. No automatic
force-stop or cleanup retry policy is added.

This applies to resources registered with the supervisor. It does not attest to
an external host's resources or an upstream telemetry exporter's internals.
Unknown remote mutation outcomes remain unknown after local cleanup.

Source: `src/uar/runtime/lifecycle.rs` and `src/embedded.rs`. Runtime and host
handoffs are `lifecycle-budget-runtime-handoff.md` and
`lifecycle-budget-lead-handoff.md`. The checklist records source completion and
remaining evidence separately. No new tests or verification gates were added.
No compilation, execution, timer-race or shutdown behavior is certified for
this API while verification is CLOSED. The earlier embedding pass predates it.
