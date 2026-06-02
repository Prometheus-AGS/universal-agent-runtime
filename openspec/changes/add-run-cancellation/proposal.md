# add-run-cancellation

## Why

When an SSE client disconnects mid-run, the run is **orphaned, not cancelled** — there is zero cancellation on the run path (`assessment.md` D2). The agent task is `tokio::spawn`ed with its `JoinHandle` discarded (`manager.rs:887`), and the SSE handler just `recv`s on a `broadcast::Receiver` with no disconnect detection, so dropping the client leaves the producer running to completion. The orphaned run keeps calling the LLM and executing tools — including destructive MCP/native tools matched by the `delete|remove|write|drop|truncate|destroy` approval heuristic — burning tokens and taking side-effecting actions after the user has left. This is a live cost + safety defect, the highest-leverage item in the `uar-harness-parity` phase.

## What Changes

- Add a per-run `tokio_util::sync::CancellationToken` created in `RunManager::start_run` (`manager.rs:441`) and stored in `RunStreamState`.
- Add `RunManager::cancel_run(run_id)` and a new endpoint **`POST /api/uar/runs/{id}/cancel`** so a run can be stopped out-of-band — modeled on the existing approval-gate seam (`pending_approvals` + `resolve_approval`, `manager.rs:105/407`), NOT a new bus.
- Thread a child token into `Orchestrator::chat_with_history` via a `with_cancellation()` builder mirroring `with_tool_approval_gate` (`orchestrator.rs:237`). `tokio::select!` the token against the driver stream call (`orchestrator.rs:376`), the stream-consumption loop (`orchestrator.rs:449`), and each MCP/native/sandbox tool `.await` (`orchestrator.rs:685/725/730`); add an `is_cancelled()` check atop the tool loop (`orchestrator.rs:600`).
- SSE drop-guard in the stream handler (`server.rs:~3895`) that cancels the run **only on last-subscriber-drop** — it must count active `broadcast` receivers and respect late joiners reconnecting via `history_since`, so a single viewer leaving does not kill a run others are watching. (Decision R2.)
- Derive every run token from a root `CancellationToken` cancelled by `shutdown_signal` (`server.rs:1140`), so graceful shutdown aborts in-flight runs within the drain window instead of relying on process teardown.
- Emit a terminal **cancelled** event on the normalized event stream so the UI and Runtime Console show a run ended by cancellation (distinct from `done`/`error`).
- Frontend: add a **stop button** on an in-flight run that calls the cancel endpoint, and render the cancelled terminal state.

Non-goal (explicitly out): replacing the brittle 6-keyword `tool_requires_approval` heuristic (`manager.rs:249`) — that Cedar migration is tracked in the deferred `uar-safety-and-evals` phase.

## Capabilities

### New Capabilities
- **`run-cancellation`** — `specs/run-cancellation/spec.md`. Defines the requirements: per-run cancellation token lifecycle, the explicit cancel endpoint, last-subscriber-drop auto-cancel semantics, propagation through LLM + tool execution, shutdown-driven cancellation, and the terminal cancelled event.

### Modified Capabilities
- **`tool-approval-workflow`** — the cancellation propagation interleaves with the approval gate inside the orchestrator tool loop; the spec's requirements gain a cancellation interaction (a pending approval must resolve/abort cleanly when its run is cancelled). Delta spec required only if behavior at the requirement level changes; otherwise leave as implementation detail in `design.md`.
- **`runtime-event-replay-entity-sync`** — the new terminal cancelled event participates in the run event stream + 512-event replay buffer; confirm replay/dedup treats it as a terminal state.

(`runtime-console` consumes the cancelled event for display but its requirements are unchanged — UI-only.)

## Impact

- **Affected code:** `src/uar/runtime/manager.rs` (token lifecycle, `cancel_run`, last-receiver accounting), `src/uar/runtime/orchestrator.rs` (`with_cancellation`, `select!` on LLM + tool awaits), `src/server.rs` (SSE drop-guard, root token, shutdown wiring, response surface), `src/uar/runtime/routes.rs` (`POST /runs/{id}/cancel`), the normalized event enum (cancelled terminal variant), and the frontend chat store/UI (stop button + cancelled state).
- **APIs:** new `POST /api/uar/runs/{id}/cancel`; new terminal `cancelled` SSE event. No breaking changes to existing endpoints.
- **Provider compatibility:** unaffected — cancellation wraps the liter-llm driver call generically; no provider-specific behavior.
- **Realtime state:** runs now reach a terminal `cancelled` state in the live event stream / Runtime Console; the broadcast bus must not cancel a run while any subscriber (including a reconnecting one) is still attached.
- **Runtime UX:** users get a working stop button; disconnects no longer silently burn tokens or trigger side-effecting tools.
- **Dependencies:** `tokio-util`'s `CancellationToken` (already a direct dependency). No new crates.
- **KBD workflow state:** YES — this is change #1 of the `uar-harness-parity` phase; `progress.json` / waypoint already track it. Implementation is gated behind Round 0 (merge prior-phase branches C2→C1→C3→C4 to `main`) because the assessment baseline lacks them.
