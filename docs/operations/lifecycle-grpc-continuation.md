# gRPC SDK lifecycle continuation

## Scope and authority

This source-only continuation belongs to
`uar-outstanding-issues-v1-1-1::function-first-release-certification`.
The operator approved the narrowly scoped project-owned SurrealDB lifecycle
overlay under `vendor/patches/surrealdb-lifecycle`; this is not an edit to an
installed registry source, dependency pin, submodule, or the separate pinned
Surreal Memory export.

Verification remains closed. No test, build, formatter, lint, typecheck,
compiler probe, review gate, or acceptance script ran.

## Changed source and contract

`vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/engine/remote/grpc.rs`
now gives each `GrpcEngine` one retained `DriverScope`. The session lifecycle
worker is registered with that scope, and live-notification and byte-export
delivery pumps are registered as cancelable read-only streams. Shutdown closes
the retained session receiver before joining the driver. `async_channel`
receiver closure stops new session events while allowing events already accepted
by the queue to drain. The session worker then calls the existing
`SessionRegistry::close_and_drain` API and acknowledges each remaining remote
session detach attempt.

The engine records detach transport failures separately while continuing local
cleanup. Its `SurrealEngine::shutdown` joins all owned tasks, consumes those
failure records, then drops its final `SurrealDbServiceClient<Channel>` from a
`Mutex<Option<_>>`. Subsequent internal access receives a lifecycle-closed
error.

The endpoint now receives a custom `hyper::rt::Executor` backed by a second,
separate `DriverScope`. Pinned tonic 0.14.6 passes its endpoint executor to
both `Channel::connect`, which launches the channel `Buffer` worker, and the
Hyper HTTP/2 connection builder, which launches the connection future. On
startup failure after either task can exist, the local client is dropped and
the network scope joins before the error returns. On normal shutdown, the
session and stream scope joins first, then the final client handle is dropped,
then the network scope joins. The two scopes never await themselves.

This is an acknowledgement of the local Buffer and HTTP/2 driver futures
registered through tonic's endpoint executor. A failed remote detach remains
an explicit shutdown error and does not become a claimed remote release.

Each live-query subscription now reserves a UUID-indexed child scope before
the subscribe RPC. The child scope admits its delivery pump only while still
open, so a concurrent `kill` or engine shutdown cannot leave a late stream
unowned. `kill` waits for the remote RPC outcome, then closes and joins the
specific local pump without waiting for a `KILL` notification that the protocol
does not promise. A failed RPC still closes the local pump and returns the
remote error, preserving that remote state may require reconciliation. Engine
shutdown drains and joins all remaining live-pump scopes concurrently with its
other local task scope before releasing the client.

## Source-only scenarios

1. Closing a gRPC engine prevents new session lifecycle events, drains an
   accepted attach/clone/drop sequence, then waits for its task before dropping
   the final client handle.
2. A live-query or export receiver that remains blocked when shutdown begins is
   canceled through its retained read-only driver scope and joined before local
   client release.
3. A remote detach lost to transport failure is retained as a close failure
   while the local client handle is still relinquished. The patch does not
   infer the server-side session state.
4. A connection whose capability handshake fails drops its only local client
   handle and joins already registered transport tasks before returning the
   connection error.
5. A retained SDK facade after closure reaches the shared admission gate before
   this engine and cannot reopen the cleared client slot.
6. `KILL` succeeds only after its matching local delivery pump has stopped and
   joined; it never waits for an assumed terminal stream notification.
7. A `KILL` transport or server error still closes and joins the local pump,
   while the returned remote error preserves the unresolved remote outcome.

## Remaining limits

Tonic's channel API does not expose a peer-close acknowledgement. Joining the
registered local futures establishes that the local Buffer and HTTP/2 driver
futures completed; it does not establish physical TCP close at the peer or
remote-session release. A permanently silent detach remains a reported
outstanding remote outcome; the implementation does not use a timer to
fabricate a result.
