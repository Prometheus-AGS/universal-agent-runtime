# Watcher shutdown ordering handoff

## Scope

This records the watcher-order correction in the active
`uar-outstanding-issues-v1-1-1::function-first-release-certification` phase.
Verification remains closed. No test, build, formatter, lint, typecheck, or
review command has run.

## Changed files and contracts

- `src/server/lifecycle_graph.rs` makes `configuration` the first close stage
  and `ingestion-files` the second. Each watcher scope depends on the resources
  that must remain available until its close acknowledgement. The former
  reverse edges from persistence and memory to configuration are removed.
  Process-signal observation is added as a prerequisite of every other present
  scope, keeping it alive through teardown.
- `src/server.rs` registers HTTP and A2A gRPC as external intake through
  `RuntimeSupervisor::register_external`. The supervisor closes those scopes'
  admission atomically when host shutdown begins, before finalizer ordering;
  admitted requests still drain under their existing scopes.
- `src/config_manager.rs` admits every explicit and filesystem reload through
  the watcher scope. `close_watchers` closes that admission before cancelling
  and joining. Its watcher loop gives stop precedence over a new filesystem
  event, while an already-admitted reload remains awaited and publishes only its
  actual result.
- `src/uar/runtime/skills/watcher.rs` moves `RecommendedWatcher` ownership into
  the lifecycle-owned event task. Scope closure now waits for the task that
  releases the native watcher before it produces a close report. The stop paths
  have priority over another queued event but do not interrupt an accepted
  refresh. The native watcher and its bounded event receiver are placed in one
  construction guard before native registration. That guard closes the receiver
  before dropping the watcher, including registration errors, rejected task
  admission, cancellation, and panic unwinding. It releases a callback blocked
  in `blocking_send` before `RecommendedWatcher` joins its native thread.

The graph is intentionally dynamic over registered resource IDs. A newly added
runtime resource remains downstream of the two known watcher stages without a
separate static list. This is a source-level ordering contract, not evidence of
the historical embedded-store lock root cause.

## Ordering

1. The supervisor atomically closes HTTP and A2A intake admission when shutdown
   starts.
2. The configuration watcher stops and its native watch task joins.
3. The ingestion file watcher stops and joins.
4. Remaining runtime and persistence scopes finalize under their existing DAG.
5. OS signal observation releases only after every other registered scope.

Immutable `ConfigManager::current()` snapshots remain readable throughout this
sequence. Watcher shutdown only stops future reload intake.

## Source-only scenarios (not executed)

1. A shutdown concurrent with an HTTP request rejects later requests through
   the external scope while the admitted request retains its lease.
2. A shutdown concurrent with `/api/config/reload` closes reload admission; an
   already admitted reload completes or fails before configuration watcher
   release is acknowledged.
3. A filesystem event queued beside watcher cancellation does not start another
   reload or skill refresh after stop wins; a refresh already executing is
   joined before native watcher release.
4. A database finalizer cannot become graph-eligible until configuration and
   ingestion watcher close reports have completed.
5. A skill callback blocked because all 64 event slots are full observes the
   receiver closing during shutdown before native watcher drop attempts to join
   that callback. The construction lease keeps a concurrent scope close pending
   until this native-registration ownership is transferred to or released by
   the tracked task.

## Limits

`ingestion-files` is the server-owned file-ingestion producer. The skill file
watcher now has correct scoped ownership, but this server composition does not
currently instantiate it, so no additional server resource is registered for
that optional compatibility API. A permanently silent admitted reload or file
operation still keeps its watcher closure pending; elapsed time is not treated
as completion.
