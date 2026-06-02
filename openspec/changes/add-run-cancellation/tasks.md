# Tasks — add-run-cancellation

## 0. Bootstrap & prerequisites

- [x] 0.1 **Round 0 gate**: prior-phase branches merged to `main` (C2/C1/C3/C4); `cargo check --features postgres-backend` green on the merge
- [x] 0.2 Work isolated on branch `feat/run-cancellation` (used a branch rather than a worktree to conserve disk — see Notes)
- [x] 0.3 Confirmed `tokio_util::sync::CancellationToken` available under current `tokio-util` features (already used by `ingestion_worker.rs`); no Cargo change needed

## 1. Cancellation token tree (D1)

- [x] 1.1 Root `CancellationToken` on `RunManager` (`root_cancellation`); exposed via `root_cancellation_token()`
- [x] 1.2 `start_run` creates `root.child_token()`, registers it in `run_cancellations: HashMap<run_id, CancellationToken>`
- [x] 1.3 Token removed from `run_cancellations` on every terminal path (done/error/cancelled, incl. graph branch)
- [x] 1.4 `RunManager::cancel_run(run_id)` — cancels the token, idempotent (false for unknown/terminal)

## 2. Orchestrator propagation (D2 — refined, see Notes)

- [x] 2.1 Cancellation threaded into the run via the **consumption-loop seam** (manager) rather than a `with_cancellation()` builder on the orchestrator
- [x] 2.2 `tokio::select!` on the run token vs the orchestrator stream `next()` (drops the in-flight LLM/tool await on cancel)
- [x] 2.3 `is_cancelled()` is implicit: a cancelled token wins the `select!` immediately at the top of each loop turn (`biased`)
- [x] 2.4 Tool execution (MCP/native/sandbox) aborts because it runs inside the orchestrator stream future that the `select!` drops
- [x] 2.5 On cancel the loop exits with a `run_cancelled` flag → terminal `Cancelled` (distinct from done/error)
- [x] 2.6 Graph-execution branch wraps `graph.execute().await` in `select!` for the graph-driven path

## 3. Cancel endpoint + terminal event (D3, D5)

- [x] 3.1 `POST /api/uar/runs/{run_id}/cancel` (`routes.rs`) → `cancel_run`; idempotent `{ "cancelled": bool }`
- [x] 3.2 `NormalizedEvent::Cancelled { run_id }` (`events.rs`) emitted via `RunEventEmitter` (lands in 512-event replay buffer); mapped to `agui.cancelled` (`sse.rs`)
- [x] 3.3 Single-terminal guaranteed structurally: one consumption loop exits exactly once (cancel OR done OR error), so no duplicate terminal event

## 4. Pending-approval interaction (D6)

- [x] 4.1 `cancel_run` removes + fires the run's `pending_approvals` sender (aborts a run parked on the approval gate)
- [x] 4.2 Cancel-during-approval terminates `cancelled` without dispatching the tool (token also drops the gate future)

## 5. Client-disconnect last-subscriber-drop (D4)

- [x] 5.1 `RunDisconnectGuard` (RAII) tied to the SSE stream lifetime; on drop, after a 250ms grace, checks `receiver_count()`
- [x] 5.2 Cancels only when no subscriber remains (`cancel_run_if_no_subscribers`) — multi-viewer + reconnect safe
- [x] 5.3 Guard wired into BOTH SSE paths: the resumable `/runs/{id}/stream` (`routes.rs`) and the chat completion stream (`server.rs`)

## 6. Graceful-shutdown cancellation (D1)

- [x] 6.1 Root token cancelled at the start of the shutdown task (before pool drain + connection drain) in `server.rs`
- [ ] 6.2 Manual SIGTERM-with-active-runs verification — pending a live dev environment (see Notes)

## 7. Frontend (stop button + cancelled state)

- [x] 7.1 Server `run_id` exposed via `x-uar-run-id` response header; captured in `chat-stream-store`; existing Stop button (`enhanced-thread.tsx` → `use-chat-runtime` → `cancelStream`) now calls `cancelRun()` service (explicit, deterministic) in addition to aborting
- [x] 7.2 `agui.cancelled` handled as a terminal event (finalizes the stream cleanly, distinct from done/error)
- [ ] 7.2a Distinct *visual* "cancelled" badge on the message — deferred (minor polish; finishStream already stops the spinner). See Notes.

## 8. Validation (gate)

- [x] 8.1 `cargo check --features postgres-backend` clean; zero new warnings (fixed one new missing-backticks doc warning; remaining clippy items pre-existing)
- [x] 8.2 Backend: 218 lib tests pass, no regressions. Dedicated cancellation integration test deferred — requires a mock-LLM `RunManager` harness that does not yet exist (see Notes)
- [ ] 8.3 Manual: multi-tab last-drop, reconnect race, SIGTERM drain — pending live dev environment
- [x] 8.4 Frontend: touched files (`chat-stream-store.ts`, `chat-stream-api.ts`) pass `eslint` clean; `tsc` errors are all pre-existing in unrelated files
- [ ] 8.5 `openspec validate add-run-cancellation --strict`; update `.kbd-orchestrator` progress (done in this change's wrap-up)

## Notes

- **Design refinement (D2):** the design proposed threading a child token into the orchestrator via a `with_cancellation()` builder and `select!`-ing each individual await. During implementation the architecture showed the orchestrator runs as an `async_stream` **consumed by the manager's spawned task**, so a single `select!` at the consumption boundary (token vs `stream.next()`) drops the orchestrator future and aborts whatever await (LLM stream, tool call, or approval gate) it is parked on. This is a cleaner, more robust single-seam realization that satisfies all `run-cancellation` spec requirements without touching `orchestrator.rs`. There are two event models (`crate::normalized` in the orchestrator vs `uar::domain::events` for SSE); the `Cancelled` variant lives in the latter and is emitted by the manager.
- **Worktree vs branch (0.2):** used branch `feat/run-cancellation` instead of a worktree to avoid a ~10 GB worktree right after a disk cleanup. Convention otherwise unchanged.
- **Cooperative cancellation only:** a tool blocked in a non-cancellable FFI/syscall aborts at its next await; the orchestrator still proceeds to terminal `Cancelled`.
- **Out of scope:** replacing the 6-keyword `tool_requires_approval` heuristic (→ `uar-safety-and-evals`); cancelled-run persistence/metrics (→ HP2 observability); distinct cancelled-badge visual (minor polish).
- **Pending manual verification (6.2, 8.3):** SIGTERM drain and multi-tab/reconnect behavior need a live dev environment with a real LLM run to exercise; not runnable headlessly here.
