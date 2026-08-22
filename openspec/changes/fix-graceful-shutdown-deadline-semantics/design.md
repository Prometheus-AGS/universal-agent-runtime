## Context

See `proposal.md` for the release blocker and `specs/graceful-shutdown/spec.md` for the `server-full` behavior contract. Today `serve_on_listener` receives the process signal in a detached Tokio task, cancels runtime work, waits for ingestion-pool cleanup, cancels the shared HTTP token, and then makes each Axum shutdown future sleep for the full configured timeout. Axum does not stop accepting connections until that future resolves, so an idle server waits 30 seconds before draining.

Axum 0.8.9 spawns accepted connections as independent Tokio tasks. Resolving its graceful-shutdown signal stops the accept loop and asks them to drain, but dropping the top-level serve future cannot force-close every spawned connection while an embedding runtime remains alive. No new crate dependency or custom Hyper accept loop is permitted in this release child.

The existing caller-owned HTTP cancellation path is intentionally distinct from OS-signal process shutdown. Embedded callers can stop HTTP first while the `server-full` future remains alive awaiting A2A; a later SIGTERM or SIGINT initiates process-scoped cleanup. That distinction must remain observable.

## Goals / Non-Goals

**Goals:**

- Start both HTTP listeners draining as soon as SIGINT or SIGTERM is observed.
- Give cleanup, HTTP/SSE drain, and A2A completion one graceful window measured from signal observation.
- Complete normally when all shutdown work finishes in time; otherwise enforce process exit independently of Tokio executor progress.
- Distinguish `graceful_complete` from `deadline_enforced` and make the forced marker observable even though process exit bypasses destructors.
- Preserve caller-owned HTTP cancellation without terminating the host process.
- Prove idle, active-completion, active-timeout, listener-refusal, SIGINT, sidecar, cleanup, and non-root container behavior locally.

**Non-Goals:**

- Replacing Axum, adding a server-control dependency, or implementing a custom Hyper accept loop.
- Changing the public sidecar API, configuration key, default timeout, provider routing, realtime schemas, or persistence model.
- Redesigning A2A, ingestion-worker, MCP, database-pool, or run-cancellation ownership.
- Claiming shutdown behavior outside the locally verified `server-full` profile.

## Decisions

### 1. Separate process-shutdown coordination from the HTTP drain token

Add a crate-private shutdown coordinator with distinct signals for process shutdown started, cleanup complete, and full process shutdown complete. Retain the existing shared `http_shutdown` token for primary and companion listeners.

On SIGINT or SIGTERM, the signal task marks process shutdown started, cancels the root run token, and cancels `http_shutdown` before starting ingestion-pool cleanup, explicit MCP transport closure, and live-query task shutdown concurrently. Neither cleanup waits for another to start. Both Axum signal futures return immediately after the shared token is cancelled; neither sleeps. The signal task marks cleanup complete only after the blocking worker-pool shutdown and async resource closure return. The outer server path marks full completion only after the HTTP listeners, cleanup, A2A task, ingestion watcher, and embedded datastore lock release have all finished.

Alternative considered: reuse only `http_shutdown`. Rejected because caller-owned HTTP cancellation is not process shutdown and cannot represent process-scoped cleanup completion.

### 2. Use a dedicated OS watchdog thread for the absolute graceful deadline

At OS-signal observation, arm one standard-library watchdog thread with the configured duration and a shared `Mutex`/`Condvar` completion flag. The thread waits independently of Tokio. Normal completion sets the flag and notifies the condition variable. If the timeout expires first, the watchdog enforces process termination.

All blocking ingestion cleanup remains on `spawn_blocking`, but executor starvation or a wedged cleanup future cannot prevent the watchdog from firing. The configured value is the graceful-drain window; forced exit begins at expiry and must be observed within the specification's one-second tolerance.

Alternatives considered:

- A Tokio timeout task: rejected because the timer would share the executor with work it is supposed to bound.
- Starting the timer after ingestion cleanup: rejected because cleanup time would extend the configured window.

### 3. Make forced termination bounded and explicitly non-graceful

At signal observation, arm both the deadline watchdog and an independent hard-stop fuse. At deadline expiry, the watchdog opens the process stderr device with the platform non-blocking flag, makes one single-line write of `UAR_SHUTDOWN outcome=deadline_enforced`, and calls `std::process::exit(0)` without acquiring Rust's ordinary stderr lock or flushing an unbounded buffered writer. The line is smaller than the atomic pipe-write limit. If the sink is backpressured, the write may return `WouldBlock`; the watchdog proceeds directly to exit. The hard-stop fuse calls `std::process::exit(0)` after a bounded sub-second evidence allowance if the first watchdog is itself stalled. The normal path disarms both and logs `graceful_complete` only after the outer server path has observed HTTP, cleanup, and A2A completion.

The emergency writer uses only `std::fs::OpenOptions` plus the existing platform-specific `OpenOptionsExt::custom_flags` API; it adds no crate dependency. A process test deliberately holds the ordinary Rust stderr lock while the deadline expires. A second test backpressures the captured stderr pipe. Both must still exit inside the one-second observation tolerance; when the non-blocking write is accepted, the marker remains mandatory.

The watchdog is armed only by SIGINT/SIGTERM. Cancelling the caller-owned sidecar HTTP token never arms it and never invokes process exit.

Alternatives considered:

- Dropping or aborting only the Axum serve future: rejected because Axum connection tasks are independently spawned.
- Adding `axum-server` or directly owning a Hyper accept loop: rejected by the no-new-dependency and minimum-change constraints.
- Treating forced exit as graceful: rejected because cleanup may be incomplete and that claim would contradict the evidence.
- Returning an error at expiry: rejected because enforcement of the operator's shutdown policy should not turn a controlled termination into a failure exit.

### 4. Define normal and forced cleanup guarantees separately

Every process shutdown immediately signals run cancellation, HTTP drain, ingestion cleanup, configured MCP transport closure, and A2A shutdown. On the normal branch, the coordinator is completed only after those joins finish. The crate-private MCP shutdown path cancels each current `rmcp::RunningService` and waits until its transport is closed, so stdio children observe EOF before normal completion. A held-ingestion barrier test requires MCP EOF to occur before the ingestion barrier is released, proving concurrent initiation rather than eventual sequencing.

SurrealDB 3.x closes embedded connections when all client handles are dropped, but its local router performs datastore shutdown asynchronously and exposes no public awaitable client-close operation. UAR therefore retains and joins its ingestion watcher, and `LiveQueryBus` retains shared cancellation state plus every topic supervisor handle so a crate-private shutdown operation can cancel and join them. The private outer server-lifetime function keeps Tokio alive after the inner server future returns and drops all remaining clients. For a filesystem-backed `surrealkv://` endpoint, normal completion then opens the existing `LOCK` file without creating or truncating it, polls stable `std::fs::File::try_lock`, and immediately unlocks after success. `WouldBlock` means cleanup is still incomplete; any other I/O error is surfaced. The already-armed OS watchdog bounds this wait.

The process harness runs UAR on a dedicated server thread and Tokio runtime. After the server thread joins, the harness publishes `resources-released` but keeps the helper process alive until the parent publishes `allow-exit`. The parent must start a second UAR process on the identical SurrealKV path and observe readiness before allowing the first helper process to exit. This removes OS process teardown as an alternative explanation for the reopened path while exercising the real server composition.

`server-full` enables embedded SurrealDB through `minimal`; it does not enable the optional `postgres-backend`, so SQLx is absent from the resolved profile graph. Redis appears only as a transitive OpenDAL implementation behind `liter-llm`; UAR owns no Redis client or connection. Those resources are explicit profile exclusions, not unobserved cleanup successes. On the forced branch, unfinished active cleanup is permitted only after the graceful deadline expires, and the outcome is deadline-enforced, never graceful or cleanup-complete.

This is an explicit correction to the previously unconditional cleanup wording. It names the uncomfortable condition: a stuck cleanup operation cannot both finish before exit and be bounded by a hard operator deadline.

### 5. Exercise real process behavior with test-only held work

Add crate-local child-process fixtures in `src/server.rs` that run the real `serve_on_listener` path with a small test router and configurable short shutdown deadline. The router exposes a real `text/event-stream` handler that either finishes its stream inside the drain window or remains pending past it. A separate injected registered-cleanup fixture can remain pending past the same deadline. Parent tests start the child, establish the SSE stream or cleanup barrier, deliver real SIGTERM or SIGINT, and observe the process boundary.

Required focused cases:

- idle SIGTERM and idle SIGINT exit normally within one second;
- a real active SSE stream completes within the drain window, while post-signal connection attempts to both listeners are refused;
- a held SSE stream survives until the internal deadline, then the child exits 0 without an external kill, the connection closes, stderr contains `deadline_enforced` when writable, and no graceful-complete marker appears;
- deliberately held registered cleanup independently reaches the same forced branch, exit bound, and marker/label assertions;
- an ordinary stderr lock and a backpressured stderr pipe cannot delay forced exit beyond the one-second tolerance;
- the existing integration sidecar path proves caller-owned HTTP cancellation leaves the child alive and emits no deadline marker until the later OS signal;
- normal-path cleanup tests prove ingestion and A2A completion, a second UAR becoming ready on the same SurrealKV path while the first helper remains alive at a pre-exit barrier, and explicit MCP stdio EOF before normal completion;
- a held-ingestion barrier proves MCP cancellation begins concurrently rather than after blocking cleanup;
- feature-graph evidence proves SQLx and Redis are outside the `server-full` claim.

The immutable failed candidate at SHA `32afa53d510c8b840b3e98b2be9d9f5dee149531`, whose non-root container exited 137, remains the behavioral negative control. A missing helper symbol is not accepted as negative evidence.

### 6. Exercise the Docker margin with held work

Keep UAR's default internal deadline at 30 seconds and set the local Docker stop deadline to 35 seconds. Extend the non-root journey with a held request that remains active through the internal deadline. Record internal and external limits, elapsed time, exit code, `deadline_enforced` outcome, request termination, and Docker event evidence showing no SIGKILL.

Retain a separate idle shutdown in the focused process tests for the one-second contract. An idle Docker exit does not prove the 30/35-second margin and will not be used for that claim.

## Risks / Trade-offs

- [Forced `process::exit(0)` bypasses Rust destructors] → Allow it only after the graceful deadline, define the cleanup exception explicitly, and never label forced exit graceful or cleanup-complete.
- [Forced evidence could block termination] → Use one non-blocking write that never acquires the ordinary stderr lock, plus an independent hard-stop fuse; parent-observed elapsed time and exit status remain authoritative when a broken sink rejects evidence.
- [A Tokio stall could delay enforcement] → Use a dedicated standard-library watchdog thread and test it while async work remains blocked.
- [Timing tests can be flaky under load] → Use short but nonzero internal deadlines with a stated one-second observation tolerance, readiness/handler-start markers, and parent-side monotonic timing.
- [A test-only held route could leak into production] → Compile it only inside the crate's test module and drive the real private listener function without adding a production route.
- [Self-delivered signals differ from Docker delivery] → Use child-process tests for fast semantics and keep the held-work non-root Docker lifecycle as the authoritative external escalation check.

## Migration Plan

1. Correct and independently re-review all OpenSpec artifacts before entering Execute.
2. Preserve the failed candidate's exit-137 evidence and add process fixtures before accepting product behavior.
3. Implement immediate drain, the OS watchdog, outcome markers, and completion wiring with Tier 0 after each source edit.
4. Run focused unit/process/cleanup tests, then update and exercise the local non-root container boundary.
5. Run strict OpenSpec, artifact-refiner, diff, Tier 0, and scoped Clippy gates and write row-form verification evidence.
   Confirm `Cargo.toml` and `Cargo.lock` are unchanged and inspect added Rust visibility so the implementation introduces only crate-private lifecycle APIs.
6. Commit and reflect the child, create a new immutable candidate SHA, and restart the complete 10,800-second local operational-resilience certification from zero. Do not weaken timing limits after a failure.
