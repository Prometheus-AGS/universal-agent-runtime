# Pre-exit SurrealKV lock negative control

Date: 2026-08-22

Profile: `server-full` only. This result transfers to no other profile.

Source base SHA: `32afa53d510c8b840b3e98b2be9d9f5dee149531`

The focused C-12 test was strengthened so the original helper remains alive
after its dedicated UAR server thread and Tokio runtime join. The parent then
starts a second UAR on the identical SurrealKV path before allowing the first
helper process to exit.

Command:

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration \
  live::capability_cases::l4_c12_persistence_round_trip \
  -- --exact --test-threads=1 --nocapture
```

Observed result:

```text
running 1 test
test live::capability_cases::l4_c12_persistence_round_trip ... FAILED

start_server_sidecar exited with error: Failed to initialize SurrealDB at
'surrealkv:///var/folders/.../.tmpTwwFH6/surrealkv': There was a problem with
the datastore: Other error: Database at
/var/folders/.../.tmpTwwFH6/surrealkv/LOCK is already locked by another process

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 93 filtered out
```

This is the required negative control: process exit had been the mechanism
releasing the embedded lock. Joining the UAR server thread/runtime did not
release it while the helper stayed alive.

Read-only ownership inspection identified two UAR-owned background owners:

- `LiveQueryBus::start` spawns one untracked topic supervisor per entity topic;
  every supervisor retains a cloned `Surreal<Any>` client indefinitely.
- the ingestion file watcher is spawned without retaining its `JoinHandle` and
  retains an `IngestService`, which retains the persistence provider.

SurrealDB 3.2.4 documents automatic close on client drop but exposes no public
awaitable client `close`, `disconnect`, or `shutdown` method. Its local router
performs datastore shutdown asynchronously after client channels close. UAR
must therefore stop and join its background client owners, drop the remaining
clients while Tokio is still running, and observe release of the SurrealKV lock
before reporting `graceful_complete`.
