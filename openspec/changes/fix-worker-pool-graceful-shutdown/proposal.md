# fix-worker-pool-graceful-shutdown

## Why
The server cannot be shut down cleanly — operators have had to `kill` it. Root cause is twofold (assessment D1):
1. UAR pins `prometheus_parking_lot` to rev `32b481d6` (`Cargo.toml:253`), which **predates** the fix PR. The required fixes are on upstream `origin/main` HEAD `ebb7c3c`: CR-01 (`retrieve_async` blocking-thread leak → process won't exit), CR-02 (`WorkerPool::shutdown()` honors join timeout + detaches wedged workers → stuck tasks).
2. Even with the bump, UAR never shuts the pool down: `IngestionWorkerPool::shutdown(self)` (`ingestion_worker.rs:405`) is **dead code** — unreachable because the pool is `Arc`-wrapped (`server.rs:771, 840`). The server's "graceful shutdown" is a bare `tokio::time::sleep` (`server.rs:1105-1142`) that never touches worker threads. Two independent pools are created and both leaked.

This is the **blocking defect** for the phase.

## What changes
- Bump `Cargo.toml:253` `prometheus_parking_lot` rev `32b481d6…` → `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0`; verify `shutdown`/`cancellation`/`hooks` modules are available under the `tokio-runtime` feature (add feature flag if gated).
- Fix compile against the new `#[non_exhaustive]` error taxonomy (CR-05) at all `PoolError` match sites.
- **Hoist + consolidate pool ownership**: build ONE `IngestionWorkerPool` before router assembly, clone its `Arc` into both knowledge router states (replacing the two separate constructions at `server.rs:762` and `833`), and retain its `ShutdownHandle`.
- Replace `shutdown_signal()` (`server.rs:1105-1142`) with `prometheus_parking_lot::wait_for_signal()` + a `ShutdownHandle` driven by a `ShutdownPolicy` carrying `config.server.shutdown_timeout_secs` (`config.rs:155`). Drive Axum's `with_graceful_shutdown` from the same signal future.
- Change `IngestionWorkerPool::shutdown(self)` → `async fn shutdown(&self)` (callable through `Arc`) delegating to the pool's timeout-honoring `shutdown().await`.
- Thread a `CancellationToken` into `DocumentIngestionExecutor::execute` (`ingestion_worker.rs:80`); check it between extraction/chunking/embedding (`:140-222`); populate `TaskMetadata.deadline_ms` (currently always `None`, `:386`).

## Impact
- Affected: `Cargo.toml`, `src/server.rs`, `src/uar/rag/ingestion_worker.rs`, `src/uar/api/knowledge.rs`, any `PoolError` match sites.
- Behavior: SIGINT/SIGTERM now drains in-flight ingestion within the configured timeout, cancels/detaches wedged workers, and exits cleanly. No more `kill`.
- Risk: medium-high — touches startup wiring + a dependency bump. Land the bump + compile-fix in the first commit so breakage is isolated.
- Unlocks: the parking-lot `Hook`/`LifecycleEvent` bus reused by `wire-runtime-console-events` (C4) and the cancellation primitives reused by the parity backlog.
