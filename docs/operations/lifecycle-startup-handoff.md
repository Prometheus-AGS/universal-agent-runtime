# Startup lifecycle cancellation handoff

## Scope

This handoff records the server-startup slice of the active
`uar-outstanding-issues-v1-1-1::function-first-release-certification` phase.
The verification gate remains closed. The source changes have not been built,
formatted, typechecked, or tested.

## Changed source

`src/server.rs` now keeps startup initialization inside the retained host-owner
future from its first registration through `run_server_with_listener`.

- `startup_cancellation_checkpoint` observes a process-lifecycle cancellation
  request at named startup boundaries.
- `ConfigManager` starts its watcher before it enters server startup, so its
  finalizer is always registered first. A pre-cancelled startup still closes
  that acquired watcher; a cancellation before readiness leaves the scope
  registered for the supervisor to close and it is not published ready.
- Configuration registration errors and startup cancellation errors are now
  captured by the same panic-catching owner path that invokes
  `RuntimeSupervisor::shutdown`. They can no longer bypass finalization through
  an early `?` in the spawned owner task.
- `run_server_with_listener` checks cancellation immediately before registering
  the `startup` scope and immediately after registration before admitting its
  initialization operation. If cancellation wins that race, the registered
  scope remains in the supervisor and no initialization operation is admitted.
- The `startup` scope owns a cancellation observer. A process cancellation
  closes startup admission and emits a structured lifecycle transition while
  the same retained initializer continues to an acknowledged result. Explicit
  checkpoints follow listener, store, realtime, memory, MCP, run-manager,
  worker, transport, and readiness transitions. They prevent the next startup
  step from beginning after cancellation; they never drop a step already
  accepted by the owner.

The caller still owns only its wait. Dropping that waiter cancels the process
token, while the spawned host owner retains initialization and later invokes the
supervisor's retained cleanup result. Cancellation is a request, not proof that
an acquired resource has been released.

## Contract and limits

The contract deliberately does not call `RuntimeSupervisor::shutdown` when a
cancellation request arrives during initialization: startup may already be in a
constructor that will register further resources, and closing the registry at
that point would race acquisition. The owner first waits for the initialization
future to return, then runs supervisor finalization.

This means a permanently silent constructor or accepted external operation can
keep completion pending. The process token makes the request observable at the
implemented boundaries; it does not fabricate a completion acknowledgement or
interrupt an operation whose API has no acknowledged cancellation contract.

There remains an unavoidable cancellation race between a checkpoint and the
following registration. The post-registration checkpoint retains that scope for
cleanup and prevents its next admission. Eliminating every simultaneous
registration would require a supervisor-wide registration-admission gate used by
every server initializer. Such a gate is not added in this server slice.

## Author acceptance scenarios (not executed)

1. Cancel before startup: the pre-existing configuration watcher is registered
   and closed by the retained owner; no startup initialization operation is
   admitted.
2. Cancel after configuration registration: configuration is finalized by the
   retained owner and no server initialization is admitted.
3. Cancel between startup-scope registration and admission: the scope appears
   in the supervisor closure report, but it has no admitted initialization
   operation.
4. Drop a startup waiter during initialization: the host owner continues to the
   authoritative shutdown report; waiter drop does not cancel finalization.
5. Cancel while a store readiness call is pending: the startup scope shows
   admission closed and its outstanding initialization operation until the call
   produces an authoritative result; the owner then closes all registered
   resources.
