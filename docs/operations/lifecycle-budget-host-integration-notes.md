# Budgeted host-shutdown integration notes

## Existing host surfaces

`src/embedded.rs` is the smallest appropriate public host surface. Its
`EmbeddedRuntime::shutdown` first cancels the run-manager root token and then
awaits `RuntimeSupervisor::shutdown`. That ordering stops new run work before
the supervisor drains the retained resource graph. Borrowed persistence and
borrowed memory remain host-owned because their existing finalizers close only
their UAR facades.

`EmbeddedRuntime::lifecycle` returns a `RuntimeHandle`, but that handle retains
only the supervisor `Arc`. It does not retain `EmbeddedLifecycle` or its
run-cancellation token. A budgeted embedded-host operation therefore must stay
on `EmbeddedRuntime` (or receive an internal, equally trusted cancellation
action); delegating directly to the current `RuntimeHandle` would omit the
existing run-cancellation step.

`src/uar/runtime/lifecycle.rs` owns the retained finalizer semantics. Once
`ResourceScope::close` has started, it spawns the finalizer and later callers
only observe the retained report. `RuntimeSupervisor::shutdown` similarly
publishes a retained supervisor finalizer before awaiting its report. A
caller-budget API must ensure that initiating that retained work itself is not
cancelled by the caller's deadline: start it in host-owned retained work, then
race only the observation wait against the caller budget. A pending result
must keep an observation capability, not fabricate release or abort engines.

The current `EmbeddedLifecycle::Drop` is deliberately fallback-only: it cancels
the root token and, only when a Tokio handle exists, spawns an unbounded
supervisor shutdown. It logs an explicit missing acknowledgement outside Tokio.
The budgeted API must not rely on `Drop` for either initiation or completion;
the caller needs an explicit retained observation path while the runtime is
still alive.

## Server and sidecar constraints

`src/server.rs` owns server-wide intake shutdown: its handler cancels the run
root and HTTP intake, joins server cleanup, and then the server owner awaits
the supervisor report. `ShutdownCoordinator::begin` records shutdown start;
the current `Duration` is only carried for server logging and is not a release
deadline. A host observation budget must never replace that owner or force a
partial engine close.

`src/server/lifecycle_control.rs` exposes only trusted-pipe `snapshot` and
generation-checked `reconnect`. `src/bin/uar-sidecar/control.rs` routes those
commands over the inherited authenticated launch pipe, while parent EOF already
cancels the sidecar process token. No sidecar protocol expansion is needed for
an embedded-host budgeted shutdown: adding a sidecar shutdown command would
create a second initiation path that bypasses the server's process-token,
listener, and cleanup ordering.

If a future requirement explicitly needs budgeted server-process observation,
it should wrap the existing server owner/coordinator rather than expose raw
supervisor shutdown through `ServerLifecycleControl`. That design is outside
the approved embedded-host capability.

## Minimal implementation boundary

1. Add the retained, caller-budgeted observation mechanism beside
   `RuntimeSupervisor`/`RuntimeHandle` only if it can start shutdown before
   returning `Pending` and preserve `CloseReport` retrieval.
2. Expose the embedded-host entry point in `src/embedded.rs`, where it cancels
   the existing run token before asking the retained lifecycle mechanism to
   close.
3. Keep `EmbeddedRuntime::shutdown` unbounded and unchanged as the completion
path; do not alter ownership flags, resource finalizers, server lifecycle
control, or the sidecar wire format.

These are source observations only. No runtime behavior, cancellation race, or
shutdown acknowledgement has been verified in this research pass.
