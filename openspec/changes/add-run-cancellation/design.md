## Context

Today a run has no cancellation path (assessment D2). `RunManager::start_run` spawns the agent loop with `tokio::spawn` and discards the `JoinHandle` (`manager.rs:887`); the SSE handler (`server.rs:~3895`) consumes a `broadcast::Receiver` with a bare `while let Ok(event) = rx.recv().await` and no disconnect detection; `shutdown_signal` (`server.rs:1140`) just sleeps a drain timer. Dropping the client therefore orphans the run — it keeps calling the LLM driver and dispatching tools (`orchestrator.rs:376/449/685/725/730`) to completion.

There is already a working precedent for an out-of-band, run-id-keyed interrupt: the **approval gate**. `RunManager` holds `pending_approvals: HashMap<run_id, oneshot::Sender<bool>>` (`manager.rs:105`), the orchestrator awaits it via `with_tool_approval_gate` (`orchestrator.rs:237`), and `resolve_approval` (`manager.rs:407`) fires it from an HTTP handler. Cancellation is structurally the same problem and should reuse this seam rather than introduce a parallel mechanism.

Constraints: `tokio-util` (source of `CancellationToken`) is already a direct dependency — no new crate. The run event stream is a multi-subscriber `broadcast` with a 512-event replay buffer and `history_since` late-join support, so any disconnect-driven behavior must be subscriber-count aware. This change is gated behind merging the prior phase's branches to `main` (Round 0) because the assessment baseline (`8b3c503`) predates them — notably the graceful-shutdown wiring (`fix/worker-pool-graceful-shutdown`) and `runtime.*` events (`fix/wire-runtime-console-events`) this change builds on.

## Goals / Non-Goals

**Goals:**
- A run aborts its in-flight LLM and tool work promptly when cancelled — via explicit endpoint, last-subscriber drop, or shutdown.
- Cancellation reuses the approval-gate pattern (run-id-keyed, stored in `RunManager`, fired from an HTTP handler).
- Multi-viewer and reconnect scenarios are safe: a run dies only when nobody is watching it anymore.
- A distinct terminal `cancelled` event is emitted and replayable.

**Non-Goals:**
- Replacing the brittle 6-keyword `tool_requires_approval` heuristic (`manager.rs:249`) — deferred to `uar-safety-and-evals`.
- A new event/lifecycle bus (parking-lot `HookBus`) — explicitly rejected by the assessment as redundant with `RunEventEmitter`.
- Mid-tool rollback/compensation — we abort at await points; tools already in a non-cancellable syscall run to their next await. Best-effort cooperative cancellation only.

## Decisions

### D1 — `CancellationToken` tree, rooted at the process and cancelled by shutdown
Create one process-level root `CancellationToken` at server start. Each run gets `root.child_token()` in `start_run`, stored in `RunStreamState`. `shutdown_signal` cancels the root, which fans out to every run.
- **Why:** child tokens give per-run cancellation + free shutdown fan-out with no bookkeeping. *Alternative:* a `HashMap<run_id, AbortHandle>` from the spawned `JoinHandle` — rejected because `abort()` is abrupt (drops futures at arbitrary points, no clean terminal event) whereas a cooperative token lets the loop emit `cancelled` and unwind tool/approval state.

### D2 — `with_cancellation()` builder mirroring `with_tool_approval_gate`
Thread the child token into `Orchestrator::chat_with_history` through a builder identical in shape to `with_tool_approval_gate` (`orchestrator.rs:237`). Inside the loop, wrap the cancellable awaits in `tokio::select!`:
- the driver `stream` call and the stream-consumption loop (`orchestrator.rs:376`, `:449`);
- each tool `.await` (MCP/native/sandbox at `:685/:725/:730`);
- an `is_cancelled()` fast-path at the top of the tool loop (`orchestrator.rs:600`).
- **Why:** keeps the orchestrator's interruption model uniform (approval + cancellation handled the same way) and localizes `select!` to the genuine await points. *Alternative:* checking `is_cancelled()` only between iterations — rejected because a single long LLM stream or a hung tool call would ignore cancellation for its full duration.

### D3 — `cancel_run` + `POST /api/uar/runs/{id}/cancel`
Add `RunManager::cancel_run(run_id)` that looks up the run's token and calls `.cancel()`; expose it via a new route in `routes.rs`. Idempotent: unknown/terminal runs return success without re-emitting a terminal event.
- **Why:** symmetric with `resolve_approval` + the existing `/tool-approval` route. *Alternative:* a generic `/runs/{id}/control` verb endpoint — rejected as over-general for one action.

### D4 — Last-subscriber-drop auto-cancel (decision R2)
Track the live subscriber count per run. The `broadcast` channel's `receiver_count()` is the source of truth, but it must be sampled in the SSE handler's drop path with a guard so a reconnect (new `history_since` subscription) that races a drop does not trip cancellation. Implementation: a drop-guard future in the SSE stream closure that, on disconnect, re-checks `receiver_count()` after yielding; if it reaches zero (and no terminal event has fired), call `cancel_run`.
- **Why:** the stream is explicitly multi-subscriber with late joiners; first-drop cancel (the naive option) would kill runs other clients are watching. *Alternatives considered:* (a) explicit-stop-only, no auto-cancel — rejected because passive disconnects (closed laptop, dropped network) would still orphan runs; (b) a debounce timer before cancelling on zero subscribers to absorb reconnect races — deferred as a tuning knob (see Open Questions), starting with the immediate re-check.

### D5 — Distinct terminal `cancelled` event
Add a `cancelled` terminal variant to the normalized event enum, emitted through the existing `RunEventEmitter` so it lands in the 512-event replay buffer like `done`/`error`. The frontend maps it to a cancelled run state.
- **Why:** reuses the existing broadcast+replay path (no new bus, per D-non-goals) and makes the terminal state observable to reconnecting clients. *Alternative:* reusing `error` with a flag — rejected; cancellation is a normal outcome, not an error, and conflating them muddies UI + metrics.

### D6 — Cancel resolves a pending approval as aborted
When `cancel_run` fires, if the run has an entry in `pending_approvals`, resolve it (aborted) so the orchestrator's approval await unblocks and the loop can observe cancellation and exit.
- **Why:** without this, a run paused on approval would ignore the cancelled token until the 5-minute approval timeout. Firing both keeps the two interrupt mechanisms consistent.

## Risks / Trade-offs

- **[Reconnect race on last-drop]** A client reconnecting in the gap between drop and the zero-count check could be cancelled erroneously → Mitigation: re-check `receiver_count()` after a yield in the drop guard; only cancel if still zero and no terminal event emitted. A debounce window is available if races are observed (Open Questions).
- **[Non-cooperative tool awaits]** A tool blocked in a non-cancellable FFI/syscall won't abort until its next await point → Mitigation: accept best-effort cancellation; the `select!` abandons the await wrapper so the orchestrator proceeds to terminal `cancelled` even if a detached tool future is still settling. Document this limitation.
- **[Token leak]** Tokens accumulating in `RunManager` for finished runs → Mitigation: release the run's token entry on every terminal path (`done`/`error`/`cancelled`), mirroring how `pending_approvals` entries are cleared.
- **[Double terminal event]** Explicit cancel racing a natural `done` → Mitigation: guard terminal emission with a "already terminal" check per run so only the first terminal event is emitted/replayed.
- **[Shutdown vs drain window]** Root cancellation must fire before the Axum graceful-shutdown drain completes, or runs are still killed abruptly → Mitigation: cancel the root token at the start of `shutdown_signal`, before the drain sleep, so runs unwind within the window (this composes with the C1 graceful-shutdown work merged in Round 0).

## Migration Plan

1. **Round 0 (prerequisite):** merge prior-phase branches C2→C1→C3→C4 to `main`; confirm `cargo build` + `cargo test` green. This change is authored now but implemented only after Round 0.
2. Land the token tree + `with_cancellation()` builder and `select!` points (no behavior change until a cancel source fires).
3. Add `cancel_run` + the cancel route + terminal `cancelled` event.
4. Add the SSE last-subscriber-drop guard.
5. Wire the root token into `shutdown_signal`.
6. Frontend stop button + cancelled-state rendering.
- **Rollback:** the change is additive (new endpoint, new event variant, new token plumbing); reverting the commit restores prior behavior with no schema/data migration.

## Open Questions

- **Debounce on zero-subscriber cancellation:** start with an immediate re-check (D4); do we need a short grace window (e.g. 1–2s) to absorb reconnect churn before cancelling? Decide from observed reconnect behavior after step 4.
- **Cancelled-run persistence:** should a `cancelled` outcome be persisted to run history/metrics distinctly (for a future cancellation-rate metric), or only emitted on the live stream for now? Leaning live-only this change; persistence folds into the observability change (HP2).
