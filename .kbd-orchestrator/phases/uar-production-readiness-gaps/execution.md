# Execution: uar-production-readiness-gaps

**Date:** 2026-06-02
**Backend:** OpenSpec (claude-code direct implementation)
**Agent:** claude-code

## Dispatch Strategy

Execute in dependency order. C1 is the blocking defect and hook-bus provider for C4:

| Wave | Changes | Strategy |
|---|---|---|
| W1 | C1 `fix-worker-pool-graceful-shutdown` | Serial (P0, blocking, unblocks C4) |
| W2 | C2 `make-config-authoritative-on-boot` ∥ C3 `persist-builtin-agents` | Parallel capable (both independent) |
| W3 | C4 `wire-runtime-console-events` | After C1 merged (needs hook bus) |
| W4 | C5 `add-otel-agent-tracing` | Partial/deferred — scaffold if time |

## Backend confirmation

- OpenSpec: ✅ `openspec/changes/<id>/{proposal.md,tasks.md}` present for all 4 in-phase changes
- QA gate: `/opsx:verify` + artifact-refiner after each change before archive
- QA skip rule: fewer than 3 files → skip (no changes qualify; all touch 3+ files)

## Key API notes (inform execution)

`prometheus_parking_lot` new APIs (after rev bump to `ebb7c3c`):
- `WorkerPool::shutdown(&self)` — now **`&self`**, callable through `Arc` (CR-02 fix)
- `ShutdownHandle::new(CancellationToken)` + `shutdown_with(ShutdownPolicy)` + `shutdown_on_signal(policy).await`
- `wait_for_signal()` — standalone fn: awaits SIGINT/SIGTERM, replaces the manual `shutdown_signal()` in server.rs
- `ShutdownPolicy::DrainThenCancel { deadline }` — default policy, replaces the `tokio::time::sleep` grace period
- `CancellationToken` (re-exported from `tokio_util::sync`) — thread into executor for cancellation

## C1 implementation approach

1. Rev bump: `Cargo.toml:253` → `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0`
2. Fix `PoolError` match sites for `#[non_exhaustive]` (add `_ => unreachable!()` or `..` wildcard)
3. `IngestionWorkerPool`: add `cancellation_token: CancellationToken` field; change `shutdown(self)` → `shutdown(&self)` delegating to `self.pool.shutdown()`; thread token into executor
4. `server.rs`: hoist one pool before router assembly; clone `Arc` into both knowledge states; store `Arc<IngestionWorkerPool>` alongside the server; replace `shutdown_signal()` with `wait_for_signal().await` + pool `shutdown()`
5. Executor: check cancellation token between processing stages

## QA gates per change

All changes require:
- `cargo build --release` clean
- `cargo clippy` clean  
- `cargo test` green
- Behavioral verification per tasks.md §5

## Progress tracking

Changes initialized in progress.json with `changes_total: 4, changes_completed: 0`.
