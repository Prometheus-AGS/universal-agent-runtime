# Tasks — add-run-cancellation

## 0. Bootstrap & prerequisites

- [ ] 0.1 **Round 0 gate**: confirm prior-phase branches are merged to `main` (C2 make-config-authoritative-on-boot, C1 worker-pool-graceful-shutdown, C3 persist-builtin-agents, C4 wire-runtime-console-events); `cargo build` + `cargo test` green before any code below
- [ ] 0.2 Create worktree via `scripts/worktree-new.sh add-run-cancellation`
- [ ] 0.3 Confirm `tokio-util` `CancellationToken` is importable (already a direct dep — no Cargo change expected); note the merged C1 graceful-shutdown wiring in `server.rs` that 5.x builds on

## 1. Cancellation token tree (D1)

- [ ] 1.1 Create a process-level root `CancellationToken` at server start; thread it into `RunManager` (or app state) so runs can derive children
- [ ] 1.2 In `RunManager::start_run` (`manager.rs:441`) create `root.child_token()`, store it in `RunStreamState`
- [ ] 1.3 Release the run's token entry on every terminal path (`done`/`error`/`cancelled`) to prevent accumulation
- [ ] 1.4 Add `RunManager::cancel_run(run_id)` that looks up the token and calls `.cancel()`; idempotent for unknown/terminal runs

## 2. Orchestrator propagation (D2)

- [ ] 2.1 Add a `with_cancellation(token)` builder on the orchestrator mirroring `with_tool_approval_gate` (`orchestrator.rs:237`)
- [ ] 2.2 `tokio::select!` the run token against the driver `stream` call and the stream-consumption loop (`orchestrator.rs:376`, `:449`)
- [ ] 2.3 `is_cancelled()` fast-path check at the top of the tool-execution loop (`orchestrator.rs:600`)
- [ ] 2.4 `tokio::select!` the run token against each tool `.await` — MCP/native/sandbox (`orchestrator.rs:685/725/730`); on cancel, abandon the await and stop dispatching remaining tools
- [ ] 2.5 On observed cancellation, exit the loop with a `cancelled` outcome (distinct from `done`/`error`)

## 3. Cancel endpoint + terminal event (D3, D5)

- [ ] 3.1 Add `POST /api/uar/runs/{id}/cancel` in `routes.rs` calling `RunManager::cancel_run`; idempotent success for unknown/terminal runs
- [ ] 3.2 Add a `cancelled` terminal variant to the normalized event enum; emit it through `RunEventEmitter` so it lands in the 512-event replay buffer
- [ ] 3.3 Guard terminal emission with an "already terminal" per-run check so an explicit cancel racing a natural `done` emits only one terminal event

## 4. Pending-approval interaction (D6)

- [ ] 4.1 In `cancel_run`, if the run has a `pending_approvals` entry (`manager.rs:105`), resolve it as aborted so the orchestrator approval await unblocks
- [ ] 4.2 Verify a run paused on the approval gate, when cancelled, terminates `cancelled` without dispatching the tool (spec scenario)

## 5. Client-disconnect last-subscriber-drop (D4)

- [ ] 5.1 Add a drop-guard in the SSE stream closure (`server.rs:~3895`) that, on disconnect, yields then re-checks `broadcast` `receiver_count()`
- [ ] 5.2 Call `cancel_run` only when the re-checked count is zero AND no terminal event has fired (multi-viewer + reconnect safe)
- [ ] 5.3 Verify: one of several viewers disconnecting does NOT cancel; last viewer disconnecting DOES cancel; reconnect via `history_since` is unaffected

## 6. Graceful-shutdown cancellation (D1)

- [ ] 6.1 Cancel the root token at the start of `shutdown_signal` (`server.rs:1140`), before the drain sleep, so in-flight runs abort within the drain window
- [ ] 6.2 Verify SIGTERM with active runs: runs reach `cancelled`, server exits cleanly within the configured timeout

## 7. Frontend (stop button + cancelled state)

- [ ] 7.1 Capture the server `run_id` client-side and add a **Stop** affordance on an in-flight run that calls `POST /runs/{id}/cancel` (respect frontend layering: component → hook → store → service)
- [ ] 7.2 Render the terminal `cancelled` state distinctly from `done`/`error` in the chat stream and Runtime Console

## 8. Validation (gate)

- [ ] 8.1 `cargo check` / `cargo clippy --all-targets` clean (zero warnings per AGENTS.md)
- [ ] 8.2 Unit/integration: explicit cancel aborts an in-flight LLM stream; cancel aborts an in-flight tool; cancel-during-approval aborts cleanly
- [ ] 8.3 Manual: multi-tab last-drop semantics; reconnect race does not over-cancel; SIGTERM drains in-flight runs
- [ ] 8.4 Frontend: `bun run lint` + manual stop-button + cancelled-state render
- [ ] 8.5 `openspec validate add-run-cancellation --strict`; update `.kbd-orchestrator` progress (change #1 of uar-harness-parity)

## Notes

- Cooperative cancellation only — a tool blocked in a non-cancellable FFI/syscall aborts at its next await; the orchestrator still proceeds to terminal `cancelled` (design Risks).
- Out of scope: replacing the 6-keyword `tool_requires_approval` heuristic (→ `uar-safety-and-evals`); cancelled-run persistence/metrics (→ HP2 observability).
- Validation step `cargo build --release` full may be pending if the frontend submodule is absent in the worktree — validate on merge (per prior-change convention).
