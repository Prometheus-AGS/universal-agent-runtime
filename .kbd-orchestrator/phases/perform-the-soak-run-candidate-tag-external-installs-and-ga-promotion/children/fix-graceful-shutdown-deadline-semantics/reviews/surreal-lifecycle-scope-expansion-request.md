# Surreal lifecycle scope-expansion request

Date: 2026-08-22

Status: authorized by the operator on 2026-08-22

## Observed blocker

The accepted pre-exit C-12 design was executed and failed. A second UAR cannot
open the identical SurrealKV path after the first UAR server thread/runtime has
joined while the helper process remains alive. The exact database error is
recorded in
`openspec/changes/fix-graceful-shutdown-deadline-semantics/evidence/pre-exit-surrealkv-lock-negative-control.md`.

This falsifies the follow-up conclusion in `approved-scope-critic.md` that the
dedicated runtime boundary alone makes `src/uar/realtime/surreal_bus.rs`
unnecessary. The critic's original ownership finding was correct.

## Requested write-surface expansion

Add exactly this product path to `scope.json`:

```text
src/uar/realtime/surreal_bus.rs
```

No provider, persistence-trait, dependency, manifest, public API, protocol,
configuration, UI, submodule, or GitHub Actions change is requested.

## Proposed implementation

1. Give `LiveQueryBus` a shared cancellation token and retained topic-task join
   handles. Add a crate-private, idempotent async shutdown operation that
   cancels every topic supervisor and joins every task. Existing clones share
   the same shutdown state. The public `RealtimeBus` trait remains unchanged.
2. Retain the existing ingestion file-watcher `JoinHandle` in `src/server.rs`.
   Abort and join it during normal process shutdown so it cannot retain the
   persistence provider after cleanup completes.
3. Move full-completion ownership to an outer private server-lifetime function.
   The inner server future performs HTTP, ingestion, MCP, live-query, and A2A
   cleanup and then returns, dropping all remaining SurrealDB clients while the
   caller's Tokio runtime is still alive.
4. For a filesystem-backed `surrealkv://` endpoint, poll the existing `LOCK`
   file with stable `std::fs::File::try_lock`. Report normal completion only
   after the exclusive lock can be acquired and immediately unlocked. The
   already-armed OS watchdog remains the bound if SDK cleanup stalls. Network
   SurrealDB and non-Surreal providers skip this filesystem assertion.
5. Re-run the focused C-12 case and its different-path negative control. The
   positive case must start the second UAR before the original helper exits;
   the negative control must still fail because the resource is absent.

## Why this is the minimum correct boundary

Cancelling only HTTP and run work does not stop the live-query supervisors or
ingestion watcher. Sleeping after runtime teardown cannot work because
SurrealDB's asynchronous datastore shutdown needs that runtime. Deleting the
`LOCK` file would bypass the database's safety boundary and is forbidden.
Changing the public persistence trait would widen the release and API surface
without solving the untracked task owners.

The proposed lock observation does not modify or remove the lock file. It uses
the same OS-level exclusive-lock mechanism as SurrealKV and releases the
observer lock immediately. It converts SDK-drop timing from an assumption into
an observed normal-completion condition.
