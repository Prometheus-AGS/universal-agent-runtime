# SurrealDB lifecycle patch set

This directory is a source patch overlay for the UAR-pinned SurrealDB 3.3.0
crate family. It was copied from the local Cargo registry source cache on
2026-10-20; registry sources were not modified.

Patched crates:

- `surrealdb-3.3.0`
- `surrealdb-engine-api-3.3.0`
- `surrealdb-engine-local-3.3.0`
- `surrealdb-core-3.3.0`
- `surrealdb-kvs-surrealkv-3.3.0`
- `surrealdb-kvs-rocksdb-3.3.0`

The root Cargo manifest activates all six crates through `patch.crates-io`;
their lockfile entries are local source entries. The base crate export records
upstream commit `238bfeb11f5725bebed370167656748df8067595` in
`.cargo_vcs_info.json`. Upstream source provenance:
[SurrealDB source](https://github.com/surrealdb/surrealdb/tree/238bfeb11f5725bebed370167656748df8067595).
The UAR state owner maintains the connection/session patch; the runtime owner
maintains admission and task scopes; the trust/tools owner maintains physical
engine release. Manifest and cross-crate integration belong to the lead.

## Lifecycle contract added here

`Surreal::shutdown()` first closes shared SDK admission and then awaits the
engine close contract. `Stream::close()` sends `KILL` with the stream's original
session, awaits command completion, and removes its local receiver without
waiting for a `Killed` notification. Embedded engines retain and join local
session and notification tasks, await maintenance and index-builder completion,
and propagate flush and close failures. Route-channel engines close the local
command channel; a remote durable operation still needs its own receipt
reconciliation.

## Removal criterion

Remove this overlay only after an upstream SurrealDB release exposes equivalent
shared admission closure, awaited stream closure, joined embedded task shutdown,
error-preserving engine close behavior, explicit transport-loss observation and
generation-checked reconnection with acknowledged session replay. Record the upstream release and the
replacement proof in the lifecycle handoff before removing the Cargo overrides.

## Evidence boundary

This overlay has not been compiled or executed in this phase. The operator gate
is closed. Its maintenance obligation includes native and wasm task ownership,
remote HTTP/WebSocket drivers, both embedded engines, and error preservation.
The presence of an API named `shutdown` is not physical-release evidence.

## Task 5.4 continuation

The `Any` native and WASM HTTP/WebSocket constructors now reuse their typed
transport constructors, so they cannot bypass the retained driver scopes.
HTTP export delivery belongs to a cancelable read-only driver task; delivery
errors remain errors rather than becoming an apparently successful truncated
stream. Stream admission and driver joining share a lock, preventing a late
subscription pump from starting after its owner has acknowledged closure.

The optional gRPC transport owns session, notification and export work separately
from its network executor. The endpoint's custom Hyper executor retains the
Tonic Buffer and HTTP/2 connection futures. Application work drains before the
last channel handle is dropped and those network futures join. This is local
driver completion; it does not prove remote session release. Detailed limits
and pinned-source observations are recorded in
`docs/operations/lifecycle-grpc-continuation.md` in the UAR root.

Index shutdown tracks actual builder tasks independently of the lookup map.
Removing a map entry cannot make a running builder disappear from shutdown
accounting. Rollback waits for builder completion before deleting durable state;
one authoritative cleanup attempt reports contention instead of sleeping and
retrying until a deadline. Existing cleanup-budget arguments are compatibility
inputs and no longer establish completion.

The gRPC executor promotes the existing locked Hyper 1.11.0 resolution to an
optional direct dependency. Core index tracking enables the existing
Tokio-util `rt` feature on native and WASM targets. Neither change upgrades the
pinned dependency base. These changes remain uncompiled and unexecuted.
