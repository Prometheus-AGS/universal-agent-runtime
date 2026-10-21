# Approved embedding-gate state repair — 2026-10-21

## Trigger and boundary

The lead-owned approved embedding Cargo gate stopped with exit 101 during
compilation, before the embedding test began. Its complete diagnostic record is
`/tmp/uar-embedding-gate-complete-diagnostics-20261021.log`. The concrete
failure was E0277 at `tests/embedding_lifecycle_failover.rs:518`, `:524`, and
`:534`: `sha2::digest::Output<Sha256>` does not implement `LowerHex`.

This repair changes source only. No test, build, Cargo command, lint, format,
compiler probe, or review gate was run by this writer.

## Changed paths and interface effects

- `tests/embedding_lifecycle_failover.rs`
  - Adds a private byte-wise `hex_encode` helper matching
    `src/uar/rag/embeddings/fastembed.rs` and uses it for the three SHA-256
    encodings. The fixture still compares actual asset and model-weight
    digests; it adds no dependency and no mock-vector route.
- `src/uar/memory/service.rs`
  - Adds redacted manual `Debug` implementations for the two public opaque
    handles. They report only whether their scope/stream/lease is open and do
    not expose the Surreal handle, stream payload, or lifecycle lease.
  - Retains the crate-private lifecycle and transport-generation accessors,
    with targeted dead-code allowances for trusted-host wiring not selected by
    this build. No lifecycle API was removed.
- `src/uar/memory/scoped_storage.rs`
- `src/uar/persistence/scoped.rs`
  - Removes only redundant Rust 2024 `Future` imports. The retained-worker
    dispatch bounds continue to refer to the Rust 2024 prelude trait; admission,
    completion, and uncertain-write reconciliation are unchanged.
- `vendor/patches/surrealdb-lifecycle/surrealdb-core-3.3.0/src/kvs/ds.rs`
  - Removes three unused imports and limits the archive-outcome helper to its
    existing unit-test users. Datastore node-update, timeout, and shutdown
    production code remains unchanged.
- `vendor/patches/surrealdb-lifecycle/surrealdb-core-3.3.0/src/kvs/tasks/mod.rs`
  - Keeps `FutureExt` available for the wasm `catch_unwind` path and omits it
    only on native builds where it is unused.
- `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/conn/driver_scope.rs`
  - Gates read-only stream-pump admission helpers to the HTTP/gRPC protocols
    that call them. Both native and wasm implementations, cancellation, task
    joining, and error collection remain available for those protocols.
- `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/conn/mod.rs`
- `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/engine/mod.rs`
- `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/engine/remote/ws/mod.rs`
- `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/engine/remote/ws/native.rs`
  - Removes imports with no local call site, retains the WS interval helper
    under its existing protocol feature, and makes the replay-ready helper
    private because only its native and wasm child modules call it. This avoids
    exposing private `SessionState` in a broader signature.
- `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/method/{authenticate,begin,cancel,commit,content,create,delete,export,health,import,insert,insert_relation,invalidate,live,merge,patch,query,run,select,set,signin,signup,unset,update,upsert,use_db,use_defaults,use_ns,version}.rs`
  - Removes only unused `OnceLockExt` imports; `live.rs` also removes its
    unused `Arc` import. No method body or public SDK API changed.

The byte-preserved first-party memory export at
`vendor/git/surreal-memory-server/**` was not edited.

## Remaining limits

This writer did not repair warnings outside the assigned state/SDK scope:
`src/sandbox/session_manager.rs`, `src/server.rs`,
`src/server/lifecycle_reconnect.rs`, `src/mcp/registry.rs`,
`src/session/thread.rs`, `src/uar/realtime/lifecycle.rs`, and
`src/uar/runtime/lifecycle.rs`. The terminal compile failure means neither the
fixture's real local model inference nor its lifecycle assertions have runtime
evidence yet. The lead must renew the full source inventory before running the
exact approved failed-gate rerun.
