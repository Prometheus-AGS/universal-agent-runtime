# Lifecycle state implementation handoff

## Scope completed by the persistence/state lane

This handoff records source changes only. The active KBD phase is
`function-first-release-certification`; the operator verification gate remains
closed. No test, format, lint, typecheck, Cargo command, build, or review gate
ran for this work.

### UAR persistence interface

`src/uar/persistence/providers/surreal.rs` now owns the one parsed endpoint
model used by provider connection setup and lifecycle reporting:

- `SurrealEndpoint::parse(&str)` distinguishes embedded files, embedded memory,
  remote transports, and a borrowed host client. It lowercases URI schemes
  without changing path or authority casing. `rocksdb://` remains a recorded
  compatibility alias that connects through the established SurrealKV path.
- `SurrealEndpoint::path_identity()` canonicalizes an existing raw ancestor
  before appending the missing suffix. It deliberately avoids lexical
  simplification before symlink resolution, and preserves leading `..` in its
  fallback identity.
- `PersistenceOwnership` identifies owned embedded, owned remote, and borrowed
  host scope. `SurrealDbProvider::close_scope()` returns
  `PersistenceCloseReport`, separating scope release, local connection release,
  embedded engine acknowledgement, remote durable uncertainty, and failure.
  Only owned providers call `Surreal::shutdown()`.

The server lifecycle authority must call `close_scope()` only after dependent
operations have drained. A successful report is not a receipt for uncertain
remote writes.

### SDK source patch overlay

`vendor/patches/surrealdb-lifecycle/` contains copied SurrealDB 3.3.0 registry
sources plus a narrow lifecycle overlay. It is inert until the lead adds the
corresponding Cargo patch overrides and lockfile update. Registry sources were
not edited.

The overlay provides:

- `Surreal::shutdown().await`, which closes SDK admission shared by clones and
  awaits the engine close operation.
- `Stream::close().await`, which sends `KILL` under the original stream
  session, awaits command completion, then removes the local receiver. It does
  not await an assumed `Killed` notification.
- engine shutdown capability across the SDK and local engine, route sender
  closure for remote client scope release, retained/joined embedded session and
  notification tasks, acknowledged maintenance/index-builder draining, and
  propagation of SurrealKV and RocksDB flush/close failures.

The overlay does not make a remote server's already submitted mutation known.
Callers must preserve an operation receipt or outcome-unknown result. It also
cannot force an uncooperative engine task to acknowledge termination; shutdown
remains pending rather than treating elapsed time as release.

### First-party Surreal Memory change

The UAR-pinned upstream base commit exists and was used in an isolated worktree:

`/Users/gqadonis/.claude/worktrees/uar-memory-lifecycle`

Local commit `9e36b1cd8b83a657366eb1e3ef8cd837178336de` is based directly on
`432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8` through its sole predecessor
`3a767608d53481ebc3c9085b8f5b4b82316ad8cb`, with no push and no vendored-export
edit. It adds `SurrealLifecycleMode::HostManaged`, a host connection constructor,
explicit failure/replacement publication, no autonomous retry/deadline outcome
logic while host managed, ownership-aware close behavior, and owned-client
cleanup when storage readiness fails after connection. The lead must advance the
pinned export only to this exact commit and reconcile the SurrealDB SDK version
before enabling it.

UAR exposes this through `MemoryService::from_host_connection(config, db,
embeddings)` and `MemoryService::close_scope()`. The typed
`MemoryCloseReport` records ownership, local scope release, owned-engine
acknowledgement, and failure. Closing the facade marks its storage unavailable,
so a retained UAR `MemoryService` cannot continue using a borrowed host client.

## Integration still owned by the lead

1. Add patch activation for the six overlay crates, update the lockfile, and
   resolve any dependency graph/version mismatch without changing unrelated
   pins.
2. Register provider and memory close reports in the host supervisor, including
   ordering of streams, task scopes, and database-dependent finalizers.
3. Use `SurrealEndpoint::path_identity()` to reject independently owned memory
   and persistence stores that resolve to one physical directory before opening
   the second store.
4. Integrate the pinned surreal-memory commit through the existing export path;
   do not copy its source into `vendor/git/surreal-memory-server` by hand.
5. Define remote transport-loss and explicit reconnect events at the UAR
   supervisor boundary. The SDK overlay only closes local route admission.

## Unverified and uncomfortable limits

No compilation or runtime behavior is claimed. The source overlay changes
internal SurrealDB crate contracts and must be compiled only at the authorised
whole-phase gate. Cancellation of an SDK `shutdown()` caller is not itself a
termination acknowledgement; the UAR supervisor must retain finalization so it
survives a waiter cancellation. A permanently silent remote peer or engine task
can leave an outstanding obligation indefinitely, which is reported rather than
converted to a timeout-based success.

### Later source additions in this lane

`PersistenceLayer::list_sessions(owner_id)` is the owner-scoped snapshot port
for authenticated thread clients. The InMemory, Postgres, and Surreal providers
select only records whose serialized owner is `owner_id`, then validate that
owner after deserialization. It deliberately does not migrate or expose legacy
ownerless rows during a list operation, so a snapshot cannot become an
all-tenant fallback. `ScopedPersistence` must delegate this method under its
existing admission lease before it is consumed by the server snapshot route.

`PersistenceLayer` now carries two retention scheduling methods in
`src/uar/persistence/mod.rs`:

- `subscribe_persisted_session_changes()` returns a local successful-commit
  generation receiver where the provider can supply one.
- `next_persisted_session_expiry(idle_timeout)` returns the earliest strictly
  future persisted idle deadline.

The Postgres and Surreal providers implement both methods. A notification is a
wakeup only; the runtime must query the deadline again and revalidate any
removal transaction. The methods intentionally make no completion claim about
an overdue or in-flight session.

`MemoryService::subscription_client()` returns a table-scoped,
admission-checked `MemorySubscriptionClient`. It permits `subscribe_memory()`
and transport-state observation without exposing a general raw memory database
handle. The lifecycle supervisor must stop the resulting bus and close its
streams before `MemoryService::close_scope()`.

`MemoryService::borrowed_scope().await` creates a separately admitted,
`HostBorrowed` facade over the source service's already-selected client and
embedding configuration. Closing that facade only releases its local storage
scope; it does not shut down the source facade or its engine. Each facade
rejects new operations after its own `close_scope()`.

Each facade also owns a local `ResourceScope`. Owner-bound memory access and
the public administrative operations acquire an `OperationLease` before they
call storage. `close_scope()` closes this admission gate and awaits the scope's
accepted operations before closing storage or an owned engine. The trusted-host
`admin_storage()` compatibility accessor returns a `ScopedMemoryStorage`
capability, not the underlying `SurrealStorage`. Every method of the pinned
`MemoryStorage` trait registers an operation lease before delegation. A dropped
waiter records an outcome-unknown identity only for a mutation; snapshots and
searches have no durable effect to reconcile. A completed mutation records the
same uncertainty only when its error chain contains the SDK's typed connection
error. Its `as_any()` returns the wrapper itself, so administrative callers
cannot downcast through the lifecycle boundary to retain a raw client.

`close_scope()` starts one retained close task and publishes one retained
`MemoryCloseReport` to all callers. Dropping a close waiter does not cancel the
scope drain, storage closure, or owned-engine shutdown. A close-task panic is
published as a failed report rather than leaving waiters to infer completion.

### Retained post-acquisition construction

`PostgresProvider::new`, `SurrealDbProvider::new_with_auth`, and
`MemoryService::new` now transfer their post-acquisition initialization into a
retained Tokio task before awaiting its one-shot result. If the startup waiter
is canceled while migrations, authentication, schema initialization, or memory
control setup is still running, that task continues and releases an
undelivered successful provider. Panics in that block become a construction
error after the owned pool or SDK client has been closed. Surreal and memory
preserve both the initialization error and any SDK shutdown error in the
returned error context. SQLx's `PgPool::close` has no failure result, so its
cleanup is awaited but cannot contribute a second error.

`MemoryService::from_host_connection` uses the same retained delivery rule for
its borrowed facade. It closes an undelivered local memory scope, never calls
SDK shutdown on the supplied host client, and leaves host ownership unchanged.
The delivery boundary is the one-shot handoff: once it succeeds, the caller has
the resource and must register it with the host supervisor; a canceled waiter
before that handoff remains the initializer's cleanup responsibility.

`MemoryService::reconnect(expected_generation)` is an admitted, explicit
transport action. It forwards one generation-checked request to the retained
SDK router and returns the acknowledged generation. It rejects after memory
admission closes and contains no automatic retry or reconnect loop.

Memory expiry now has two local lifecycle ports:

- `subscribe_committed_memory_changes()` returns a successful-local-commit
  generation receiver.
- `next_expiry_due_at()` returns the earliest strictly future `memory.valid_until`
  or durable capture-control `expires_at` deadline.

`run_ttl_worker` consumes those ports. It waits for a durable change, shutdown,
or the one actual business expiry date; after that date it awaits
`expire_stale()` before calculating another deadline. Its legacy interval
argument remains source-compatible but does not cause polling. Writes that can
change an expiry boundary publish the local generation only after their durable
operation succeeds. An expiry failure or an unchanged past-due row waits for a
committed change or shutdown before any next due-time read, avoiding a
zero-duration retry loop. The local generation does not replace the
memory-store realtime bus for changes made through another client.

The SDK overlay additionally exposes `Surreal::subscribe_transport_state()`
and `Surreal::transport_generation()`. WebSocket loss publishes
`Unavailable`, fails registered work, and performs no reconnect retry or
keepalive poll. `Surreal::reconnect(expected_generation)` and router-control
plumbing have been introduced as source API, but the retained-session router
control loop still needs completion before this method is usable. It must retain
the route/session registries while unavailable, reject new work deterministically,
and replay only after a single host-authorized successful connection. A fresh
router replacement would lose clone/session state and is not an acceptable
substitute.

### Explicit WebSocket reconnect completion

Both native and WASM WebSocket routers now implement the same retained-router
control protocol. `Surreal::reconnect(expected_generation)` sends one command
to the existing router. While unavailable, the router rejects new operations,
queues session-lifecycle events, and keeps the established session registry.
A matching-generation request attempts one connection, replays the retained
session state, advances the generation, and acknowledges the new generation.
A failed attempt remains unavailable and reports that failure; no loop, retry
sleep, ping interval, or automatic reconnection is used. A stale generation or
a closed engine is rejected. Pending work interrupted by loss receives an
outcome-unknown transport error; durable callers must reconcile through their
own operation identity and receipt.

### Remote HTTP lifecycle completion

The HTTP native and WASM routers now share the router's retained
`TransportState`. A request that loses its transport while the result may have
been applied is returned as a connection error whose message preserves the
outcome-unknown condition. That transition closes HTTP admission for the
facade; subsequent routes are rejected, and recovery requires a trusted host
to construct a new client rather than the SDK retrying work. A received HTTP
status remains an authoritative server response and does not manufacture a
transport-loss transition.

HTTP router tasks and their per-request workers remain owned by `DriverScope`.
`Surreal::shutdown()` closes the route sender through `LifecycleEngine` before
joining that scope, so retained client clones cannot keep the route receiver
alive. The WASM constructor now joins a driver whose startup probe failed
before it returns the startup error. No timeout, polling loop, or reconnect
backoff is involved.

### Owner-authorized ingestion status SSE

`GET /api/uar/knowledge-bases/{id}/documents/{doc_id}/ingestion/events` and
its `/api/knowledge` alias now provide an owner-authorized ingestion status
stream. The route subscribes to the internal `runtime_principal_ingestion_jobs`
topic before it reads the existing owner-scoped ingestion job. The initial
`ingestion.status` event carries that authoritative `IngestionJob`, the topic
generation, and readiness. A normal committed wakeup, a lifecycle reset, or a
consumer-lag signal triggers the same owner-scoped read before another status
is emitted.

The route treats the private topic as an invalidation source only. It never
serializes its `LiveEvent` ID or data: PostgreSQL notifications intentionally
use an opaque wildcard ID, and a Surreal live event may represent another
principal. Subscription unavailability, lifecycle transition, consumer lag,
source closure, a missing job, or a failed authorized read emits
`ingestion.gap` with `requires_snapshot: true`; source closure and failed
authoritative reconciliation end the stream. The stream has no polling loop,
deadline, or keepalive-based correctness condition.

Author acceptance scenario, not executed: create an owner A ingestion job,
subscribe to its events before reading its status, and transition that job.
The stream must produce an initial owner A status and a re-read status. A job
for owner B may wake the private topic but must only cause an owner A lookup;
its ID, payload, and status must not appear in owner A's stream. A forced bus
reset or receiver lag must produce `ingestion.gap` followed by an authorized
owner A reconciliation, while a closed source must produce the closure gap and
end without any retry timer.
