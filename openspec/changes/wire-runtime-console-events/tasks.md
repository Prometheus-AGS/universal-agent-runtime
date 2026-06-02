# Tasks — wire-runtime-console-events
# Depends on: fix-worker-pool-graceful-shutdown (C1 hook bus)
# Commit: 50d4a23 on branch fix/wire-runtime-console-events
# Worktree: ~/.claude/worktrees/wire-runtime-console-events

## §1 Emit run/tool-call events via SSE (P1)
- [x] Add `to_runtime_entity_event()` to `src/uar/api/sse.rs` mapping NormalizedEvent variants
  (RunStart → run_started, RunDone/RunDoneWithUsage → run_finished, Error → run_failed,
  ToolStart → tool_call_started, ToolEnd → tool_call_finished/tool_call_failed,
  ToolCallApprovalRequired → approval_requested) to `runtime.*` SSE events
- [x] Emit `runtime.*` events in `process_stream_event` macro in `server.rs` alongside `agui.*`
- [x] `RuntimeStep` emission deferred — no per-step NormalizedEvent variant exists; would require
  orchestrator instrumentation (future phase)
- [x] Parking-lot `Hook`/`AuditSink` bus integration deferred to C5/next phase (hooks module not
  yet wired into UAR's run path at time of implementation)

## §2 Real HITL approvals (P1)
- [x] `POST /api/uar/runs/{run_id}/approval` endpoint added in `server.rs`
- [x] Wires to existing `RunManager::resolve_approval()` (already implemented)
- [x] 200 response if run found and resolved; 404 if no pending approval
- [x] Approval entity emitted via `to_runtime_entity_event` on `ToolCallApprovalRequired` event

## §3 Frontend Approve/Deny handlers (P1)
- [x] `RuntimeApprovalsPage` in `runtime-console-page.tsx`: real `onClick` on Approve/Deny buttons
- [x] `resolveApproval()` callback: optimistic entity upsert → POST to approval endpoint → revert on error
- [x] `ingestRuntimeEvent` imported in `chat-stream-store.ts`; `runtime.*` SSE blocks dispatched to entity graph

## §4 Gate un-backed panels (P2)
- [x] DEV `window` replay helper confirmed already gated by `import.meta.env.DEV` in `main.tsx:16` — no prod change needed
- [x] `runtime-ingest.ts` entity dispatch infrastructure already present; this change wires live event source
- [/] `RuntimeStep` entity and `A2uiSurface`/full Protocols — rendering as-is (panels show empty when
  no data; not actively broken). Explicit hide/gate deferred to next phase cleanup pass

## §5 Validation (gate)
- [x] `cargo check` clean (SKIP_FRONTEND_BUILD=1)
- [x] `cargo test --lib` — 218/218 passed (confirmed post-commit)
- [ ] Manual: start a chat run → verify Cockpit/Runs entity appears in Runtime Console — pending merge
- [ ] Manual: trigger tool approval → Approve/Deny → verify run continues/aborts — pending merge
- [ ] Frontend vitest for Approve/Deny handlers — deferred to integration test phase

## Scope delta vs plan
- `RuntimeStep` emission not implemented (no per-step NormalizedEvent variant; needs orchestrator work)
- Parking-lot Hook bus not wired in this change (deferred to parity backlog phase)
- Protocols page not explicitly gated (it remains non-destructive empty state, not actively broken)
- Everything else in §1-§4 delivered as planned
