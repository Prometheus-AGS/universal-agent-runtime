# Tasks — fix-worker-pool-graceful-shutdown

## §0 Bootstrap
- [x] Create worktree via `scripts/worktree-new.sh fix-worker-pool-graceful-shutdown`
- [x] Read upstream new APIs in `/Users/gqadonis/Projects/prometheus/prometheus-parking-lot-rs/src/core/{shutdown,cancellation,hooks}.rs`

## §1 Dependency bump (P0 — unblocks all)
- [x] `Cargo.toml:253` rev `32b481d6…` → `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0`
- [x] `cargo fetch` / confirm `core::shutdown`, `core::cancellation` resolve under `features=["tokio-runtime"]` — confirmed (no extra feature flag needed)
- [x] `cargo check` — fixed CR-05 compile breakage: `idempotency_key: None` in TaskMetadata; `kind`/`origin` defaults in test; `surreal_ns`/`surreal_db` in settings_persistence test

## §2 Hoist & consolidate pool ownership (P0)
- [x] Build one `IngestionWorkerPool` before router assembly in `start_server` (after `state` is built, before router `.nest()`)
- [x] Clone `Arc` into both `/api/uar/knowledge-bases` and `/api/knowledge` router states; deleted both inline constructions
- [x] Pool retained as `ingestion_pool_shared: Option<Arc<IngestionWorkerPool>>` visible to shutdown spawn

## §3 Real graceful shutdown (P0)
- [x] Removed `shutdown_signal()` fn entirely
- [x] Replaced with `wait_for_signal()` from `prometheus_parking_lot::core::shutdown` in a spawned task
- [x] Pool `shutdown(&self)` called before oneshot-firing Axum graceful-shutdown
- [x] Axum `with_graceful_shutdown` wired to `shutdown_rx.await` + connection drain sleep
- [x] `IngestionWorkerPool::shutdown(self)` → `fn shutdown(&self)` — callable through `Arc`; delegates to fixed `WorkerPool::shutdown(&self)`

## §4 Cancellation + deadlines for stuck tasks (P1)
- [x] `CancellationToken` added to `DocumentIngestionExecutor` (field + constructor arg)
- [x] Token checked at start of `execute()` (before status update) and between extraction/embedding stages in `process_document()`
- [x] `TaskMetadata.deadline_ms` populated from submit-time `Option<u128>` offset; `with_task_deadline_ms()` builder on pool
- [x] `tokio-util` already a direct dep — no new dependency needed

## §5 Validation (gate)
- [x] `cargo check` clean (SKIP_FRONTEND_BUILD=1) — no errors
- [x] `cargo test --lib -- ingestion_worker` — 3/3 passed
- [ ] `cargo build --release` full — pending (frontend submodule not installed in worktree; validate on merge)
- [ ] Manual: SIGTERM test — pending merge to local dev environment
- [ ] Confirm single pool construction log — pending manual run

## Notes
- `hooks` module not yet integrated (deferred to C4 wire-runtime-console-events)
- `config_integration` test suite failures confirmed pre-existing on main; not introduced by this change
- Worktree: `~/.claude/worktrees/fix-worker-pool-graceful-shutdown`, branch `fix/worker-pool-graceful-shutdown`, commit `86bb7ab`
