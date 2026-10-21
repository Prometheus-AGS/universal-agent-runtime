# Lifecycle producer/state continuation

## Scope and phase

The active KBD phase is `function-first-release-certification`, Execute. This note records source-level lifecycle work in the state lane. The operator verification gate is closed. No test, formatter, lint, Cargo command, compiler probe, review gate, build, or acceptance script has run for this work.

## Changed source and current contracts

### `src/uar/persistence/scoped.rs`

`ScopedPersistence` now routes every delegated asynchronous `PersistenceLayer` operation through one retained `dispatch` path. Dispatch first registers a named task with `ResourceScope`, then invokes the underlying provider inside that task and returns its actual result through a one-shot channel.

A caller that cancels its response waiter no longer drops an admitted provider future. Scope closure joins the registered task before the physical provider finalizer can run. Each dispatch creates a `PersistenceCompletion` with the operation identity. A transport error on a mutating operation records an unresolved durable effect; reads are explicitly classified so a canceled snapshot does not claim a mutation outcome. Normal domain errors are returned to the caller and do not become cleanup failures.

All 137 delegated persistence operations use this path. The A2A methods that previously constructed `PersistenceCompletion` with the obsolete two-argument form now use the current method/identity contract through the common dispatch path.

### `src/uar/memory/scoped_storage.rs`

`ScopedMemoryStorage` now follows the same retained-dispatch contract for all 52 `MemoryStorage` methods. Borrowed request data is copied into the retained task before the method begins. This prevents request cancellation from invalidating borrowed strings, vectors, optional filters, or memory references while the storage operation is still live.

`MemoryCompletion` receives the stable dispatch identity rather than allocating a different identity after admission. Mutating operations with a Surreal transport failure retain an unresolved-effect receipt. Read-only methods remain explicitly classified.

This wrapper is the administrative storage boundary returned by `MemoryService`; closing a UAR scope now waits for its accepted storage work instead of treating a dropped caller as completion. It does not close a host-borrowed database engine.

### `src/uar/memory/service.rs`

Memory live-query subscriptions now return `MemoryLiveStream`, which retains its `OperationLease` until the original SDK stream receives close-command completion. Explicit `close()` propagates a close error and records an unresolved obligation. Cancellation while that close command is pending transfers the same stream and lease to one retained close worker. Dropping a stream uses that retained worker too; it does not release the parent lease before command acknowledgement. If a runtime is unavailable for retained cleanup, the lease is recorded unresolved.

Subscription registration is also retained independently of the caller's response waiter. A canceled requester that receives a successfully registered stream causes that stream to enter the same retained close path. Registration errors and panics retain an unresolved receipt rather than representing registration as completed.

The duplicate owned-client capture in `MemoryService::close_scope()` was removed. The existing retained initializer and close result remain the authority for constructor cancellation and repeated close callers.

### `src/uar/realtime/surreal_bus.rs`

A Surreal transport `Available` notification no longer automatically requests a subscription reconnect. Availability is evidence only. A trusted host must issue `request_reconnect`, which advances the subscription generation. `BusLifecycle::generation()` exposes that retained generation for host-side compare-and-swap control. This prevents transport delivery from silently creating a parallel subscription generation while persistence and realtime controls are separately registered.

The existing supervisor still turns stream EOF or stream error into an unavailable topic state, awaits explicit reconnect control, explicitly closes the original stream, and records close failure in the bus lifecycle. `MemoryService::transport_generation()` exposes the active SDK-retained generation without admitting a new operation; `MemorySubscriptionClient::transport_generation()` remains observation-only after normal memory admission closes. The host pairs the active generation with its explicit `reconnect(expected_generation)` command.

### `src/uar/realtime/postgres_bus.rs`

The dedicated PostgreSQL listener pool remains the acknowledgment boundary for listener release. When shutdown is already requested, a failure from listener cleanup is now returned to the lifecycle task instead of being discarded. The listener still uses `eager_reconnect(false)` and waits for an explicit reconnect event after a connection loss.

## Source evidence used

- `ResourceScope::spawn` holds a task-tracker entry and a scope lease until its future returns; its panic and error result are retained in the resource report.
- The pinned SDK stream close implementation uses the original stream session, clears the local receiver, and returns the `KILL` command result. It does not promise a terminal `Killed` notification.
- The first-party surreal-memory source declares `Clone` for every value copied by the generated `ScopedMemoryStorage` dispatch surface: memory, entity, relation, mindmap, task stream, and task-step types.
- `LiveQueryBus` is the only in-tree consumer of `MemorySubscriptionClient`; it closes each returned stream before its worker ends.

## Remaining production integration and limits

**Integration addendum:** the server owner subsequently implemented item 1 in
`src/server/lifecycle_reconnect.rs` and integrated both aliases into the private
control handler. One shared pair gate now orders transport recovery before
subscription generation advancement and actual topic readiness. The Boss
consumer understands the correlated partial-failure response. See
`lifecycle-reconnect-continuation.md`; the original lane handoff below preserves
the earlier ownership transfer and does not describe current unfinished code.

1. `server.rs` currently registers persistence, memory transport, persistence realtime, and memory realtime reconnect handlers separately. The trusted host still needs one authorized sequence that reconnects the owned transport and only then advances the dependent bus generation. This is a server-composition change outside this lane.
2. `PostgresNotifyBus` can acknowledge local listener-pool release, but SQLx exposes no receipt for the outcome of a remote mutation that lost its response. The scoped persistence receipt records that distinction for mutating transport errors.
3. A permanently silent remote peer or an SDK close operation that never resolves remains an outstanding lifecycle obligation. This design exposes it through the retained task/report; it does not replace it with a deadline-based success claim.
4. The memory live-stream drop path needs an active Tokio runtime to begin retained asynchronous close. In a no-runtime drop context it records the close as unresolved rather than fabricating release.
5. The new retained dispatches require the current provider futures and copied payloads to satisfy their Rust `Send + 'static` contracts. Source inspection shows owned values for every wrapper argument, but compilation is intentionally deferred under the phase gate.

## Verification status

No verification command was run. The statements above are source-graph evidence only; compiled behavior, provider wire behavior, engine closure, and sidecar/release acceptance remain unverified.

## Authored integration acceptance (not executed)

`tests/lifecycle_owned_resources.rs` is an integration acceptance target authored for the lifecycle ownership contracts. It uses `RuntimeSupervisor` and `ResourceScope` directly, and its database scenario uses a real `SurrealDbProvider` backed by a unique embedded SurrealKV directory. Its holds and releases are one-shot or blocking-channel barriers; it does not use delays, retry loops, polling, deadline-based assertions, or lock probes.

The target covers five required contracts:

1. Dropping a blocking worker's join waiter does not release that worker's scope. The worker remains admitted until the test-owned release channel permits it to exit, after which shutdown may enter its finalizer.
2. Two concurrent shutdown callers, including one canceled caller, observe the same retained finalizer. The finalizer is released only by an owned one-shot and the test asserts that it was invoked once before a repeated shutdown reads the retained result.
3. A database-dependent producer owns an admitted `ScopedPersistence` call while an engine scope is held as its lifecycle dependency. The real provider call must finish before the owned provider's acknowledged close begins; only after that acknowledgement does the test immediately reopen the same embedded store once.
4. A UAR `SurrealDbProvider::from_existing_client` facade reports only local scope release through the supervisor. Its host-selected embedded SDK client then performs a real query before the host itself acknowledges engine shutdown.
5. When a real owned SDK client has already acknowledged closure through an independently retained clone, the provider's later `close_scope()` failure reaches both the resource and supervisor terminal reports. The supervisor must not report release.

The target has not been compiled or executed because the operator verification gate remains closed. It does not cover remote transport loss and uncertain mutation reconciliation, a physical embedded-engine shutdown fault injection, memory-store closure, or full sidecar process reaping. The public SDK/provider surface has no deterministic physical close-failure injection; the owned-close scenario therefore covers the available production-path condition: an SDK-rejected repeated close after an authoritative earlier closure. Those remaining scenarios require the phase's later whole-system acceptance matrix rather than a narrow lifecycle-owned-resource target.
